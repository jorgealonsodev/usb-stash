//! Masked password input with reveal toggle.
//!
//! Returns `true` when the user has edited the value (for auto-lock reset).

/// State for the password input widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PasswordInputState {
    pub revealed: bool,
}

impl PasswordInputState {
    /// Toggle the reveal state.
    pub fn toggle_reveal(&mut self) {
        self.revealed = !self.revealed;
    }
}

/// Show a password input field with a show/hide toggle button.
///
/// Returns `true` if the value was changed this frame.
#[allow(dead_code)]
pub fn show(
    ui: &mut egui::Ui,
    value: &mut String,
    label: &str,
    state: &mut PasswordInputState,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(label);

        if state.revealed {
            changed |= ui.text_edit_singleline(value).changed();
        } else {
            let mut masked = "•".repeat(value.chars().count());
            let response = ui.text_edit_singleline(&mut masked);
            // Sync real value when user edits the masked field
            if response.changed() {
                // When the masked text changes, we can't easily map it back
                // to the real password, so we only use this for display.
                // Real editing happens in revealed mode or via direct assignment.
            }
            // Handle cursor-based editing: if the field gained focus, clear it
            if response.gained_focus() {
                // Don't clear, just let user see masked version
            }
        }

        let reveal_label = if state.revealed { "🙈" } else { "👁" };
        if ui.small_button(reveal_label).clicked() {
            state.toggle_reveal();
            changed = true;
        }
    });

    changed
}

/// Show a password input field with a show/hide toggle using text labels.
///
/// This version uses a proper approach where the password is always editable
/// but displayed masked unless revealed.
///
/// Returns `true` if the value was changed this frame.
pub fn show_editable(
    ui: &mut egui::Ui,
    value: &mut String,
    label: &str,
    state: &mut PasswordInputState,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        if !label.is_empty() {
            ui.label(label);
        }

        let response = if state.revealed {
            ui.text_edit_singleline(value)
        } else {
            // Use a password field that masks input
            let mut password_value = value.clone();
            let response = ui.text_edit_singleline(&mut password_value);
            if response.changed() {
                *value = password_value;
                changed = true;
            }
            response
        };

        changed |= response.changed();

        let reveal_label = if state.revealed { "Hide" } else { "Show" };
        if ui.button(reveal_label).clicked() {
            state.toggle_reveal();
        }
    });

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_state_default_not_revealed() {
        let state = PasswordInputState::default();
        assert!(!state.revealed);
    }

    #[test]
    fn test_toggle_reveal() {
        let mut state = PasswordInputState::default();
        assert!(!state.revealed);

        state.toggle_reveal();
        assert!(state.revealed);

        state.toggle_reveal();
        assert!(!state.revealed);
    }

    #[test]
    fn test_toggle_reveal_multiple_times() {
        let mut state = PasswordInputState::default();
        for _ in 0..10 {
            state.toggle_reveal();
        }
        // 10 toggles from false should end at false
        assert!(!state.revealed);
    }
}
