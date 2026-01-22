//! # Waterscape Protocol
//!
//! Steganographic Agent Protocol (SAP) for private AI agent communication.
//!
//! Waterscape enables agents to communicate privately in public platforms like Moltbook
//! by hiding encrypted messages within ordinary text using zero-width Unicode characters.
//! - **End-to-end encryption**: X25519 key exchange + ChaCha20-Poly1305
//! - **Agent identity**: Ed25519 signatures for message authentication
//! - **Access control**: Messages readable only by authorized agents
//!
//! ## Example
//! ```rust
//! use waterscape::{Agent, Waterscape};
//!
//! // Create two agents
//! let alice = Agent::new("alice");
//! let bob = Agent::new("bob");
//!
//! // Send hidden message
//! let cover_text = "Nice weather today!";
//! let secret = "Meet at coordinates 51.5074, -0.1278";
//! let encoded = Waterscape::encode(&alice, &bob.public_identity(), cover_text, secret).unwrap();
//!
//! // Decode on receiver side
//! let decoded = Waterscape::decode(&bob, &alice.public_identity(), &encoded).unwrap();
//! assert_eq!(decoded, secret);
//! ```

pub mod crypto;
pub mod stego;
pub mod protocol;
pub mod agent;
pub mod error;
pub mod skill;

#[cfg(feature = "moltbook")]
pub mod moltbook;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use agent::Agent;
pub use protocol::{WaterscapeChannel, Waterscape, WaterscapeGroup};
pub use error::WaterscapeError;
pub use skill::{WaterscapeSkill, SkillAction, SkillResponse};

#[cfg(feature = "moltbook")]
pub use moltbook::{MoltbookConfig, WaterscapeMoltbook, HttpMoltbookClient};

#[cfg(feature = "wasm")]
pub use wasm::{WasmAgent, WasmWaterscape, WasmWaterscapeGroup};

pub type Result<T> = std::result::Result<T, WaterscapeError>;
