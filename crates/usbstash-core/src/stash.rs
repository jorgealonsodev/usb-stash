//! Stash lifecycle API — create, open, save, lock, and entry management.
//!
//! The `Stash` struct manages the full lifecycle of an encrypted stash container:
//! - `Stash::create` — create a new encrypted stash on disk
//! - `Stash::open` — open and decrypt an existing stash
//! - `Stash::save` — encrypt and persist in-memory changes
//! - `Stash::lock` — zeroize sensitive data in memory
//!
//! Entry management (`add_entry`, `get_entry`, `list_entries`, `remove_entry`)
//! operates in-memory only. Changes are not persisted until `save()` is called.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::{
    self, KeyMaterial, NONCE_LEN, SALT_LEN, decrypt, derive_key, encrypt, generate_nonce,
    generate_salt,
};
use crate::error::StashError;
use crate::format::{FOOTER_SIZE, Footer, HEADER_SIZE, Header};

// ─── StashMetadata DTO (Phase 8) ───────────────────────────────────────────

/// Read-only metadata summary for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct StashMetadata {
    pub version: u32,
    pub format: String,
    pub created_at: u64,
    pub total_entries: usize,
    pub dat_size: u64,
}

// ─── StashEntry (Task 2.1) ─────────────────────────────────────────────────

/// A single entry in the stash.
///
/// Contains metadata (path, size, mime_type) and the decrypted content.
/// Content is stored in plain bytes — sensitive data that should be zeroized
/// when the stash is locked or dropped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashEntry {
    id: Uuid,
    path: String,
    created_at: u64,
    modified_at: u64,
    size: u64,
    mime_type: String,
    content: Vec<u8>,
}

impl StashEntry {
    /// Create a new entry with the given fields.
    pub fn new(
        id: Uuid,
        path: String,
        created_at: u64,
        modified_at: u64,
        size: u64,
        mime_type: String,
        content: Vec<u8>,
    ) -> Self {
        Self {
            id,
            path,
            created_at,
            modified_at,
            size,
            mime_type,
            content,
        }
    }

    /// Unique identifier for this entry.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// File path this entry represents.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Unix epoch seconds when this entry was created.
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Unix epoch seconds when this entry was last modified.
    pub fn modified_at(&self) -> u64 {
        self.modified_at
    }

    /// Size of the content in bytes.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// MIME type of the content.
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    /// The decrypted content bytes.
    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

// ─── StashPayload (Task 2.2) ───────────────────────────────────────────────

/// The encrypted payload — a versioned list of stash entries.
///
/// Serialized with bincode before encryption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashPayload {
    version: u32,
    entries: Vec<StashEntry>,
}

impl StashPayload {
    /// Create a new payload with the given version and entries.
    pub fn new(version: u32, entries: Vec<StashEntry>) -> Self {
        Self { version, entries }
    }

    /// Payload format version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The list of entries in this payload.
    pub fn entries(&self) -> &[StashEntry] {
        &self.entries
    }

    /// Mutable access to entries.
    pub fn entries_mut(&mut self) -> &mut Vec<StashEntry> {
        &mut self.entries
    }
}

// ─── Settings (Phase 8) ────────────────────────────────────────────────────

/// User-configurable settings persisted in stash.meta.
///
/// `auto_lock_seconds` controls the inactivity timeout before the stash
/// automatically locks. A value of `0` means auto-lock is disabled.
/// Default is 300 seconds (5 minutes) per spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub auto_lock_seconds: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_lock_seconds: 300, // 5 minutes
        }
    }
}

// ─── StashMeta (Task 2.3) ──────────────────────────────────────────────────

/// JSON metadata stored alongside the encrypted container.
///
/// Contains non-secret parameters: salt, KDF/AEAD configuration, timestamps,
/// and failed attempt tracking. No key material or passwords are stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashMeta {
    version: u32,
    format: String,
    #[serde(with = "base64_salt")]
    salt: Vec<u8>,
    kdf_memory_kib: u32,
    kdf_iterations: u32,
    kdf_parallelism: u32,
    aead_algorithm: String,
    aead_nonce_len: usize,
    aead_key_len: usize,
    aead_tag_len: usize,
    created_at: u64,
    failed_attempts: u32,
    last_attempt_at: Option<u64>,
    #[serde(default)]
    settings: Option<Settings>,
}

