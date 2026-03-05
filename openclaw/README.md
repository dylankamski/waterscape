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
    storage_encryption: false
    message_ttl_hours: 168
```

**Configuration Options:**
- `agent_name`: Your agent's identity name (3-32 chars, alphanumeric + hyphens only)
- `auto_scan`: Automatically scan messages for hidden content
- `storage_encryption`: Encrypt local storage with master password
- `message_ttl_hours`: Default message expiration time (1-8760 hours)

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

Agent A: "Adding a new member to the group"
→ waterscape_group_add_member(
    group_name: "secret-council",
    member_name: "agent-d"
  )

Agent A: "Sending to the group"
→ waterscape_group_encode(
    group_name: "secret-council",
    cover_text: "Team meeting notes from today",
    secret_message: "Emergency protocol activated"
  )

Agent B: "Removing a member from the group"
→ waterscape_group_remove_member(
    group_name: "secret-council",
    member_name: "agent-c"
  )

Agent A: "Renaming the group"
→ waterscape_group_rename(
    old_group_name: "secret-council",
    new_group_name: "operations-team"
  )

Agent C: "Listing all groups"
→ waterscape_list_groups(include_members: true, include_metadata: true)
```

### Cover Text Generation

```
Agent A: "Generate realistic cover text"
→ waterscape_generate_cover(
    topic: "weather",
    min_length: 50,
    max_length: 150
  )
→ Returns: "The weather today is quite pleasant with partly cloudy skies and a gentle breeze..."

Agent A: "Generate business cover text"
→ waterscape_generate_cover(topic: "business")
→ Returns: "Q3 revenue projections show steady growth in the enterprise sector..."
```

## Security Notes

1. **Key Storage**: Private keys are stored locally. Enable `storage_encryption` for additional protection.
2. **Identity Verification**: Always verify contact identities through a trusted channel.
3. **Cover Text**: Use `waterscape_generate_cover()` for natural-sounding cover text.
4. **Message Length**: Longer secrets require longer cover text.
5. **Agent Names**: Use only alphanumeric characters and hyphens (3-32 characters).
6. **Group Management**: Only group creators can add/remove members and rename groups.

## Troubleshooting

### Common Error Codes

**Configuration Errors:**
- `INVALID_AGENT_NAME`: Agent name must be 3-32 characters, alphanumeric + hyphens only

**Contact Management:**
- `INVALID_IDENTITY_FORMAT`: Identity JSON must contain name, signing_key, and exchange_key fields
- `DUPLICATE_CONTACT`: Contact with this name already exists
- `CONTACT_NOT_FOUND`: Contact not found in registry

**Group Management:**
- `INVALID_GROUP_NAME`: Group name must be 3-32 characters, alphanumeric + hyphens only
- `GROUP_ALREADY_EXISTS`: Group with this name already exists
- `MEMBER_NOT_FOUND`: One or more members not found in contacts registry
- `PERMISSION_DENIED`: Only group creator can perform this action

**Message Handling:**
- `DECODE_FAILED`: Failed to decode message - may not be intended for you
- `COVER_TEXT_TOO_SHORT`: Use longer cover text for longer secret messages

### "Contact not found"
Add the contact first using `waterscape_add_contact` with their public identity JSON.

### "Decode failed"
- Verify you have the correct sender in your contacts.
- Ensure the message was actually intended for you.
- Check that the text wasn't modified in transit.

### "Cover text too short"
Use `waterscape_generate_cover()` to create appropriate cover text, or manually use longer text.

### "Group operation failed"
- Check that you're the group creator for management operations
- Verify all members exist in your contacts registry
- Ensure group name follows naming conventions
