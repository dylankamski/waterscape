import init, { WasmAgent, WasmWaterscape, WasmWaterscapeGroup } from '/wasm/waterscape.js';

let wasmReady = false;
let alice = null;
let bob = null;
let eve = null;
let senderIsAlice = true;
let lastEncodedText = null;

let groupObj = null;
let groupAgents = {};
let groupOutsider = null;
let lastGroupEncoded = null;

function $(id) { return document.getElementById(id); }

function show(id) { const el = $(id); if (el) el.style.display = ''; }
function hide(id) { const el = $(id); if (el) el.style.display = 'none'; }

function setText(id, val) { const el = $(id); if (el) el.textContent = val; }

function countZeroWidth(text) {
  return (text.match(/[\u200B\u200C\u200D\u2060\uFEFF]/g) || []).length;
}

function formatZeroWidthForDisplay(zw) {
  return zw
    .replace(/\u200B/g, '<span class="zw zw-0">0</span>')
    .replace(/\u200C/g, '<span class="zw zw-1">1</span>')
    .replace(/\u200D/g, '<span class="zw zw-sep">|</span>')
    .replace(/\u2060/g, '<span class="zw zw-start">[</span>')
    .replace(/\uFEFF/g, '<span class="zw zw-end">]</span>');
}

function copyToClipboard(text, btnId) {
  if (!text) return;
  navigator.clipboard.writeText(text).then(() => {
    const btn = $(btnId);
    if (!btn) return;
    const orig = btn.textContent;
    btn.textContent = 'Copied!';
    setTimeout(() => { btn.textContent = orig; }, 1500);
  });
}

async function initWasm() {
  const dot = $('status-dot');
  const lbl = $('status-label');
  try {
    await init('/wasm/waterscape_bg.wasm');
    wasmReady = true;
    if (dot) dot.className = 'status-dot ok';
    if (lbl) lbl.textContent = 'WASM ready.';
    createP2PAgents();
    createGroupAgents();
    enableButtons();
    buildGroupMembersList();
  } catch (e) {
    if (dot) dot.className = 'status-dot err';
    if (lbl) lbl.textContent = 'WASM failed: ' + e.message;
  }
}

function enableButtons() {
  ['encode-btn', 'decode-btn', 'detect-btn', 'create-group-btn'].forEach(id => {
    const el = $(id);
    if (el) el.disabled = false;
  });
}

document.querySelectorAll('.tab').forEach(tab => {
  tab.addEventListener('click', () => {
    const target = tab.dataset.tab;
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    tab.classList.add('active');
    const panel = $('tab-' + target);
    if (panel) panel.classList.add('active');
  });
});

function createP2PAgents() {
  if (alice) alice.free();
  if (bob)   bob.free();
  if (eve)   eve.free();
  alice = new WasmAgent('Alice');
  bob   = new WasmAgent('Bob');
  eve   = new WasmAgent('Eve');
  updateP2PAgentUI();
}

function updateP2PAgentUI() {
  const sender   = senderIsAlice ? alice : bob;
  const receiver = senderIsAlice ? bob   : alice;
  setText('alice-name', alice.name);
  setText('bob-name',   bob.name);
  setText('alice-fp',   alice.fingerprint);
  setText('bob-fp',     bob.fingerprint);
  setText('encode-direction', `${sender.name} → ${receiver.name}`);
  setText('decode-direction', `${receiver.name} ← ${sender.name}`);
}

function showIdentityPanel(agent, label) {
  try {
    const json = JSON.parse(agent.publicIdentityJson());
    const pre = $('identity-json');
    if (pre) pre.textContent = JSON.stringify(json, null, 2);
    setText('id-panel-owner', label);
    show('identity-panel');
  } catch (e) { /* ignore */ }
}

function clearP2PResults() {
  hide('encode-result');
  hide('stego-viz');
  hide('decode-result');
  hide('decode-error');
  hide('unauthorized-box');
  lastEncodedText = null;
  const di = $('decode-input');
  if (di) di.value = '';
  const ueBtn = $('use-encoded-btn');
  if (ueBtn) ueBtn.disabled = true;
  const eveBtn = $('eve-btn');
  if (eveBtn) eveBtn.disabled = true;
}

$('agent-alice')?.addEventListener('click', e => {
  if (e.target.closest('button')) return;
  if (alice) showIdentityPanel(alice, 'Alice');
});
$('agent-bob')?.addEventListener('click', e => {
  if (e.target.closest('button')) return;
  if (bob) showIdentityPanel(bob, 'Bob');
});

$('regen-alice')?.addEventListener('click', () => {
  if (!wasmReady) return;
  if (alice) alice.free();
  alice = new WasmAgent('Alice');
  updateP2PAgentUI();
  showIdentityPanel(alice, 'Alice');
  clearP2PResults();
});

$('regen-bob')?.addEventListener('click', () => {
  if (!wasmReady) return;
  if (bob) bob.free();
  bob = new WasmAgent('Bob');
  updateP2PAgentUI();
  showIdentityPanel(bob, 'Bob');
  clearP2PResults();
});

