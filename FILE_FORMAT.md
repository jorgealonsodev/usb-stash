# File Format Specification — USB Stash

> **v1.0** · 2026-05-26 · Reference implementation: `crates/usbstash-core/src/format.rs`

This document specifies the binary format for USB Stash containers. It is
sufficient to implement an independent encoder/decoder.

### Quick Reference

| Byte offset | Size | What |
|-------------|------|------|
| 0 | 4 | Magic `STSH` |
| 4 | 2 | Version (u16 LE) |
| 6 | 2 | Flags |
| 8 | 16 | Salt |
| 24 | 24 | Nonce |
| 48 | variable | Encrypted payload (XChaCha20-Poly1305) |
| -16 | 4 | Footer magic `STSH` |
| -12 | 4 | Chunk count (u32 LE) |
| -8 | 8 | Plaintext size (u64 LE) |

---

## 1. Overview

USB Stash stores data in two files:

| File | Type | Contents |
|------|------|----------|
| `stash.dat` | Binary | Encrypted container with header, ciphertext chunks, and footer |
| `stash.meta` | JSON (text) | Public metadata: KDF parameters, algorithm identifiers, salt |

All multi-byte integer fields are stored in **little-endian** byte order.

## 2. `stash.meta` — JSON Metadata

### 2.1 Schema

```json
{
  "version": 1,
  "format": "usbstash",
  "kdf": {
    "algorithm": "argon2id",
    "memory_kib": 65536,
    "iterations": 3,
    "parallelism": 4,
    "salt": "<base64-encoded 16 bytes>"
  },
  "aead": {
    "algorithm": "xchacha20poly1305",
    "chunk_size": 1048576
  },
  "created_at": "2026-05-26T10:00:00Z",
  "failed_attempts": 0,
  "last_attempt_at": null,
  "settings": {
    "auto_lock_minutes": 5
  }
}
```

### 2.2 Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | integer | Yes | Format version (currently 1) |
| `format` | string | Yes | Must be `"usbstash"` |
| `kdf.algorithm` | string | Yes | Must be `"argon2id"` |
| `kdf.memory_kib` | integer | Yes | Argon2id memory cost in KiB (default: 65536 = 64 MB) |
| `kdf.iterations` | integer | Yes | Argon2id iteration count (default: 3) |
| `kdf.parallelism` | integer | Yes | Argon2id parallelism (default: 4) |
| `kdf.salt` | string (base64) | Yes | 16-byte random salt, base64-encoded |
| `aead.algorithm` | string | Yes | Must be `"xchacha20poly1305"` |
| `aead.chunk_size` | integer | Yes | Maximum plaintext bytes per chunk (default: 1048576 = 1 MB) |
| `created_at` | string (ISO 8601) | Yes | UTC timestamp of stash creation |
| `failed_attempts` | integer | Yes | Count of consecutive failed unlock attempts |
| `last_attempt_at` | string (ISO 8601) or null | Yes | Timestamp of last unlock attempt, or null |
| `settings` | object | No | Application settings (non-sensitive) |

### 2.3 Constraints

- `format` must exactly equal `"usbstash"`.
- `kdf.algorithm` must exactly equal `"argon2id"`.
- `aead.algorithm` must exactly equal `"xchacha20poly1305"`.
- `salt` decodes to exactly 16 bytes.
- `memory_kib` must be ≥ 1024 (1 MB).
- `iterations` must be ≥ 1.
- `parallelism` must be ≥ 1.

## 3. `stash.dat` — Binary Container

### 3.1 Overall Structure

```
┌─────────────────────────────────────────┐
│ HEADER (48 bytes)                       │
├─────────────────────────────────────────┤
│ ENCRYPTED PAYLOAD (variable length)     │
├─────────────────────────────────────────┤
│ FOOTER (16 bytes)                       │
└─────────────────────────────────────────┘
```

### 3.2 Header (48 bytes)

| Offset | Size | Field | Type | Description |
|--------|------|-------|------|-------------|
| 0 | 4 | `magic` | `[u8; 4]` | Magic bytes: `0x53 0x54 0x53 0x48` ("STSH") |
| 4 | 2 | `version` | `u16 LE` | Format version (currently 1) |
| 6 | 2 | `flags` | `u16 LE` | Reserved flags (must be 0) |
| 8 | 16 | `salt` | `[u8; 16]` | Argon2id salt (copy of value in stash.meta) |
| 24 | 24 | `nonce` | `[u8; 24]` | XChaCha20 base nonce (24 random bytes) |
| 46 | 2 | `_reserved` | `[u8; 2]` | Reserved, must be zero |

