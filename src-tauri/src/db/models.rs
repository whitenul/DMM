use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
    pub icon: Option<String>,
    pub folder_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: i64,
    pub category_id: i64,
    pub name: String,
    pub pinyin_name: Option<String>,
    pub item_type: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub arguments: Option<String>,
    pub working_dir: Option<String>,
    pub sort_order: i32,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedApp {
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub app_type: String,
    pub arguments: Option<String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub item: Item,
    pub category_name: String,
    pub match_type: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub window: WindowSettings,
    pub shortcut: ShortcutSettings,
    pub appearance: AppearanceSettings,
    pub scan: ScanSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub edge_snap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutSettings {
    pub global_search: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme: String,
    pub effect: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSettings {
    pub auto_scan_on_start: bool,
    pub scan_start_menu: bool,
    pub scan_uwp: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(String),
    #[error("文件未找到: {0}")]
    NotFound(String),
    #[error("权限不足: {0}")]
    Permission(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let code = match self {
            AppError::Database(_) => "DATABASE",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Permission(_) => "PERMISSION",
            AppError::Io(_) => "IO",
        };
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", code)?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}
