use serde::Serialize;
use tauri::State;
use usbstash_core::StashEntry;

use crate::state::AppState;

/// DTO for serializing entry metadata to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct EntrySummary {
    pub id: String,
    pub path: String,
    pub size: u64,
    pub mime_type: String,
    pub created_at: u64,
    pub modified_at: u64,
}

impl EntrySummary {
    /// Convert a `StashEntry` to an `EntrySummary` DTO.
    pub fn from_stash_entry(entry: &StashEntry) -> Self {
        Self {
            id: entry.id().to_string(),
            path: entry.path().to_string(),
            size: entry.size(),
            mime_type: entry.mime_type().to_string(),
            created_at: entry.created_at(),
            modified_at: entry.modified_at(),
        }
    }
}

// ─── Tauri Commands ────────────────────────────────────────────────────────

/// Check if a stash exists at the given path.
#[tauri::command]
pub fn stash_exists(path: String) -> Result<bool, String> {
    let meta_path = std::path::Path::new(&path).join("stash.meta");
    let dat_path = std::path::Path::new(&path).join("stash.dat");
    Ok(meta_path.exists() && dat_path.exists())
}

/// Create a new encrypted stash at the given path.
#[tauri::command]
pub fn create_stash(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> Result<(), String> {
    let _state = state; // reserved for Phase 5
    let _path = path;
    let _password = password;
    // TODO: delegate to usbstash_core::Stash::create
    Err("not yet implemented".to_string())
}

/// Open an existing stash with the given password.
#[tauri::command]
pub fn open_stash(
    state: State<'_, AppState>,
    path: String,
    password: String,
) -> Result<(), String> {
    let _state = state;
    let _path = path;
    let _password = password;
    // TODO: delegate to usbstash_core::Stash::open
    Err("not yet implemented".to_string())
}

/// Lock the currently open stash, zeroizing sensitive data.
#[tauri::command]
pub fn lock_stash(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|e| format!("state lock error: {e}"))?;
    if let Some(stash) = guard.as_mut() {
        stash.lock();
    }
    Ok(())
}

/// List all entries in the currently open stash.
#[tauri::command]
pub fn list_entries(state: State<'_, AppState>) -> Result<Vec<EntrySummary>, String> {
    let guard = state
        .0
        .lock()
        .map_err(|e| format!("state lock error: {e}"))?;
    let stash = guard.as_ref().ok_or("no stash open")?;
    let entries = stash.list_entries().map_err(|e| e.to_string())?;
    Ok(entries.iter().map(|e| EntrySummary::from_stash_entry(e)).collect())
}

/// Add a new entry to the currently open stash.
#[tauri::command]
pub fn add_entry(
    state: State<'_, AppState>,
    path: String,
    content: Vec<u8>,
) -> Result<String, String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|e| format!("state lock error: {e}"))?;
    let stash = guard.as_mut().ok_or("no stash open")?;
    let id = stash.add_entry(path, content).map_err(|e| e.to_string())?;
    Ok(id.to_string())
}

/// Extract an entry from the currently open stash to a file.
#[tauri::command]
pub fn extract_entry(
    state: State<'_, AppState>,
    entry_path: String,
    output: String,
) -> Result<(), String> {
    let _state = state;
    let _entry_path = entry_path;
    let _output = output;
    // TODO: delegate to usbstash_core
    Err("not yet implemented".to_string())
}

/// Delete an entry from the currently open stash (in-memory only).
/// Changes persist only after `save_stash` is called.
#[tauri::command]
pub fn delete_entry(
    state: State<'_, AppState>,
    entry_path: String,
) -> Result<(), String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|e| format!("state lock error: {e}"))?;
    let stash = guard.as_mut().ok_or("no stash open")?;
    let removed = stash.remove_entry(&entry_path).map_err(|e| e.to_string())?;
    if !removed {
        return Err(format!("entry not found: {entry_path}"));
    }
    Ok(())
}

/// Rename an entry in the currently open stash (in-memory only).
/// Removes the entry at `entry_path` and re-adds it with `new_path`,
/// preserving content and MIME type. Changes persist only after `save_stash`.
#[tauri::command]
pub fn rename_entry(
    state: State<'_, AppState>,
    entry_path: String,
    new_path: String,
) -> Result<(), String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|e| format!("state lock error: {e}"))?;
    let stash = guard.as_mut().ok_or("no stash open")?;

    // Get the original entry data
    let entry = stash.get_entry(&entry_path).map_err(|e| e.to_string())?;
    let content = entry.content().to_vec();

    // Remove old entry
    stash.remove_entry(&entry_path).map_err(|e| e.to_string())?;

    // Re-add with new path (mime type is re-guessed from new path)
    stash
        .add_entry(new_path, content)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Persist the current in-memory stash state to disk.