#### Magic Bytes

The magic bytes are the ASCII characters `STSH` (from "Stash"):

```
0x53 0x54 0x53 0x48
  S    T    S    H
```

A valid `stash.dat` file must begin with these four bytes.

### 3.3 Encrypted Payload

The payload is the ciphertext produced by encrypting the serialized
`StashPayload` structure with XChaCha20-Poly1305.

#### Encryption Process

1. Serialize `StashPayload` using **bincode** (little-endian, no length prefixes for fixed-size fields).
2. For stashes ≤ 1 MB: encrypt the entire payload with the base nonce.
3. For stashes > 1 MB: split into chunks of `chunk_size` bytes (from `stash.meta`).
   - Each chunk is encrypted independently.
   - Chunk nonce = `nonce_base XOR (chunk_index as u64, padded to 24 bytes)`.
   - Each chunk produces ciphertext + 16-byte Poly1305 tag.
4. Concatenate all ciphertext chunks (each followed by its tag).

#### Payload Size

The encrypted payload size is:
- For single-chunk: `plaintext_size + 16` (16 bytes for the Poly1305 tag)
- For multi-chunk: `sum(ciphertext_i + 16)` for each chunk

### 3.4 Footer (16 bytes)

| Offset | Size | Field | Type | Description |
|--------|------|-------|------|-------------|
| 0 | 4 | `magic` | `[u8; 4]` | Magic bytes: `0x53 0x54 0x53 0x48` ("STSH") |
| 4 | 4 | `chunk_count` | `u32 LE` | Number of encrypted chunks |
| 8 | 8 | `payload_size` | `u64 LE` | Original plaintext size in bytes (before encryption) |

### 3.5 File Size Calculation

```
total_file_size = HEADER_SIZE (48) + encrypted_payload_size + FOOTER_SIZE (16)
```

## 4. `StashPayload` — Decrypted Content

After decryption, the plaintext is a bincode-serialized structure:

```rust
struct StashPayload {
    version: u32,
    entries: Vec<StashEntry>,
}

struct StashEntry {
    id: Uuid,              // 16 bytes
    path: String,          // bincode: 8-byte length + UTF-8 bytes
    created_at: SystemTime, // bincode: (u64 secs, u32 nanos)
    modified_at: SystemTime,
    size: u64,
    mime_type: Option<String>, // bincode: 1-byte variant + data
    content: Vec<u8>,      // bincode: 8-byte length + bytes
}
```

### 4.1 Bincode Encoding Details

- **Integers:** Little-endian, fixed-width.
- **Strings:** 8-byte little-endian length prefix followed by UTF-8 bytes.
- **Vectors:** 8-byte little-endian length prefix followed by elements.
- **Options:** 1-byte discriminant (0 = None, 1 = Some) followed by the value.
- **Uuid:** 16 bytes (standard UUID binary representation).
- **SystemTime:** Tuple of `(u64 seconds_since_unix_epoch, u32 nanoseconds)`.

### 4.2 MIME Type Detection

The `mime_type` field is populated when the file is added to the stash:
- Text files: `"text/plain"`
- JSON files: `"application/json"`
- PDF files: `"application/pdf"`
- Images: `"image/png"`, `"image/jpeg"`, `"image/gif"`, etc.
- Unknown: `None`

## 5. Versioning

### 5.1 Current Version

- Header version: `1`
- Meta version: `1`
- Payload version: `1`

### 5.2 Version Compatibility

- A reader MUST reject files with an unknown header version.
- A reader MAY support multiple versions if backward-compatible changes are made.
- Version bumps are required for:
  - Changes to header/footer structure
  - Changes to bincode serialization format
  - Changes to encryption algorithm or parameters

### 5.3 Future Versioning Strategy

- Minor version bumps (e.g., 1 → 2) indicate incompatible changes.
- The `stash.meta` file stores KDF and AEAD parameters separately, allowing
  algorithm upgrades without changing the container format version.

## 6. Validation Checklist

To validate a `stash.dat` file:

1. File size ≥ 64 bytes (header + footer minimum).
2. First 4 bytes = `STSH` magic.
3. Last 16 bytes begin with `STSH` magic.
4. Header version matches supported version.
5. Header salt matches `stash.meta` salt.
6. Footer `chunk_count` ≥ 1.
7. Footer `payload_size` matches expected decrypted size.
8. AEAD tag verification succeeds for each chunk.

## 7. Reference Implementation

The reference implementation is in `crates/usbstash-core/src/format.rs`:
- `Header::read()` / `Header::write()` — header serialization
- `Footer::read()` / `Footer::write()` — footer serialization
