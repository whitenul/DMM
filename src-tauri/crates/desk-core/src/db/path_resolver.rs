use crate::error::AppError;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppDataPath {
    /// 开发环境：项目根目录下的 .dev/data/
    DevRelative,
    /// 生产环境：Tauri 标准的 app_data_dir()（%APPDATA%/{identifier}）
    InstalledRelative,
}

impl AppDataPath {
    /// 通过编译配置决定使用哪个路径
    pub fn detect() -> Self {
        #[cfg(debug_assertions)]
        {
            AppDataPath::DevRelative
        }
        #[cfg(not(debug_assertions))]
        {
            AppDataPath::InstalledRelative
        }
    }
}

/// 解析应用数据目录
///
/// - 开发环境（debug build）：`<project_root>/.dev/data/`
/// - 安装环境（release build）：`<app_data_dir>/` (Tauri 标准)
pub fn resolve_app_data_dir<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<PathBuf, AppError> {
    let path_type = AppDataPath::detect();
    let path = match path_type {
        AppDataPath::DevRelative => resolve_dev_path()?,
        AppDataPath::InstalledRelative => resolve_installed_path(app)?,
    };
    Ok(path)
}

fn resolve_dev_path() -> Result<PathBuf, AppError> {
    if let Ok(custom) = std::env::var("DESK_MANAGER_DEV_DATA_DIR") {
        let p = PathBuf::from(custom);
        std::fs::create_dir_all(&p)?;
        return Ok(p);
    }

    let exe_path = std::env::current_exe().map_err(AppError::Io)?;

    let mut current = exe_path.parent().map(|p| p.to_path_buf());
    let project_root = loop {
        let Some(c) = current.take() else {
            return Err(AppError::Config(
                "Cannot locate project root for dev data path".into(),
            ));
        };
        if c.join("src-tauri").exists() && c.join("package.json").exists() {
            break c;
        }
        current = c.parent().map(|p| p.to_path_buf());
    };

    let dev_data = project_root.join(".dev").join("data");
    std::fs::create_dir_all(&dev_data)?;
    Ok(dev_data)
}

fn resolve_installed_path<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<PathBuf, AppError> {
    if let Ok(custom) = std::env::var("DESK_MANAGER_DATA_DIR") {
        let p = PathBuf::from(custom);
        std::fs::create_dir_all(&p)?;
        return Ok(p);
    }
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Config(e.to_string()))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 数据库文件路径
pub fn resolve_db_path<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<PathBuf, AppError> {
    let dir = resolve_app_data_dir(app)?;
    Ok(dir.join("data.db"))
}

/// 图标存储目录
pub fn resolve_icons_dir<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<PathBuf, AppError> {
    let dir = resolve_app_data_dir(app)?;
    let icons = dir.join("icons");
    std::fs::create_dir_all(&icons)?;
    Ok(icons)
}

/// 日志存储目录
pub fn resolve_logs_dir<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<PathBuf, AppError> {
    let dir = resolve_app_data_dir(app)?;
    let logs = dir.join("logs");
    std::fs::create_dir_all(&logs)?;
    Ok(logs)
}

/// 配置文件路径
pub fn resolve_config_path<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<PathBuf, AppError> {
    let dir = resolve_app_data_dir(app)?;
    Ok(dir.join("config.toml"))
}

/// 主题文件目录
pub fn resolve_themes_dir<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<PathBuf, AppError> {
    let dir = resolve_app_data_dir(app)?;
    let themes = dir.join("themes");
    std::fs::create_dir_all(&themes)?;
    Ok(themes)
}
