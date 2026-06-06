use desk_core::db::compute_pinyin;
use desk_core::db::{DbState, SqliteCategoryRepo, SqliteItemRepo};
use desk_core::domain::category::CategoryRepo;
use desk_core::domain::item::ItemRepo;
use desk_core::domain::scanned_app::ScannedApp;
use desk_core::error::AppError;
use std::path::{Path, PathBuf};
use tauri::{plugin::TauriPlugin, Manager, Runtime};
use walkdir::WalkDir;

pub mod migrations;
pub use migrations::DeskScanMigrations;

// --- 扫描状态 ---

pub struct ScanState {
    item_repo: Box<dyn ItemRepo>,
    category_repo: Box<dyn CategoryRepo>,
    /// 保留用于直接数据库访问
    #[allow(dead_code)]
    db: DbState,
}

// --- LNK 解析 ---

struct LnkInfo {
    target: String,
    arguments: Option<String>,
    working_dir: Option<String>,
}

fn parse_lnk_info(path: &std::path::Path) -> Option<LnkInfo> {
    let shortcut = lnk::ShellLink::open(path, lnk::encoding::WINDOWS_1252).ok()?;
    let target = shortcut
        .link_target()
        .unwrap_or_else(|| path.to_string_lossy().to_string());
    let string_data = shortcut.string_data();
    let arguments = string_data.command_line_arguments().clone();
    let working_dir = string_data.working_dir().clone();
    Some(LnkInfo {
        target,
        arguments,
        working_dir,
    })
}

// --- 扫描函数 ---

pub fn scan_start_menu() -> Result<Vec<ScannedApp>, AppError> {
    let mut apps = Vec::new();
    let app_data = std::env::var("APPDATA").unwrap_or_default();
    let common_start = std::env::var("PROGRAMDATA").unwrap_or_default();

    let start_menu_dirs: Vec<PathBuf> = vec![
        PathBuf::from(&app_data).join("Microsoft\\Windows\\Start Menu\\Programs"),
        PathBuf::from(&common_start).join("Microsoft\\Windows\\Start Menu\\Programs"),
    ];

    let mut seen_paths = std::collections::HashSet::new();

    for dir in &start_menu_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("lnk") {
                continue;
            }

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();

            let lnk_info = parse_lnk_info(path);
            let target = lnk_info
                .as_ref()
                .map(|i| i.target.clone())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            if seen_paths.contains(&target) {
                continue;
            }
            seen_paths.insert(target.clone());

            apps.push(ScannedApp {
                name,
                path: target,
                icon_path: None,
                app_type: "App".to_string(),
                arguments: lnk_info.as_ref().and_then(|i| i.arguments.clone()),
                working_dir: lnk_info.as_ref().and_then(|i| i.working_dir.clone()),
            });
        }
    }

    apps.sort_by_key(|a| a.name.to_lowercase());
    Ok(apps)
}

/// 扫描单个文件，若为支持的类型则返回 ScannedApp
pub fn scan_single_file(path: &Path) -> Option<ScannedApp> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !["exe", "lnk", "url", "bat"].contains(&ext) {
        return None;
    }

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let app_type = match ext {
        "exe" => "App",
        "url" => "Web",
        _ => "File",
    };

    let path_str = path.to_string_lossy().to_string();

    if ext == "lnk" {
        let lnk_info = parse_lnk_info(path);
        Some(ScannedApp {
            name,
            path: lnk_info
                .as_ref()
                .map(|i| i.target.clone())
                .unwrap_or(path_str),
            icon_path: None,
            app_type: app_type.to_string(),
            arguments: lnk_info.as_ref().and_then(|i| i.arguments.clone()),
            working_dir: lnk_info.as_ref().and_then(|i| i.working_dir.clone()),
        })
    } else {
        Some(ScannedApp {
            name,
            path: path_str,
            icon_path: None,
            app_type: app_type.to_string(),
            arguments: None,
            working_dir: None,
        })
    }
}

