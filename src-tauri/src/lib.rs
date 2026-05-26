mod commands;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::stash_exists,
            commands::create_stash,
            commands::open_stash,
            commands::lock_stash,
            commands::list_entries,
            commands::add_entry,
            commands::extract_entry,
            commands::delete_entry,
            commands::rename_entry,
            commands::save_stash,
        ])
        .run(tauri::generate_context!())
        .expect("error while running USB Stash");
}
