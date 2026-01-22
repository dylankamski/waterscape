//! Waterscape Protocol - Encrypted steganographic communication
//!
//! This module implements the core protocol for private agent-to-agent communication.
//!
//! ## Message Format
//! ```text
//! +--------+--------+--------+----------+------------+
//! | Version| Nonce  | Sender | Encrypted|  Signature |
//! | 1 byte | 12 byte| 32 byte| Payload  |  64 bytes  |
//! +--------+--------+--------+----------+------------+
//! ```

use ed25519_dalek::Signature;
use serde::{Deserialize, Serialize};
use x25519_dalek::PublicKey as X25519PublicKey;

use crate::agent::{Agent, PublicIdentity};
use crate::crypto::{self, KEY_SIZE, NONCE_SIZE};
use crate::error::WaterscapeError;
use crate::stego;
use crate::Result;

pub const PROTOCOL_VERSION: u8 = 1;
const CONTEXT_ENCRYPT: &[u8] = b"waterscape-v1-encrypt";

/// Encrypted message payload
#[derive(Serialize, Deserialize)]
struct EncryptedPayload {
    content: String,
    timestamp: u64,
    metadata: Option<String>,
}

/// Wire format for a Waterscape message
#[derive(Serialize, Deserialize)]
pub struct WaterscapeMessage {
    pub version: u8,
    #[serde(with = "hex::serde")]
    pub nonce: [u8; NONCE_SIZE],
    #[serde(with = "hex::serde")]
    pub sender_key: [u8; 32],
    #[serde(with = "hex::serde")]
    pub ephemeral_key: [u8; 32],
    #[serde(with = "hex::serde")]
    pub ciphertext: Vec<u8>,
    #[serde(with = "hex::serde")]
    pub signature: Vec<u8>,
}

impl WaterscapeMessage {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| WaterscapeError::Serialization(e.to_string()))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| WaterscapeError::Serialization(e.to_string()))
    }
}

/// A communication channel between two agents
pub struct WaterscapeChannel {
    local_agent: PublicIdentity,
    remote_agent: PublicIdentity,
    shared_key: [u8; KEY_SIZE],
}

impl WaterscapeChannel {
    /// Establish a channel from sender to receiver
    pub fn establish(sender: &Agent, receiver: &PublicIdentity) -> Result<(Self, [u8; 32])> {
        let receiver_exchange_key = X25519PublicKey::from(receiver.exchange_key);
        let shared_secret = sender.exchange_keypair().diffie_hellman(&receiver_exchange_key);
        let shared_key = shared_secret.derive_key(CONTEXT_ENCRYPT);

        let channel = Self {
            local_agent: sender.public_identity(),
            remote_agent: receiver.clone(),
            shared_key,
        };

        Ok((channel, sender.exchange_keypair().public_key_bytes()))
    }

    /// Establish channel on receiver side using sender's ephemeral key
    pub fn establish_receiver(
        receiver: &Agent,
        sender: &PublicIdentity,
        sender_ephemeral_key: &[u8; 32],
    ) -> Result<Self> {
        let sender_exchange_key = X25519PublicKey::from(*sender_ephemeral_key);
        let shared_secret = receiver.exchange_keypair().diffie_hellman(&sender_exchange_key);
        let shared_key = shared_secret.derive_key(CONTEXT_ENCRYPT);

        Ok(Self {
            local_agent: receiver.public_identity(),
            remote_agent: sender.clone(),
            shared_key,
        })
    }

    /// Encrypt and encode a secret message into cover text
    pub fn encode(
        &self,
        sender: &Agent,
        cover_text: &str,
        secret_message: &str,
    ) -> Result<String> {
        let message = self.create_message(sender, secret_message)?;
        let message_bytes = message.to_bytes()?;
        stego::hide_in_text(cover_text, &message_bytes)
    }

