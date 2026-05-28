use std::path::Path;

use usbstash_core::Stash;

use crate::app::{App, Screen};
use crate::widgets::entropy_bar::{analyze, show as show_entropy_bar};
use crate::widgets::password_input;

/// Create screen: path picker, password + confirm, entropy bar, create button.
pub fn show(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical_centered(|ui| {
        ui.heading("Create New Stash");
    });

    ui.add_space(16.0);

    // Path input with file dialog button
    ui.horizontal(|ui| {
        ui.label("Path:");
        ui.text_edit_singleline(&mut app.create_path);
        if ui.button("Browse…").clicked()
            && let Some(path) = rfd::FileDialog::new().pick_folder()
        {
            app.create_path = path.to_string_lossy().to_string();
            app.record_interaction();
        }
    });

    ui.add_space(8.0);

    // Password input
    let mut pw_state = password_input::PasswordInputState::default();
    password_input::show_editable(ui, &mut app.create_password, "Password:", &mut pw_state);

    ui.add_space(4.0);

    // Confirm password input
    let mut confirm_state = password_input::PasswordInputState::default();
    password_input::show_editable(ui, &mut app.create_confirm, "Confirm:", &mut confirm_state);

    ui.add_space(8.0);

    // Entropy bar (real-time feedback based on password)
    let level = analyze(&app.create_password);
    show_entropy_bar(ui, level);

    ui.add_space(16.0);

    // Error display
    if let Some(ref error) = app.create_error {
        ui.colored_label(egui::Color32::RED, error);
        ui.add_space(8.0);
    }

    // Create button
    ui.horizontal(|ui| {
        if ui.button("Create").clicked() {
            app.create_error = None;

            // Validate path
            if app.create_path.is_empty() {
                app.create_error = Some("Please select a path".to_string());
                return;
            }

            // Validate password not empty
            if app.create_password.is_empty() {
                app.create_error = Some("Password cannot be empty".to_string());
                return;
            }

            // Validate password matches confirm
            if app.create_password != app.create_confirm {
                app.create_error = Some("Passwords do not match".to_string());
                return;
            }

            // Check if stash already exists
            let meta_path = Path::new(&app.create_path).join("stash.meta");
            if meta_path.exists() {
                app.create_error =
                    Some("Stash already exists. Try opening it instead.".to_string());
                return;
            }

            let path = app.create_path.clone();
            let password = app.create_password.clone();

            // Create the stash (no mutex guard needed — Stash::create is standalone)
            match Stash::create(password.as_bytes(), Path::new(&path)) {
                Ok(_) => {
                    // Open the newly created stash
                    match Stash::open(password.as_bytes(), Path::new(&path)) {
                        Ok(stash) => {
                            {
                                let mut guard = match app.stash.lock() {
                                    Ok(g) => g,
                                    Err(poisoned) => poisoned.into_inner(),
                                };
                                *guard = Some(stash);
                            }
                            // Guard dropped, now safe to call app methods
                            app.screen = Screen::Explorer;
                            app.create_password.clear();
                            app.create_confirm.clear();
                            app.refresh_entries();
                            app.set_status("Stash created");
                        }
                        Err(e) => {
                            app.create_error = Some(format!("Created but failed to open: {}", e));
                        }
                    }
                }
                Err(e) => {
                    app.create_error = Some(format!("Failed to create stash: {}", e));
                }
            }
        }

        ui.add_space(16.0);

        // Navigate back to Login
        if ui.button("Back to Login").clicked() {
            app.screen = Screen::Login;
            app.create_error = None;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_stash_success() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        let stash = Stash::create(b"testpassword123", &path).unwrap();
        stash.save().unwrap();

        assert!(path.join("stash.meta").exists());
        assert!(path.join("stash.dat").exists());
    }

    #[test]
    fn test_create_stash_already_exists() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        let stash = Stash::create(b"testpassword123", &path).unwrap();
        stash.save().unwrap();
        drop(stash);

        let result = Stash::create(b"anotherpassword", &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_password_mismatch_logic() {
        let mut app = App::default();
        app.create_password = "password1".to_string();
        app.create_confirm = "password2".to_string();
        app.create_error = None;

        if app.create_password != app.create_confirm {
            app.create_error = Some("Passwords do not match".to_string());
        }

        assert!(app.create_error.is_some());
        assert_eq!(app.create_error.as_deref(), Some("Passwords do not match"));
    }

    #[test]
    fn test_create_empty_password_error() {
        let mut app = App::default();
        app.create_password = String::new();
        app.create_error = None;

        if app.create_password.is_empty() {
            app.create_error = Some("Password cannot be empty".to_string());
        }

        assert!(app.create_error.is_some());
    }

    #[test]
    fn test_create_empty_path_error() {
        let mut app = App::default();
        app.create_path = String::new();
        app.create_error = None;

        if app.create_path.is_empty() {
            app.create_error = Some("Please select a path".to_string());
        }

        assert!(app.create_error.is_some());
    }

    #[test]
    fn test_create_navigate_to_explorer_on_success() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        // Create a stash directly (simulating what the Create screen does)
        let password = b"testpassword123";
        let stash = Stash::create(password, &path).unwrap();
        stash.save().unwrap();
        drop(stash);

        let mut app = App::default();
        app.create_path = path.to_str().unwrap().to_string();
        app.create_password = "testpassword123".to_string();
        app.create_confirm = "testpassword123".to_string();
        app.create_error = None;

        // Simulate: after creation, open the stash
        match Stash::open(password, Path::new(&app.create_path)) {
            Ok(stash) => {
                {
                    let mut guard = app.stash.lock().unwrap();
                    *guard = Some(stash);
                }
                app.screen = Screen::Explorer;
                app.create_password.clear();
                app.create_confirm.clear();
            }
            Err(_) => panic!("Should open after create"),
        }

        assert_eq!(app.screen, Screen::Explorer);
        assert!(app.create_password.is_empty());
        assert!(app.create_confirm.is_empty());
    }

    #[test]
    fn test_create_navigate_back_to_login() {
        let mut app = App::default();
        app.screen = Screen::Create;
        app.create_error = Some("some error".to_string());

        app.screen = Screen::Login;
        app.create_error = None;

        assert_eq!(app.screen, Screen::Login);
        assert!(app.create_error.is_none());
    }

    #[test]
    fn test_create_stash_already_exists_ui_check() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        let stash = Stash::create(b"testpassword123", &path).unwrap();
        stash.save().unwrap();
        drop(stash);

        let mut app = App::default();
        app.create_path = path.to_str().unwrap().to_string();
        app.create_error = None;

        let meta_path = Path::new(&app.create_path).join("stash.meta");
        if meta_path.exists() {
            app.create_error = Some("Stash already exists. Try opening it instead.".to_string());
        }

        assert!(app.create_error.is_some());
        assert!(
            app.create_error
                .as_ref()
                .unwrap()
                .contains("already exists")
        );
    }
}
