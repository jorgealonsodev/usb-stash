//! Core cryptographic operations for USB Stash.
//!
//! Provides Argon2id key derivation, XChaCha20-Poly1305 AEAD encryption/decryption,
//! and cryptographically secure random generation via `OsRng`.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::TryRngCore;
use rand::rngs::OsRng;
use zeroize::ZeroizeOnDrop;

use crate::error::CryptoError;

// ─── Constants (Task 3.1) ──────────────────────────────────────────────────

/// Argon2id memory cost in KiB (64 MB).
pub const ARGON2_MEMORY_KIB: u32 = 65_536;

/// Argon2id iteration count.
pub const ARGON2_ITERATIONS: u32 = 3;

/// Argon2id parallelism factor.
pub const ARGON2_PARALLELISM: u32 = 4;

/// Salt length in bytes.
pub const SALT_LEN: usize = 16;

/// XChaCha20 nonce length in bytes (192-bit).
pub const NONCE_LEN: usize = 24;

/// Derived key length in bytes (256-bit).
pub const KEY_LEN: usize = 32;

/// AEAD authentication tag length in bytes.
pub const TAG_LEN: usize = 16;

// ─── KeyMaterial (Task 3.2) ────────────────────────────────────────────────

/// Wrapper for key material that guarantees zeroization on drop.
///
/// Uses volatile writes via the `zeroize` crate to prevent compiler
/// optimization from eliding the zeroization.
#[derive(ZeroizeOnDrop)]
pub struct KeyMaterial(pub(crate) Box<[u8; KEY_LEN]>);

impl KeyMaterial {
    /// Create a `KeyMaterial` from raw key bytes.
    ///
    /// This is primarily useful for testing with known keys. In production,
    /// keys should always come from [`derive_key`].
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self(Box::new(bytes))
    }
}

// ─── Random Generation (Tasks 3.3, 3.4) ────────────────────────────────────

/// Generate a cryptographically secure random salt using `OsRng`.
pub fn generate_salt() -> Result<[u8; SALT_LEN], CryptoError> {
    let mut salt = [0u8; SALT_LEN];
    OsRng
        .try_fill_bytes(&mut salt)
        .map_err(|e| CryptoError::RngError(e.to_string()))?;
    Ok(salt)
}

/// Generate a cryptographically secure random nonce using `OsRng`.
pub fn generate_nonce() -> Result<[u8; NONCE_LEN], CryptoError> {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng
        .try_fill_bytes(&mut nonce)
        .map_err(|e| CryptoError::RngError(e.to_string()))?;
    Ok(nonce)
}

// ─── Key Derivation (Task 3.5) ─────────────────────────────────────────────

/// Derive a 256-bit master key from a password and salt using Argon2id.
///
/// Parameters are hardcoded per PRD section 6.2:
/// - Memory: 64 MiB (65536 KiB)
/// - Iterations: 3
/// - Parallelism: 4
/// - Output: 32 bytes
pub fn derive_key(password: &[u8], salt: &[u8; SALT_LEN]) -> Result<KeyMaterial, CryptoError> {
    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(KEY_LEN),
    )
    .map_err(|e| CryptoError::KdfError(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key_bytes = Box::new([0u8; KEY_LEN]);

    argon2
        .hash_password_into(password, salt, &mut *key_bytes)
        .map_err(|e| CryptoError::KdfError(e.to_string()))?;

    Ok(KeyMaterial(key_bytes))
}

// ─── Encryption / Decryption (Tasks 3.6, 3.7) ──────────────────────────────

/// Encrypt plaintext using XChaCha20-Poly1305.
///
/// Returns ciphertext with the 16-byte authentication tag appended.
pub fn encrypt(
    plaintext: &[u8],
    key: &KeyMaterial,
    nonce: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new_from_slice(&key.0[..])
        .map_err(|e| CryptoError::EncryptError(e.to_string()))?;

    let xnonce = XNonce::from_slice(nonce);

    cipher
        .encrypt(xnonce, plaintext.as_ref())
        .map_err(|e| CryptoError::EncryptError(e.to_string()))
}

/// Decrypt ciphertext using XChaCha20-Poly1305.
///
/// The authentication tag is verified before returning plaintext.
/// Returns `InvalidTag` if the ciphertext was tampered or the key is wrong.
pub fn decrypt(
    ciphertext: &[u8],
    key: &KeyMaterial,
    nonce: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new_from_slice(&key.0[..])
        .map_err(|e| CryptoError::DecryptError(e.to_string()))?;

    let xnonce = XNonce::from_slice(nonce);

    cipher
        .decrypt(xnonce, ciphertext.as_ref())
        .map_err(|_| CryptoError::InvalidTag)
}
