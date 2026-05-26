//! Error types for cryptographic operations and stash file format.
//!
//! All cryptographic errors are expressed as typed `CryptoError` variants.
//! Error messages MUST NOT leak sensitive material (keys, passwords, plaintext).
//!
//! `StashError` is the top-level error for stash operations, wrapping `CryptoError`
//! and adding format, I/O, and lifecycle variants.

use std::path::PathBuf;

use thiserror::Error;

// ─── CryptoError (Phase 1) ─────────────────────────────────────────────────

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

// ─── StashError (Phase 2) ──────────────────────────────────────────────────

/// Top-level error for stash file operations.
///
/// Wraps `CryptoError` via `#[from]` so crypto calls can use `?` directly.
/// Format, I/O, and lifecycle errors are expressed as distinct variants.
/// Error messages never contain key bytes, passwords, or plaintext.
#[derive(Debug, Error)]
pub enum StashError {
    /// A cryptographic operation failed (wraps `CryptoError`).
    #[error("crypto error: {0}")]
    Crypto(#[from] CryptoError),

    /// The file format is invalid or corrupted.
    #[error("invalid format: {0}")]
    InvalidFormat(String),

    /// The file format version is not supported.
    #[error("unsupported version: {0}")]
    UnsupportedVersion(u16),

    /// An I/O error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization or deserialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// The requested file was not found.
    #[error("not found: {}", _0.display())]
    NotFound(PathBuf),

    /// A file already exists at the target path.
    #[error("already exists: {}", _0.display())]
    AlreadyExists(PathBuf),

    /// The stash is locked — sensitive data has been zeroized.
    #[error("stash is locked")]
    Locked,

    /// The current password provided for change_password is incorrect.
    #[error("wrong password")]
    WrongPassword,

    /// The new password does not meet minimum strength requirements.
    #[error("password too weak: must be at least 8 characters")]
    PasswordTooWeak,
}
