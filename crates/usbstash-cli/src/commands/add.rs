//! Add command handler.
//!
//! Adds a file to an existing stash.

use std::path::Path;

use usbstash_core::Stash;
use zeroize::Zeroizing;

use crate::error::CliError;

/// Add a file to an existing stash.
///
/// Opens the stash, reads the file content, adds it as an entry, and saves.
/// If `as_path` is provided, the entry is stored with that path instead of
/// the file's basename.
pub fn add(
    dir: &Path,
    file: &Path,
    as_path: Option<&str>,
    password: &Zeroizing<String>,
) -> Result<(), CliError> {
    // Check source file exists
    if !file.exists() {
        return Err(CliError::FileNotFound(file.to_path_buf()));
    }

    let content = std::fs::read(file)?;
    let entry_path = as_path
        .map(String::from)
        .or_else(|| file.file_name().map(|n| n.to_string_lossy().to_string()))
        .ok_or_else(|| CliError::FileNotFound(file.to_path_buf()))?;

    let mut stash = Stash::open(password.as_bytes(), dir)?;
    stash.add_entry(entry_path, content)?;
    stash.save()?;

    eprintln!("Added: {}", file.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use usbstash_core::Stash;
    use zeroize::Zeroizing;

    use super::add;
    use crate::commands::create;
    use crate::error::CliError;

    #[test]
    fn add_file_success() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        // Create stash first
        create::create(dir.path(), &password).unwrap();

        // Create a source file
        let src_file = dir.path().join("notes.txt");
        fs::write(&src_file, "hello world").unwrap();

        // Add to stash
        let result = add(dir.path(), &src_file, None, &password);
        assert!(result.is_ok());

        // Verify by opening and listing
        let stash = Stash::open(password.as_bytes(), dir.path()).unwrap();
        let entries = stash.list_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path(), "notes.txt");
    }

    #[test]
    fn add_file_with_as_path_override() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        create::create(dir.path(), &password).unwrap();

        let src_file = dir.path().join("notes.txt");
        fs::write(&src_file, "hello").unwrap();

        let result = add(dir.path(), &src_file, Some("renamed.txt"), &password);
        assert!(result.is_ok());

        let stash = Stash::open(password.as_bytes(), dir.path()).unwrap();
        let entries = stash.list_entries().unwrap();
        assert_eq!(entries[0].path(), "renamed.txt");
    }

    #[test]
    fn add_file_not_found() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        create::create(dir.path(), &password).unwrap();

        let missing_file = dir.path().join("does-not-exist.txt");
        let result = add(dir.path(), &missing_file, None, &password);
        assert!(result.is_err());
        match result.unwrap_err() {
            CliError::FileNotFound(_) => {}
            other => panic!("expected FileNotFound, got {:?}", other),
        }
    }

    #[test]
    fn add_wrong_password() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("correct-password".to_string());

        create::create(dir.path(), &password).unwrap();

        let src_file = dir.path().join("notes.txt");
        fs::write(&src_file, "hello").unwrap();

        let wrong_password = Zeroizing::new("wrong-password".to_string());
        let result = add(dir.path(), &src_file, None, &wrong_password);
        assert!(result.is_err());
    }
}