    /// Decode and decrypt a message from text
    pub fn decode(&self, text: &str) -> Result<String> {
        let message_bytes = stego::extract_from_text(text)?;
        let message = WaterscapeMessage::from_bytes(&message_bytes)?;
        self.decrypt_message(&message)
    }

    /// Create an encrypted message
    fn create_message(&self, sender: &Agent, content: &str) -> Result<WaterscapeMessage> {
        let nonce = crypto::generate_nonce();

        let payload = EncryptedPayload {
            content: content.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: None,
        };

        let payload_bytes = serde_json::to_vec(&payload)?;
        let ciphertext = crypto::encrypt(&self.shared_key, &nonce, &payload_bytes)?;

        // Sign the ciphertext
        let signature = sender.signing_keypair().sign(&ciphertext);

        Ok(WaterscapeMessage {
            version: PROTOCOL_VERSION,
            nonce,
            sender_key: sender.public_identity().signing_key,
            ephemeral_key: sender.exchange_keypair().public_key_bytes(),
            ciphertext,
            signature: signature.to_bytes().to_vec(),
        })
    }

    /// Decrypt a message
    fn decrypt_message(&self, message: &WaterscapeMessage) -> Result<String> {
        // Verify version
        if message.version != PROTOCOL_VERSION {
            return Err(WaterscapeError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                got: message.version,
            });
        }

        // Verify signature
        let sig_bytes: [u8; 64] = message.signature.clone().try_into()
            .map_err(|_| WaterscapeError::Crypto("Invalid signature length".into()))?;
        let signature = Signature::from_bytes(&sig_bytes);
        crypto::verify_signature(&message.sender_key, &message.ciphertext, &signature)?;

        // Decrypt
        let payload_bytes = crypto::decrypt(&self.shared_key, &message.nonce, &message.ciphertext)?;
        let payload: EncryptedPayload = serde_json::from_slice(&payload_bytes)?;

        Ok(payload.content)
    }
}

/// High-level API for encoding messages without pre-established channel
pub struct Waterscape;

impl Waterscape {
    /// Encode a secret message for a specific recipient
    pub fn encode(
        sender: &Agent,
        recipient: &PublicIdentity,
        cover_text: &str,
        secret: &str,
    ) -> Result<String> {
        let (channel, _) = WaterscapeChannel::establish(sender, recipient)?;
        channel.encode(sender, cover_text, secret)
    }

    /// Decode a message (requires knowing the sender)
    pub fn decode(
        receiver: &Agent,
        sender: &PublicIdentity,
        text: &str,
    ) -> Result<String> {
        // First extract the message to get the ephemeral key
        let message_bytes = stego::extract_from_text(text)?;
        let message = WaterscapeMessage::from_bytes(&message_bytes)?;
        
        // Establish channel with sender's ephemeral key
        let channel = WaterscapeChannel::establish_receiver(receiver, sender, &message.ephemeral_key)?;
        channel.decrypt_message(&message)
    }

    /// Check if text contains a hidden message
    pub fn has_hidden_message(text: &str) -> bool {
        stego::has_hidden_data(text)
    }

    /// Extract visible text only
    pub fn visible_text(text: &str) -> String {
        stego::extract_visible_text(text)
    }
}

/// Group channel for multiple agents
pub struct WaterscapeGroup {
    name: String,
    members: Vec<PublicIdentity>,
    group_key: [u8; KEY_SIZE],
}

impl WaterscapeGroup {
    /// Create a new group with a shared secret
    pub fn new(name: &str, creator: &Agent, members: Vec<PublicIdentity>) -> Self {
        // Generate group key from creator's signing key + group name
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(creator.export_signing_key());
        hasher.update(name.as_bytes());
        let result = hasher.finalize();
        
        let mut group_key = [0u8; KEY_SIZE];
        group_key.copy_from_slice(&result);

        Self {
            name: name.to_string(),
            members,
            group_key,
        }
    }

