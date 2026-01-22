//! Cryptographic primitives for the Waterscape protocol
//!
//! - X25519 for key exchange
//! - ChaCha20-Poly1305 for authenticated encryption
//! - Ed25519 for digital signatures
//! - HKDF for key derivation

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::Sha256;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

use crate::error::WaterscapeError;
use crate::Result;

pub const NONCE_SIZE: usize = 12;
pub const KEY_SIZE: usize = 32;
pub const SIGNATURE_SIZE: usize = 64;

/// Key pair for X25519 key exchange
pub struct KeyExchangePair {
    secret: StaticSecret,
    public: X25519PublicKey,
}

impl KeyExchangePair {
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);
        Self { secret, public }
    }

    pub fn public_key(&self) -> &X25519PublicKey {
        &self.public
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public.to_bytes()
    }

    /// Perform Diffie-Hellman key exchange
    pub fn diffie_hellman(&self, their_public: &X25519PublicKey) -> SharedSecret {
        let shared = self.secret.diffie_hellman(their_public);
        SharedSecret(shared.to_bytes())
    }
}

/// Shared secret from key exchange
pub struct SharedSecret([u8; 32]);

impl SharedSecret {
    /// Derive encryption key using HKDF
    pub fn derive_key(&self, context: &[u8]) -> [u8; KEY_SIZE] {
        let hk = Hkdf::<Sha256>::new(None, &self.0);
        let mut key = [0u8; KEY_SIZE];
        hk.expand(context, &mut key)
            .expect("HKDF expand should not fail with valid length");
        key
    }
}

impl Drop for SharedSecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Signing key pair for Ed25519
pub struct SigningKeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl SigningKeyPair {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(bytes);
        let verifying_key = signing_key.verifying_key();
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<()> {
        self.verifying_key
            .verify(message, signature)
            .map_err(|_| WaterscapeError::InvalidSignature)
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    pub fn signing_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

/// Verify a signature with a public verifying key
pub fn verify_signature(
    verifying_key_bytes: &[u8; 32],
    message: &[u8],
    signature: &Signature,
) -> Result<()> {
    let verifying_key = VerifyingKey::from_bytes(verifying_key_bytes)
        .map_err(|_| WaterscapeError::Crypto("Invalid verifying key".into()))?;
    verifying_key
        .verify(message, signature)
        .map_err(|_| WaterscapeError::InvalidSignature)
}

/// Encrypt data using ChaCha20-Poly1305
pub fn encrypt(key: &[u8; KEY_SIZE], nonce: &[u8; NONCE_SIZE], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| WaterscapeError::Crypto("Invalid key length".into()))?;
    let nonce = Nonce::from_slice(nonce);
    cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| WaterscapeError::Crypto("Encryption failed".into()))
}

/// Decrypt data using ChaCha20-Poly1305
pub fn decrypt(
    key: &[u8; KEY_SIZE],
    nonce: &[u8; NONCE_SIZE],
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| WaterscapeError::Crypto("Invalid key length".into()))?;
    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| WaterscapeError::AuthenticationFailed)
}

/// Generate a random nonce
pub fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    rand::RngCore::fill_bytes(&mut OsRng, &mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_exchange() {
        let alice = KeyExchangePair::generate();
        let bob = KeyExchangePair::generate();

        let alice_shared = alice.diffie_hellman(bob.public_key());
        let bob_shared = bob.diffie_hellman(alice.public_key());

        let alice_key = alice_shared.derive_key(b"test");
        let bob_key = bob_shared.derive_key(b"test");

        assert_eq!(alice_key, bob_key);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; KEY_SIZE];
        let nonce = generate_nonce();
        let plaintext = b"Hello, secret world!";

        let ciphertext = encrypt(&key, &nonce, plaintext).unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_signing() {
        let keypair = SigningKeyPair::generate();
        let message = b"Sign this message";

        let signature = keypair.sign(message);
        assert!(keypair.verify(message, &signature).is_ok());

        // Verify with wrong message should fail
        let wrong_message = b"Wrong message";
        assert!(keypair.verify(wrong_message, &signature).is_err());
    }
}
