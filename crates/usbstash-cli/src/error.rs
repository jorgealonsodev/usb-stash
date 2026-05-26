//! CLI-specific error types wrapping core and I/O errors.

use std::path::PathBuf;

use thiserror::Error;
use usbstash_core::StashError;

/// Top-level error for the CLI binary.
///
/// Wraps `StashError` from core, `std::io::Error`, and CLI-specific variants
/// (password mismatch, file-not-found for CLI args, etc.).
#[derive(Debug, Error)]
pub enum CliError {
    /// A stash operation failed (wraps `StashError`).
    #[error("{0}")]
    Stash(#[from] StashError),

    /// An I/O operation failed.
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Interactive password confirmation did not match.
    #[error("passwords do not match")]
    PasswordMismatch,

    /// A source file specified on the CLI was not found.
    #[error("file not found: {}", _0.display())]
    FileNotFound(PathBuf),

    /// An output file already exists (extract without --overwrite).
    #[error("file already exists: {}", _0.display())]
    FileAlreadyExists(PathBuf),
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::PathBuf;

    use usbstash_core::StashError;

    use super::CliError;

    #[test]
    fn cli_error_displays_stash_error_already_exists() {
        let stash_err = StashError::AlreadyExists(PathBuf::from("/tmp/stash"));
        let cli_err = CliError::Stash(stash_err);
        assert_eq!(format!("{}", cli_err), "already exists: /tmp/stash");
    }

    #[test]
    fn cli_error_displays_stash_error_not_found() {
        let stash_err = StashError::NotFound(PathBuf::from("/tmp/missing"));
        let cli_err = CliError::Stash(stash_err);
        assert_eq!(format!("{}", cli_err), "not found: /tmp/missing");
    }

    #[test]
    fn cli_error_displays_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let cli_err = CliError::Io(io_err);
        assert!(format!("{}", cli_err).contains("file not found"));
    }

    #[test]
    fn cli_error_displays_password_mismatch() {
        let cli_err = CliError::PasswordMismatch;
        assert_eq!(format!("{}", cli_err), "passwords do not match");
    }

    #[test]
    fn cli_error_displays_file_not_found() {
        let cli_err = CliError::FileNotFound(PathBuf::from("notes.txt"));
        assert_eq!(format!("{}", cli_err), "file not found: notes.txt");
    }

    #[test]
    fn cli_error_displays_file_already_exists() {
        let cli_err = CliError::FileAlreadyExists(PathBuf::from("/tmp/out.txt"));
        assert_eq!(format!("{}", cli_err), "file already exists: /tmp/out.txt");
    }

    #[test]
    fn cli_error_from_stash_error() {
        let stash_err = StashError::Locked;
        let cli_err: CliError = stash_err.into();
        match cli_err {
            CliError::Stash(StashError::Locked) => {}
            other => panic!("expected Stash(Locked), got {:?}", other),
        }
    }

    #[test]
    fn cli_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let cli_err: CliError = io_err.into();
        match cli_err {
            CliError::Io(_) => {}
            other => panic!("expected Io, got {:?}", other),
        }
    }
}
