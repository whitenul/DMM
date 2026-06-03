use crate::db::models::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub shortcut: ShortcutConfig,
    pub appearance: AppearanceConfig,
    pub scan: ScanConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub edge_snap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub global_search: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub theme: String,
    pub effect: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub auto_scan_on_start: bool,
    pub scan_start_menu: bool,
    pub scan_uwp: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                x: 100,
                y: 100,
                width: 800,
                height: 600,
                edge_snap: true,
            },
            shortcut: ShortcutConfig {
                global_search: "Ctrl+Space".to_string(),
            },
            appearance: AppearanceConfig {
                theme: "system".to_string(),
                effect: "auto".to_string(),
                language: "zh-CN".to_string(),
            },
            scan: ScanConfig {
                auto_scan_on_start: true,
                scan_start_menu: true,
                scan_uwp: true,
            },
        }
    }
}

pub struct ConfigState(pub Mutex<AppConfig>);

impl ConfigState {
    pub fn new(app_data_dir: &std::path::Path) -> Result<Self, AppError> {
        let config_path = app_data_dir.join("config.toml");
        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            AppConfig::default()
        };
        Ok(Self(Mutex::new(config)))
    }

    pub fn save(&self, app_data_dir: &std::path::Path) -> Result<(), AppError> {
        let config_path = app_data_dir.join("config.toml");
        let config = self.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
        let content =
            toml::to_string_pretty(&*config).map_err(|e| AppError::Database(e.to_string()))?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}