#[tauri::command]
pub fn save_stash(state: State<'_, AppState>) -> Result<(), String> {
    let guard = state
        .0
        .lock()
        .map_err(|e| format!("state lock error: {e}"))?;
    let stash = guard.as_ref().ok_or("no stash open")?;
    stash.save().map_err(|e| e.to_string())
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn make_test_entry() -> StashEntry {
        StashEntry::new(
            Uuid::new_v4(),
            "/docs/secret.txt".to_string(),
            1_700_000_000,
            1_700_000_100,
            42,
            "text/plain".to_string(),
            b"hello world".to_vec(),
        )
    }

    /// Create a test stash with entries, returning the stash and temp dir.
    fn make_test_stash() -> (usbstash_core::Stash, TempDir) {
        let tmp = TempDir::new().expect("create temp dir");
        let mut stash =
            usbstash_core::Stash::create(b"test-password", tmp.path()).expect("create test stash");
        stash
            .add_entry("/docs/notes.txt".to_string(), b"some notes".to_vec())
            .expect("add entry");
        stash
            .add_entry("/img/logo.png".to_string(), b"fake png".to_vec())
            .expect("add entry");
        (stash, tmp)
    }

    // ─── EntrySummary tests ────────────────────────────────────────────

    #[test]
    fn entry_summary_from_stash_entry_maps_all_fields() {
        let entry = make_test_entry();
        let summary = EntrySummary::from_stash_entry(&entry);

        assert_eq!(summary.path, "/docs/secret.txt");
        assert_eq!(summary.size, 42);
        assert_eq!(summary.mime_type, "text/plain");
        assert_eq!(summary.created_at, 1_700_000_000);
        assert_eq!(summary.modified_at, 1_700_000_100);
        assert_eq!(summary.id.len(), 36);
    }

    #[test]
    fn entry_summary_id_is_valid_uuid_string() {
        let fixed_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let entry = StashEntry::new(
            fixed_id,
            "/test.bin".to_string(),
            0,
            0,
            0,
            "application/octet-stream".to_string(),
            vec![],
        );
        let summary = EntrySummary::from_stash_entry(&entry);
        assert_eq!(summary.id, "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn stash_exists_returns_false_for_nonexistent_path() {
        let result = stash_exists("/nonexistent/path/that/does/not/exist".to_string()).unwrap();
        assert!(!result);
    }

    #[test]
    fn entry_summary_serializes_to_json() {
        let entry = make_test_entry();
        let summary = EntrySummary::from_stash_entry(&entry);
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"path\":\"/docs/secret.txt\""));
        assert!(json.contains("\"size\":42"));
        assert!(json.contains("\"mime_type\":\"text/plain\""));
    }

    // ─── delete_entry (tested via Stash.remove_entry) ──────────────────

    #[test]
    fn remove_entry_deletes_existing_entry() {
        let (mut stash, _tmp) = make_test_stash();

        // Verify entry exists
        let entries = stash.list_entries().unwrap();
        assert!(entries.iter().any(|e| e.path() == "/docs/notes.txt"));

        // Delete
        let removed = stash.remove_entry("/docs/notes.txt").unwrap();
        assert!(removed);

        // Verify gone
        let entries = stash.list_entries().unwrap();
        assert!(!entries.iter().any(|e| e.path() == "/docs/notes.txt"));
        assert!(entries.iter().any(|e| e.path() == "/img/logo.png"));
    }

    #[test]
    fn remove_entry_returns_false_for_nonexistent_path() {
        let (mut stash, _tmp) = make_test_stash();

        let removed = stash.remove_entry("/does/not/exist.txt").unwrap();
        assert!(!removed);
    }

    #[test]
    fn remove_entry_on_locked_stash_returns_error() {
        let (mut stash, _tmp) = make_test_stash();
        stash.lock();

        let result = stash.remove_entry("/docs/notes.txt");
        assert!(result.is_err());
    }

    // ─── rename_entry (remove + re-add pattern) ────────────────────────

    #[test]
    fn rename_entry_via_remove_and_add_preserves_content() {
        let (mut stash, _tmp) = make_test_stash();

        // Get original entry data
        let original = stash.get_entry("/docs/notes.txt").unwrap();
        let content = original.content().to_vec();
        let mime = original.mime_type().to_string();
        let size = original.size();

        // Rename: remove + re-add
        let removed = stash.remove_entry("/docs/notes.txt").unwrap();
        assert!(removed);
        let _id = stash
            .add_entry("/docs/notes-renamed.txt".to_string(), content)
            .unwrap();

        // Verify old path gone
        let entries = stash.list_entries().unwrap();
        assert!(!entries.iter().any(|e| e.path() == "/docs/notes.txt"));

        // Verify new path exists with preserved content
        let renamed = stash.get_entry("/docs/notes-renamed.txt").unwrap();
        assert_eq!(renamed.size(), size);
        assert_eq!(renamed.mime_type(), mime);
        assert_eq!(renamed.content(), b"some notes");
    }

    #[test]
    fn rename_entry_on_locked_stash_fails() {
        let (mut stash, _tmp) = make_test_stash();
        stash.lock();

        let result = stash.remove_entry("/docs/notes.txt");
        assert!(result.is_err());
    }

    // ─── save_stash (tested via Stash.save) ────────────────────────────

    #[test]
    fn save_persists_in_memory_changes() {
        let (mut stash, tmp) = make_test_stash();

        // Delete an entry (in-memory change)
        stash.remove_entry("/docs/notes.txt").unwrap();

        // Save to disk
        stash.save().expect("save should succeed");

        // Re-open and verify deletion persisted
        let reopened =
            usbstash_core::Stash::open(b"test-password", tmp.path()).expect("reopen stash");
        let entries = reopened.list_entries().unwrap();
        assert!(!entries.iter().any(|e| e.path() == "/docs/notes.txt"));
        assert!(entries.iter().any(|e| e.path() == "/img/logo.png"));
    }

    #[test]
    fn save_on_locked_stash_returns_error() {
        let (mut stash, _tmp) = make_test_stash();
        stash.lock();

        let result = stash.save();
        assert!(result.is_err());
    }
}
