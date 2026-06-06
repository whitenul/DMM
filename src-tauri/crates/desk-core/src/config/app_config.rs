use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub shortcut: ShortcutConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub scan: ScanConfig,
    /// 用户点击 X 按钮或系统触发关闭时的行为
    /// - "ask"              弹确认对话框（前端 X 按钮路径）
    /// - "minimize_to_tray" 隐藏到托盘
    /// - "quit"             退出应用
    #[serde(default = "default_close_behavior")]
    pub close_behavior: String,
    #[serde(default)]
    pub autostart: bool,
}

fn default_close_behavior() -> String {
    "ask".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_i32_100")]
    pub x: i32,
    #[serde(default = "default_i32_100")]
    pub y: i32,
    #[serde(default = "default_u32_800")]
    pub width: u32,
    #[serde(default = "default_u32_600")]
    pub height: u32,
    #[serde(default = "default_true")]
    pub edge_snap: bool,
}

fn default_i32_100() -> i32 { 100 }
fn default_u32_800() -> u32 { 800 }
fn default_u32_600() -> u32 { 600 }
fn default_true() -> bool { true }

impl Default for WindowConfig {
    fn default() -> Self {
        Self { x: 100, y: 100, width: 800, height: 600, edge_snap: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    #[serde(default = "default_global_search")]
    pub global_search: String,
}

fn default_global_search() -> String { "Ctrl+Shift+Space".to_string() }

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self { global_search: "Ctrl+Shift+Space".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_effect")]
    pub effect: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme_id")]
    pub theme_id: String,
    #[serde(default = "default_accent_source")]
    pub accent_source: String,
    #[serde(default)]
    pub custom_accent_color: Option<String>,
    #[serde(default)]
    pub background_image: Option<String>,
    #[serde(default = "default_bg_blur")]
    pub bg_blur: f32,
    #[serde(default = "default_app_opacity")]
    pub app_opacity: f32,
}

fn default_theme() -> String { "system".to_string() }
fn default_effect() -> String { "auto".to_string() }
fn default_language() -> String { "zh-CN".to_string() }
fn default_theme_id() -> String { "default".to_string() }
fn default_accent_source() -> String { "system".to_string() }
fn default_bg_blur() -> f32 { 0.0 }
fn default_app_opacity() -> f32 { 0.0 }

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            effect: "auto".to_string(),
            language: "zh-CN".to_string(),
            theme_id: "default".to_string(),
            accent_source: "system".to_string(),
            custom_accent_color: None,
            background_image: None,
            bg_blur: 0.0,
            app_opacity: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    #[serde(default = "default_true")]
    pub auto_scan_on_start: bool,
    #[serde(default = "default_true")]
    pub scan_start_menu: bool,
    #[serde(default = "default_true")]
    pub scan_uwp: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self { auto_scan_on_start: true, scan_start_menu: true, scan_uwp: true }
    }
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
                global_search: "Ctrl+Shift+Space".to_string(),
            },
            appearance: AppearanceConfig::default(),
            scan: ScanConfig {
                auto_scan_on_start: true,
                scan_start_menu: true,
                scan_uwp: true,
            },
            close_behavior: "ask".to_string(),
            autostart: false,
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

    pub fn get(&self) -> Result<AppConfig, AppError> {
        self.0
            .lock()
            .map(|c| c.clone())
            .map_err(|e| AppError::Config(e.to_string()))
    }

    pub fn update(&self, new_config: &AppConfig) -> Result<(), AppError> {
        let mut config = self.0.lock().map_err(|e| AppError::Config(e.to_string()))?;
        *config = new_config.clone();
        Ok(())
    }

    pub fn save(&self, app_data_dir: &std::path::Path) -> Result<(), AppError> {
        let config = self.0.lock().map_err(|e| AppError::Config(e.to_string()))?;
        let config_path = app_data_dir.join("config.toml");
        let content = toml::to_string_pretty(&*config)
            .map_err(|e| AppError::Config(e.to_string()))?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}
