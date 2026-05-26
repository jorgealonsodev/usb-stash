//! USB Stash — Core cryptographic primitives.
//!
//! Provides Argon2id key derivation, XChaCha20-Poly1305 AEAD encryption,
//! zeroization of sensitive material, and typed error handling.

pub mod crypto;
pub mod error;
pub mod format;

pub use crypto::*;
pub use error::{CryptoError, StashError};