$('swap-btn')?.addEventListener('click', () => {
  senderIsAlice = !senderIsAlice;
  updateP2PAgentUI();
  hide('decode-result');
  hide('decode-error');
});

$('encode-btn')?.addEventListener('click', () => {
  if (!wasmReady) return;
  const cover  = $('cover-text')?.value.trim()  || '';
  const secret = $('secret-text')?.value.trim() || '';
  if (!cover || !secret) return;

  const sender    = senderIsAlice ? alice : bob;
  const receiver  = senderIsAlice ? bob   : alice;

  try {
    const recipientJson = receiver.publicIdentityJson();
    const encoded = WasmWaterscape.encode(sender, recipientJson, cover, secret);
    lastEncodedText = encoded;

    const visible = WasmWaterscape.visibleText(encoded);
    const zwCount = countZeroWidth(encoded);

    setText('encoded-text', visible);
    setText('stat-visible', visible.length.toString());
    setText('stat-hidden',  new TextEncoder().encode(secret).length + ' B');
    setText('stat-total',   encoded.length.toString());
    setText('stat-zw',      zwCount.toString());
    const zwChars = [...encoded].filter(c => '\u200B\u200C\u200D\u2060\uFEFF'.includes(c)).join('');
    const vizStego = $('viz-stego');
    if (vizStego) vizStego.innerHTML = formatZeroWidthForDisplay(zwChars.slice(0, 300)) + (zwChars.length > 300 ? '…' : '');
    setText('viz-public', visible);
    setText('viz-crypto', `X25519 ECDH · ChaCha20-Poly1305 · Ed25519 sig · ${zwCount} zero-width chars embedded`);

    show('encode-result');
    show('stego-viz');
    show('unauthorized-box');

    const ueBtn = $('use-encoded-btn');
    if (ueBtn) ueBtn.disabled = false;
    const eveBtn = $('eve-btn');
    if (eveBtn) eveBtn.disabled = false;

    const di = $('decode-input');
    if (di) di.value = encoded;

    const note = $('examples-note');
    if (note) note.style.display = 'none';

  } catch (e) {
    alert('Encode error: ' + e);
  }
});

$('decode-btn')?.addEventListener('click', () => decodeP2P('legit'));
$('eve-btn')?.addEventListener('click',    () => decodeP2P('eve'));

function decodeP2P(mode) {
  if (!wasmReady) return;
  const text = $('decode-input')?.value || '';
  if (!text) return;

  const sender    = senderIsAlice ? alice : bob;
  const receiver  = senderIsAlice ? bob   : alice;
  const decodeAgent = mode === 'eve' ? eve : receiver;

  hide('decode-result');
  hide('decode-error');

  try {
    const senderJson = sender.publicIdentityJson();
    const decoded = WasmWaterscape.decode(decodeAgent, senderJson, text);
    setText('decoded-text', decoded);
    show('decode-result');
  } catch (e) {
    setText('error-text', e.toString());
    show('decode-error');
  }
}

$('use-encoded-btn')?.addEventListener('click', () => {
  if (lastEncodedText) {
    const di = $('decode-input');
    if (di) di.value = lastEncodedText;
  }
});

$('copy-encoded')?.addEventListener('click', () => copyToClipboard(lastEncodedText, 'copy-encoded'));

$('detect-btn')?.addEventListener('click', () => {
  if (!wasmReady) return;
  const text = $('detect-input')?.value || '';
  if (!text) return;
  runDetect(text);
});

function runDetect(text) {
  const hasHidden = WasmWaterscape.hasHiddenMessage(text);
  const visible   = WasmWaterscape.visibleText(text);
  const zwCount   = countZeroWidth(text);
  const total     = text.length;
  const visLen    = visible.length;

  const statusEl = $('detect-status');
  if (statusEl) statusEl.className = 'detect-status ' + (hasHidden ? 'detected' : 'clean');

  setText('detect-icon',     hasHidden ? '∃' : '∅');
  setText('detect-headline', hasHidden ? 'Hidden payload detected' : 'No hidden data found');
  setText('detect-detail',
    hasHidden
      ? `${zwCount} zero-width characters found (${((zwCount / total) * 100).toFixed(1)}% of total length)`
      : 'Text appears clean — no steganographic markers detected.'
  );

  setText('detect-original',   visible);
  setText('detect-visible',    visible);
  setText('detect-total-len',  `${total} total chars`);
  setText('detect-visible-len',`${visLen} visible chars · ${total - visLen} hidden chars`);

  const barsEl = $('breakdown-bars');
  if (barsEl) {
    const pct = (n) => total > 0 ? (n / total * 100).toFixed(1) : '0';
    barsEl.innerHTML = [
      { label: 'Visible chars', count: visLen,        cls: '' },
      { label: 'Zero-width',    count: zwCount,       cls: 'zw' },
    ].map(({ label, count, cls }) => `
      <div class="breakdown-bar-row">
        <span class="breakdown-bar-label">${label}</span>
        <div class="breakdown-bar-track">
          <div class="breakdown-bar-fill ${cls}" style="width:${pct(count)}%"></div>
        </div>
        <span class="breakdown-bar-count">${count} (${pct(count)}%)</span>
      </div>
    `).join('');
  }

  show('detect-result');
}

