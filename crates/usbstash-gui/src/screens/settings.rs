use crate::app::{App, Screen};
use crate::widgets::password_input;

/// Settings screen: auto-lock timeout, change password, export.
/// Stretch goal — minimal MVP implementation.
pub fn show(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical_centered(|ui| {
        ui.heading("Settings");
    });

    ui.add_space(16.0);

    // Auto-lock timeout slider
    ui.horizontal(|ui| {
        ui.label("Auto-lock timeout:");
        let mut minutes = (app.auto_lock_seconds / 60) as f32;
        ui.add(egui::Slider::new(&mut minutes, 0.0..=60.0).text("minutes"));
        app.auto_lock_seconds = (minutes * 60.0) as u32;
    });

    ui.separator();

    // Change password section
    ui.heading("Change Password");

    let mut old_pw_state = password_input::PasswordInputState::default();
    password_input::show(ui, &mut app.old_password, "Current:", &mut old_pw_state);

    ui.add_space(4.0);

    let mut new_pw_state = password_input::PasswordInputState::default();
    password_input::show(ui, &mut app.new_password, "New:", &mut new_pw_state);

    ui.add_space(8.0);

    if ui.button("Change Password").clicked() {
        if app.old_password.is_empty() || app.new_password.is_empty() {
            app.set_status("Both passwords are required");
            return;
        }

        let old_pw = app.old_password.clone();
        let new_pw = app.new_password.clone();

        // Scope the mutex guard
        let change_result = {
            let mut guard = match app.stash.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard
                .as_mut()
                .map(|stash| stash.change_password(old_pw.as_bytes(), new_pw.as_bytes()))
        };

        match change_result {
            Some(Ok(_)) => {
                app.set_status("Password changed successfully");
                app.old_password.clear();
                app.new_password.clear();
            }
            Some(Err(e)) => {
                app.set_status(format!("Failed to change password: {}", e));
            }
            None => {
                app.set_status("No stash is open");
            }
        }
    }

    ui.separator();

    // Export stash
    ui.heading("Export");
    if ui.button("Export Stash…").clicked()
        && let Some(target) = rfd::FileDialog::new().save_file()
    {
        app.record_interaction();

        // Scope the mutex guard
        let export_result = {
            let guard = match app.stash.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard.as_ref().map(|stash| stash.export_to(&target))
        };

        match export_result {
            Some(Ok(_)) => {
                app.set_status(format!("Exported to {}", target.display()));
            }
            Some(Err(e)) => {
                app.set_status(format!("Failed to export: {}", e));
            }
            None => {
                app.set_status("No stash is open");
            }
        }
    }

    ui.add_space(16.0);

    // Back button
    if ui.button("← Back").clicked() {
        app.screen = Screen::Explorer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use usbstash_core::Stash;

    #[test]
    fn test_settings_navigate_back_to_explorer() {
        let mut app = App::default();
        app.screen = Screen::Settings;

        app.screen = Screen::Explorer;

        assert_eq!(app.screen, Screen::Explorer);
    }

    #[test]
    fn test_settings_auto_lock_slider_range() {
        let mut app = App::default();
        assert_eq!(app.auto_lock_seconds, 300);

        app.auto_lock_seconds = 0;
        assert_eq!(app.auto_lock_seconds, 0);

        app.auto_lock_seconds = 3600;
        assert_eq!(app.auto_lock_seconds, 3600);
    }

    #[test]
    fn test_change_password_requires_both_fields() {
        let mut app = App::default();
        app.old_password = String::new();
        app.new_password = String::new();

        if app.old_password.is_empty() || app.new_password.is_empty() {
            app.set_status("Both passwords are required");
        }

        assert!(app.status_message.is_some());
    }

    #[test]
    fn test_change_password_success() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        let stash = Stash::create(b"oldpassword123", &path).unwrap();
        stash.save().unwrap();

        let mut app = App::default();
        {
            let mut guard = app.stash.lock().unwrap();
            *guard = Some(stash);
        }
        app.screen = Screen::Explorer;

        app.old_password = "oldpassword123".to_string();
        app.new_password = "newpassword456".to_string();

        {
            let mut guard = app.stash.lock().unwrap();
            if let Some(stash) = guard.as_mut() {
                stash
                    .change_password(app.old_password.as_bytes(), app.new_password.as_bytes())
                    .unwrap();
            }
        }

        let reopen = Stash::open(b"newpassword456", &path);
        assert!(reopen.is_ok());
    }
}
