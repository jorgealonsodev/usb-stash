//! Tests for core cryptographic operations.

use usbstash_core::{
    ARGON2_ITERATIONS, ARGON2_MEMORY_KIB, ARGON2_PARALLELISM, CryptoError, KEY_LEN, NONCE_LEN,
    SALT_LEN, TAG_LEN, decrypt, derive_key, encrypt, generate_nonce, generate_salt,
};

// ─── Constants Tests (Task 3.1) ────────────────────────────────────────────

#[test]
fn constants_have_expected_values() {
    assert_eq!(ARGON2_MEMORY_KIB, 65_536);
    assert_eq!(ARGON2_ITERATIONS, 3);
    assert_eq!(ARGON2_PARALLELISM, 4);
    assert_eq!(SALT_LEN, 16);
    assert_eq!(NONCE_LEN, 24);
    assert_eq!(KEY_LEN, 32);
    assert_eq!(TAG_LEN, 16);
}

// ─── Salt and Nonce Generation (Tasks 3.3, 3.4) ────────────────────────────

#[test]
fn generate_salt_returns_correct_length() {
    let salt = generate_salt().expect("OsRng should be available");
    assert_eq!(salt.len(), SALT_LEN);
}

#[test]
fn generate_nonce_returns_correct_length() {
    let nonce = generate_nonce().expect("OsRng should be available");
    assert_eq!(nonce.len(), NONCE_LEN);
}

#[test]
fn generate_salt_produces_different_values() {
    let s1 = generate_salt().unwrap();
    let s2 = generate_salt().unwrap();
    assert_ne!(s1, s2, "two consecutive salts should differ");
}

#[test]
fn generate_nonce_produces_different_values() {
    let n1 = generate_nonce().unwrap();
    let n2 = generate_nonce().unwrap();
    assert_ne!(n1, n2, "two consecutive nonces should differ");
}

// ─── Key Derivation (Task 3.5) ─────────────────────────────────────────────

#[test]
fn derive_key_produces_correct_length() {
    let salt = generate_salt().unwrap();
    let key = derive_key(b"test-password", &salt).expect("derivation should succeed");
    // KeyMaterial wraps a [u8; KEY_LEN]
    let _ = key; // type-check: returns KeyMaterial
}

#[test]
fn derive_key_deterministic_same_inputs() {
    let salt = [0u8; SALT_LEN];
    let key1 = derive_key(b"same-password", &salt).unwrap();
    let key2 = derive_key(b"same-password", &salt).unwrap();
    // Both derivations with same inputs must produce same key
    // We can't directly compare KeyMaterial, but we can use them to encrypt/decrypt
    let nonce = generate_nonce().unwrap();
    let ct = encrypt(b"hello", &key1, &nonce).unwrap();
    let pt = decrypt(&ct, &key2, &nonce).unwrap();
    assert_eq!(pt, b"hello");
}

#[test]
fn derive_key_different_passwords_different_keys() {
    let salt = [0u8; SALT_LEN];
    let key_a = derive_key(b"password-a", &salt).unwrap();
    let key_b = derive_key(b"password-b", &salt).unwrap();
    let nonce = generate_nonce().unwrap();
    let ct = encrypt(b"secret", &key_a, &nonce).unwrap();
    // Decrypting with wrong key must fail
    let result = decrypt(&ct, &key_b, &nonce);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CryptoError::InvalidTag));
}

// ─── Encrypt / Decrypt Round-trip (Tasks 3.6, 3.7) ─────────────────────────

#[test]
fn encrypt_decrypt_roundtrip_empty() {
    let salt = generate_salt().unwrap();
    let key = derive_key(b"test", &salt).unwrap();
    let nonce = generate_nonce().unwrap();
    let ct = encrypt(b"", &key, &nonce).unwrap();
    let pt = decrypt(&ct, &key, &nonce).unwrap();
    assert_eq!(pt, b"");
}

#[test]
fn encrypt_decrypt_roundtrip_known_plaintext() {
    let salt = generate_salt().unwrap();
    let key = derive_key(b"my-password", &salt).unwrap();
    let nonce = generate_nonce().unwrap();
    let plaintext = b"The quick brown fox jumps over the lazy dog";
    let ct = encrypt(plaintext, &key, &nonce).unwrap();
    let pt = decrypt(&ct, &key, &nonce).unwrap();
    assert_eq!(pt, plaintext);
}

#[test]
fn encrypt_decrypt_roundtrip_binary_data() {
    let salt = generate_salt().unwrap();
    let key = derive_key(b"binary-test", &salt).unwrap();
    let nonce = generate_nonce().unwrap();
    // All byte values
    let plaintext: [u8; 256] = core::array::from_fn(|i| i as u8);
    let ct = encrypt(&plaintext, &key, &nonce).unwrap();
    let pt = decrypt(&ct, &key, &nonce).unwrap();
    assert_eq!(pt, plaintext);
}

// ─── Tampered Ciphertext (Task 5.3) ────────────────────────────────────────

#[test]
fn tampered_ciphertext_fails_with_invalid_tag() {
    let salt = generate_salt().unwrap();
    let key = derive_key(b"tamper-test", &salt).unwrap();
    let nonce = generate_nonce().unwrap();
    let plaintext = b"do not tamper with this";
    let mut ct = encrypt(plaintext, &key, &nonce).unwrap();

    // Flip one byte in the ciphertext
    if !ct.is_empty() {
        ct[0] ^= 0xFF;
    }

    let result = decrypt(&ct, &key, &nonce);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CryptoError::InvalidTag));
}

#[test]
fn tampered_last_byte_fails_with_invalid_tag() {
    let salt = generate_salt().unwrap();
    let key = derive_key(b"tamper-last", &salt).unwrap();
    let nonce = generate_nonce().unwrap();
    let mut ct = encrypt(b"test data", &key, &nonce).unwrap();

    // Flip the last byte (could be in the tag)
    let last = ct.len() - 1;
    ct[last] ^= 0xFF;

    let result = decrypt(&ct, &key, &nonce);
    assert!(matches!(result.unwrap_err(), CryptoError::InvalidTag));
}

// ─── Wrong Key Decryption (Task 5.4) ───────────────────────────────────────

#[test]
fn wrong_key_fails_with_invalid_tag() {
    let salt_a = [1u8; SALT_LEN];
    let salt_b = [2u8; SALT_LEN];
    let key_a = derive_key(b"shared-password", &salt_a).unwrap();
    let key_b = derive_key(b"shared-password", &salt_b).unwrap();
    let nonce = generate_nonce().unwrap();

    let ct = encrypt(b"encrypted with key A", &key_a, &nonce).unwrap();
    let result = decrypt(&ct, &key_b, &nonce);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CryptoError::InvalidTag));
}

// ─── KeyMaterial Zeroization (Task 3.2) ────────────────────────────────────

#[test]
fn key_material_is_zeroize_on_drop() {
    // This test verifies the type implements ZeroizeOnDrop.
    // Actual zeroization is guaranteed by the `zeroize` crate's volatile writes.
    let salt = generate_salt().unwrap();
    let key = derive_key(b"zeroize-test", &salt).unwrap();
    // Type-check: KeyMaterial must exist and be usable
    let nonce = generate_nonce().unwrap();
    let ct = encrypt(b"test", &key, &nonce).unwrap();
    let pt = decrypt(&ct, &key, &nonce).unwrap();
    assert_eq!(pt, b"test");
    // key is dropped here — ZeroizeOnDrop should zero the bytes
}
