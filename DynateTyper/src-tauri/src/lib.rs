mod commands;
mod config;
mod conflict_detector;
mod crypto;
mod key_manager;
mod models;

use commands::AppState;
use parking_lot::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load config dari TOML file
    let app_config = config::load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            config: Mutex::new(app_config),
        })
        .invoke_handler(tauri::generate_handler![
            // Key capture
            commands::start_key_capture,
            commands::stop_key_capture,
            // Key entry CRUD
            commands::add_key_entry,
            commands::edit_key_entry,
            commands::delete_key_entry,
            commands::clear_all_entries,
            commands::get_entries,
            // Conflict detection
            commands::check_key_conflict,
            // Profile management
            commands::list_profiles,
            commands::create_profile,
            commands::switch_profile,
            commands::delete_profile,
            commands::get_active_profile,
            // Encryption
            commands::toggle_encryption,
            commands::get_encryption_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