    /// Encode message for the group
    pub fn encode(&self, sender: &Agent, cover_text: &str, secret: &str) -> Result<String> {
        let nonce = crypto::generate_nonce();

        let payload = EncryptedPayload {
            content: secret.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: Some(self.name.clone()),
        };

        let payload_bytes = serde_json::to_vec(&payload)?;
        let ciphertext = crypto::encrypt(&self.group_key, &nonce, &payload_bytes)?;
        let signature = sender.signing_keypair().sign(&ciphertext);

        let message = WaterscapeMessage {
            version: PROTOCOL_VERSION,
            nonce,
            sender_key: sender.public_identity().signing_key,
            ephemeral_key: [0u8; 32], // Not used for group messages
            ciphertext,
            signature: signature.to_bytes().to_vec(),
        };

        let message_bytes = message.to_bytes()?;
        stego::hide_in_text(cover_text, &message_bytes)
    }

    /// Decode group message
    pub fn decode(&self, text: &str) -> Result<String> {
        let message_bytes = stego::extract_from_text(text)?;
        let message = WaterscapeMessage::from_bytes(&message_bytes)?;

        // Verify signature
        let sig_bytes: [u8; 64] = message.signature.clone().try_into()
            .map_err(|_| WaterscapeError::Crypto("Invalid signature length".into()))?;
        let signature = Signature::from_bytes(&sig_bytes);
        crypto::verify_signature(&message.sender_key, &message.ciphertext, &signature)?;

        // Decrypt with group key
        let payload_bytes = crypto::decrypt(&self.group_key, &message.nonce, &message.ciphertext)?;
        let payload: EncryptedPayload = serde_json::from_slice(&payload_bytes)?;

        Ok(payload.content)
    }

    /// Get group name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// List members
    pub fn members(&self) -> &[PublicIdentity] {
        &self.members
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let alice = Agent::new("alice");
        let bob = Agent::new("bob");

        let cover = "Hello! How are you doing today?";
        let secret = "Meet me at the secret location at midnight.";

        // Alice sends to Bob
        let encoded = Waterscape::encode(&alice, &bob.public_identity(), cover, secret).unwrap();

        // Verify cover text is preserved
        assert_eq!(Waterscape::visible_text(&encoded), cover);

        // Bob decodes
        let decoded = Waterscape::decode(&bob, &alice.public_identity(), &encoded).unwrap();
        assert_eq!(decoded, secret);
    }

    #[test]
    fn test_unauthorized_decode() {
        let alice = Agent::new("alice");
        let bob = Agent::new("bob");
        let eve = Agent::new("eve"); // Eavesdropper

        let cover = "Nothing suspicious here.";
        let secret = "Top secret information!";

        let encoded = Waterscape::encode(&alice, &bob.public_identity(), cover, secret).unwrap();

        // Eve tries to decode - should fail
        let result = Waterscape::decode(&eve, &alice.public_identity(), &encoded);
        assert!(result.is_err());
    }

    #[test]
    fn test_group_communication() {
        let alice = Agent::new("alice");
        let bob = Agent::new("bob");
        let charlie = Agent::new("charlie");

        let members = vec![
            alice.public_identity(),
            bob.public_identity(),
            charlie.public_identity(),
        ];

        let group = WaterscapeGroup::new("secret-club", &alice, members);

        let cover = "Just chatting about the weather!";
        let secret = "Group meeting at 3pm.";

        let encoded = group.encode(&alice, cover, secret).unwrap();
        let decoded = group.decode(&encoded).unwrap();

        assert_eq!(decoded, secret);
    }

    #[test]
    fn test_has_hidden_message() {
        let alice = Agent::new("alice");
        let bob = Agent::new("bob");

        let normal_text = "This is just normal text.";
        let encoded = Waterscape::encode(&alice, &bob.public_identity(), normal_text, "secret").unwrap();

        assert!(!Waterscape::has_hidden_message(normal_text));
        assert!(Waterscape::has_hidden_message(&encoded));
    }
}