pub fn scan_folder(folder_path: &str) -> Result<Vec<ScannedApp>, AppError> {
    let mut apps = Vec::new();
    let dir = PathBuf::from(folder_path);
    if !dir.exists() {
        return Ok(apps);
    }

    let extensions = ["exe", "lnk", "url", "bat"];
    let mut seen = std::collections::HashSet::new();

    for entry in WalkDir::new(&dir).max_depth(2).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !extensions.contains(&ext) {
            continue;
        }

        let path_str = path.to_string_lossy().to_string();
        if seen.contains(&path_str) {
            continue;
        }
        seen.insert(path_str.clone());

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let app_type = match ext {
            "exe" => "App",
            "url" => "Web",
            _ => "File",
        };

        if ext == "lnk" {
            let lnk_info = parse_lnk_info(path);
            apps.push(ScannedApp {
                name,
                path: lnk_info
                    .as_ref()
                    .map(|i| i.target.clone())
                    .unwrap_or(path_str),
                icon_path: None,
                app_type: app_type.to_string(),
                arguments: lnk_info.as_ref().and_then(|i| i.arguments.clone()),
                working_dir: lnk_info.as_ref().and_then(|i| i.working_dir.clone()),
            });
        } else {
            apps.push(ScannedApp {
                name,
                path: path_str,
                icon_path: None,
                app_type: app_type.to_string(),
                arguments: None,
                working_dir: None,
            });
        }
    }

    apps.sort_by_key(|a| a.name.to_lowercase());
    Ok(apps)
}

pub fn scan_uwp_apps() -> Result<Vec<ScannedApp>, AppError> {
    use windows::Management::Deployment::PackageManager;
    use windows_core::HSTRING;

    let mut apps = Vec::new();
    let manager = PackageManager::new().map_err(|e| AppError::Scan(e.to_string()))?;

    // FindPackages() 枚举所有包，非管理员可能返回 0x80070005
    // 降级方案：尝试 FindPackages，权限不足时返回空列表而非报错
    let packages = match manager.FindPackages() {
        Ok(p) => p,
        Err(e) => {
            let code = e.code();
            // 0x80070005 = E_ACCESSDENIED
            if code == windows_core::HRESULT(0x80070005_u32 as i32) {
                tracing::warn!("UWP scan: FindPackages access denied (not running as admin), skipping UWP scan");
                return Ok(apps);
            }
            return Err(AppError::Scan(e.to_string()));
        }
    };

    for package in packages {
        let is_framework: bool = package.IsFramework().unwrap_or(false);
        if is_framework {
            continue;
        }

        let display_name_hstr: Option<HSTRING> = package.DisplayName().ok();
        let package_id = package.Id().ok();
        let name_hstr: Option<HSTRING> = package_id.as_ref().and_then(|i| i.Name().ok());
        let full_name_hstr: Option<HSTRING> = package_id.as_ref().and_then(|i| i.FullName().ok());
        let install_path_hstr: Option<HSTRING> = package.InstalledPath().ok();

        let app_name = display_name_hstr
            .as_ref()
            .map(|n| n.to_string_lossy())
            .filter(|n: &String| !n.is_empty())
            .or_else(|| name_hstr.as_ref().map(|n| n.to_string_lossy()))
            .or_else(|| full_name_hstr.as_ref().map(|f| f.to_string_lossy()))
            .unwrap_or_else(|| "Unknown UWP App".to_string());

        let path: String = install_path_hstr
            .as_ref()
            .map(|p| p.to_string_lossy())
            .unwrap_or_default();

        if path.is_empty() {
            continue;
        }

        let exe_path = find_uwp_executable(&path);

        apps.push(ScannedApp {
            name: app_name,
            path: exe_path.unwrap_or(path),
            icon_path: None,
            app_type: "App".to_string(),
            arguments: None,
            working_dir: None,
        });
    }

    apps.sort_by_key(|a| a.name.to_lowercase());
    apps.dedup_by(|a, b| a.path == b.path);
    Ok(apps)
}

fn find_uwp_executable(install_dir: &str) -> Option<String> {
    let manifest_path = std::path::Path::new(install_dir).join("AppxManifest.xml");
    if !manifest_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&manifest_path).ok()?;
    let start = content.find("Executable=\"")? + "Executable=\"".len();
    let end = content[start..].find('"')? + start;
    let exe_relative = &content[start..end];
    let exe_path = std::path::Path::new(install_dir).join(exe_relative);
    if exe_path.exists() {
        Some(exe_path.to_string_lossy().to_string())
    } else {
        None
    }
}

