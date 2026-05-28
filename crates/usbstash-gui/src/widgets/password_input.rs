//! Masked password input with reveal toggle.
//!
//! Uses egui's built-in password masking when not revealed.

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
pub fn show(
    ui: &mut egui::Ui,
    value: &mut String,
    label: &str,
    state: &mut PasswordInputState,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(label);

        let response = if state.revealed {
            ui.text_edit_singleline(value)
        } else {
            // Use egui's password mode: displays bullets but edits the real value
            ui.add(egui::TextEdit::singleline(value).password(true))
        };

        changed = response.changed();

        let reveal_label = if state.revealed { "Hide" } else { "Show" };
        if ui.button(reveal_label).clicked() {
            state.toggle_reveal();
        }
    });

    changed
}

/// Show a password input field without a label (compact version).
///
/// Returns `true` if the value was changed this frame.
#[allow(dead_code)]
pub fn show_compact(ui: &mut egui::Ui, value: &mut String, state: &mut PasswordInputState) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        let response = if state.revealed {
            ui.text_edit_singleline(value)
        } else {
            ui.add(egui::TextEdit::singleline(value).password(true))
        };

        changed = response.changed();

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
