//! Binary container format for `stash.dat`.
//!
//! The file format consists of:
//! - A 48-byte header (magic, version, flags, salt, nonce, reserved)
//! - Encrypted payload chunks
//! - A 16-byte footer (magic, chunk_count, payload_size)
//!
//! All multi-byte fields are stored in little-endian byte order.

use std::io::{self, Read, Write};

use crate::error::StashError;

// ─── Constants ──────────────────────────────────────────────────────────────

/// Size of the header in bytes.
pub const HEADER_SIZE: usize = 48;

/// Size of the footer in bytes.
pub const FOOTER_SIZE: usize = 16;

// ─── Header ─────────────────────────────────────────────────────────────────

/// Container header (48 bytes, little-endian).
///
/// Layout:
/// | Field      | Offset | Size |
/// |------------|--------|------|
/// | magic      | 0      | 4    |
/// | version    | 4      | 2    |
/// | flags      | 6      | 2    |
/// | salt       | 8      | 16   |
/// | nonce      | 24     | 24   |
/// | _reserved  | 46     | 2    |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    magic: [u8; 4],
    version: u16,
    flags: u16,
    salt: [u8; 16],
    nonce: [u8; 24],
    _reserved: [u8; 2],
}

impl Header {
    /// Magic bytes identifying a valid stash container.
    pub const MAGIC: [u8; 4] = *b"STSH";

    /// Supported format version.
    pub const VERSION: u16 = 1;

    /// Create a new header with the given salt and nonce.
    /// Uses default version (1) and zero flags.
    pub fn new(salt: [u8; 16], nonce: [u8; 24]) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            flags: 0,
            salt,
            nonce,
            _reserved: [0; 2],
        }
    }

    /// Create a header with a specific version (for testing).
    pub fn new_with_version(salt: [u8; 16], nonce: [u8; 24], version: u16) -> Self {
        Self {
            magic: Self::MAGIC,
            version,
            flags: 0,
            salt,
            nonce,
            _reserved: [0; 2],
        }
    }

    /// Create a header with specific flags (for testing).
    pub fn new_with_flags(salt: [u8; 16], nonce: [u8; 24], flags: u16) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            flags,
            salt,
            nonce,
            _reserved: [0; 2],
        }
    }

    /// Read a header from a byte stream.
    ///
    /// Validates magic bytes and version. Returns `InvalidFormat` if magic
    /// doesn't match, or `UnsupportedVersion` if version is not supported.
    pub fn read<R: Read>(r: &mut R) -> Result<Self, StashError> {
        let mut buf = [0u8; HEADER_SIZE];
        r.read_exact(&mut buf).map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                StashError::InvalidFormat("header too short".to_string())
            } else {
                StashError::Io(e)
            }
        })?;

        let magic: [u8; 4] = buf[0..4].try_into().map_err(|_| {
            StashError::InvalidFormat("header magic slice incorrect length".to_string())
        })?;
        if magic != Self::MAGIC {
            return Err(StashError::InvalidFormat(format!(
                "invalid magic: expected STSH, got {:?}",
                String::from_utf8_lossy(&magic)
            )));
        }

        let version = u16::from_le_bytes([buf[4], buf[5]]);
        if version != Self::VERSION {
            return Err(StashError::UnsupportedVersion(version));
        }

        let flags = u16::from_le_bytes([buf[6], buf[7]]);
        let salt: [u8; 16] = buf[8..24].try_into().map_err(|_| {
            StashError::InvalidFormat("header salt slice incorrect length".to_string())
        })?;
        let nonce: [u8; 24] = buf[24..48].try_into().map_err(|_| {
            StashError::InvalidFormat("header nonce slice incorrect length".to_string())
        })?;

        Ok(Self {
            magic,
            version,
            flags,
            salt,
            nonce,
            _reserved: [0; 2],
        })
    }

    /// Write the header to a byte stream.
    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), StashError> {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        buf[6..8].copy_from_slice(&self.flags.to_le_bytes());
        buf[8..24].copy_from_slice(&self.salt);
        buf[24..48].copy_from_slice(&self.nonce);
        // _reserved stays zero

        w.write_all(&buf).map_err(StashError::Io)
    }

    /// Accessor: magic bytes.
    pub fn magic(&self) -> [u8; 4] {
        self.magic
    }

    /// Accessor: format version.
    pub fn version(&self) -> u16 {
        self.version
    }

    /// Accessor: flags field.
    pub fn flags(&self) -> u16 {
        self.flags
    }

    /// Accessor: salt bytes.
    pub fn salt(&self) -> [u8; 16] {
        self.salt
    }

    /// Accessor: nonce bytes.
    pub fn nonce(&self) -> [u8; 24] {
        self.nonce
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::new([0u8; 16], [0u8; 24])
    }
}

// ─── Footer ─────────────────────────────────────────────────────────────────

/// Container footer (16 bytes, little-endian).
///
/// Layout:
/// | Field        | Offset | Size |
/// |--------------|--------|------|
/// | magic        | 0      | 4    |
/// | chunk_count  | 4      | 4    |
/// | payload_size | 8      | 8    |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Footer {
    magic: [u8; 4],
    chunk_count: u32,
    payload_size: u64,
}

impl Footer {
    /// Magic bytes identifying a valid stash container footer.
    pub const MAGIC: [u8; 4] = *b"STSH";

    /// Create a new footer with the given chunk count and payload size.
    pub fn new(chunk_count: u32, payload_size: u64) -> Self {
        Self {
            magic: Self::MAGIC,
            chunk_count,
            payload_size,
        }
    }

    /// Read a footer from a byte stream.
    ///
    /// Validates magic bytes. Returns `InvalidFormat` if magic doesn't match.
    pub fn read<R: Read>(r: &mut R) -> Result<Self, StashError> {
        let mut buf = [0u8; FOOTER_SIZE];
        r.read_exact(&mut buf).map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                StashError::InvalidFormat("footer too short".to_string())
            } else {
                StashError::Io(e)
            }
        })?;

        let magic: [u8; 4] = buf[0..4].try_into().map_err(|_| {
            StashError::InvalidFormat("footer magic slice incorrect length".to_string())
        })?;
        if magic != Self::MAGIC {
            return Err(StashError::InvalidFormat(format!(
                "invalid footer magic: expected STSH, got {:?}",
                String::from_utf8_lossy(&magic)
            )));
        }

        let chunk_count = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let payload_size = u64::from_le_bytes([
            buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
        ]);

        Ok(Self {
            magic,
            chunk_count,
            payload_size,
        })
    }

    /// Write the footer to a byte stream.
    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), StashError> {
        let mut buf = [0u8; FOOTER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..8].copy_from_slice(&self.chunk_count.to_le_bytes());
        buf[8..16].copy_from_slice(&self.payload_size.to_le_bytes());

        w.write_all(&buf).map_err(StashError::Io)
    }

    /// Accessor: magic bytes.
    pub fn magic(&self) -> [u8; 4] {
        self.magic
    }

    /// Accessor: number of encrypted chunks.
    pub fn chunk_count(&self) -> u32 {
        self.chunk_count
    }

    /// Accessor: original plaintext size in bytes.
    pub fn payload_size(&self) -> u64 {
        self.payload_size
    }
}

impl Default for Footer {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
