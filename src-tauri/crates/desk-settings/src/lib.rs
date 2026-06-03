use desk_core::config::ConfigState;
use desk_core::error::AppError;
use std::path::PathBuf;
use tauri::{plugin::TauriPlugin, Manager, Runtime, State};

/// Plugin-scoped state that stores the app data directory,
/// so commands do not need an `AppHandle<R>` parameter.
pub struct SettingsState {
    pub app_data_dir: PathBuf,
}

mod commands {
    use super::*;

    #[tauri::command]
    pub fn load_settings(config: State<'_, ConfigState>) -> Result<desk_core::config::AppConfig, AppError> {
        config.get()
    }

    #[tauri::command]
    pub fn update_settings(
        config: State<'_, ConfigState>,
        settings_state: State<'_, SettingsState>,
        settings: desk_core::config::AppConfig,
    ) -> Result<(), AppError> {
        config.update(&settings)?;
        config.save(&settings_state.app_data_dir)?;
        Ok(())
    }

    #[tauri::command]
    pub fn save_window_position(
        config: State<'_, ConfigState>,
        settings_state: State<'_, SettingsState>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<(), AppError> {
        let mut cfg = config.get()?;
        cfg.window.x = x;
        cfg.window.y = y;
        cfg.window.width = width;
        cfg.window.height = height;
        config.update(&cfg)?;
        config.save(&settings_state.app_data_dir)?;
        Ok(())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("desk-settings")
        .invoke_handler(tauri::generate_handler![
            commands::load_settings,
            commands::update_settings,
            commands::save_window_position,
        ])
        .setup(|app, _api| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Config(e.to_string()))?;
            app.manage(SettingsState { app_data_dir });
            Ok(())
        })
        .build()
}
