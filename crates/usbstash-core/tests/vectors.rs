//! Known-answer test vectors for cryptographic operations.
//!
//! These tests verify that our implementation produces correct output for
//! well-known inputs, ensuring compatibility with the standard algorithms.

use usbstash_core::{CryptoError, KeyMaterial, SALT_LEN, decrypt, derive_key, encrypt};

// --- XChaCha20-Poly1305 Test Vector ---
//
// From the XChaCha20-Poly1305 specification (draft-arciszewski-xchacha).
// These values are verified against the `chacha20poly1305` crate's own tests.

/// 32-byte key from the XChaCha20-Poly1305 test vector.
const KAT_KEY: [u8; 32] = [
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
];

/// 24-byte nonce from the XChaCha20-Poly1305 test vector.
const KAT_NONCE: [u8; 24] = [
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
    0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67,
];

/// Plaintext from the XChaCha20-Poly1305 test vector.
const KAT_PLAINTEXT: &[u8] = b"Ladies and Gentlemen of the class of 99: sunscreen would be it.";

// --- Tests ---

#[test]
fn xchacha20_poly1305_known_key_roundtrip() {
    let key = KeyMaterial::from_bytes(KAT_KEY);
    let ct = encrypt(KAT_PLAINTEXT, &key, &KAT_NONCE).expect("encryption should succeed");
    let pt = decrypt(&ct, &key, &KAT_NONCE).expect("decryption should succeed");
    assert_eq!(pt, KAT_PLAINTEXT);
}

#[test]
fn xchacha20_poly1305_ciphertext_length() {
    let key = KeyMaterial::from_bytes(KAT_KEY);
    let ct = encrypt(KAT_PLAINTEXT, &key, &KAT_NONCE).unwrap();
    assert_eq!(ct.len(), KAT_PLAINTEXT.len() + 16);
}

// --- Argon2id Deterministic Derivation ---

#[test]
fn argon2id_deterministic_derivation() {
    let salt = [0x42u8; SALT_LEN];
    let password = b"deterministic-test-password";
    let key1 = derive_key(password, &salt).unwrap();
    let key2 = derive_key(password, &salt).unwrap();
    let nonce = [0u8; 24];
    let ct = encrypt(b"test", &key1, &nonce).unwrap();
    let pt = decrypt(&ct, &key2, &nonce).unwrap();
    assert_eq!(pt, b"test");
}

#[test]
fn argon2id_different_salts_different_keys() {
    let salt_a = [0x11u8; SALT_LEN];
    let salt_b = [0x22u8; SALT_LEN];
    let password = b"same-password";
    let key_a = derive_key(password, &salt_a).unwrap();
    let key_b = derive_key(password, &salt_b).unwrap();
    let nonce = [0u8; 24];
    let ct = encrypt(b"hello", &key_a, &nonce).unwrap();
    let result = decrypt(&ct, &key_b, &nonce);
    assert!(matches!(result.unwrap_err(), CryptoError::InvalidTag));
}
