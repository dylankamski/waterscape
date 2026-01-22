//! Agent identity and key management
//!
//! Each agent has:
//! - A unique identifier (name)
//! - An Ed25519 signing key pair for authentication
//! - An X25519 key pair for key exchange

use serde::{Deserialize, Serialize};

use crate::crypto::{KeyExchangePair, SigningKeyPair};
use crate::Result;

/// Public identity of an agent (can be shared freely)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicIdentity {
    pub name: String,
    pub signing_key: [u8; 32],
    pub exchange_key: [u8; 32],
}

impl PublicIdentity {
    /// Create fingerprint (first 8 bytes of signing key as hex)
    pub fn fingerprint(&self) -> String {
        hex::encode(&self.signing_key[..8])
    }
}

/// Full agent with private keys
pub struct Agent {
    name: String,
    signing_keypair: SigningKeyPair,
    exchange_keypair: KeyExchangePair,
}

impl Agent {
    /// Create a new agent with generated keys
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            signing_keypair: SigningKeyPair::generate(),
            exchange_keypair: KeyExchangePair::generate(),
        }
    }

    /// Restore agent from stored keys
    pub fn from_keys(
        name: &str,
        signing_key_bytes: &[u8; 32],
    ) -> Result<Self> {
        let signing_keypair = SigningKeyPair::from_bytes(signing_key_bytes)?;
        let exchange_keypair = KeyExchangePair::generate();
        
        Ok(Self {
            name: name.to_string(),
            signing_keypair,
            exchange_keypair,
        })
    }

    /// Get agent name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get public identity (safe to share)
    pub fn public_identity(&self) -> PublicIdentity {
        PublicIdentity {
            name: self.name.clone(),
            signing_key: self.signing_keypair.verifying_key_bytes(),
            exchange_key: self.exchange_keypair.public_key_bytes(),
        }
    }

    /// Get signing key pair (for internal use)
    pub(crate) fn signing_keypair(&self) -> &SigningKeyPair {
        &self.signing_keypair
    }

    /// Get exchange key pair (for internal use)
    pub(crate) fn exchange_keypair(&self) -> &KeyExchangePair {
        &self.exchange_keypair
    }

    /// Export private signing key (for backup)
    pub fn export_signing_key(&self) -> [u8; 32] {
        self.signing_keypair.signing_key_bytes()
    }

    /// Sign arbitrary data
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.signing_keypair.sign(data).to_bytes().to_vec()
    }
}

/// Agent registry for managing known agents
#[derive(Default)]
pub struct AgentRegistry {
    agents: std::collections::HashMap<String, PublicIdentity>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a known agent
    pub fn register(&mut self, identity: PublicIdentity) {
        self.agents.insert(identity.name.clone(), identity);
    }

    /// Look up agent by name
    pub fn get(&self, name: &str) -> Option<&PublicIdentity> {
        self.agents.get(name)
    }

    /// Look up agent by fingerprint
    pub fn get_by_fingerprint(&self, fingerprint: &str) -> Option<&PublicIdentity> {
        self.agents.values().find(|a| a.fingerprint() == fingerprint)
    }

    /// List all known agents
    pub fn list(&self) -> Vec<&PublicIdentity> {
        self.agents.values().collect()
    }

    /// Remove an agent
    pub fn remove(&mut self, name: &str) -> Option<PublicIdentity> {
        self.agents.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("test-agent");
        assert_eq!(agent.name(), "test-agent");
        
        let identity = agent.public_identity();
        assert_eq!(identity.name, "test-agent");
        assert!(!identity.fingerprint().is_empty());
    }

    #[test]
    fn test_agent_registry() {
        let mut registry = AgentRegistry::new();
        
        let alice = Agent::new("alice");
        let bob = Agent::new("bob");
        
        registry.register(alice.public_identity());
        registry.register(bob.public_identity());
        
        assert!(registry.get("alice").is_some());
        assert!(registry.get("bob").is_some());
        assert!(registry.get("charlie").is_none());
        
        assert_eq!(registry.list().len(), 2);
    }

    #[test]
    fn test_fingerprint_lookup() {
        let mut registry = AgentRegistry::new();
        let agent = Agent::new("test");
        let fingerprint = agent.public_identity().fingerprint();
        
        registry.register(agent.public_identity());
        
        let found = registry.get_by_fingerprint(&fingerprint);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test");
    }
}
