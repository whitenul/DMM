mod db;
mod commands;
mod services;
mod config;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            db::connection::init_db(app)?;
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e: tauri::Error| e.to_string())?;
            let config_state =
                config::app_config::ConfigState::new(&app_data_dir).map_err(|e| e.to_string())?;
            app.manage(config_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::category::get_categories,
            commands::category::create_category,
            commands::category::update_category,
            commands::category::delete_category,
            commands::category::reorder_categories,
            commands::item::get_items_by_category,
            commands::item::create_item,
            commands::item::launch_item,
            commands::item::move_item,
            commands::item::reorder_items,
            commands::item::delete_item,
            commands::item::toggle_pin_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
