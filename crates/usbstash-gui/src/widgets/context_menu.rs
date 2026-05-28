#![allow(dead_code)]
//! Context menu widget for entry actions (Extract, Rename, Delete).
//! Not yet wired into the explorer table rows.

/// Action available from the context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextAction {
    Extract,
    Rename,
    Delete,
}

impl ContextAction {
    /// Get the display label for this action.
    pub fn label(&self) -> &'static str {
        match self {
            ContextAction::Extract => "Extract",
            ContextAction::Rename => "Rename",
            ContextAction::Delete => "Delete",
        }
    }
}

/// State for the context menu widget.
#[derive(Debug, Clone, Default)]
pub struct ContextMenuState {
    /// Whether the context menu is currently open.
    pub open: bool,
    /// The path of the entry the menu was opened for.
    pub entry_path: Option<String>,
    /// The screen position where the menu was opened.
    pub position: Option<egui::Pos2>,
}

impl ContextMenuState {
    /// Open the context menu for the given entry at the given position.
    pub fn open_for(&mut self, entry_path: String, position: egui::Pos2) {
        self.open = true;
        self.entry_path = Some(entry_path);
        self.position = Some(position);
    }

    /// Close the context menu.
    pub fn close(&mut self) {
        self.open = false;
        self.entry_path = None;
        self.position = None;
    }

    /// Check if the menu is open for a specific entry.
    pub fn is_open_for(&self, entry_path: &str) -> bool {
        self.open && self.entry_path.as_deref() == Some(entry_path)
    }
}

/// Show a context menu overlay within the parent egui area.
///
/// This renders an egui `Area` positioned at the stored cursor position,
/// with buttons for each available action.
///
/// Returns the selected action if the user clicked one, or `None`.
/// After returning an action (or when the user clicks outside), the state is closed.
pub fn show(ui: &mut egui::Ui, state: &mut ContextMenuState) -> Option<ContextAction> {
    if !state.open {
        return None;
    }

    let position = match state.position {
        Some(p) => p,
        None => {
            state.close();
            return None;
        }
    };

    let mut selected_action: Option<ContextAction> = None;

    // Use egui Area for the context menu overlay
    let id = egui::Id::new("context_menu");
    egui::Area::new(id)
        .fixed_pos(position)
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(120.0);

                for action in [
                    ContextAction::Extract,
                    ContextAction::Rename,
                    ContextAction::Delete,
                ] {
                    let label = action.label();
                    let button = ui.button(label);

                    if button.clicked() {
                        selected_action = Some(action);
                    }
                }
            });
        });

    // If an action was selected, close the state
    if selected_action.is_some() {
        state.close();
    }

    selected_action
}

/// Alternative: show context menu using egui's built-in context_menu mechanism.
/// This is called when the user right-clicks on an entry row.
pub fn show_for_entry(
    _ui: &mut egui::Ui,
    entry_path: &str,
    response: &egui::Response,
) -> Option<ContextAction> {
    let mut selected_action: Option<ContextAction> = None;

    response.context_menu(|ui| {
        ui.set_min_width(120.0);

        for action in [
            ContextAction::Extract,
            ContextAction::Rename,
            ContextAction::Delete,
        ] {
            let label = action.label();
            let button = ui.button(label);

            if button.clicked() {
                selected_action = Some(action);
                ui.close();
            }
        }

        // Show the entry path as a label
        ui.separator();
        ui.small(format!("Entry: {}", entry_path));
    });

    selected_action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_state_default_closed() {
        let state = ContextMenuState::default();
        assert!(!state.open);
        assert!(state.entry_path.is_none());
        assert!(state.position.is_none());
    }

    #[test]
    fn test_open_for_sets_state() {
        let mut state = ContextMenuState::default();
        let pos = egui::Pos2::new(100.0, 200.0);
        state.open_for("docs/file.txt".to_string(), pos);

        assert!(state.open);
        assert_eq!(state.entry_path, Some("docs/file.txt".to_string()));
        assert_eq!(state.position, Some(pos));
    }

    #[test]
    fn test_close_clears_state() {
        let mut state = ContextMenuState::default();
        state.open_for("test.txt".to_string(), egui::Pos2::ZERO);
        state.close();

        assert!(!state.open);
        assert!(state.entry_path.is_none());
        assert!(state.position.is_none());
    }

    #[test]
    fn test_is_open_for_matching() {
        let mut state = ContextMenuState::default();
        state.open_for("docs/file.txt".to_string(), egui::Pos2::ZERO);

        assert!(state.is_open_for("docs/file.txt"));
        assert!(!state.is_open_for("other.txt"));
    }

    #[test]
    fn test_is_open_for_when_closed() {
        let state = ContextMenuState::default();
        assert!(!state.is_open_for("anything.txt"));
    }

    #[test]
    fn test_context_action_labels() {
        assert_eq!(ContextAction::Extract.label(), "Extract");
        assert_eq!(ContextAction::Rename.label(), "Rename");
        assert_eq!(ContextAction::Delete.label(), "Delete");
    }

    #[test]
    fn test_context_action_equality() {
        let a = ContextAction::Extract;
        let b = ContextAction::Extract;
        let c = ContextAction::Delete;

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_open_for_replaces_previous() {
        let mut state = ContextMenuState::default();
        state.open_for("first.txt".to_string(), egui::Pos2::new(0.0, 0.0));
        state.open_for("second.txt".to_string(), egui::Pos2::new(50.0, 50.0));

        assert_eq!(state.entry_path, Some("second.txt".to_string()));
        assert_eq!(state.position, Some(egui::Pos2::new(50.0, 50.0)));
    }
}