/// 将扫描到的应用导入指定分类，返回新导入的应用数量
pub fn import_scanned_apps(
    item_repo: &dyn ItemRepo,
    category_id: i64,
    apps: &[ScannedApp],
) -> Result<usize, AppError> {
    let mut imported = 0usize;

    // 通过 get_by_category 确定当前最大 sort_order
    let existing = item_repo.get_by_category(category_id)?;
    let max_order = existing.iter().map(|i| i.sort_order).max().unwrap_or(-1);

    for (i, app) in apps.iter().enumerate() {
        let exists = item_repo.exists_by_path_and_category(&app.path, category_id)?;
        if exists {
            continue;
        }

        let pinyin_name = compute_pinyin(&app.name);
        let sort_order = max_order + 1 + i as i32;

        item_repo.create_with_pinyin(
            category_id,
            &app.name,
            &pinyin_name,
            &app.app_type,
            &app.path,
            app.arguments.as_deref(),
            app.working_dir.as_deref(),
            sort_order,
        )?;

        imported += 1;
    }

    Ok(imported)
}

/// 应用启动时自动扫描，返回导入的应用数量和默认分类 ID
pub fn auto_scan_on_start(
    item_repo: &dyn ItemRepo,
    category_repo: &dyn CategoryRepo,
    config: &desk_core::config::AppConfig,
) -> Result<(u32, i64), AppError> {
    if !config.scan.auto_scan_on_start {
        return Ok((0, 0));
    }

    // 查找或创建默认分类
    let categories = category_repo.get_all()?;
    let default_cat = categories.iter().find(|c| c.name == "默认");
    let default_cat_id = match default_cat {
        Some(cat) => cat.id,
        None => {
            let cat = category_repo.create("默认", None, None)?;
            cat.id
        }
    };

    let mut total_imported = 0u32;

    if config.scan.scan_start_menu {
        if let Ok(apps) = scan_start_menu() {
            if let Ok(count) = import_scanned_apps(item_repo, default_cat_id, &apps) {
                total_imported += count as u32;
            }
        }
    }

    if config.scan.scan_uwp {
        if let Ok(apps) = scan_uwp_apps() {
            if let Ok(count) = import_scanned_apps(item_repo, default_cat_id, &apps) {
                total_imported += count as u32;
            }
        }
    }

    tracing::info!("Auto scan on start: imported {} new apps", total_imported);
    Ok((total_imported, default_cat_id))
}

// --- 文件夹监听 ---

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct FolderWatcher {
    _watcher: RecommendedWatcher,
    stop_flag: std::sync::Arc<std::sync::Mutex<bool>>,
    _rx_thread: Option<std::thread::JoinHandle<()>>,
}

impl FolderWatcher {
    pub fn start(app: AppHandle, db: DbState) -> Result<Self, AppError> {
        let (tx, rx) = mpsc::channel::<Event>();
        let stop_flag = std::sync::Arc::new(std::sync::Mutex::new(false));

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| AppError::Scan(e.to_string()))?;

        // 使用 SqliteCategoryRepo 查询带有 folder_path 的分类
        let category_repo = SqliteCategoryRepo::new(db.clone());
        let categories: Vec<(i64, String)> = category_repo
            .get_all()?
            .into_iter()
            .filter(|c| c.folder_path.as_ref().is_some_and(|p| !p.is_empty()))
            .map(|c| (c.id, c.folder_path.unwrap()))
            .collect();

