use desk_core::config::ConfigState;
use desk_core::db::resolve_app_data_dir;
use desk_core::error::AppError;
use std::path::PathBuf;
use tauri::{plugin::TauriPlugin, Manager, Runtime, State};

/// 插件作用域状态，存储应用数据目录
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

    #[tauri::command]
    pub fn get_system_accent_color() -> Result<String, String> {
        #[cfg(windows)]
        {
            use windows_registry::CURRENT_USER;
            let key = CURRENT_USER
                .open(r"Software\Microsoft\Windows\DWM")
                .map_err(|e| e.to_string())?;
            let dword: u32 = key
                .get_u32("AccentColor")
                .map_err(|e| e.to_string())?;
            // DWM 强调色: ABGR → #RRGGBB
            let r = (dword & 0x0000FF) as u8;
            let g = ((dword >> 8) & 0xFF) as u8;
            let b = ((dword >> 16) & 0xFF) as u8;
            Ok(format!("#{:02X}{:02X}{:02X}", r, g, b))
        }
        #[cfg(not(windows))]
        {
            Err("Not supported on this platform".to_string())
        }
    }

    #[tauri::command]
    pub fn list_custom_themes(
        settings_state: State<'_, SettingsState>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let themes_dir = settings_state.app_data_dir.join("themes");
        if !themes_dir.exists() {
            return Ok(vec![]);
        }
        let mut themes = Vec::new();
        let entries = std::fs::read_dir(&themes_dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    // 只返回非预装主题
                    if val.get("isBuiltIn").and_then(|v| v.as_bool()).unwrap_or(false) {
                        continue;
                    }
                    themes.push(val);
                }
            }
        }
        Ok(themes)
    }

    #[tauri::command]
    pub fn save_custom_theme(
        settings_state: State<'_, SettingsState>,
        theme: serde_json::Value,
    ) -> Result<(), String> {
        let themes_dir = settings_state.app_data_dir.join("themes");
        std::fs::create_dir_all(&themes_dir).map_err(|e| e.to_string())?;
        let id = theme.get("id").and_then(|v| v.as_str()).ok_or("Missing theme id")?;
        let mode = theme.get("mode").and_then(|v| v.as_str()).ok_or("Missing theme mode")?;
        let filename = format!("{}.{}.json", id, mode);
        let path = themes_dir.join(&filename);
        let content = serde_json::to_string_pretty(&theme).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    #[tauri::command]
    pub fn delete_custom_theme(
        settings_state: State<'_, SettingsState>,
        theme_id: String,
    ) -> Result<(), String> {
        let themes_dir = settings_state.app_data_dir.join("themes");
        if !themes_dir.exists() {
            return Err("Themes directory not found".to_string());
        }
        let entries = std::fs::read_dir(&themes_dir).map_err(|e| e.to_string())?;
        let mut deleted = false;
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            if filename.starts_with(&theme_id) && filename.ends_with(".json") {
                std::fs::remove_file(&path).map_err(|e| e.to_string())?;
                deleted = true;
            }
        }
        if !deleted {
            return Err(format!("Theme '{}' not found", theme_id));
        }
        Ok(())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("desk-settings")
        .invoke_handler(tauri::generate_handler![
            commands::load_settings,
            commands::update_settings,
            commands::save_window_position,
            commands::get_system_accent_color,
            commands::list_custom_themes,
            commands::save_custom_theme,
            commands::delete_custom_theme,
        ])
        .setup(|app, _api| {
            // 使用与主应用一致的 resolve_app_data_dir，确保开发/生产环境路径一致
            let app_data_dir = resolve_app_data_dir(app.app_handle())
                .map_err(|e| AppError::Config(e.to_string()))?;
            app.manage(SettingsState { app_data_dir });
            Ok(())
        })
        .build()
}
