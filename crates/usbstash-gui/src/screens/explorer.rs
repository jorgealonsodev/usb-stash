use crate::app::{App, Screen};
use crate::widgets::file_table::{SortColumn, show as show_file_table};
use crate::widgets::tree_view::{build_tree, show as show_tree_view};

/// Explorer screen: tree view sidebar, file table, search, action buttons.
pub fn show(ui: &mut egui::Ui, app: &mut App) {
    // Top bar with search and actions
    ui.horizontal(|ui| {
        ui.label("Search:");
        ui.text_edit_singleline(&mut app.search_query);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Lock button
            if ui.button("🔒 Lock").clicked() {
                app.lock_stash();
            }

            // Settings button
            if ui.button("⚙ Settings").clicked() {
                app.screen = Screen::Settings;
            }
        });
    });

    ui.separator();

    // Main content: left tree, right file table
    ui.horizontal(|ui| {
        // Left panel: Tree view
        ui.vertical(|ui| {
            ui.set_min_width(200.0);
            ui.set_max_width(250.0);
            ui.heading("Folders");
            ui.separator();

            let tree_nodes = build_tree(&app.entries);

            egui::ScrollArea::vertical().show(ui, |ui| {
                show_tree_view(
                    ui,
                    &tree_nodes,
                    &mut app.expanded_paths,
                    &mut app.selected_path,
                );
            });
        });

        ui.separator();

        // Right panel: File table
        ui.vertical(|ui| {
            ui.heading("Files");
            ui.separator();

            // Filter entries by selected path and search query
            let filtered: Vec<_> = app
                .entries
                .iter()
                .filter(|e| {
                    // Filter by selected directory
                    if let Some(ref selected) = app.selected_path {
                        let entry_path = e.path();
                        let parent = entry_path
                            .rsplit_once('/')
                            .map(|(p, _)| p.to_string())
                            .unwrap_or_default();
                        if parent != *selected && entry_path != *selected {
                            return false;
                        }
                    }

                    // Filter by search query
                    if !app.search_query.is_empty() {
                        let name = entry_filename(e.path());
                        if !name
                            .to_lowercase()
                            .contains(&app.search_query.to_lowercase())
                        {
                            return false;
                        }
                    }

                    true
                })
                .cloned()
                .collect();

            // Sort state (per-frame for now)
            let mut sort = SortColumn::Name;

            egui::ScrollArea::vertical().show(ui, |ui| {
                show_file_table(ui, &filtered, &mut sort, &mut app.selected_path);
            });

            // Action buttons
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("➕ Add").clicked()
                    && let Some(file_path) = rfd::FileDialog::new().pick_file()
                {
                    app.record_interaction();
                    let content = match std::fs::read(&file_path) {
                        Ok(c) => c,
                        Err(e) => {
                            app.set_status(format!("Failed to read file: {}", e));
                            return;
                        }
                    };
                    let relative_path = file_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| file_path.to_string_lossy().to_string());

                    // Scope the mutex guard
                    let add_result = {
                        let mut guard = match app.stash.lock() {
                            Ok(g) => g,
                            Err(poisoned) => poisoned.into_inner(),
                        };
                        guard
                            .as_mut()
                            .map(|stash| stash.add_entry(relative_path.clone(), content))
                    };

                    match add_result {
                        Some(Ok(_)) => {
                            app.is_dirty = true;
                            app.set_status(format!("Added: {}", relative_path));
                        }
                        Some(Err(e)) => {
                            app.set_status(format!("Failed to add: {}", e));
                        }
                        None => {
                            app.set_status("No stash is open");
                        }
                    }
                }

                if ui.button("📥 Extract").clicked() {
                    let selected_path = app.selected_path.clone();
                    if let Some(ref selected) = selected_path {
                        // Scope the mutex guard
                        let extract_result = {
                            let guard = match app.stash.lock() {
                                Ok(g) => g,
                                Err(poisoned) => poisoned.into_inner(),
                            };
                            if let Some(stash) = guard.as_ref() {
                                match stash.get_entry(selected) {
                                    Ok(entry) => Some(entry.content().to_vec()),
                                    Err(e) => {
                                        drop(guard);
                                        app.set_status(format!("Failed to get entry: {}", e));
                                        None
                                    }
                                }
                            } else {
                                drop(guard);
                                app.set_status("No stash is open");
                                None
                            }
                        };

                        if let Some(content) = extract_result {
                            let default_name = entry_filename(selected);
                            if let Some(save_path) = rfd::FileDialog::new()
                                .set_file_name(default_name)
                                .save_file()
                            {
                                app.record_interaction();
                                match std::fs::write(&save_path, content) {
                                    Ok(_) => {
                                        app.set_status(format!(
                                            "Extracted: {}",
                                            save_path.display()
                                        ));
                                    }
                                    Err(e) => {
                                        app.set_status(format!("Failed to extract: {}", e));
                                    }
                                }
                            }
                        }
                    } else {
                        app.set_status("Select a file to extract");
                    }
                }

                if ui.button("🗑 Delete").clicked() {
                    if app.selected_path.is_some() {
                        app.show_delete_confirmation = true;
                    } else {
                        app.set_status("Select a file to delete");
                    }
                }

                if ui.button("✏️ Rename").clicked() {
                    if let Some(ref selected) = app.selected_path {
                        app.rename_entry_path = Some(selected.clone());
                        app.rename_new_name = entry_filename(selected).to_string();
                    } else {
                        app.set_status("Select a file to rename");
                    }
                }
            });

            // ─── Delete Confirmation Dialog ────────────────────────────
            if app.show_delete_confirmation {
                if let Some(selected) = app.selected_path.clone() {
                    let file_name = entry_filename(&selected);
                    egui::Window::new("Confirm Delete")
                        .collapsible(false)
                        .resizable(false)
                        .default_width(320.0)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ui.ctx(), |ui| {
                            ui.label(format!(
                                "Are you sure you want to delete \"{}\"?",
                                file_name
                            ));
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button("Cancel").clicked() {
                                    app.show_delete_confirmation = false;
                                }
                                if ui.button("Delete").clicked() {
                                    let delete_result = {
                                        let mut guard = match app.stash.lock() {
                                            Ok(g) => g,
                                            Err(poisoned) => poisoned.into_inner(),
                                        };
                                        guard.as_mut().map(|stash| stash.remove_entry(&selected))
                                    };

                                    match delete_result {
                                        Some(Ok(true)) => {
                                            app.is_dirty = true;
                                            app.selected_path = None;
                                            app.set_status(format!("Deleted: {}", selected));
                                        }
                                        Some(Ok(false)) => {
                                            app.set_status("Entry not found");
                                        }
                                        Some(Err(e)) => {
                                            app.set_status(format!("Failed to delete: {}", e));
                                        }
                                        None => {
                                            app.set_status("No stash is open");
                                        }
                                    }
                                    app.show_delete_confirmation = false;
                                }
                            });
                        });
                } else {
                    app.show_delete_confirmation = false;
                }
            }

            // ─── Rename Dialog ─────────────────────────────────────────
            if let Some(entry_path) = app.rename_entry_path.clone() {
                egui::Window::new("Rename Entry")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(320.0)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {
                        ui.label("New name:");
                        ui.text_edit_singleline(&mut app.rename_new_name);
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                app.rename_entry_path = None;
                                app.rename_new_name.clear();
                            }
                            if ui.button("Rename").clicked() {
                                app.rename_entry_path = None;
                                let new_name = app.rename_new_name.clone();
                                // Build new path: same directory, new filename
                                let new_path = if let Some((dir, _)) = entry_path.rsplit_once('/') {
                                    format!("{}/{}", dir, new_name)
                                } else {
                                    new_name.clone()
                                };

                                let rename_result = {
                                    let mut guard = match app.stash.lock() {
                                        Ok(g) => g,
                                        Err(poisoned) => poisoned.into_inner(),
                                    };
                                    guard
                                        .as_mut()
                                        .map(|stash| stash.rename_entry(&entry_path, &new_path))
                                };

                                match rename_result {
                                    Some(Ok(())) => {
                                        app.is_dirty = true;
                                        app.selected_path = Some(new_path.clone());
                                        app.set_status(format!("Renamed to: {}", new_path));
                                    }
                                    Some(Err(e)) => {
                                        app.set_status(format!("Failed to rename: {}", e));
                                    }
                                    None => {
                                        app.set_status("No stash is open");
                                    }
                                }
                                app.rename_new_name.clear();
                            }
                        });
                    });
            }

            // Status bar
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                let file_count = app.entries.len();
                let total_size: u64 = app.entries.iter().map(|e| e.size()).sum();
                ui.small(format!("{} files", file_count));
                ui.small("·");
                ui.small(format_bytes(total_size));
                if app.is_dirty {
                    ui.small("·");
                    ui.small(egui::RichText::new("●").color(egui::Color32::YELLOW));
                    if ui.small_button("Save").clicked() {
                        app.record_interaction();
                        // Scope the mutex guard to avoid borrow conflicts
                        let save_result = {
                            let mut guard = match app.stash.lock() {
                                Ok(g) => g,
                                Err(poisoned) => poisoned.into_inner(),
                            };
                            guard.as_mut().map(|stash| stash.save())
                        };
                        match save_result {
                            Some(Ok(_)) => {
                                app.is_dirty = false;
                                app.set_status("Saved");
                            }
                            Some(Err(e)) => {
                                app.set_status(format!("Save failed: {}", e));
                            }
                            None => {
                                app.set_status("No stash is open");
                            }
                        }
                    }
                }
            });
        });
    });
}

