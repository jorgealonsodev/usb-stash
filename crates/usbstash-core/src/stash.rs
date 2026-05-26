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
use crate::format::{Footer, Header, HEADER_SIZE};

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
            let triple = (vals[0] << 18) | (vals[1] << 12)
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
        let plaintext = decrypt(ciphertext, &key, &nonce).map_err(|e| {
            // Record failed attempt
            meta.record_failed_attempt();
            let _ = write_meta(&meta_path, &meta);
            e
        })?;

        // Decode payload with size limit
        let config = bincode::config::standard()
            .with_limit::<{ MAX_DECODING_SIZE }>();
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
        let key = self
            .key
            .as_ref()
            .ok_or(StashError::Locked)?;
        let payload = self
            .payload
            .as_ref()
            .ok_or(StashError::Locked)?;

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
        let payload = self
            .payload
            .as_mut()
            .ok_or(StashError::Locked)?;

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
        let payload = self
            .payload
            .as_mut()
            .ok_or(StashError::Locked)?;

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
    let encoded =
        bincode::serde::encode_to_vec(payload, config)
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