        for (id, path) in &categories {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                let _ = watcher.watch(&path_buf, RecursiveMode::Recursive);
                tracing::info!("Watching folder for category {}: {}", id, path);
            }
        }

        let stop = stop_flag.clone();
        let rx_thread = std::thread::Builder::new()
            .name("folder-watcher".into())
            .spawn(move || {
                let debounce = Duration::from_millis(500);
                let mut pending_paths: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
                let mut last_flush = std::time::Instant::now();

                loop {
                    if *stop.lock().unwrap() {
                        break;
                    }

                    match rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(event) => {
                            if matches!(event.kind,
                                EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
                            ) {
                                for path in &event.paths {
                                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                                    if ["exe", "lnk", "url", "bat"].contains(&ext) {
                                        pending_paths.insert(path.clone());
                                    }
                                }
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }

                    // 防抖：等待事件稳定后再处理
                    if !pending_paths.is_empty() && last_flush.elapsed() >= debounce {
                        let paths_to_process: Vec<PathBuf> = pending_paths.drain().collect();
                        last_flush = std::time::Instant::now();

                        // 增量更新：仅处理变更的文件
                        let item_repo = SqliteItemRepo::new(db.clone());
                        for path in &paths_to_process {
                            // 查找路径所属分类
                            for (cat_id, folder) in &categories {
                                let folder_path = PathBuf::from(folder);
                                if path.starts_with(&folder_path) {
                                    if path.exists() {
                                        // 文件新增/修改：扫描并导入
                                        if let Some(app) = scan_single_file(path) {
                                            import_scanned_apps(&item_repo, *cat_id, &[app]).ok();
                                        }
                                    } else {
                                        // 文件删除：按路径从数据库删除
                                        item_repo.exists_by_path_and_category(
                                            &path.to_string_lossy(), *cat_id
                                        ).ok();
                                        // 目前缺少 delete_by_path 方法，回退为全量重扫
                                        let apps = scan_folder(folder).unwrap_or_default();
                                        import_scanned_apps(&item_repo, *cat_id, &apps).ok();
                                    }
                                    let _ = app.emit("folder-changed", *cat_id);
                                    break;
                                }
                            }
                        }
                    }
                }
            })
            .map_err(|e| AppError::Scan(e.to_string()))?;

        Ok(Self {
            _watcher: watcher,
            stop_flag,
            _rx_thread: Some(rx_thread),
        })
    }

    pub fn stop(&mut self) {
        *self.stop_flag.lock().unwrap() = true;
        if let Some(thread) = self._rx_thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for FolderWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 将文件夹关联到分类并扫描导入
pub fn link_folder(
    category_repo: &dyn CategoryRepo,
    item_repo: &dyn ItemRepo,
    category_id: i64,
    folder_path: &str,
) -> Result<(), AppError> {
    category_repo.link_folder(category_id, folder_path)?;
    let apps = scan_folder(folder_path)?;
    import_scanned_apps(item_repo, category_id, &apps)?;
    Ok(())
}

/// 取消文件夹与分类的关联
pub fn unlink_folder(
    category_repo: &dyn CategoryRepo,
    category_id: i64,
) -> Result<(), AppError> {
    category_repo.unlink_folder(category_id)
}

// --- Tauri 命令 ---

mod commands {
    use super::*;

    #[tauri::command]
    pub fn scan_start_menu() -> Result<Vec<ScannedApp>, AppError> {
        super::scan_start_menu()
    }

    #[tauri::command]
    pub fn scan_uwp_apps() -> Result<Vec<ScannedApp>, AppError> {
        super::scan_uwp_apps()
    }

    #[tauri::command]
    pub fn scan_folder(folder_path: String) -> Result<Vec<ScannedApp>, AppError> {
        super::scan_folder(&folder_path)
    }

    #[tauri::command]
    pub fn import_scanned_apps(
        state: tauri::State<'_, ScanState>,
        category_id: i64,
        apps: Vec<ScannedApp>,
    ) -> Result<usize, AppError> {
        super::import_scanned_apps(&*state.item_repo, category_id, &apps)
    }

    #[tauri::command]
    pub fn auto_scan_on_start<R: Runtime>(
        state: tauri::State<'_, ScanState>,
        config: tauri::State<'_, desk_core::config::ConfigState>,
        app: tauri::AppHandle<R>,
    ) -> Result<u32, AppError> {
        let cfg = config.get()?;
        let (count, default_cat_id) =
            super::auto_scan_on_start(&*state.item_repo, &*state.category_repo, &cfg)?;
        if count > 0 {
            let _ = app.emit("folder-changed", default_cat_id);
        }
        Ok(count)
    }
}

// --- 插件初始化 ---

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("desk-scan")
        .invoke_handler(tauri::generate_handler![
            commands::scan_start_menu,
            commands::scan_uwp_apps,
            commands::scan_folder,
            commands::import_scanned_apps,
            commands::auto_scan_on_start,
        ])
        .setup(|app, _api| {
            let db_state = app.state::<DbState>().inner().clone();
            let item_repo = Box::new(SqliteItemRepo::new(db_state.clone())) as Box<dyn ItemRepo>;
            let category_repo =
                Box::new(SqliteCategoryRepo::new(db_state.clone())) as Box<dyn CategoryRepo>;
            app.manage(ScanState {
                item_repo,
                category_repo,
                db: db_state,
            });
            Ok(())
        })
        .build()
}