/// Extract the filename from a path.
fn entry_filename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Format bytes into a human-readable string.
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use usbstash_core::Stash;

    #[test]
    fn test_entry_filename_simple() {
        assert_eq!(entry_filename("file.txt"), "file.txt");
    }

    #[test]
    fn test_entry_filename_nested() {
        assert_eq!(entry_filename("docs/report.pdf"), "report.pdf");
    }

    #[test]
    fn test_entry_filename_deep() {
        assert_eq!(entry_filename("a/b/c/d/file.txt"), "file.txt");
    }

    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn test_format_bytes_kb() {
        assert_eq!(format_bytes(1024), "1.0 KB");
    }

    #[test]
    fn test_format_bytes_mb() {
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_format_bytes_gb() {
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_explorer_lock_transitions_to_login() {
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
        app.entries.push(usbstash_core::StashEntry::new(
            uuid::Uuid::new_v4(),
            "test.txt".to_string(),
            0,
            0,
            100,
            "text/plain".to_string(),
            vec![0; 100],
        ));

        app.lock_stash();

        assert_eq!(app.screen, Screen::Login);
        assert!(app.entries.is_empty());
        assert!(app.selected_path.is_none());
    }

    #[test]
    fn test_explorer_add_entry_updates_entries() {
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

        {
            let mut guard = app.stash.lock().unwrap();
            if let Some(stash) = guard.as_mut() {
                stash
                    .add_entry("hello.txt".to_string(), b"hello world".to_vec())
                    .unwrap();
            }
        }

        app.refresh_entries();

        assert_eq!(app.entries.len(), 1);
        assert_eq!(app.entries[0].path(), "hello.txt");
    }

    #[test]
    fn test_explorer_delete_entry() {
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

        {
            let mut guard = app.stash.lock().unwrap();
            if let Some(stash) = guard.as_mut() {
                stash
                    .add_entry("delete_me.txt".to_string(), b"content".to_vec())
                    .unwrap();
            }
        }
        app.refresh_entries();
        assert_eq!(app.entries.len(), 1);

        {
            let mut guard = app.stash.lock().unwrap();
            if let Some(stash) = guard.as_mut() {
                stash.remove_entry("delete_me.txt").unwrap();
            }
        }
        app.refresh_entries();

        assert_eq!(app.entries.len(), 0);
    }

    #[test]
    fn test_explorer_navigate_to_settings() {
        let mut app = App::default();
        app.screen = Screen::Explorer;

        app.screen = Screen::Settings;

        assert_eq!(app.screen, Screen::Settings);
    }

    #[test]
    fn test_status_bar_total_size() {
        let mut app = App::default();

        let e1 = usbstash_core::StashEntry::new(
            uuid::Uuid::new_v4(),
            "a.txt".to_string(),
            0,
            0,
            100,
            "text/plain".to_string(),
            vec![],
        );
        let e2 = usbstash_core::StashEntry::new(
            uuid::Uuid::new_v4(),
            "b.txt".to_string(),
            0,
            0,
            200,
            "text/plain".to_string(),
            vec![],
        );
        app.entries.push(e1);
        app.entries.push(e2);

        let total: u64 = app.entries.iter().map(|e| e.size()).sum();
        assert_eq!(total, 300);
    }

    #[test]
    fn test_delete_confirmation_starts_closed() {
        let app = App::default();
        assert!(!app.show_delete_confirmation);
    }

    #[test]
    fn test_rename_state_starts_empty() {
        let app = App::default();
        assert!(app.rename_entry_path.is_none());
        assert!(app.rename_new_name.is_empty());
    }

    #[test]
    fn test_explorer_rename_entry_in_core() {
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

        {
            let mut guard = app.stash.lock().unwrap();
            if let Some(stash) = guard.as_mut() {
                stash
                    .add_entry("original.txt".to_string(), b"content".to_vec())
                    .unwrap();
            }
        }
        app.refresh_entries();
        assert_eq!(app.entries.len(), 1);

        // Simulate rename via core
        {
            let mut guard = app.stash.lock().unwrap();
            if let Some(stash) = guard.as_mut() {
                stash.rename_entry("original.txt", "renamed.txt").unwrap();
            }
        }
        app.refresh_entries();

        assert_eq!(app.entries.len(), 1);
        assert_eq!(app.entries[0].path(), "renamed.txt");
    }
}
