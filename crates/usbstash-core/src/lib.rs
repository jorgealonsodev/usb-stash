//! USB Stash — Core cryptographic primitives.
//!
//! Provides Argon2id key derivation, XChaCha20-Poly1305 AEAD encryption,
//! zeroization of sensitive material, typed error handling, and the Stash
//! lifecycle API for encrypted file containers.

pub mod crypto;
pub mod error;
pub mod format;
pub mod stash;

pub use crypto::*;
pub use error::{CryptoError, StashError};
pub use stash::{Settings, Stash, StashEntry, StashMeta, StashMetadata, StashPayload};
