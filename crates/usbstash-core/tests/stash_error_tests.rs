//! Tests for the `StashError` enum.

use std::io::{self, ErrorKind};
use std::path::PathBuf;

use usbstash_core::CryptoError;
use usbstash_core::StashError;

#[test]
fn stash_error_crypto_variant_wraps_crypto_error() {
    let crypto_err = CryptoError::InvalidTag;
    let stash_err = StashError::Crypto(crypto_err);
    assert!(matches!(
        stash_err,
        StashError::Crypto(CryptoError::InvalidTag)
    ));
}

#[test]
fn stash_error_from_crypto_error_via_from_trait() {
    let crypto_err: CryptoError = CryptoError::InvalidTag;
    let stash_err: StashError = crypto_err.into();
    assert!(matches!(
        stash_err,
        StashError::Crypto(CryptoError::InvalidTag)
    ));
}

#[test]
fn stash_error_invalid_format_carries_message() {
    let err = StashError::InvalidFormat("bad magic bytes".to_string());
    let msg = err.to_string();
    assert!(msg.contains("bad magic bytes"));
}

#[test]
fn stash_error_unsupported_version_carries_version() {
    let err = StashError::UnsupportedVersion(42);
    let msg = err.to_string();
    assert!(msg.contains("42"));
}

#[test]
fn stash_error_io_variant_wraps_io_error() {
    let io_err = io::Error::new(ErrorKind::NotFound, "file missing");
    let stash_err = StashError::Io(io_err);
    assert!(matches!(stash_err, StashError::Io(_)));
    assert!(stash_err.to_string().contains("file missing"));
}

#[test]
fn stash_error_serialization_carries_message() {
    let err = StashError::Serialization("bincode decode failed".to_string());
    let msg = err.to_string();
    assert!(msg.contains("bincode decode failed"));
}

#[test]
fn stash_error_not_found_carries_path() {
    let path = PathBuf::from("/tmp/stash.dat");
    let err = StashError::NotFound(path.clone());
    let msg = err.to_string();
    assert!(msg.contains("/tmp/stash.dat"));
}

#[test]
fn stash_error_already_exists_carries_path() {
    let path = PathBuf::from("/tmp/stash.meta");
    let err = StashError::AlreadyExists(path.clone());
    let msg = err.to_string();
    assert!(msg.contains("/tmp/stash.meta"));
}

#[test]
fn stash_error_locked_has_display() {
    let err = StashError::Locked;
    let msg = err.to_string();
    assert!(!msg.is_empty());
}

#[test]
fn stash_error_debug_derived() {
    let err = StashError::Locked;
    let debug = format!("{:?}", err);
    assert!(debug.contains("Locked"));
}

#[test]
fn stash_error_display_no_secrets() {
    // Error messages must NOT contain key/password/plaintext bytes
    let invalid = StashError::InvalidFormat("corrupt header".to_string());
    let msg = invalid.to_string();
    assert!(!msg.contains("password"));
    assert!(!msg.contains("key"));

    let version = StashError::UnsupportedVersion(99);
    let msg = version.to_string();
    assert!(!msg.contains("password"));
}
