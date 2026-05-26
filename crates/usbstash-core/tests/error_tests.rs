//! Tests for the `CryptoError` enum.

use usbstash_core::CryptoError;

#[test]
fn error_kdf_variant_exists() {
    let err = CryptoError::KdfError("invalid parameters".to_string());
    assert!(matches!(err, CryptoError::KdfError(_)));
}

#[test]
fn error_encrypt_variant_exists() {
    let err = CryptoError::EncryptError("cipher failure".to_string());
    assert!(matches!(err, CryptoError::EncryptError(_)));
}

#[test]
fn error_decrypt_variant_exists() {
    let err = CryptoError::DecryptError("decryption failed".to_string());
    assert!(matches!(err, CryptoError::DecryptError(_)));
}

#[test]
fn error_invalid_tag_variant_exists() {
    let err = CryptoError::InvalidTag;
    assert!(matches!(err, CryptoError::InvalidTag));
}

#[test]
fn error_rng_variant_exists() {
    let err = CryptoError::RngError("os rng unavailable".to_string());
    assert!(matches!(err, CryptoError::RngError(_)));
}

#[test]
fn error_display_no_secrets() {
    // Error messages must NOT contain key/password/plaintext bytes
    let kdf = CryptoError::KdfError("bad params".to_string());
    let msg = kdf.to_string();
    assert!(msg.contains("key derivation failed"));
    assert!(!msg.contains("secret"));

    let encrypt = CryptoError::EncryptError("oops".to_string());
    assert!(encrypt.to_string().contains("encryption failed"));

    let decrypt = CryptoError::DecryptError("oops".to_string());
    assert!(decrypt.to_string().contains("decryption failed"));

    let tag = CryptoError::InvalidTag;
    assert!(tag.to_string().contains("authentication"));
    assert!(!tag.to_string().contains("key"));
    assert!(!tag.to_string().contains("password"));

    let rng = CryptoError::RngError("no entropy".to_string());
    assert!(rng.to_string().contains("random number generation"));
}

#[test]
fn error_debug_derived() {
    let err = CryptoError::InvalidTag;
    let debug = format!("{:?}", err);
    assert!(debug.contains("InvalidTag"));
}
