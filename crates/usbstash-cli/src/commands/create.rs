//! Create command handler.
//!
//! Creates a new encrypted stash at the given directory path.

use std::path::Path;

use usbstash_core::Stash;
use zeroize::Zeroizing;

use crate::error::CliError;

/// Create a new stash at `dir`.
///
/// Delegates to `Stash::create`. Returns `CliError::Stash(AlreadyExists)` if
/// stash files already exist, or other `StashError` variants on failure.
pub fn create(dir: &Path, password: &Zeroizing<String>) -> Result<(), CliError> {
    Stash::create(password.as_bytes(), dir)?;
    eprintln!("Stash created at {}", dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use zeroize::Zeroizing;

    use super::create;

    #[test]
    fn create_stash_success() {
        let dir = tempdir().unwrap();
        // Use the tempdir itself as the stash path (it already exists)
        let password = Zeroizing::new("test-password".to_string());

        let result = create(dir.path(), &password);
        assert!(result.is_ok());

        // Verify files were created
        assert!(dir.path().join("stash.meta").exists());
        assert!(dir.path().join("stash.dat").exists());
    }

    #[test]
    fn create_stash_already_exists() {
        let dir = tempdir().unwrap();
        let password = Zeroizing::new("test-password".to_string());

        // Create once
        create(dir.path(), &password).unwrap();

        // Try to create again
        let result = create(dir.path(), &password);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("already exists"));
    }

    #[test]
    fn create_stash_parent_not_found() {
        let dir = tempdir().unwrap();
        // Path with non-existent parent chain
        let stash_path = dir.path().join("nonexistent").join("deep").join("stash");

        let password = Zeroizing::new("test-password".to_string());
        let result = create(&stash_path, &password);
        // Stash::create doesn't create parent dirs — it will fail when trying
        // to write files to a non-existent directory
        assert!(result.is_err());
    }
}
