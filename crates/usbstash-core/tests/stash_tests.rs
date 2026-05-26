//! Tests for the Stash API — data structures (tasks 2.1-2.3) and lifecycle (tasks 2.4-2.9).

use std::io::Cursor;

use tempfile::TempDir;
use usbstash_core::StashError;
use usbstash_core::format::{Footer, HEADER_SIZE};
use usbstash_core::stash::{Stash, StashEntry, StashMeta, StashPayload};

// ─── StashEntry (Task 2.1) ─────────────────────────────────────────────────

#[test]
fn stash_entry_has_expected_fields() {
    let id = uuid::Uuid::new_v4();
    let entry = StashEntry::new(
        id,
        "secret.txt".to_string(),
        1_700_000_000,
        1_700_000_001,
        5,
        "text/plain".to_string(),
        b"hello".to_vec(),
    );

    assert_eq!(entry.id(), id);
    assert_eq!(entry.path(), "secret.txt");
    assert_eq!(entry.created_at(), 1_700_000_000);
    assert_eq!(entry.modified_at(), 1_700_000_001);
    assert_eq!(entry.size(), 5);
    assert_eq!(entry.mime_type(), "text/plain");
    assert_eq!(entry.content(), b"hello");
}

#[test]
fn stash_entry_empty_content() {
    let id = uuid::Uuid::new_v4();
    let entry = StashEntry::new(
        id,
        "empty.txt".to_string(),
        1_700_000_000,
        1_700_000_000,
        0,
        "text/plain".to_string(),
        vec![],
    );

    assert_eq!(entry.size(), 0);
    assert!(entry.content().is_empty());
}

#[test]
fn stash_entry_binary_content() {
    let id = uuid::Uuid::new_v4();
    let content: Vec<u8> = (0..256).map(|i| i as u8).collect();
    let entry = StashEntry::new(
        id,
        "binary.bin".to_string(),
        1_700_000_000,
        1_700_000_000,
        256,
        "application/octet-stream".to_string(),
        content.clone(),
    );

    assert_eq!(entry.content(), &content);
    assert_eq!(entry.size(), 256);
}

// ─── StashPayload (Task 2.2) ───────────────────────────────────────────────

#[test]
fn stash_payload_empty_entries() {
    let payload = StashPayload::new(1, vec![]);
    assert_eq!(payload.version(), 1);
    assert!(payload.entries().is_empty());
}

#[test]
fn stash_payload_with_entries() {
    let id = uuid::Uuid::new_v4();
    let entry = StashEntry::new(
        id,
        "test.txt".to_string(),
        1_700_000_000,
        1_700_000_000,
        4,
        "text/plain".to_string(),
        b"test".to_vec(),
    );
    let payload = StashPayload::new(1, vec![entry]);
    assert_eq!(payload.version(), 1);
    assert_eq!(payload.entries().len(), 1);
    assert_eq!(payload.entries()[0].path(), "test.txt");
}

#[test]
fn stash_payload_bincode_roundtrip() {
    let id = uuid::Uuid::new_v4();
    let entry = StashEntry::new(
        id,
        "roundtrip.txt".to_string(),
        1_700_000_000,
        1_700_000_001,
        9,
        "text/plain".to_string(),
        b"roundtrip".to_vec(),
    );
    let payload = StashPayload::new(1, vec![entry]);

    let config = bincode::config::standard();
    let encoded = bincode::serde::encode_to_vec(&payload, config).unwrap();

    let (decoded, _len): (StashPayload, usize) =
        bincode::serde::decode_from_slice(&encoded, config).unwrap();

    assert_eq!(decoded.version(), payload.version());
    assert_eq!(decoded.entries().len(), payload.entries().len());
    assert_eq!(decoded.entries()[0].path(), payload.entries()[0].path());
    assert_eq!(
        decoded.entries()[0].content(),
        payload.entries()[0].content()
    );
}

// ─── StashMeta (Task 2.3) ──────────────────────────────────────────────────

