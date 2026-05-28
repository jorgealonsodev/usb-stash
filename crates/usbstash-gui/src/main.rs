#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod screens;
mod widgets;

use app::App;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("USB Stash"),
        ..Default::default()
    };

    eframe::run_native(
        "USB Stash",
        native_options,
        Box::new(|cc| {
            // Set up custom styling
            let mut style = (*cc.egui_ctx.style()).clone();
            style.spacing.button_padding = egui::vec2(12.0, 6.0);
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.visuals.button_frame = true;
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 55);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 75);
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(70, 70, 85);
            style.visuals.selection.bg_fill = egui::Color32::from_rgb(59, 130, 246);
            style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 35);
            style.visuals.panel_fill = egui::Color32::from_rgb(30, 30, 35);
            style.visuals.extreme_bg_color = egui::Color32::from_rgb(20, 20, 25);
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 225));
            style.visuals.warn_fg_color = egui::Color32::from_rgb(251, 191, 36);
            style.visuals.error_fg_color = egui::Color32::from_rgb(239, 68, 68);
            style.visuals.hyperlink_color = egui::Color32::from_rgb(59, 130, 246);
            cc.egui_ctx.set_style(style);

            Ok(Box::new(App::default()))
        }),
    )
}
