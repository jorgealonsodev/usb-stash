//! List command handler.
//!
//! Lists all entries in a stash as a table.

use std::path::Path;

use usbstash_core::Stash;
use zeroize::Zeroizing;

use crate::error::CliError;

/// List all entries in a stash.
///
/// Opens the stash and prints a table with columns: path, size (human-readable),
/// and mime_type.
pub fn list(dir: &Path, password: &Zeroizing<String>) -> Result<Vec<ListEntry>, CliError> {
    let stash = Stash::open(password.as_bytes(), dir)?;
    let entries = stash.list_entries()?;

    let result: Vec<ListEntry> = entries
        .iter()
        .map(|e| ListEntry {
            path: e.path().to_string(),
            size: e.size(),
            mime_type: e.mime_type().to_string(),
        })
        .collect();

    Ok(result)
}

/// A single entry row for display.
#[derive(Debug, Clone)]
pub struct ListEntry {
    pub path: String,
    pub size: u64,
    pub mime_type: String,
}

/// Format bytes into a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use zeroize::Zeroizing;

    use super::{format_bytes, list};
    use crate::commands::{add, create};

    #[test]
    fn list_empty_stash() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        create::create(dir.path(), &password).unwrap();

        let entries = list(dir.path(), &password).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn list_stash_with_entries() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        create::create(dir.path(), &password).unwrap();

        // Add two files
        let file1 = dir.path().join("notes.txt");
        fs::write(&file1, "hello world").unwrap();
        add::add(dir.path(), &file1, None, &password).unwrap();

        let file2 = dir.path().join("data.json");
        fs::write(&file2, r#"{"key":"value"}"#).unwrap();
        add::add(dir.path(), &file2, None, &password).unwrap();

        let entries = list(dir.path(), &password).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "notes.txt");
        assert_eq!(entries[0].mime_type, "text/plain");
        assert_eq!(entries[1].path, "data.json");
        assert_eq!(entries[1].mime_type, "application/json");
    }

    #[test]
    fn list_wrong_password() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("correct".to_string());

        create::create(dir.path(), &password).unwrap();

        let wrong = Zeroizing::new("wrong".to_string());
        let result = list(dir.path(), &wrong);
        assert!(result.is_err());
    }

    #[test]
    fn format_bytes_small() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn format_bytes_kilobytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
    }

    #[test]
    fn format_bytes_megabytes() {
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn format_bytes_gigabytes() {
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }
}
