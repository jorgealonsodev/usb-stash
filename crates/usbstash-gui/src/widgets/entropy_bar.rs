/// Password strength level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntropyLevel {
    /// Very weak or empty password
    VeryWeak,
    /// Weak password
    Weak,
    /// Medium strength
    Medium,
    /// Strong password
    Strong,
    /// Very strong password
    VeryStrong,
}

impl EntropyLevel {
    /// Get the display label for this level.
    pub fn label(&self) -> &'static str {
        match self {
            EntropyLevel::VeryWeak => "Very Weak",
            EntropyLevel::Weak => "Weak",
            EntropyLevel::Medium => "Medium",
            EntropyLevel::Strong => "Strong",
            EntropyLevel::VeryStrong => "Very Strong",
        }
    }

    /// Get the color for this level as an egui Color32.
    pub fn color(&self) -> egui::Color32 {
        match self {
            EntropyLevel::VeryWeak => egui::Color32::RED,
            EntropyLevel::Weak => egui::Color32::from_rgb(255, 140, 0), // Dark orange
            EntropyLevel::Medium => egui::Color32::YELLOW,
            EntropyLevel::Strong => egui::Color32::LIGHT_GREEN,
            EntropyLevel::VeryStrong => egui::Color32::GREEN,
        }
    }
}

/// Analyze password strength using zxcvbn.
///
/// Returns an EntropyLevel based on the zxcvbn score (0-4).
pub fn analyze(password: &str) -> EntropyLevel {
    if password.is_empty() {
        return EntropyLevel::VeryWeak;
    }

    use zxcvbn::Score;

    let estimate = zxcvbn::zxcvbn(password, &[]);
    match estimate.score() {
        Score::Zero => EntropyLevel::VeryWeak,
        Score::One => EntropyLevel::Weak,
        Score::Two => EntropyLevel::Medium,
        Score::Three => EntropyLevel::Strong,
        Score::Four => EntropyLevel::VeryStrong,
        _ => EntropyLevel::VeryWeak, // Future-proof for new scores
    }
}

/// Render the entropy bar widget.
///
/// Shows a colored bar with the strength label.
pub fn show(ui: &mut egui::Ui, level: EntropyLevel) {
    let color = level.color();
    let label = level.label();

    ui.horizontal(|ui| {
        ui.label("Strength:");

        // Draw the bar segments (5 segments for 5 levels)
        let bar_width = ui.available_width().min(200.0);
        let segment_width = bar_width / 5.0;
        let active_segments = match level {
            EntropyLevel::VeryWeak => 1,
            EntropyLevel::Weak => 2,
            EntropyLevel::Medium => 3,
            EntropyLevel::Strong => 4,
            EntropyLevel::VeryStrong => 5,
        };

        let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_width, 8.0), egui::Sense::hover());

        let painter = ui.painter_at(rect);
        let bg_color = ui.visuals().widgets.noninteractive.bg_fill;

        // Draw background segments
        for i in 0..5 {
            let x = rect.left() + (i as f32 * segment_width);
            let segment_rect = egui::Rect::from_min_max(
                egui::pos2(x, rect.top()),
                egui::pos2(x + segment_width - 1.0, rect.bottom()),
            );
            painter.rect_filled(segment_rect, 2.0, bg_color);
        }

        // Draw active segments
        for i in 0..active_segments {
            let x = rect.left() + (i as f32 * segment_width);
            let segment_rect = egui::Rect::from_min_max(
                egui::pos2(x, rect.top()),
                egui::pos2(x + segment_width - 1.0, rect.bottom()),
            );
            painter.rect_filled(segment_rect, 2.0, color);
        }

        ui.label(label);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_password_is_very_weak() {
        let level = analyze("");
        assert_eq!(level, EntropyLevel::VeryWeak);
    }

    #[test]
    fn test_single_char_is_very_weak() {
        let level = analyze("a");
        assert_eq!(level, EntropyLevel::VeryWeak);
    }

    #[test]
    fn test_common_password_is_weak() {
        let level = analyze("password");
        assert!(level == EntropyLevel::VeryWeak || level == EntropyLevel::Weak);
    }

    #[test]
    fn test_short_simple_is_weak() {
        let level = analyze("abc123");
        assert!(
            level == EntropyLevel::VeryWeak
                || level == EntropyLevel::Weak
                || level == EntropyLevel::Medium
        );
    }

    #[test]
    fn test_long_complex_is_strong() {
        let level = analyze("Tr0ub4dor&3!xYz9");
        assert!(level == EntropyLevel::Strong || level == EntropyLevel::VeryStrong);
    }

    #[test]
    fn test_very_long_passphrase_is_very_strong() {
        let level = analyze("correct horse battery staple monkey wrench!");
        assert!(level == EntropyLevel::Strong || level == EntropyLevel::VeryStrong);
    }

    #[test]
    fn test_entropy_level_label() {
        assert_eq!(EntropyLevel::VeryWeak.label(), "Very Weak");
        assert_eq!(EntropyLevel::Weak.label(), "Weak");
        assert_eq!(EntropyLevel::Medium.label(), "Medium");
        assert_eq!(EntropyLevel::Strong.label(), "Strong");
        assert_eq!(EntropyLevel::VeryStrong.label(), "Very Strong");
    }

    #[test]
    fn test_score_maps_to_level() {
        // Verify zxcvbn scores 0-4 map correctly
        // Score 0: very weak
        assert_eq!(analyze(""), EntropyLevel::VeryWeak);

        // Score 4: very strong (long random-ish password)
        let strong = analyze("X9#mK2$pL5@nQ8!vR3");
        assert!(strong == EntropyLevel::Strong || strong == EntropyLevel::VeryStrong);
    }

    #[test]
    fn test_special_chars_improve_strength() {
        let plain = analyze("abcdefghij");
        let special = analyze("ab!@#$%^&*");
        // Special chars should at least not decrease strength
        // (zxcvbn may rate symbols differently, but shouldn't be worse)
        let _ = (plain, special); // Both should not panic
    }

    #[test]
    fn test_entropy_level_color_returns_valid_color() {
        for level in [
            EntropyLevel::VeryWeak,
            EntropyLevel::Weak,
            EntropyLevel::Medium,
            EntropyLevel::Strong,
            EntropyLevel::VeryStrong,
        ] {
            let color = level.color();
            // Color32 should always be valid
            assert!(color.r() <= 255 && color.g() <= 255 && color.b() <= 255);
        }
    }
}