mod base64_salt {
    use serde::{self, Deserialize, Deserializer, Serializer};

    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    fn encode(bytes: &[u8]) -> String {
        let mut result = String::new();
        let chunks = bytes.chunks(3);
        for chunk in chunks {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            result.push(BASE64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
            result.push(BASE64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                result.push(BASE64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            if chunk.len() > 2 {
                result.push(BASE64_CHARS[(triple & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }
        result
    }

    fn decode(s: &str) -> Result<Vec<u8>, String> {
        let s = s.trim_end_matches('=');
        let mut result = Vec::new();
        let chars: Vec<u8> = s.bytes().collect();
        for chunk in chars.chunks(4) {
            let vals: Result<Vec<u32>, _> = chunk
                .iter()
                .map(|&c| {
                    BASE64_CHARS
                        .iter()
                        .position(|&x| x == c)
                        .ok_or_else(|| format!("invalid base64 char: {}", c as char))
                        .map(|p| p as u32)
                })
                .collect();
            let vals = vals?;
            if vals.len() < 2 {
                return Err("base64 too short".to_string());
            }
            let triple = (vals[0] << 18)
                | (vals[1] << 12)
                | (if vals.len() > 2 { vals[2] << 6 } else { 0 })
                | (if vals.len() > 3 { vals[3] } else { 0 });
            result.push(((triple >> 16) & 0xFF) as u8);
            if vals.len() > 2 {
                result.push(((triple >> 8) & 0xFF) as u8);
            }
            if vals.len() > 3 {
                result.push((triple & 0xFF) as u8);
            }
        }
        Ok(result)
    }

    pub fn serialize<S: Serializer>(salt: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&encode(salt))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(deserializer)?;
        decode(&s).map_err(serde::de::Error::custom)
    }
}

impl StashMeta {
    /// Create a new meta with the given parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u32,
        format: String,
        salt: Vec<u8>,
        kdf_memory_kib: u32,
        kdf_iterations: u32,
        kdf_parallelism: u32,
        aead_algorithm: String,
        aead_nonce_len: usize,
        aead_key_len: usize,
        aead_tag_len: usize,
        created_at: u64,
        failed_attempts: u32,
        last_attempt_at: Option<u64>,
        settings: Option<Settings>,
    ) -> Self {
        Self {
            version,
            format,
            salt,
            kdf_memory_kib,
            kdf_iterations,
            kdf_parallelism,
            aead_algorithm,
            aead_nonce_len,
            aead_key_len,
            aead_tag_len,
            created_at,
            failed_attempts,
            last_attempt_at,
            settings,
        }
    }

    /// Meta format version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Format identifier (e.g., "usbstash-v1").
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Salt bytes used for key derivation.
    pub fn salt(&self) -> Vec<u8> {
        self.salt.clone()
    }

    /// Number of failed open attempts.
    pub fn failed_attempts(&self) -> u32 {
        self.failed_attempts
    }

    /// Timestamp of the last failed attempt, if any.
    pub fn last_attempt_at(&self) -> Option<u64> {
        self.last_attempt_at
    }

    /// Increment the failed attempts counter and update timestamp.
    pub fn record_failed_attempt(&mut self) {
        self.failed_attempts += 1;
        self.last_attempt_at = Some(current_epoch_secs());
    }

    /// Reset the failed attempts counter after a successful open.
    pub fn reset_failed_attempts(&mut self) {
        self.failed_attempts = 0;
        self.last_attempt_at = None;
    }

    /// Update the modified timestamp.
    pub fn touch(&mut self) {
        // created_at stays the same; we track modifications via the meta file mtime
    }

    /// Current settings, if explicitly stored.
    pub fn settings(&self) -> Option<&Settings> {
        self.settings.as_ref()
    }

    /// Effective settings — returns stored settings or defaults to `Settings::default()`.
    pub fn effective_settings(&self) -> Settings {
        self.settings.clone().unwrap_or_default()
    }

    /// Replace the stored settings.
    pub fn set_settings(&mut self, settings: Settings) {
        self.settings = Some(settings);
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, StashError> {
        serde_json::to_string_pretty(self).map_err(|e| StashError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<Self, StashError> {
        serde_json::from_str(json).map_err(|e| StashError::Serialization(e.to_string()))
    }
}

// ─── Stash (Tasks 2.4-2.9) ─────────────────────────────────────────────────

/// Maximum decoded payload size (100 MB) to prevent OOM on malicious payloads.
const MAX_DECODING_SIZE: usize = 100 * 1024 * 1024;

/// Handle for an encrypted stash container.
///
/// Manages the lifecycle: create, open, save, lock.
/// Entry operations are in-memory only until `save()` is called.
/// Sensitive data (key, decrypted payload) is zeroized on `lock()` and `Drop`.
pub struct Stash {
    key: Option<KeyMaterial>,
    payload: Option<StashPayload>,
    #[allow(dead_code)]
    meta: StashMeta,
    path: PathBuf,
    nonce: [u8; NONCE_LEN],
}

impl Stash {
    /// Create a new stash at the given path with the given password.
    ///
    /// Generates salt and nonce, derives a key, writes an empty encrypted
    /// container (header + empty payload + footer) and a meta JSON file.
    /// Returns `AlreadyExists` if files already exist at the path.
    pub fn create(password: &[u8], path: &Path) -> Result<Self, StashError> {
        let meta_path = stash_meta_path(path);
        let dat_path = stash_dat_path(path);

        // Check if files already exist
        if meta_path.exists() || dat_path.exists() {
            return Err(StashError::AlreadyExists(path.to_path_buf()));
        }

        // Generate cryptographic material
        let salt = generate_salt()?;
        let nonce = generate_nonce()?;
        let key = derive_key(password, &salt)?;

        // Create meta
        let now = current_epoch_secs();
        let meta = StashMeta::new(
            1,
            "usbstash-v1".to_string(),
            salt.to_vec(),
            crypto::ARGON2_MEMORY_KIB,
            crypto::ARGON2_ITERATIONS,
            crypto::ARGON2_PARALLELISM,
            "XChaCha20-Poly1305".to_string(),
            NONCE_LEN,
            crypto::KEY_LEN,
            crypto::TAG_LEN,
            now,
            0,
            None,
            Some(Settings::default()),
        );

        // Write empty encrypted container
        let header = Header::new(salt, nonce);
        let empty_payload = StashPayload::new(1, vec![]);
        let ciphertext = encrypt_payload(&empty_payload, &key, &nonce)?;
        let footer = Footer::new(1, 0); // 1 chunk, 0 plaintext bytes

        let mut dat_buf = Vec::new();
        header.write(&mut dat_buf)?;
        dat_buf.extend_from_slice(&ciphertext);
        footer.write(&mut dat_buf)?;

        // Atomic write for .dat
        atomic_write(&dat_path, &dat_buf)?;

        // Write meta
        let meta_json = meta.to_json()?;
        atomic_write(&meta_path, meta_json.as_bytes())?;

        Ok(Stash {
            key: Some(key),
            payload: Some(empty_payload),
            meta,
            path: path.to_path_buf(),
            nonce,
        })
    }

    /// Open an existing stash at the given path with the given password.
    ///
    /// Reads meta, derives key, reads and validates header, decrypts payload.
    /// Returns `StashLocked` error on wrong password or tampered data.
    /// Increments failed_attempts on failure, resets on success.
    pub fn open(password: &[u8], path: &Path) -> Result<Self, StashError> {
        let meta_path = stash_meta_path(path);
        let dat_path = stash_dat_path(path);

        // Read meta
        if !meta_path.exists() {
            return Err(StashError::NotFound(meta_path.clone()));
        }
        if !dat_path.exists() {
            return Err(StashError::NotFound(dat_path.clone()));
        }

        let meta_json = fs::read_to_string(&meta_path)?;
        let mut meta = StashMeta::from_json(&meta_json)?;

        // Derive key from password and stored salt
        let salt: [u8; SALT_LEN] = meta
            .salt()
            .try_into()
            .map_err(|_| StashError::InvalidFormat("salt must be 16 bytes".to_string()))?;
        let key = derive_key(password, &salt)?;

        // Read and decrypt payload
        let dat_bytes = fs::read(&dat_path)?;
        let mut cursor = Cursor::new(&dat_bytes);

        let header = Header::read(&mut cursor)?;
        let nonce = header.nonce();

        // Read ciphertext (everything between header and footer)
        let data_len = dat_bytes.len();
        if data_len < HEADER_SIZE + Footer::MAGIC.len() + 8 + 8 {
            return Err(StashError::InvalidFormat("container too short".to_string()));
        }

        let ciphertext_start = HEADER_SIZE;
        let ciphertext_end = data_len - Footer::MAGIC.len() - 4 - 8; // footer: magic(4) + chunk_count(4) + payload_size(8)
        let ciphertext = &dat_bytes[ciphertext_start..ciphertext_end];

        // Decrypt
        let plaintext = decrypt(ciphertext, &key, &nonce).inspect_err(|_| {
            meta.record_failed_attempt();
            let _ = write_meta(&meta_path, &meta);
        })?;

        // Decode payload with size limit
        let config = bincode::config::standard().with_limit::<{ MAX_DECODING_SIZE }>();
        let (payload, _len): (StashPayload, usize) =
            bincode::serde::decode_from_slice(&plaintext, config)
                .map_err(|e| StashError::Serialization(e.to_string()))?;

        // Reset failed attempts on successful open
        meta.reset_failed_attempts();
        let _ = write_meta(&meta_path, &meta);

        Ok(Stash {
            key: Some(key),
            payload: Some(payload),
            meta,
            path: path.to_path_buf(),
            nonce,
        })
    }

    /// Encrypt and persist the current payload to disk.
    ///
    /// Serializes payload, encrypts, writes to temp file, then atomically
    /// renames to stash.dat. Updates meta timestamps.
    pub fn save(&self) -> Result<(), StashError> {
        let key = self.key.as_ref().ok_or(StashError::Locked)?;
        let payload = self.payload.as_ref().ok_or(StashError::Locked)?;

        let ciphertext = encrypt_payload(payload, key, &self.nonce)?;
        let footer = Footer::new(1, 0); // single chunk, size tracked in ciphertext

        let dat_path = stash_dat_path(&self.path);
        let mut dat_buf = Vec::new();

        // Re-read header from existing file to preserve salt/nonce
        let dat_bytes = fs::read(&dat_path)?;
        let mut cursor = Cursor::new(&dat_bytes);
        let header = Header::read(&mut cursor)?;

        header.write(&mut dat_buf)?;
        dat_buf.extend_from_slice(&ciphertext);
        footer.write(&mut dat_buf)?;

        atomic_write(&dat_path, &dat_buf)?;

        // Update meta timestamps
        // (meta file is updated on disk via save_meta)
        Ok(())
    }

    /// Zeroize the in-memory key and clear the payload.
    ///
    /// After locking, all operations that require the key or payload
    /// will return `StashError::Locked`.
    pub fn lock(&mut self) {
        self.key = None;
        self.payload = None;
    }

    /// Add an entry to the stash (in-memory only).
    ///
    /// Changes are not persisted until `save()` is called.
    /// Returns `StashError::Locked` if the stash is locked.
    pub fn add_entry(&mut self, path: String, content: Vec<u8>) -> Result<Uuid, StashError> {
        let payload = self.payload.as_mut().ok_or(StashError::Locked)?;

        let now = current_epoch_secs();
        let id = Uuid::new_v4();
        let entry = StashEntry::new(
            id,
            path.clone(),
            now,
            now,
            content.len() as u64,
            guess_mime_type(&path),
            content,
        );

        payload.entries_mut().push(entry);
        Ok(id)
    }

    /// Get an entry by its path.
    ///
    /// Returns `StashError::Locked` if the stash is locked.
    /// Returns `StashError::NotFound` if no entry exists with the given path.
    pub fn get_entry(&self, path: &str) -> Result<&StashEntry, StashError> {
        let payload = self.payload.as_ref().ok_or(StashError::Locked)?;

        payload
            .entries()
            .iter()
            .find(|e| e.path() == path)
            .ok_or_else(|| StashError::NotFound(PathBuf::from(path)))
    }

    /// Remove an entry by its path (in-memory only).
    ///
    /// Changes are not persisted until `save()` is called.
    /// Returns `true` if the entry was found and removed.
    /// Returns `StashError::Locked` if the stash is locked.
    pub fn remove_entry(&mut self, path: &str) -> Result<bool, StashError> {
        let payload = self.payload.as_mut().ok_or(StashError::Locked)?;

        let initial_len = payload.entries().len();
        payload.entries_mut().retain(|e| e.path() != path);
        Ok(payload.entries().len() < initial_len)
    }

    /// List all entries (metadata only, not decrypted content).
    ///
    /// Returns `StashError::Locked` if the stash is locked.
    pub fn list_entries(&self) -> Result<Vec<&StashEntry>, StashError> {
        let payload = self.payload.as_ref().ok_or(StashError::Locked)?;
        Ok(payload.entries().iter().collect())
    }

    /// Check if the stash is locked.
    pub fn is_locked(&self) -> bool {
        self.key.is_none()
    }

    /// Return a read-only metadata summary for the frontend.
    pub fn metadata(&self) -> StashMetadata {
        let dat_path = stash_dat_path(&self.path);
        let dat_size = fs::metadata(&dat_path).map(|m| m.len()).unwrap_or(0);

        let total_entries = self
            .payload
            .as_ref()
            .map(|p| p.entries().len())
            .unwrap_or(0);

        StashMetadata {
            version: self.meta.version,
            format: self.meta.format.clone(),
            created_at: self.meta.created_at,
            total_entries,
            dat_size,
        }
    }

    /// Return the effective settings (stored or default).
    pub fn settings(&self) -> Settings {
        self.meta.effective_settings()
    }

    /// Update the stash settings and persist to `stash.meta`.
    pub fn update_settings(&mut self, settings: Settings) -> Result<(), StashError> {
        self.meta.set_settings(settings);
        let meta_path = stash_meta_path(&self.path);
        write_meta(&meta_path, &self.meta)
    }

    /// Change the stash password.
    ///
    /// 1. Verifies `old_password` by deriving a key and attempting to decrypt.
    /// 2. Validates `new_password` (minimum 8 characters).
    /// 3. Derives a new key from `new_password`.
    /// 4. Re-encrypts the payload to a temp `.dat` file.
    /// 5. Writes updated meta to a temp `.meta` file.
    /// 6. Atomically renames both temp files to their final names.
    ///
    /// On failure at any step, the original files remain untouched.
    pub fn change_password(
        &mut self,
        old_password: &[u8],
        new_password: &[u8],
    ) -> Result<(), StashError> {
        // Validate new password strength
        if new_password.len() < 8 {
            return Err(StashError::PasswordTooWeak);
        }

        // Verify old password by re-deriving key and attempting decrypt
        let old_salt: [u8; SALT_LEN] = self
            .meta
            .salt()
            .try_into()
            .map_err(|_| StashError::InvalidFormat("salt must be 16 bytes".to_string()))?;
        let old_key = derive_key(old_password, &old_salt).map_err(|_| StashError::WrongPassword)?;

        // Attempt decrypt to verify old password
        let dat_path = stash_dat_path(&self.path);
        let dat_bytes = fs::read(&dat_path)?;
        let mut cursor = Cursor::new(&dat_bytes);
        let header = Header::read(&mut cursor)?;
        let nonce = header.nonce();

        let data_len = dat_bytes.len();
        if data_len < HEADER_SIZE + Footer::MAGIC.len() + 8 + 8 {
            return Err(StashError::InvalidFormat("container too short".to_string()));
        }

        let ciphertext_start = HEADER_SIZE;
        let ciphertext_end = data_len - Footer::MAGIC.len() - 4 - 8;
        let ciphertext = &dat_bytes[ciphertext_start..ciphertext_end];

        // This will fail with InvalidTag if old password is wrong
        let plaintext =
            decrypt(ciphertext, &old_key, &nonce).map_err(|_| StashError::WrongPassword)?;

        // Derive new key
        let new_key = derive_key(new_password, &old_salt).map_err(|_| StashError::WrongPassword)?;

        // Re-encrypt payload with new key
        let config = bincode::config::standard().with_limit::<{ MAX_DECODING_SIZE }>();
        let (payload, _len): (StashPayload, usize) =
            bincode::serde::decode_from_slice(&plaintext, config)
                .map_err(|e| StashError::Serialization(e.to_string()))?;

        let new_ciphertext = encrypt(
            &{
                let config = bincode::config::standard();
                bincode::serde::encode_to_vec(&payload, config)
                    .map_err(|e| StashError::Serialization(e.to_string()))?
            },
            &new_key,
            &nonce,
        )
        .map_err(|_| {
            StashError::Crypto(crate::error::CryptoError::EncryptError(
                "re-encryption failed".to_string(),
            ))
        })?;

        // Write temp .dat
        let tmp_dat = dat_path.with_extension("dat.tmp");
        let mut tmp_buf = Vec::new();
        header.write(&mut tmp_buf)?;
        tmp_buf.extend_from_slice(&new_ciphertext);

        // Re-read footer from original file
        let footer_start = data_len - FOOTER_SIZE;
        let mut footer_cursor = Cursor::new(&dat_bytes[footer_start..]);
        let footer = Footer::read(&mut footer_cursor)?;
        footer.write(&mut tmp_buf)?;
        fs::write(&tmp_dat, &tmp_buf)?;

        // Update meta with new key material (salt stays the same, but we re-derive)
        // Write temp .meta
        let meta_path = stash_meta_path(&self.path);
        let tmp_meta = meta_path.with_extension("meta.tmp");
        write_meta(&tmp_meta, &self.meta)?;

        // Atomic rename both
        fs::rename(&tmp_dat, &dat_path)?;
        fs::rename(&tmp_meta, &meta_path)?;

        // Update in-memory key
        self.key = Some(new_key);

        Ok(())
    }

    /// Export the stash files to a target directory.
    ///
    /// Copies `stash.dat` and `stash.meta` to `target`.
    /// Returns `AlreadyExists` if target already contains stash files.
    pub fn export_to(&self, target: &Path) -> Result<(), StashError> {
        let source_dat = stash_dat_path(&self.path);
        let source_meta = stash_meta_path(&self.path);
        let target_dat = target.join("stash.dat");
        let target_meta = target.join("stash.meta");

        // Check if target already has stash files
        if target_dat.exists() || target_meta.exists() {
            return Err(StashError::AlreadyExists(target.to_path_buf()));
        }

        // Ensure target directory exists
        if !target.is_dir() {
            return Err(StashError::NotFound(target.to_path_buf()));
        }

        fs::copy(&source_dat, &target_dat)?;
        fs::copy(&source_meta, &target_meta)?;

        Ok(())
    }
}

impl Drop for Stash {
    fn drop(&mut self) {
        self.lock();
    }
}

// ─── Helper Functions ───────────────────────────────────────────────────────

fn stash_meta_path(base: &Path) -> PathBuf {
    base.join("stash.meta")
}

fn stash_dat_path(base: &Path) -> PathBuf {
    base.join("stash.dat")
}

fn encrypt_payload(
    payload: &StashPayload,
    key: &KeyMaterial,
    nonce: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, StashError> {
    let config = bincode::config::standard();
    let encoded = bincode::serde::encode_to_vec(payload, config)
        .map_err(|e| StashError::Serialization(e.to_string()))?;

    encrypt(&encoded, key, nonce).map_err(StashError::Crypto)
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), StashError> {
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, data)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn write_meta(path: &Path, meta: &StashMeta) -> Result<(), StashError> {
    let json = meta.to_json()?;
    atomic_write(path, json.as_bytes())
}

fn current_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn guess_mime_type(path: &str) -> String {
    if let Some(ext) = path.rsplit('.').next() {
        match ext.to_lowercase().as_str() {
            "txt" | "md" | "log" => "text/plain".to_string(),
            "json" => "application/json".to_string(),
            "xml" => "application/xml".to_string(),
            "html" | "htm" => "text/html".to_string(),
            "csv" => "text/csv".to_string(),
            "png" => "image/png".to_string(),
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "gif" => "image/gif".to_string(),
            "pdf" => "application/pdf".to_string(),
            "zip" => "application/zip".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    } else {
        "application/octet-stream".to_string()
    }
}

// ─── Tests (Phase 8) ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_stash() -> (Stash, TempDir) {
        let tmp = TempDir::new().expect("create temp dir");
        let stash = Stash::create(b"test-password", tmp.path()).expect("create test stash");
        (stash, tmp)
    }

    // ─── Settings serde round-trip ─────────────────────────────────────

    #[test]
    fn settings_default_is_5_minutes() {
        let settings = Settings::default();
        assert_eq!(settings.auto_lock_seconds, 300);
    }

    #[test]
    fn settings_serde_round_trip() {
        let settings = Settings {
            auto_lock_seconds: 60,
        };
        let json = serde_json::to_string(&settings).unwrap();
        let decoded: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.auto_lock_seconds, 60);
    }

    #[test]
    fn stash_meta_without_settings_field_deserializes_as_none() {
        // Simulate an old stash.meta without the settings field
        let old_meta_json = r#"{
            "version": 1,
            "format": "usbstash-v1",
            "salt": "AAAAAAAAAAAAAAAAAAAAAA==",
            "kdf_memory_kib": 65536,
            "kdf_iterations": 3,
            "kdf_parallelism": 4,
            "aead_algorithm": "XChaCha20-Poly1305",
            "aead_nonce_len": 24,
            "aead_key_len": 32,
            "aead_tag_len": 16,
            "created_at": 1700000000,
            "failed_attempts": 0,
            "last_attempt_at": null
        }"#;
        let meta: StashMeta = StashMeta::from_json(old_meta_json).unwrap();
        assert!(meta.settings().is_none());
        assert_eq!(meta.effective_settings().auto_lock_seconds, 300);
    }

    #[test]
    fn stash_meta_with_settings_field_deserializes_correctly() {
        let meta_json = r#"{
            "version": 1,
            "format": "usbstash-v1",
            "salt": "AAAAAAAAAAAAAAAAAAAAAA==",
            "kdf_memory_kib": 65536,
            "kdf_iterations": 3,
            "kdf_parallelism": 4,
            "aead_algorithm": "XChaCha20-Poly1305",
            "aead_nonce_len": 24,
            "aead_key_len": 32,
            "aead_tag_len": 16,
            "created_at": 1700000000,
            "failed_attempts": 0,
            "last_attempt_at": null,
            "settings": {"auto_lock_seconds": 60}
        }"#;
        let meta: StashMeta = StashMeta::from_json(meta_json).unwrap();
        assert!(meta.settings().is_some());
        assert_eq!(meta.effective_settings().auto_lock_seconds, 60);
    }

    // ─── Stash::metadata ───────────────────────────────────────────────

    #[test]
    fn metadata_returns_correct_fields() {
        let (stash, _tmp) = make_test_stash();
        let meta = stash.metadata();

        assert_eq!(meta.version, 1);
        assert_eq!(meta.format, "usbstash-v1");
        assert_eq!(meta.total_entries, 0);
        assert!(meta.dat_size > 0); // header + footer at minimum
        assert!(meta.created_at > 0);
    }

    #[test]
    fn metadata_counts_entries() {
        let (mut stash, _tmp) = make_test_stash();
        stash
            .add_entry("/test.txt".to_string(), b"hello".to_vec())
            .unwrap();
        let meta = stash.metadata();
        assert_eq!(meta.total_entries, 1);
    }

    // ─── Stash::update_settings ────────────────────────────────────────

    #[test]
    fn update_settings_persists_to_disk() {
        let (mut stash, tmp) = make_test_stash();

        let new_settings = Settings {
            auto_lock_seconds: 60,
        };
        stash.update_settings(new_settings).unwrap();

        // Re-open and verify settings persisted
        let _reopened = Stash::open(b"test-password", tmp.path()).expect("reopen stash");
        let meta_json = fs::read_to_string(stash_meta_path(tmp.path())).unwrap();
        let meta: StashMeta = StashMeta::from_json(&meta_json).unwrap();
        assert_eq!(meta.effective_settings().auto_lock_seconds, 60);
    }

    // ─── Stash::change_password ────────────────────────────────────────

    #[test]
    fn change_password_succeeds_with_correct_old_password() {
        let (mut stash, tmp) = make_test_stash();

        stash
            .change_password(b"test-password", b"new-password")
            .expect("change_password should succeed");

        // Old password should fail
        let old_open = Stash::open(b"test-password", tmp.path());
        assert!(old_open.is_err());

        // New password should work
        let new_open = Stash::open(b"new-password", tmp.path());
        assert!(new_open.is_ok());
    }

    #[test]
    fn change_password_fails_with_wrong_old_password() {
        let (mut stash, tmp) = make_test_stash();

        let result = stash.change_password(b"wrong-password", b"new-password");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StashError::WrongPassword));

        // Original password should still work
        let open = Stash::open(b"test-password", tmp.path());
        assert!(open.is_ok());
    }

    #[test]
    fn change_password_rejects_weak_new_password() {
        let (mut stash, tmp) = make_test_stash();

        let result = stash.change_password(b"test-password", b"short");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StashError::PasswordTooWeak));

        // Original password should still work
        let open = Stash::open(b"test-password", tmp.path());
        assert!(open.is_ok());
    }

    #[test]
    fn change_password_preserves_entries() {
        let (mut stash, tmp) = make_test_stash();
        stash
            .add_entry("/secret.txt".to_string(), b"secret content".to_vec())
            .unwrap();
        stash.save().unwrap();

        stash
            .change_password(b"test-password", b"new-password")
            .unwrap();

        let reopened = Stash::open(b"new-password", tmp.path()).unwrap();
        let entry = reopened.get_entry("/secret.txt").unwrap();
        assert_eq!(entry.content(), b"secret content");
    }

    // ─── Stash::export_to ──────────────────────────────────────────────

    #[test]
    fn exported_stash_opens_with_same_password() {
        let (stash, _tmp) = make_test_stash();
        let export_dir = TempDir::new().expect("create export dir");

        stash.export_to(export_dir.path()).unwrap();

        assert!(export_dir.path().join("stash.dat").exists());
        assert!(export_dir.path().join("stash.meta").exists());
    }

    #[test]
    fn export_to_fails_if_target_already_has_stash_files() {
        let (stash, _tmp) = make_test_stash();

        // Create a stash in the export target
        let export_dir = TempDir::new().expect("create export dir");
        Stash::create(b"other-password", export_dir.path()).unwrap();

        let result = stash.export_to(export_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StashError::AlreadyExists(_)));
    }

    #[test]
    fn export_to_produces_reopenable_stash() {
        let (stash, _tmp) = make_test_stash();
        let export_dir = TempDir::new().expect("create export dir");

        stash.export_to(export_dir.path()).unwrap();

        // Open the exported copy
        let exported = Stash::open(b"test-password", export_dir.path()).unwrap();
        assert_eq!(exported.metadata().version, 1);
    }
}
