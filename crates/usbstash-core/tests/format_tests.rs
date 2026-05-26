//! Tests for the binary container format (Header/Footer).

use std::io::Cursor;

use usbstash_core::StashError;
use usbstash_core::format::{Footer, Header};

// ─── Header round-trip ─────────────────────────────────────────────────────

#[test]
fn header_write_and_read_roundtrip() {
    let nonce = [0xABu8; 24];
    let salt = [0xCDu8; 16];
    let header = Header::new(salt, nonce);

    let mut buf = Vec::new();
    header.write(&mut buf).unwrap();

    assert_eq!(buf.len(), 48, "Header must be exactly 48 bytes");

    let mut cursor = Cursor::new(&buf);
    let read = Header::read(&mut cursor).unwrap();

    assert_eq!(read.magic(), Header::MAGIC);
    assert_eq!(read.version(), Header::VERSION);
    assert_eq!(read.salt(), salt);
    assert_eq!(read.nonce(), nonce);
}

#[test]
fn header_default_produces_valid_header() {
    let header = Header::default();
    assert_eq!(header.magic(), Header::MAGIC);
    assert_eq!(header.version(), Header::VERSION);
}

#[test]
fn header_magic_constant_is_stsh() {
    assert_eq!(Header::MAGIC, *b"STSH");
}

#[test]
fn header_version_constant_is_one() {
    assert_eq!(Header::VERSION, 1u16);
}

// ─── Header validation ─────────────────────────────────────────────────────

#[test]
fn header_rejects_invalid_magic() {
    // Write a valid header then corrupt the magic bytes
    let header = Header::default();
    let mut buf = Vec::new();
    header.write(&mut buf).unwrap();

    // Corrupt magic: change first byte
    buf[0] = b'X';

    let mut cursor = Cursor::new(&buf);
    let result = Header::read(&mut cursor);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), StashError::InvalidFormat(_)));
}

#[test]
fn header_rejects_unsupported_version() {
    // Write a valid header then change the version
    let header = Header::default();
    let mut buf = Vec::new();
    header.write(&mut buf).unwrap();

    // Version is at offset 4 (after 4-byte magic), u16 LE
    buf[4] = 0x02; // version = 2
    buf[5] = 0x00;

    let mut cursor = Cursor::new(&buf);
    let result = Header::read(&mut cursor);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), StashError::UnsupportedVersion(2)));
}

#[test]
fn header_rejects_short_input() {
    let short = vec![0u8; 20]; // Less than 48 bytes
    let mut cursor = Cursor::new(&short);
    let result = Header::read(&mut cursor);

    assert!(result.is_err());
}

// ─── Footer round-trip ─────────────────────────────────────────────────────

#[test]
fn footer_write_and_read_roundtrip() {
    let footer = Footer::new(3, 1024);

    let mut buf = Vec::new();
    footer.write(&mut buf).unwrap();

    assert_eq!(buf.len(), 16, "Footer must be exactly 16 bytes");

    let mut cursor = Cursor::new(&buf);
    let read = Footer::read(&mut cursor).unwrap();

    assert_eq!(read.magic(), Footer::MAGIC);
    assert_eq!(read.chunk_count(), 3);
    assert_eq!(read.payload_size(), 1024);
}

#[test]
fn footer_magic_constant_is_stsh() {
    assert_eq!(Footer::MAGIC, *b"STSH");
}

#[test]
fn footer_rejects_invalid_magic() {
    let footer = Footer::new(1, 0);
    let mut buf = Vec::new();
    footer.write(&mut buf).unwrap();

    // Corrupt magic
    buf[0] = b'X';

    let mut cursor = Cursor::new(&buf);
    let result = Footer::read(&mut cursor);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), StashError::InvalidFormat(_)));
}

#[test]
fn footer_rejects_short_input() {
    let short = vec![0u8; 8]; // Less than 16 bytes
    let mut cursor = Cursor::new(&short);
    let result = Footer::read(&mut cursor);

    assert!(result.is_err());
}

// ─── Header endianness ─────────────────────────────────────────────────────

#[test]
fn header_version_is_little_endian() {
    // Version 0x0102 should serialize as [0x02, 0x01] in LE
    let salt = [0u8; 16];
    let nonce = [0u8; 24];
    let header = Header::new_with_version(salt, nonce, 0x0102);

    let mut buf = Vec::new();
    header.write(&mut buf).unwrap();

    // Bytes 4-5 are version in LE
    assert_eq!(buf[4], 0x02);
    assert_eq!(buf[5], 0x01);
}

#[test]
fn header_flags_is_little_endian() {
    let salt = [0u8; 16];
    let nonce = [0u8; 24];
    let header = Header::new_with_flags(salt, nonce, 0x0304);

    let mut buf = Vec::new();
    header.write(&mut buf).unwrap();

    // Flags are at offset 6-7 (after magic[4] + version[2])
    assert_eq!(buf[6], 0x04);
    assert_eq!(buf[7], 0x03);
}

// ─── Footer endianness ─────────────────────────────────────────────────────

#[test]
fn footer_chunk_count_is_little_endian() {
    let footer = Footer::new(0x01020304, 0);
    let mut buf = Vec::new();
    footer.write(&mut buf).unwrap();

    // chunk_count is at offset 4 (after magic[4]), u32 LE
    assert_eq!(buf[4], 0x04);
    assert_eq!(buf[5], 0x03);
    assert_eq!(buf[6], 0x02);
    assert_eq!(buf[7], 0x01);
}

#[test]
fn footer_payload_size_is_little_endian() {
    let footer = Footer::new(0, 0x0102030405060708);
    let mut buf = Vec::new();
    footer.write(&mut buf).unwrap();

    // payload_size is at offset 8 (after magic[4] + chunk_count[4]), u64 LE
    assert_eq!(buf[8], 0x08);
    assert_eq!(buf[9], 0x07);
    assert_eq!(buf[10], 0x06);
    assert_eq!(buf[11], 0x05);
    assert_eq!(buf[12], 0x04);
    assert_eq!(buf[13], 0x03);
    assert_eq!(buf[14], 0x02);
    assert_eq!(buf[15], 0x01);
}
