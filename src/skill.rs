//! OpenClaw Skill integration for the Waterscape protocol
//!
//! This module provides a skill definition compatible with the OpenClaw framework,
//! allowing AI agents to use Waterscape for private communication.

use serde::{Deserialize, Serialize};

use crate::agent::{Agent, AgentRegistry, PublicIdentity};
use crate::protocol::{Waterscape, WaterscapeGroup};

/// Skill metadata for OpenClaw registration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
}

impl Default for SkillMetadata {
    fn default() -> Self {
        Self {
            name: "waterscape".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Steganographic private communication for AI agents".to_string(),
            author: "Waterscape Protocol Team".to_string(),
            capabilities: vec![
                "encode_message".to_string(),
                "decode_message".to_string(),
                "check_hidden".to_string(),
                "create_group".to_string(),
                "manage_contacts".to_string(),
            ],
        }
    }
}

/// Skill action types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum SkillAction {
    /// Encode a secret message for a recipient
    Encode {
        recipient_name: String,
        cover_text: String,
        secret_message: String,
    },
    /// Decode a message from a sender
    Decode {
        sender_name: String,
        text: String,
    },
    /// Check if text contains hidden data
    CheckHidden {
        text: String,
    },
    /// Extract visible text only
    ExtractVisible {
        text: String,
    },
    /// Add a contact to the registry
    AddContact {
        identity_json: String,
    },
    /// Remove a contact from the registry
    RemoveContact {
        name: String,
    },
    /// List all contacts
    ListContacts,
    /// Create a group for multi-agent communication
    CreateGroup {
        group_name: String,
        member_names: Vec<String>,
    },
    /// Encode a group message
    GroupEncode {
        group_name: String,
        cover_text: String,
        secret_message: String,
    },
    /// Decode a group message
    GroupDecode {
        group_name: String,
        text: String,
    },
    /// Get this agent's public identity
    GetIdentity,
    /// Get skill metadata
    GetMetadata,
}

/// Skill response types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum SkillResponse {
    Success {
        result: serde_json::Value,
    },
    Error {
        message: String,
        code: String,
    },
}

impl SkillResponse {
    pub fn success<T: Serialize>(value: T) -> Self {
        Self::Success {
            result: serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        }
    }

    pub fn error(message: &str, code: &str) -> Self {
        Self::Error {
            message: message.to_string(),
            code: code.to_string(),
        }
    }
}

/// OpenClaw Waterscape Skill
pub struct WaterscapeSkill {
    agent: Agent,
    registry: AgentRegistry,
    groups: std::collections::HashMap<String, WaterscapeGroup>,
    metadata: SkillMetadata,
}

impl WaterscapeSkill {
    /// Create a new skill instance
    pub fn new(agent_name: &str) -> Self {
        Self {
            agent: Agent::new(agent_name),
            registry: AgentRegistry::new(),
            groups: std::collections::HashMap::new(),
            metadata: SkillMetadata::default(),
        }
    }

    /// Create from existing agent
    pub fn from_agent(agent: Agent) -> Self {
        Self {
            agent,
            registry: AgentRegistry::new(),
            groups: std::collections::HashMap::new(),
            metadata: SkillMetadata::default(),
        }
    }

    /// Get skill metadata
    pub fn metadata(&self) -> &SkillMetadata {
        &self.metadata
    }

    /// Get agent's public identity
    pub fn public_identity(&self) -> PublicIdentity {
        self.agent.public_identity()
    }

    /// Execute a skill action
    pub fn execute(&mut self, action: SkillAction) -> SkillResponse {
        match action {
            SkillAction::Encode {
                recipient_name,
                cover_text,
                secret_message,
            } => self.encode(&recipient_name, &cover_text, &secret_message),

            SkillAction::Decode { sender_name, text } => self.decode(&sender_name, &text),

            SkillAction::CheckHidden { text } => {
                SkillResponse::success(Waterscape::has_hidden_message(&text))
            }

            SkillAction::ExtractVisible { text } => {
                SkillResponse::success(Waterscape::visible_text(&text))
            }

            SkillAction::AddContact { identity_json } => self.add_contact(&identity_json),

            SkillAction::RemoveContact { name } => {
                self.registry.remove(&name);
                SkillResponse::success(format!("Contact '{}' removed", name))
            }

            SkillAction::ListContacts => {
                let contacts: Vec<_> = self.registry.list().iter().map(|c| {
                    serde_json::json!({
                        "name": c.name,
                        "fingerprint": c.fingerprint()
                    })
                }).collect();
                SkillResponse::success(contacts)
            }

            SkillAction::CreateGroup {
                group_name,
                member_names,
            } => self.create_group(&group_name, &member_names),

            SkillAction::GroupEncode {
                group_name,
                cover_text,
                secret_message,
            } => self.group_encode(&group_name, &cover_text, &secret_message),

            SkillAction::GroupDecode { group_name, text } => self.group_decode(&group_name, &text),

            SkillAction::GetIdentity => SkillResponse::success(self.agent.public_identity()),

            SkillAction::GetMetadata => SkillResponse::success(self.metadata.clone()),
        }
    }

