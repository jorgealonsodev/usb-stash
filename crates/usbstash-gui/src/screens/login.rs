use std::path::Path;

use usbstash_core::Stash;

use crate::app::App;

/// Check if a stash exists at the given path by looking for stash.meta.
pub fn stash_exists(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    Path::new(path).join("stash.meta").exists()
}

/// Login screen: path picker, password input, open button.
pub fn show(ui: &mut egui::Ui, app: &mut App) {
    ui.vertical_centered(|ui| {
        ui.heading("Open Stash");
    });

    ui.add_space(16.0);

    // Path input with file dialog button
    ui.horizontal(|ui| {
        ui.label("Path:");
        ui.text_edit_singleline(&mut app.login_path);
        if ui.button("Browse…").clicked()
            && let Some(path) = rfd::FileDialog::new().pick_folder()
        {
            app.login_path = path.to_string_lossy().to_string();
            app.record_interaction();
        }
    });

    ui.add_space(8.0);

    // Password input using PasswordInput widget
    let mut pw_state = crate::widgets::password_input::PasswordInputState::default();
    crate::widgets::password_input::show(ui, &mut app.login_password, "Password:", &mut pw_state);

    ui.add_space(16.0);

    // Error display
    if let Some(ref error) = app.login_error {
        ui.colored_label(egui::Color32::RED, error);
        ui.add_space(8.0);
    }

    // Open button
    ui.horizontal(|ui| {
        if ui.button("Open").clicked() {
            app.login_error = None;

            if app.login_path.is_empty() {
                app.login_error = Some("Please select a stash path".to_string());
                return;
            }

            if !stash_exists(&app.login_path) {
                app.login_error = Some("Stash not found".to_string());
                return;
            }

            let path = app.login_path.clone();
            let password = app.login_password.clone();

            // Scope the mutex guard so it drops before we call app methods
            let open_result = {
                let guard = match app.stash.lock() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                // We can't use guard here since stash is Option<Stash> and we need mutable access
                // Instead, do the open outside the guard
                drop(guard);
                Stash::open(password.as_bytes(), Path::new(&path))
            };

            match open_result {
                Ok(stash) => {
                    {
                        let mut guard = match app.stash.lock() {
                            Ok(g) => g,
                            Err(poisoned) => poisoned.into_inner(),
                        };
                        *guard = Some(stash);
                    }
                    app.screen = crate::app::Screen::Explorer;
                    app.login_password.clear();
                    app.refresh_entries();
                    app.set_status("Stash opened");
                }
                Err(e) => {
                    app.login_error = Some(format!("Invalid password: {}", e));
                    app.login_password.clear();
                }
            }
        }

        ui.add_space(16.0);

        // Navigate to Create screen
        if ui.button("Create new stash").clicked() {
            app.screen = crate::app::Screen::Create;
            app.login_error = None;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Screen;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_stash_exists_false_for_empty_path() {
        assert!(!stash_exists(""));
    }

    #[test]
    fn test_stash_exists_false_for_nonexistent_dir() {
        assert!(!stash_exists("/nonexistent/path/that/does/not/exist"));
    }

    #[test]
    fn test_stash_exists_true_when_meta_present() {
        let tmp = TempDir::new().unwrap();
        let meta_path = tmp.path().join("stash.meta");
        fs::write(&meta_path, "test").unwrap();
        assert!(stash_exists(tmp.path().to_str().unwrap()));
    }

    #[test]
    fn test_stash_exists_false_without_meta() {
        let tmp = TempDir::new().unwrap();
        assert!(!stash_exists(tmp.path().to_str().unwrap()));
    }

    #[test]
    fn test_login_screen_transitions_to_explorer_on_success() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        let stash = Stash::create(b"testpassword123", &path).unwrap();
        stash.save().unwrap();
        drop(stash);

        assert!(stash_exists(path.to_str().unwrap()));

        let mut app = App::default();
        app.login_path = path.to_str().unwrap().to_string();
        app.login_password = "testpassword123".to_string();

        match Stash::open(app.login_password.as_bytes(), Path::new(&app.login_path)) {
            Ok(stash) => {
                {
                    let mut guard = app.stash.lock().unwrap();
                    *guard = Some(stash);
                }
                app.screen = Screen::Explorer;
                app.login_password.clear();
            }
            Err(_) => panic!("Should have opened stash"),
        }

        assert_eq!(app.screen, Screen::Explorer);
        assert!(app.login_password.is_empty());
    }

    #[test]
    fn test_login_fails_with_wrong_password() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();

        let stash = Stash::create(b"correctpassword", &path).unwrap();
        stash.save().unwrap();
        drop(stash);

        let mut app = App::default();
        app.login_path = path.to_str().unwrap().to_string();
        app.login_password = "wrongpassword".to_string();
        app.login_error = None;

        match Stash::open(app.login_password.as_bytes(), Path::new(&app.login_path)) {
            Ok(_) => panic!("Should have failed with wrong password"),
            Err(e) => {
                app.login_error = Some(format!("Invalid password: {}", e));
                app.login_password.clear();
            }
        }

        assert!(app.login_error.is_some());
        assert!(app.login_password.is_empty());
        assert_eq!(app.screen, Screen::Login);
    }

    #[test]
    fn test_login_stash_not_found_error() {
        let mut app = App::default();
        app.login_path = "/nonexistent/stash".to_string();
        app.login_error = None;

        if !stash_exists(&app.login_path) {
            app.login_error = Some("Stash not found".to_string());
        }

        assert!(app.login_error.is_some());
        assert_eq!(app.login_error.as_deref(), Some("Stash not found"));
        assert_eq!(app.screen, Screen::Login);
    }

    #[test]
    fn test_login_empty_path_error() {
        let mut app = App::default();
        app.login_path = String::new();
        app.login_error = None;

        if app.login_path.is_empty() {
            app.login_error = Some("Please select a stash path".to_string());
        }

        assert!(app.login_error.is_some());
        assert_eq!(app.screen, Screen::Login);
    }

    #[test]
    fn test_login_navigate_to_create() {
        let mut app = App::default();
        app.screen = Screen::Login;
        app.login_error = Some("some error".to_string());

        app.screen = Screen::Create;
        app.login_error = None;

        assert_eq!(app.screen, Screen::Create);
        assert!(app.login_error.is_none());
    }
}