#[test]
fn stash_meta_json_roundtrip() {
    let meta = StashMeta::new(
        1,
        "usbstash-v1".to_string(),
        vec![0xAB; 16],
        65_536,
        3,
        4,
        "XChaCha20-Poly1305".to_string(),
        24,
        32,
        16,
        1_700_000_000,
        0,
        None,
        None,
    );

    let json = serde_json::to_string(&meta).unwrap();
    let decoded: StashMeta = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.version(), 1);
    assert_eq!(decoded.format(), "usbstash-v1");
    assert_eq!(decoded.salt(), vec![0xAB; 16]);
    assert_eq!(decoded.failed_attempts(), 0);
    assert!(decoded.last_attempt_at().is_none());
}

#[test]
fn stash_meta_with_failed_attempts() {
    let meta = StashMeta::new(
        1,
        "usbstash-v1".to_string(),
        vec![0xCD; 16],
        65_536,
        3,
        4,
        "XChaCha20-Poly1305".to_string(),
        24,
        32,
        16,
        1_700_000_000,
        3,
        Some(1_700_000_100),
        None,
    );

    let json = serde_json::to_string(&meta).unwrap();
    let decoded: StashMeta = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.failed_attempts(), 3);
    assert_eq!(decoded.last_attempt_at(), Some(1_700_000_100));
}

#[test]
fn stash_meta_contains_no_secrets() {
    let meta = StashMeta::new(
        1,
        "usbstash-v1".to_string(),
        vec![0x01; 16],
        65_536,
        3,
        4,
        "XChaCha20-Poly1305".to_string(),
        24,
        32,
        16,
        1_700_000_000,
        0,
        None,
        None,
    );

    let json = serde_json::to_string(&meta).unwrap();
    // JSON must not contain actual secret values (passwords, key bytes)
    // Field names like "aead_key_len" are fine — they're config params, not secrets
    assert!(!json.contains("password"));
    assert!(!json.contains("secret"));
    // Salt is stored as base64, not raw bytes — verify it's encoded
    assert!(json.contains("salt"));
}

// ─── Stash::create (Task 2.4) ──────────────────────────────────────────────

#[test]
fn stash_create_produces_dat_and_meta_files() {
    let tmp = TempDir::new().unwrap();
    let stash = Stash::create(b"test-password", tmp.path()).unwrap();

    let dat = tmp.path().join("stash.dat");
    let meta = tmp.path().join("stash.meta");

    assert!(dat.exists(), "stash.dat must exist");
    assert!(meta.exists(), "stash.meta must exist");

    // Verify .dat starts with STSH magic
    let dat_bytes = std::fs::read(&dat).unwrap();
    assert_eq!(&dat_bytes[0..4], b"STSH");

    // Verify meta is valid JSON with expected format
    let meta_json = std::fs::read_to_string(&meta).unwrap();
    let meta: StashMeta = serde_json::from_str(&meta_json).unwrap();
    assert_eq!(meta.format(), "usbstash-v1");
    assert_eq!(meta.failed_attempts(), 0);

    // Stash is not locked
    assert!(!stash.is_locked());
}

#[test]
fn stash_create_rejects_if_already_exists() {
    let tmp = TempDir::new().unwrap();

    // Create first stash
    Stash::create(b"password1", tmp.path()).unwrap();

    // Second create at same path must fail
    let result = Stash::create(b"password2", tmp.path());
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        StashError::AlreadyExists(_)
    ));
}

#[test]
fn stash_create_empty_stash_has_no_entries() {
    let tmp = TempDir::new().unwrap();
    let stash = Stash::create(b"password", tmp.path()).unwrap();

    let entries = stash.list_entries().unwrap();
    assert!(entries.is_empty());
}

// ─── Stash::open (Task 2.5) ────────────────────────────────────────────────

#[test]
fn stash_open_with_correct_password() {
    let tmp = TempDir::new().unwrap();
    Stash::create(b"correct-pw", tmp.path()).unwrap();

    let stash = Stash::open(b"correct-pw", tmp.path()).unwrap();
    assert!(!stash.is_locked());
    assert!(stash.list_entries().unwrap().is_empty());
}

#[test]
fn stash_open_wrong_password_returns_crypto_error() {
    let tmp = TempDir::new().unwrap();
    Stash::create(b"correct-pw", tmp.path()).unwrap();

    let result = Stash::open(b"wrong-pw", tmp.path());
    assert!(result.is_err());
    // Wrong password causes decryption failure (InvalidTag)
    assert!(matches!(result.err().unwrap(), StashError::Crypto(_)));
}

