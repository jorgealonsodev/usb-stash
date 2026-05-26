use std::sync::Mutex;
use usbstash_core::Stash;

/// Application state managed by Tauri.
///
/// Wraps `Mutex<Option<Stash>>` to represent the open/locked lifecycle.
/// When `None`, the stash is locked (no stash open or explicitly locked).
/// When `Some(Stash)`, the stash is open and accessible.
pub struct AppState(pub Mutex<Option<Stash>>);

impl AppState {
    /// Create a new AppState with no open stash (locked state).
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    /// Check if the stash is currently locked (None or inner Stash is locked).
    pub fn is_locked(&self) -> bool {
        let guard = self.0.lock().expect("AppState mutex poisoned");
        match guard.as_ref() {
            None => true,
            Some(stash) => stash.is_locked(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_app_state_is_locked() {
        let state = AppState::new();
        assert!(
            state.is_locked(),
            "new AppState should be locked (no stash open)"
        );
    }

    #[test]
    fn app_state_wraps_mutex_option_stash() {
        let state = AppState::new();
        let guard = state.0.lock().unwrap();
        assert!(
            guard.is_none(),
            "inner Option<Stash> should be None initially"
        );
    }
}
