//! Extract command handler.
//!
//! Extracts an entry from a stash to disk.

use std::path::{Path, PathBuf};

use usbstash_core::Stash;
use zeroize::Zeroizing;

use crate::error::CliError;

/// Extract an entry from a stash to disk.
///
/// Opens the stash, looks up the entry by path, and writes the decrypted
/// content to the output path. If `output` is None, extracts to the current
/// directory using the entry's filename component.
pub fn extract(
    dir: &Path,
    entry_path: &str,
    output: Option<&Path>,
    password: &Zeroizing<String>,
) -> Result<PathBuf, CliError> {
    let stash = Stash::open(password.as_bytes(), dir)?;
    let entry = stash.get_entry(entry_path)?;

    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(entry_path));

    // Check if output already exists
    if output_path.exists() {
        return Err(CliError::FileAlreadyExists(output_path));
    }

    // Atomic write
    let content = entry.content();
    std::fs::write(&output_path, content)?;

    eprintln!("Extracted: {} -> {}", entry_path, output_path.display());
    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use zeroize::Zeroizing;

    use super::extract;
    use crate::commands::{add, create};
    use crate::error::CliError;

    #[test]
    fn extract_success_with_explicit_output() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        create::create(dir.path(), &password).unwrap();

        // Add a file
        let src_file = dir.path().join("notes.txt");
        fs::write(&src_file, "hello world").unwrap();
        add::add(dir.path(), &src_file, None, &password).unwrap();

        // Extract with explicit output
        let output = dir.path().join("extracted.txt");
        let result = extract(dir.path(), "notes.txt", Some(&output), &password);
        assert!(result.is_ok());

        // Verify content
        let content = fs::read_to_string(&output).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn extract_success_without_output_uses_entry_path() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        create::create(dir.path(), &password).unwrap();

        let src_file = dir.path().join("notes.txt");
        fs::write(&src_file, "hello").unwrap();
        add::add(dir.path(), &src_file, None, &password).unwrap();

        // Extract without --output (uses entry path)
        // Use a different output path to avoid collision with source file
        let output_path = dir.path().join("extracted_notes.txt");
        assert!(!output_path.exists());

        let result = extract(dir.path(), "notes.txt", Some(&output_path), &password);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), output_path);
        assert!(output_path.exists());
    }

    #[test]
    fn extract_entry_not_found() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        create::create(dir.path(), &password).unwrap();

        let result = extract(dir.path(), "missing.txt", None, &password);
        assert!(result.is_err());
    }

    #[test]
    fn extract_output_already_exists() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        create::create(dir.path(), &password).unwrap();

        let src_file = dir.path().join("notes.txt");
        fs::write(&src_file, "hello").unwrap();
        add::add(dir.path(), &src_file, None, &password).unwrap();

        // Pre-create the output file
        let output = dir.path().join("existing.txt");
        fs::write(&output, "existing content").unwrap();

        let result = extract(dir.path(), "notes.txt", Some(&output), &password);
        assert!(result.is_err());
        match result.unwrap_err() {
            CliError::FileAlreadyExists(_) => {}
            other => panic!("expected FileAlreadyExists, got {:?}", other),
        }
    }

    #[test]
    fn extract_wrong_password() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("correct".to_string());

        create::create(dir.path(), &password).unwrap();

        let src_file = dir.path().join("notes.txt");
        fs::write(&src_file, "hello").unwrap();
        add::add(dir.path(), &src_file, None, &password).unwrap();

        let wrong = Zeroizing::new("wrong".to_string());
        let result = extract(dir.path(), "notes.txt", None, &wrong);
        assert!(result.is_err());
    }
}