#[test]
fn stash_open_increments_failed_attempts() {
    let tmp = TempDir::new().unwrap();
    Stash::create(b"correct-pw", tmp.path()).unwrap();

    // Wrong password
    let _ = Stash::open(b"wrong-1", tmp.path());

    // Read meta to check failed_attempts
    let meta_json = std::fs::read_to_string(tmp.path().join("stash.meta")).unwrap();
    let meta: StashMeta = serde_json::from_str(&meta_json).unwrap();
    assert_eq!(meta.failed_attempts(), 1);
    assert!(meta.last_attempt_at().is_some());
}

#[test]
fn stash_open_successful_resets_failed_attempts() {
    let tmp = TempDir::new().unwrap();
    Stash::create(b"correct-pw", tmp.path()).unwrap();

    // Fail once
    let _ = Stash::open(b"wrong", tmp.path());

    // Then succeed
    let _stash = Stash::open(b"correct-pw", tmp.path()).unwrap();

    let meta_json = std::fs::read_to_string(tmp.path().join("stash.meta")).unwrap();
    let meta: StashMeta = serde_json::from_str(&meta_json).unwrap();
    assert_eq!(meta.failed_attempts(), 0);
}

#[test]
fn stash_open_missing_files_returns_not_found() {
    let tmp = TempDir::new().unwrap();
    let result = Stash::open(b"any-password", tmp.path());
    assert!(result.is_err());
    assert!(matches!(result.err().unwrap(), StashError::NotFound(_)));
}

// ─── Stash::save and round-trip (Task 2.6) ─────────────────────────────────

#[test]
fn stash_save_and_reopen_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"roundtrip-pw", tmp.path()).unwrap();

    // Add entries
    stash
        .add_entry("file1.txt".to_string(), b"hello world".to_vec())
        .unwrap();
    stash
        .add_entry("file2.txt".to_string(), b"goodbye world".to_vec())
        .unwrap();

    // Save
    stash.save().unwrap();

    // Drop (auto-lock)
    drop(stash);

    // Reopen
    let stash2 = Stash::open(b"roundtrip-pw", tmp.path()).unwrap();
    let entries = stash2.list_entries().unwrap();
    assert_eq!(entries.len(), 2);

    let e1 = stash2.get_entry("file1.txt").unwrap();
    assert_eq!(e1.content(), b"hello world");

    let e2 = stash2.get_entry("file2.txt").unwrap();
    assert_eq!(e2.content(), b"goodbye world");
}

#[test]
fn stash_save_empty_stash_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let stash = Stash::create(b"empty-pw", tmp.path()).unwrap();
    stash.save().unwrap();
    drop(stash);

    let stash2 = Stash::open(b"empty-pw", tmp.path()).unwrap();
    assert!(stash2.list_entries().unwrap().is_empty());
}

// ─── Stash::lock (Task 2.7) ────────────────────────────────────────────────

#[test]
fn stash_lock_prevents_entry_access() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"lock-pw", tmp.path()).unwrap();
    stash
        .add_entry("secret.txt".to_string(), b"secret".to_vec())
        .unwrap();

    stash.lock();
    assert!(stash.is_locked());

    let result = stash.get_entry("secret.txt");
    assert!(matches!(result.err().unwrap(), StashError::Locked));
}

#[test]
fn stash_lock_prevents_save() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"lock-pw", tmp.path()).unwrap();
    stash
        .add_entry("test.txt".to_string(), b"data".to_vec())
        .unwrap();

    stash.lock();
    let result = stash.save();
    assert!(matches!(result.err().unwrap(), StashError::Locked));
}

#[test]
fn stash_lock_prevents_add_entry() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"lock-pw", tmp.path()).unwrap();

    stash.lock();
    let result = stash.add_entry("test.txt".to_string(), b"data".to_vec());
    assert!(matches!(result.err().unwrap(), StashError::Locked));
}

#[test]
fn stash_lock_prevents_list_entries() {
    let tmp = TempDir::new().unwrap();
    let stash = Stash::create(b"lock-pw", tmp.path()).unwrap();

    let mut stash = stash;
    stash.lock();
    let result = stash.list_entries();
    assert!(matches!(result.err().unwrap(), StashError::Locked));
}

