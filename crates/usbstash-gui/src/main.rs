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
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