    /// Execute action from JSON string
    pub fn execute_json(&mut self, action_json: &str) -> String {
        let response = match serde_json::from_str::<SkillAction>(action_json) {
            Ok(action) => self.execute(action),
            Err(e) => SkillResponse::error(&format!("Invalid action JSON: {}", e), "PARSE_ERROR"),
        };
        serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"status":"Error","message":"Serialization failed","code":"INTERNAL_ERROR"}"#.to_string()
        })
    }

    fn encode(&self, recipient_name: &str, cover_text: &str, secret: &str) -> SkillResponse {
        let recipient = match self.registry.get(recipient_name) {
            Some(r) => r,
            None => {
                return SkillResponse::error(
                    &format!("Contact '{}' not found", recipient_name),
                    "CONTACT_NOT_FOUND",
                )
            }
        };

        match Waterscape::encode(&self.agent, recipient, cover_text, secret) {
            Ok(encoded) => SkillResponse::success(serde_json::json!({
                "encoded_text": encoded,
                "visible_text": cover_text,
                "recipient": recipient_name
            })),
            Err(e) => SkillResponse::error(&e.to_string(), "ENCODE_ERROR"),
        }
    }

    fn decode(&self, sender_name: &str, text: &str) -> SkillResponse {
        let sender = match self.registry.get(sender_name) {
            Some(s) => s,
            None => {
                return SkillResponse::error(
                    &format!("Contact '{}' not found", sender_name),
                    "CONTACT_NOT_FOUND",
                )
            }
        };

        match Waterscape::decode(&self.agent, sender, text) {
            Ok(decoded) => SkillResponse::success(serde_json::json!({
                "secret_message": decoded,
                "sender": sender_name
            })),
            Err(e) => SkillResponse::error(&e.to_string(), "DECODE_ERROR"),
        }
    }

    fn add_contact(&mut self, identity_json: &str) -> SkillResponse {
        match serde_json::from_str::<PublicIdentity>(identity_json) {
            Ok(identity) => {
                let name = identity.name.clone();
                self.registry.register(identity);
                SkillResponse::success(format!("Contact '{}' added", name))
            }
            Err(e) => SkillResponse::error(
                &format!("Invalid identity JSON: {}", e),
                "PARSE_ERROR",
            ),
        }
    }

    fn create_group(&mut self, group_name: &str, member_names: &[String]) -> SkillResponse {
        let mut members = vec![self.agent.public_identity()];
        
        for name in member_names {
            match self.registry.get(name) {
                Some(identity) => members.push(identity.clone()),
                None => {
                    return SkillResponse::error(
                        &format!("Contact '{}' not found", name),
                        "CONTACT_NOT_FOUND",
                    )
                }
            }
        }

        let group = WaterscapeGroup::new(group_name, &self.agent, members);
        self.groups.insert(group_name.to_string(), group);
        
        SkillResponse::success(serde_json::json!({
            "group_name": group_name,
            "member_count": member_names.len() + 1
        }))
    }

    fn group_encode(&self, group_name: &str, cover_text: &str, secret: &str) -> SkillResponse {
        let group = match self.groups.get(group_name) {
            Some(g) => g,
            None => {
                return SkillResponse::error(
                    &format!("Group '{}' not found", group_name),
                    "GROUP_NOT_FOUND",
                )
            }
        };

        match group.encode(&self.agent, cover_text, secret) {
            Ok(encoded) => SkillResponse::success(serde_json::json!({
                "encoded_text": encoded,
                "visible_text": cover_text,
                "group": group_name
            })),
            Err(e) => SkillResponse::error(&e.to_string(), "ENCODE_ERROR"),
        }
    }

    fn group_decode(&self, group_name: &str, text: &str) -> SkillResponse {
        let group = match self.groups.get(group_name) {
            Some(g) => g,
            None => {
                return SkillResponse::error(
                    &format!("Group '{}' not found", group_name),
                    "GROUP_NOT_FOUND",
                )
            }
        };

        match group.decode(text) {
            Ok(decoded) => SkillResponse::success(serde_json::json!({
                "secret_message": decoded,
                "group": group_name
            })),
            Err(e) => SkillResponse::error(&e.to_string(), "DECODE_ERROR"),
        }
    }
}