// ─── Stash entry management (Task 2.8) ─────────────────────────────────────

#[test]
fn stash_add_and_get_entry() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"entry-pw", tmp.path()).unwrap();

    let id = stash
        .add_entry("notes.txt".to_string(), b"my notes".to_vec())
        .unwrap();
    assert!(!id.to_string().is_empty());

    let entry = stash.get_entry("notes.txt").unwrap();
    assert_eq!(entry.path(), "notes.txt");
    assert_eq!(entry.content(), b"my notes");
    assert_eq!(entry.size(), 8);
}

#[test]
fn stash_remove_entry() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"remove-pw", tmp.path()).unwrap();

    stash
        .add_entry("keep.txt".to_string(), b"keep".to_vec())
        .unwrap();
    stash
        .add_entry("remove.txt".to_string(), b"remove".to_vec())
        .unwrap();

    let removed = stash.remove_entry("remove.txt").unwrap();
    assert!(removed);

    let entries = stash.list_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path(), "keep.txt");

    // Removing non-existent entry returns false
    let removed = stash.remove_entry("nonexistent.txt").unwrap();
    assert!(!removed);
}

#[test]
fn stash_list_entries_returns_metadata() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"list-pw", tmp.path()).unwrap();

    stash
        .add_entry("a.txt".to_string(), b"aaa".to_vec())
        .unwrap();
    stash
        .add_entry("b.txt".to_string(), b"bbbb".to_vec())
        .unwrap();
    stash
        .add_entry("c.txt".to_string(), b"ccccc".to_vec())
        .unwrap();

    let entries = stash.list_entries().unwrap();
    assert_eq!(entries.len(), 3);

    // Entries contain metadata
    let paths: Vec<&str> = entries.iter().map(|e| e.path()).collect();
    assert!(paths.contains(&"a.txt"));
    assert!(paths.contains(&"b.txt"));
    assert!(paths.contains(&"c.txt"));
}

#[test]
fn stash_entry_mime_type_guessing() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"mime-pw", tmp.path()).unwrap();

    stash
        .add_entry("doc.txt".to_string(), b"text".to_vec())
        .unwrap();
    stash
        .add_entry("data.json".to_string(), b"{}".to_vec())
        .unwrap();
    stash
        .add_entry("image.png".to_string(), b"\x89PNG".to_vec())
        .unwrap();
    stash
        .add_entry("unknown".to_string(), b"??".to_vec())
        .unwrap();

    assert_eq!(
        stash.get_entry("doc.txt").unwrap().mime_type(),
        "text/plain"
    );
    assert_eq!(
        stash.get_entry("data.json").unwrap().mime_type(),
        "application/json"
    );
    assert_eq!(
        stash.get_entry("image.png").unwrap().mime_type(),
        "image/png"
    );
    assert_eq!(
        stash.get_entry("unknown").unwrap().mime_type(),
        "application/octet-stream"
    );
}

// ─── Stash::Drop auto-lock (Task 2.9) ──────────────────────────────────────

#[test]
fn stash_drop_auto_locks() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"drop-pw", tmp.path()).unwrap();
    stash
        .add_entry("secret.txt".to_string(), b"secret".to_vec())
        .unwrap();

    // stash is dropped here — auto-lock happens in Drop
    drop(stash);

    // Reopening should show empty stash (entry was never saved)
    let stash2 = Stash::open(b"drop-pw", tmp.path()).unwrap();
    assert!(stash2.list_entries().unwrap().is_empty());
}

#[test]
fn stash_unsaved_changes_lost_on_drop() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"unsaved-pw", tmp.path()).unwrap();
    stash
        .add_entry("unsaved.txt".to_string(), b"lost".to_vec())
        .unwrap();

    // Drop WITHOUT save
    drop(stash);

    // Reopen — entry should NOT be present
    let stash2 = Stash::open(b"unsaved-pw", tmp.path()).unwrap();
    let entries = stash2.list_entries().unwrap();
    assert!(entries.is_empty(), "unsaved changes should be lost");
}

// ─── Integration: Full lifecycle (Task 3.2-3.4) ────────────────────────────