$('ex-clean')?.addEventListener('click', () => {
  const input = $('detect-input');
  if (input) input.value = 'The quarterly report shows positive growth across all sectors. No hidden information here, just ordinary text.';
  hide('detect-result');
});

$('ex-encoded')?.addEventListener('click', () => {
  if (!lastEncodedText) return;
  const input = $('detect-input');
  if (input) input.value = lastEncodedText;
  hide('detect-result');
});

const GROUP_MEMBER_NAMES = ['Alice', 'Bob', 'Charlie'];

function createGroupAgents() {
  Object.values(groupAgents).forEach(a => a.free());
  groupAgents = {};
  if (groupOutsider) groupOutsider.free();
  GROUP_MEMBER_NAMES.forEach(name => { groupAgents[name] = new WasmAgent(name); });
  groupOutsider = new WasmAgent('Outsider');
}

function buildGroupMembersList() {
  const list = $('group-members-list');
  if (!list) return;
  list.innerHTML = GROUP_MEMBER_NAMES.map(name => {
    const agent = groupAgents[name];
    const fp = agent ? agent.fingerprint : '—';
    return `
      <div class="member-item">
        <div class="member-avatar">${name[0]}</div>
        <span>${name}</span>
        <span class="member-fp">${fp}</span>
      </div>`;
  }).join('');
}

$('create-group-btn')?.addEventListener('click', () => {
  if (!wasmReady) return;
  const groupName = $('group-name')?.value.trim() || 'secret-ops';
  if (!groupName) return;

  try {
    if (groupObj) groupObj.free();
    const creator  = groupAgents['Alice'];
    const members  = GROUP_MEMBER_NAMES.map(n => groupAgents[n]);
    const membersJson = JSON.stringify(members.map(m => JSON.parse(m.publicIdentityJson())));
    groupObj = new WasmWaterscapeGroup(groupName, creator, membersJson);

    const sel = $('group-sender');
    if (sel) {
      sel.innerHTML = GROUP_MEMBER_NAMES.map(n => `<option value="${n}">${n}</option>`).join('');
      sel.disabled = false;
    }

    setText('group-status-name', groupName);
    setText('member-count', GROUP_MEMBER_NAMES.length.toString());
    show('group-status');

    $('group-encode-btn').disabled = false;
    $('group-decode-btn').disabled = false;
    $('group-outsider-btn').disabled = false;

  } catch (e) {
    alert('Create group error: ' + e);
  }
});

$('group-sender')?.addEventListener('change', function() {
  setText('group-sender-label', this.value ? `Sender: ${this.value}` : '—');
});

$('group-encode-btn')?.addEventListener('click', () => {
  if (!wasmReady || !groupObj) return;
  const senderName = $('group-sender')?.value;
  if (!senderName) return;
  const cover  = $('group-cover')?.value.trim()  || '';
  const secret = $('group-secret')?.value.trim() || '';
  if (!cover || !secret) return;

  try {
    const sender = groupAgents[senderName];
    const encoded = groupObj.encode(sender, cover, secret);
    lastGroupEncoded = encoded;

    const visible = WasmWaterscape.visibleText(encoded);
    setText('group-encoded-text', visible);
    show('group-encode-result');
    show('group-outsider-box');

    const useBtn = $('group-use-encoded-btn');
    if (useBtn) useBtn.disabled = false;

    const di = $('group-decode-input');
    if (di) di.value = encoded;

  } catch (e) {
    alert('Group encode error: ' + e);
  }
});

$('group-decode-btn')?.addEventListener('click',    () => decodeGroup('member'));
$('group-outsider-btn')?.addEventListener('click',  () => decodeGroup('outsider'));

function decodeGroup(mode) {
  if (!wasmReady || !groupObj) return;
  const text = $('group-decode-input')?.value || '';
  if (!text) return;

  hide('group-decode-result');
  hide('group-decode-error');

  try {
    let decoded;
    if (mode === 'outsider') {
      const outsiderGroup = new WasmWaterscapeGroup(
        groupObj.name,
        groupOutsider,
        JSON.stringify([JSON.parse(groupOutsider.publicIdentityJson())])
      );
      try {
        decoded = outsiderGroup.decode(text);
        outsiderGroup.free();
      } catch (e) {
        outsiderGroup.free();
        throw e;
      }
    } else {
      decoded = groupObj.decode(text);
    }
    setText('group-decoded-text', decoded);
    show('group-decode-result');
  } catch (e) {
    setText('group-error-text', e.toString());
    show('group-decode-error');
  }
}

$('group-use-encoded-btn')?.addEventListener('click', () => {
  if (lastGroupEncoded) {
    const di = $('group-decode-input');
    if (di) di.value = lastGroupEncoded;
  }
});

$('copy-group-encoded')?.addEventListener('click', () => copyToClipboard(lastGroupEncoded, 'copy-group-encoded'));

initWasm();
