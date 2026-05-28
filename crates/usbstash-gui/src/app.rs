use std::sync::{Arc, Mutex};
use std::time::Instant;

use usbstash_core::{Stash, StashEntry};

/// Navigation screens for the GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Login,
    Create,
    Explorer,
    Settings,
}

/// Main application state.
pub struct App {
    /// The open stash (None = locked).
    pub stash: Arc<Mutex<Option<Stash>>>,

    /// Current navigation screen.
    pub screen: Screen,

    // ─── Login state ───────────────────────────────────────────────────
    pub login_path: String,
    pub login_password: String,
    pub login_error: Option<String>,

    // ─── Create state ──────────────────────────────────────────────────
    pub create_path: String,
    pub create_password: String,
    pub create_confirm: String,
    pub create_error: Option<String>,

    // ─── Explorer state ────────────────────────────────────────────────
    pub entries: Vec<StashEntry>,
    pub expanded_paths: std::collections::HashSet<String>,
    pub selected_path: Option<String>,
    pub search_query: String,
    pub is_dirty: bool,

    // ─── Explorer dialog state ─────────────────────────────────────────
    pub show_delete_confirmation: bool,
    pub rename_entry_path: Option<String>,
    pub rename_new_name: String,

    // ─── Settings state ────────────────────────────────────────────────
    pub auto_lock_seconds: u32,
    pub old_password: String,
    pub new_password: String,

    // ─── Auto-lock ─────────────────────────────────────────────────────
    pub last_interaction: Instant,

    // ─── Misc ──────────────────────────────────────────────────────────
    pub status_message: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            stash: Arc::new(Mutex::new(None)),
            screen: Screen::Login,
            login_path: String::new(),
            login_password: String::new(),
            login_error: None,
            create_path: String::new(),
            create_password: String::new(),
            create_confirm: String::new(),
            create_error: None,
            entries: Vec::new(),
            expanded_paths: std::collections::HashSet::new(),
            selected_path: None,
            search_query: String::new(),
            is_dirty: false,
            show_delete_confirmation: false,
            rename_entry_path: None,
            rename_new_name: String::new(),
            auto_lock_seconds: 300, // 5 minutes default
            old_password: String::new(),
            new_password: String::new(),
            last_interaction: Instant::now(),
            status_message: None,
        }
    }
}

impl App {
    /// Reset the auto-lock interaction timer.
    pub fn record_interaction(&mut self) {
        self.last_interaction = Instant::now();
    }

    /// Check if the auto-lock timeout has elapsed and lock if so.
    pub fn check_auto_lock(&mut self) {
        if self.auto_lock_seconds == 0 {
            return;
        }
        let elapsed = self.last_interaction.elapsed().as_secs();
        if elapsed >= self.auto_lock_seconds as u64 {
            self.lock_stash();
        }
    }

    /// Lock the stash and return to the login screen.
    pub fn lock_stash(&mut self) {
        let mut guard = match self.stash.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(stash) = guard.as_mut() {
            stash.lock();
        }
        *guard = None;
        self.screen = Screen::Login;
        self.entries.clear();
        self.selected_path = None;
        self.login_password.clear();
        self.status_message = Some("Stash locked".to_string());
    }

    /// Set a status message that will be displayed briefly.
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    /// Refresh the entries list from the open stash.
    pub fn refresh_entries(&mut self) {
        let guard = match self.stash.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(stash) = guard.as_ref()
            && let Ok(entries) = stash.list_entries()
        {
            self.entries = entries.into_iter().cloned().collect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_record_interaction_updates_timestamp() {
        let mut app = App::default();
        let before = app.last_interaction;
        std::thread::sleep(Duration::from_millis(10));
        app.record_interaction();
        assert!(app.last_interaction >= before);
    }

    #[test]
    fn test_auto_lock_disabled_when_zero() {
        let mut app = App::default();
        app.auto_lock_seconds = 0;
        // Even with a stash, should not lock
        app.screen = Screen::Explorer;
        app.check_auto_lock();
        assert_eq!(app.screen, Screen::Explorer);
    }

    #[test]
    fn test_auto_lock_triggers_after_timeout() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        let stash = Stash::create(b"testpassword123", &path).unwrap();
        stash.save().unwrap();

        let mut app = App::default();
        {
            let mut guard = app.stash.lock().unwrap();
            *guard = Some(stash);
        }
        app.screen = Screen::Explorer;
        app.auto_lock_seconds = 1; // 1 second timeout

        // Simulate time passing by setting last_interaction far in the past
        app.last_interaction = Instant::now() - Duration::from_secs(5);

        app.check_auto_lock();

        assert_eq!(app.screen, Screen::Login);
        assert!(app.entries.is_empty());
    }

    #[test]
    fn test_auto_lock_does_not_trigger_before_timeout() {
        let mut app = App::default();
        app.auto_lock_seconds = 300;
        app.screen = Screen::Explorer;
        // last_interaction is Instant::now() from default — should not trigger
        app.check_auto_lock();
        assert_eq!(app.screen, Screen::Explorer);
    }

    #[test]
    fn test_lock_stash_clears_state() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        let stash = Stash::create(b"testpassword123", &path).unwrap();
        stash.save().unwrap();

        let mut app = App::default();
        {
            let mut guard = app.stash.lock().unwrap();
            *guard = Some(stash);
        }
        app.screen = Screen::Explorer;
        app.entries.push(StashEntry::new(
            uuid::Uuid::new_v4(),
            "test.txt".to_string(),
            0,
            0,
            100,
            "text/plain".to_string(),
            vec![],
        ));
        app.selected_path = Some("test.txt".to_string());
        app.login_password = "old_password".to_string();

        app.lock_stash();

        assert_eq!(app.screen, Screen::Login);
        assert!(app.entries.is_empty());
        assert!(app.selected_path.is_none());
        assert!(app.login_password.is_empty());
        assert!(app.status_message.as_deref() == Some("Stash locked"));
    }

    #[test]
    fn test_screen_dispatch_enum_coverage() {
        // Verify all screen variants exist and are distinct
        assert_ne!(Screen::Login, Screen::Create);
        assert_ne!(Screen::Login, Screen::Explorer);
        assert_ne!(Screen::Login, Screen::Settings);
        assert_ne!(Screen::Create, Screen::Explorer);
        assert_ne!(Screen::Create, Screen::Settings);
        assert_ne!(Screen::Explorer, Screen::Settings);
    }

    #[test]
    fn test_set_status() {
        let mut app = App::default();
        app.set_status("hello");
        assert_eq!(app.status_message, Some("hello".to_string()));
    }

    #[test]
    fn test_default_app_starts_on_login() {
        let app = App::default();
        assert_eq!(app.screen, Screen::Login);
        assert!(app.login_path.is_empty());
        assert!(app.login_password.is_empty());
        assert!(app.login_error.is_none());
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Record interaction on any input event
        if ui.input(|i| !i.events.is_empty()) {
            self.record_interaction();
        }

        // Auto-lock check
        self.check_auto_lock();

        // Render the current screen
        match self.screen {
            Screen::Login => crate::screens::login::show(ui, self),
            Screen::Create => crate::screens::create::show(ui, self),
            Screen::Explorer => crate::screens::explorer::show(ui, self),
            Screen::Settings => crate::screens::settings::show(ui, self),
        }

        // Status bar
        if let Some(ref msg) = self.status_message {
            ui.separator();
            ui.small(msg);
        }

        // Request repaint if dirty
        if self.is_dirty {
            ui.ctx().request_repaint();
            self.is_dirty = false;
        }
    }
}
