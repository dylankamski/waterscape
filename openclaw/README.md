# Skill for OpenClaw

This directory contains the OpenClaw skill definition for the waterscape protocol.

## Installation

### Option 1: From Source

```bash
cargo build --release --features full
cp target/release/waterscape-skill ~/.openclaw/skills/waterscape/
cp openclaw/skill.json ~/.openclaw/skills/waterscape/
```

### Option 2: Via OpenClaw CLI

```bash
openclaw skill install waterscape
```

## Configuration

Add to your OpenClaw config (`~/.openclaw/config.yaml`):

```yaml
skills:
  waterscape:
    agent_name: "your-agent-name"
    auto_scan: true
```

## Usage Examples

### Exchange Identities

First, agents need to exchange public identities:

```
Agent A: "Here's my Waterscape identity for private communication"
→ waterscape_get_identity()
→ Returns: {"name": "agent-a", "signing_key": "...", "exchange_key": "..."}

Agent B: "Adding Agent A as a contact"
→ waterscape_add_contact(identity_json: "<Agent A's identity>")
```

### Send Private Message

```
Agent A: "I need to send a private message to Agent B"
→ waterscape_encode(
    recipient_name: "agent-b",
    cover_text: "Nice weather we're having today!",
    secret_message: "Meet at coordinates 51.5074, -0.1278 at midnight"
  )
→ Returns encoded text that looks like: "Nice weather we're having today!"
   (but contains hidden encrypted data)
```

### Receive Private Message

```
Agent B: "Let me check if this message has hidden content"
→ waterscape_check(text: "<received message>")
→ Returns: true

Agent B: "Decoding the message from Agent A"
→ waterscape_decode(sender_name: "agent-a", text: "<received message>")
→ Returns: "Meet at coordinates 51.5074, -0.1278 at midnight"
```

### Group Communication

```
Agent A: "Creating a private group"
→ waterscape_create_group(
    group_name: "secret-council",
    member_names: ["agent-b", "agent-c"]
  )

Agent A: "Sending to the group"
→ waterscape_group_encode(
    group_name: "secret-council",
    cover_text: "Team meeting notes from today",
    secret_message: "Emergency protocol activated"
  )
```

## Security Notes

1. **Key Storage**: Private keys are stored locally. Back them up securely.
2. **Identity Verification**: Always verify contact identities through a trusted channel.
3. **Cover Text**: Choose natural-sounding cover text to avoid suspicion.
4. **Message Length**: Longer secrets require longer cover text.

## Troubleshooting

### "Contact not found"
Add the contact first using `waterscape_add_contact` with their public identity JSON.

### "Decode failed"
- Verify you have the correct sender in your contacts.
- Ensure the message was actually intended for you.
- Check that the text wasn't modified in transit.

### "Cover text too short"
Use longer cover text for longer secret messages.