#[test]
fn full_lifecycle_create_add_save_open_verify() {
    let tmp = TempDir::new().unwrap();

    // Create
    let mut stash = Stash::create(b"master-pw", tmp.path()).unwrap();

    // Add multiple entries
    stash
        .add_entry("passwords.txt".to_string(), b"admin:password123".to_vec())
        .unwrap();
    stash
        .add_entry("api_key.txt".to_string(), b"sk-1234567890".to_vec())
        .unwrap();
    stash
        .add_entry("notes.md".to_string(), b"# Secrets\nDo not share.".to_vec())
        .unwrap();

    // Save
    stash.save().unwrap();
    drop(stash);

    // Open
    let stash2 = Stash::open(b"master-pw", tmp.path()).unwrap();

    // Verify all entries
    let entries = stash2.list_entries().unwrap();
    assert_eq!(entries.len(), 3);

    assert_eq!(
        stash2.get_entry("passwords.txt").unwrap().content(),
        b"admin:password123"
    );
    assert_eq!(
        stash2.get_entry("api_key.txt").unwrap().content(),
        b"sk-1234567890"
    );
    assert_eq!(
        stash2.get_entry("notes.md").unwrap().content(),
        b"# Secrets\nDo not share."
    );
}

#[test]
fn tampered_dat_file_detected_on_open() {
    let tmp = TempDir::new().unwrap();
    Stash::create(b"tamper-pw", tmp.path()).unwrap();

    // Tamper with the .dat file — flip a byte in the ciphertext region
    let dat_path = tmp.path().join("stash.dat");
    let mut dat_bytes = std::fs::read(&dat_path).unwrap();
    // Flip a byte after the header (in the ciphertext area)
    if dat_bytes.len() > HEADER_SIZE + 1 {
        dat_bytes[HEADER_SIZE + 1] ^= 0xFF;
    }
    std::fs::write(&dat_path, &dat_bytes).unwrap();

    // Open should fail with crypto error (InvalidTag)
    let result = Stash::open(b"tamper-pw", tmp.path());
    assert!(result.is_err());
    assert!(matches!(result.err().unwrap(), StashError::Crypto(_)));
}

#[test]
fn multiple_save_cycles_preserve_data() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"multi-pw", tmp.path()).unwrap();

    // First save cycle
    stash
        .add_entry("first.txt".to_string(), b"first".to_vec())
        .unwrap();
    stash.save().unwrap();
    drop(stash);

    // Second save cycle — add more
    let mut stash = Stash::open(b"multi-pw", tmp.path()).unwrap();
    stash
        .add_entry("second.txt".to_string(), b"second".to_vec())
        .unwrap();
    stash.save().unwrap();
    drop(stash);

    // Verify both entries
    let stash = Stash::open(b"multi-pw", tmp.path()).unwrap();
    let entries = stash.list_entries().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(stash.get_entry("first.txt").unwrap().content(), b"first");
    assert_eq!(stash.get_entry("second.txt").unwrap().content(), b"second");
}

#[test]
fn stash_get_entry_not_found() {
    let tmp = TempDir::new().unwrap();
    let stash = Stash::create(b"notfound-pw", tmp.path()).unwrap();

    let result = stash.get_entry("nonexistent.txt");
    assert!(result.is_err());
    assert!(matches!(result.err().unwrap(), StashError::NotFound(_)));
}

#[test]
fn stash_dat_file_has_valid_footer() {
    let tmp = TempDir::new().unwrap();
    let mut stash = Stash::create(b"footer-pw", tmp.path()).unwrap();
    stash
        .add_entry("test.txt".to_string(), b"test data here".to_vec())
        .unwrap();
    stash.save().unwrap();
    drop(stash);

    let dat_bytes = std::fs::read(tmp.path().join("stash.dat")).unwrap();
    let data_len = dat_bytes.len();

    // Read footer from end of file
    let footer_start = data_len - 16; // Footer is 16 bytes
    let mut cursor = Cursor::new(&dat_bytes[footer_start..]);
    let footer = Footer::read(&mut cursor).unwrap();

    assert_eq!(footer.magic(), Footer::MAGIC);
    assert_eq!(footer.chunk_count(), 1);
}