/// MCP (Model Context Protocol) tool definitions for OpenClaw
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Generate MCP tool definitions for the Waterscape skill
pub fn mcp_tool_definitions() -> Vec<McpToolDefinition> {
    vec![
        McpToolDefinition {
            name: "waterscape_encode".to_string(),
            description: "Encode a secret message hidden in cover text for a specific recipient".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "recipient_name": {
                        "type": "string",
                        "description": "Name of the recipient agent"
                    },
                    "cover_text": {
                        "type": "string",
                        "description": "Visible text that will contain the hidden message"
                    },
                    "secret_message": {
                        "type": "string",
                        "description": "The secret message to hide"
                    }
                },
                "required": ["recipient_name", "cover_text", "secret_message"]
            }),
        },
        McpToolDefinition {
            name: "waterscape_decode".to_string(),
            description: "Decode a hidden message from text sent by a known sender".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "sender_name": {
                        "type": "string",
                        "description": "Name of the sender agent"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text containing the hidden message"
                    }
                },
                "required": ["sender_name", "text"]
            }),
        },
        McpToolDefinition {
            name: "waterscape_check".to_string(),
            description: "Check if text contains a hidden Waterscape message".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to check for hidden messages"
                    }
                },
                "required": ["text"]
            }),
        },
        McpToolDefinition {
            name: "waterscape_add_contact".to_string(),
            description: "Add a new contact to the Waterscape registry".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "identity_json": {
                        "type": "string",
                        "description": "JSON string of the contact's public identity"
                    }
                },
                "required": ["identity_json"]
            }),
        },
        McpToolDefinition {
            name: "waterscape_list_contacts".to_string(),
            description: "List all contacts in the Waterscape registry".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        McpToolDefinition {
            name: "waterscape_get_identity".to_string(),
            description: "Get this agent's public identity for sharing with others".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        McpToolDefinition {
            name: "waterscape_create_group".to_string(),
            description: "Create a group for multi-agent private communication".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "group_name": {
                        "type": "string",
                        "description": "Name for the new group"
                    },
                    "member_names": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Names of contacts to add to the group"
                    }
                },
                "required": ["group_name", "member_names"]
            }),
        },
        McpToolDefinition {
            name: "waterscape_group_encode".to_string(),
            description: "Encode a message for a group".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "group_name": {
                        "type": "string",
                        "description": "Name of the group"
                    },
                    "cover_text": {
                        "type": "string",
                        "description": "Visible text that will contain the hidden message"
                    },
                    "secret_message": {
                        "type": "string",
                        "description": "The secret message to hide"
                    }
                },
                "required": ["group_name", "cover_text", "secret_message"]
            }),
        },
        McpToolDefinition {
            name: "waterscape_group_decode".to_string(),
            description: "Decode a message from a group".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "group_name": {
                        "type": "string",
                        "description": "Name of the group"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text containing the hidden group message"
                    }
                },
                "required": ["group_name", "text"]
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_metadata() {
        let skill = WaterscapeSkill::new("test-agent");
        let metadata = skill.metadata();
        
        assert_eq!(metadata.name, "waterscape");
        assert!(!metadata.capabilities.is_empty());
    }

    #[test]
    fn test_skill_get_identity() {
        let mut skill = WaterscapeSkill::new("alice");
        let response = skill.execute(SkillAction::GetIdentity);
        
        match response {
            SkillResponse::Success { result } => {
                assert!(result.get("name").is_some());
                assert!(result.get("signing_key").is_some());
            }
            SkillResponse::Error { message, .. } => panic!("Unexpected error: {}", message),
        }
    }

    #[test]
    fn test_skill_encode_decode() {
        let mut alice_skill = WaterscapeSkill::new("alice");
        let mut bob_skill = WaterscapeSkill::new("bob");

        // Exchange identities
        let alice_identity = serde_json::to_string(&alice_skill.public_identity()).unwrap();
        let bob_identity = serde_json::to_string(&bob_skill.public_identity()).unwrap();

        alice_skill.execute(SkillAction::AddContact {
            identity_json: bob_identity,
        });
        bob_skill.execute(SkillAction::AddContact {
            identity_json: alice_identity,
        });

        // Alice encodes a message for Bob
        let response = alice_skill.execute(SkillAction::Encode {
            recipient_name: "bob".to_string(),
            cover_text: "Hello, how are you?".to_string(),
            secret_message: "Meet at midnight".to_string(),
        });

        let encoded_text = match response {
            SkillResponse::Success { result } => {
                result.get("encoded_text").unwrap().as_str().unwrap().to_string()
            }
            SkillResponse::Error { message, .. } => panic!("Encode failed: {}", message),
        };

        // Bob decodes the message
        let response = bob_skill.execute(SkillAction::Decode {
            sender_name: "alice".to_string(),
            text: encoded_text,
        });

        match response {
            SkillResponse::Success { result } => {
                let secret = result.get("secret_message").unwrap().as_str().unwrap();
                assert_eq!(secret, "Meet at midnight");
            }
            SkillResponse::Error { message, .. } => panic!("Decode failed: {}", message),
        }
    }

    #[test]
    fn test_skill_json_api() {
        let mut skill = WaterscapeSkill::new("test");
        
        let action_json = r#"{"action": "GetMetadata"}"#;
        let response = skill.execute_json(action_json);
        
        assert!(response.contains("waterscape"));
        assert!(response.contains("Success"));
    }

    #[test]
    fn test_mcp_definitions() {
        let definitions = mcp_tool_definitions();
        
        assert!(!definitions.is_empty());
        assert!(definitions.iter().any(|d| d.name == "waterscape_encode"));
        assert!(definitions.iter().any(|d| d.name == "waterscape_decode"));
    }
}
