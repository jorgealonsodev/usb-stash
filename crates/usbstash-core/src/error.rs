//! Error types for cryptographic operations.
//!
//! All cryptographic errors are expressed as typed `CryptoError` variants.
//! Error messages MUST NOT leak sensitive material (keys, passwords, plaintext).

use thiserror::Error;

/// Typed error enum for all cryptographic operations.
///
/// Each variant represents a distinct failure mode. Error messages
/// are user-safe and never contain key bytes, passwords, or plaintext.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Key derivation failed (e.g., invalid Argon2id parameters).
    #[error("key derivation failed: {0}")]
    KdfError(String),

    /// Encryption failed (e.g., cipher error).
    #[error("encryption failed: {0}")]
    EncryptError(String),

    /// Decryption failed (e.g., internal cipher error, not tag mismatch).
    #[error("decryption failed: {0}")]
    DecryptError(String),

    /// Authentication tag verification failed — ciphertext was tampered
    /// or the wrong key was used.
    #[error("authentication tag verification failed")]
    InvalidTag,

    /// Random number generation failed (e.g., OS entropy source unavailable).
    #[error("random number generation failed: {0}")]
    RngError(String),
}
