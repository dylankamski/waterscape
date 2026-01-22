//! Error types for the Waterscape protocol

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WaterscapeError {
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Steganographic encoding error: {0}")]
    Encoding(String),

    #[error("Decoding error: {0}")]
    Decoding(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Key exchange failed: {0}")]
    KeyExchange(String),

    #[error("Cover text too short for payload (need {needed} chars, have {available})")]
    CoverTextTooShort { needed: usize, available: usize },

    #[error("No hidden message found in text")]
    NoHiddenMessage,

    #[error("Message authentication failed")]
    AuthenticationFailed,

    #[error("Agent not authorized to read this message")]
    Unauthorized,

    #[error("Protocol version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u8, got: u8 },

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<chacha20poly1305::Error> for WaterscapeError {
    fn from(_: chacha20poly1305::Error) -> Self {
        WaterscapeError::Crypto("AEAD encryption/decryption failed".into())
    }
}

impl From<ed25519_dalek::SignatureError> for WaterscapeError {
    fn from(_: ed25519_dalek::SignatureError) -> Self {
        WaterscapeError::InvalidSignature
    }
}

impl From<serde_json::Error> for WaterscapeError {
    fn from(e: serde_json::Error) -> Self {
        WaterscapeError::Serialization(e.to_string())
    }
}
