use desk_core::db::DbState;
use desk_core::domain::category::{Category, CategoryRepo};
use desk_core::domain::item::{Item, ItemRepo};
use desk_core::domain::scanned_app::ScannedApp;
use desk_core::error::AppError;
use rusqlite::params;
use std::path::PathBuf;
use tauri::{plugin::TauriPlugin, Manager, Runtime};
use walkdir::WalkDir;

pub mod migrations;
pub use migrations::DeskScanMigrations;

// ===========================================================================
// ScanState — managed Tauri state holding repos + DbState
// ===========================================================================

pub struct ScanState {
    item_repo: Box<dyn ItemRepo>,
    category_repo: Box<dyn CategoryRepo>,
    /// Kept for direct DB access when needed (e.g., FolderWatcher).
    #[allow(dead_code)]
    db: DbState,
}

// ===========================================================================
// SqliteItemRepo — local ItemRepo implementation backed by SQLite
// ===========================================================================

struct SqliteItemRepo {
    db: DbState,
}

impl SqliteItemRepo {
    fn new(db: DbState) -> Self {
        Self { db }
    }
}

const ITEM_COLUMNS: &str =
    "id, category_id, name, pinyin_name, item_type, path, icon_path, arguments, working_dir, \
     sort_order, is_pinned, created_at, updated_at";

fn row_to_item(row: &rusqlite::Row<'_>) -> Result<Item, rusqlite::Error> {
    Ok(Item {
        id: row.get(0)?,
        category_id: row.get(1)?,
        name: row.get(2)?,
        pinyin_name: row.get(3)?,
        item_type: row.get(4)?,
        path: row.get(5)?,
        icon_path: row.get(6)?,
        arguments: row.get(7)?,
        working_dir: row.get(8)?,
        sort_order: row.get(9)?,
        is_pinned: row.get::<_, i32>(10)? != 0,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

impl ItemRepo for SqliteItemRepo {
    fn get_by_category(&self, category_id: i64) -> Result<Vec<Item>, AppError> {
        let conn = self.db.lock()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {ITEM_COLUMNS} FROM items WHERE category_id = ?1 ORDER BY is_pinned DESC, sort_order"
        ))?;
        let rows = stmt.query_map(params![category_id], row_to_item)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    fn get_by_id(&self, id: i64) -> Result<Option<Item>, AppError> {
        let conn = self.db.lock()?;
        let result = conn.query_row(
            &format!("SELECT {ITEM_COLUMNS} FROM items WHERE id = ?1"),
            params![id],
            row_to_item,
        );
        match result {
            Ok(item) => Ok(Some(item)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn get_path_and_type(&self, id: i64) -> Result<(String, String), AppError> {
        let conn = self.db.lock()?;
        let result = conn.query_row(
            "SELECT path, item_type FROM items WHERE id = ?1",
            params![id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;
        Ok(result)
    }

    fn create(
        &self,
        category_id: i64,
        name: &str,
        item_type: &str,
        path: &str,
        arguments: Option<&str>,
        working_dir: Option<&str>,
    ) -> Result<Item, AppError> {
        let conn = self.db.lock()?;
        let pinyin_name = compute_pinyin(name);
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM items WHERE category_id = ?1",
                params![category_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);
        conn.execute(
            "INSERT INTO items (category_id, name, pinyin_name, item_type, path, arguments, \
             working_dir, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                category_id, name, pinyin_name, item_type, path, arguments, working_dir,
                max_order + 1
            ],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Item {
            id,
            category_id,
            name: name.to_string(),
            pinyin_name: Some(pinyin_name),
            item_type: item_type.to_string(),
            path: path.to_string(),
            icon_path: None,
            arguments: arguments.map(|s| s.to_string()),
            working_dir: working_dir.map(|s| s.to_string()),
            sort_order: max_order + 1,
            is_pinned: false,
            created_at: String::new(),
            updated_at: String::new(),
        })
    }

    fn update(
        &self,
        id: i64,
        name: Option<&str>,
        item_type: Option<&str>,
        path: Option<&str>,
        arguments: Option<&str>,
        working_dir: Option<&str>,
    ) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        let mut updates = Vec::new();
        let mut param_idx = 1usize;
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(n) = name {
            let pinyin = compute_pinyin(n);
            updates.push(format!("name = ?{param_idx}"));
            param_values.push(Box::new(n.to_string()));
            param_idx += 1;
            updates.push(format!("pinyin_name = ?{param_idx}"));
            param_values.push(Box::new(pinyin));
            param_idx += 1;
        }
        if let Some(t) = item_type {
            updates.push(format!("item_type = ?{param_idx}"));
            param_values.push(Box::new(t.to_string()));
            param_idx += 1;
        }
        if let Some(p) = path {
            updates.push(format!("path = ?{param_idx}"));
            param_values.push(Box::new(p.to_string()));
            param_idx += 1;
        }
        if let Some(a) = arguments {
            updates.push(format!("arguments = ?{param_idx}"));
            param_values.push(Box::new(a.to_string()));
            param_idx += 1;
        }
        if let Some(w) = working_dir {
            updates.push(format!("working_dir = ?{param_idx}"));
            param_values.push(Box::new(w.to_string()));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = datetime('now')".to_string());
        let sql = format!(
            "UPDATE items SET {} WHERE id = ?{param_idx}",
            updates.join(", ")
        );
        param_values.push(Box::new(id));
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, refs.as_slice())?;
        Ok(())
    }

    fn delete(&self, id: i64) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        conn.execute("DELETE FROM items WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn move_to_category(&self, id: i64, category_id: i64) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM items WHERE category_id = ?1",
                params![category_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);
        conn.execute(
            "UPDATE items SET category_id = ?1, sort_order = ?2, updated_at = datetime('now') \
             WHERE id = ?3",
            params![category_id, max_order + 1, id],
        )?;
        Ok(())
    }

    fn reorder(&self, orders: &[(i64, i32)]) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        for (id, sort_order) in orders {
            conn.execute(
                "UPDATE items SET sort_order = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![sort_order, id],
            )?;
        }
        Ok(())
    }

    fn toggle_pin(&self, id: i64) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        conn.execute(
            "UPDATE items SET is_pinned = CASE WHEN is_pinned = 0 THEN 1 ELSE 0 END, \
             updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    fn batch_delete(&self, ids: &[i64]) -> Result<usize, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.db.lock()?;
        let placeholders: Vec<String> = ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect();
        let sql = format!(
            "DELETE FROM items WHERE id IN ({})",
            placeholders.join(",")
        );
        let param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
            ids.iter().map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>).collect();
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let deleted = conn.execute(&sql, refs.as_slice())?;
        tracing::info!("Batch deleted {} items", deleted);
        Ok(deleted)
    }

    fn update_icon_path(&self, id: i64, icon_path: &str) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        conn.execute(
            "UPDATE items SET icon_path = ?1 WHERE id = ?2",
            params![icon_path, id],
        )?;
        Ok(())
    }

    fn find_id_by_path_and_category(
        &self,
        path: &str,
        category_id: i64,
        icon_null_only: bool,
    ) -> Result<Option<i64>, AppError> {
        let conn = self.db.lock()?;
        let sql = if icon_null_only {
            "SELECT id FROM items WHERE path = ?1 AND category_id = ?2 AND icon_path IS NULL \
             LIMIT 1"
        } else {
            "SELECT id FROM items WHERE path = ?1 AND category_id = ?2 LIMIT 1"
        };
        let result = conn.query_row(sql, params![path, category_id], |row| {
            row.get::<_, i64>(0)
        });
        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn exists_by_path_and_category(&self, path: &str, category_id: i64) -> Result<bool, AppError> {
        let conn = self.db.lock()?;
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM items WHERE path = ?1 AND category_id = ?2",
            params![path, category_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    fn create_with_pinyin(
        &self,
        category_id: i64,
        name: &str,
        pinyin_name: &str,
        item_type: &str,
        path: &str,
        arguments: Option<&str>,
        working_dir: Option<&str>,
        sort_order: i32,
    ) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        conn.execute(
            "INSERT INTO items (category_id, name, pinyin_name, item_type, path, arguments, \
             working_dir, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                category_id, name, pinyin_name, item_type, path, arguments, working_dir,
                sort_order
            ],
        )?;
        Ok(())
    }
}

// ===========================================================================
// SqliteCategoryRepo — local CategoryRepo implementation backed by SQLite
// ===========================================================================

struct SqliteCategoryRepo {
    db: DbState,
}

impl SqliteCategoryRepo {
    fn new(db: DbState) -> Self {
        Self { db }
    }
}

impl CategoryRepo for SqliteCategoryRepo {
    fn get_all(&self) -> Result<Vec<Category>, AppError> {
        let conn = self.db.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, parent_id, sort_order, icon, folder_path, created_at, updated_at \
             FROM categories ORDER BY sort_order",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                sort_order: row.get(3)?,
                icon: row.get(4)?,
                folder_path: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        let mut categories = Vec::new();
        for row in rows {
            categories.push(row?);
        }
        Ok(categories)
    }

    fn get_by_id(&self, id: i64) -> Result<Option<Category>, AppError> {
        let conn = self.db.lock()?;
        let result = conn.query_row(
            "SELECT id, name, parent_id, sort_order, icon, folder_path, created_at, updated_at \
             FROM categories WHERE id = ?1",
            params![id],
            |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent_id: row.get(2)?,
                    sort_order: row.get(3)?,
                    icon: row.get(4)?,
                    folder_path: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        );
        match result {
            Ok(category) => Ok(Some(category)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn create(
        &self,
        name: &str,
        parent_id: Option<i64>,
        icon: Option<&str>,
    ) -> Result<Category, AppError> {
        let conn = self.db.lock()?;
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM categories WHERE parent_id IS ?",
                params![parent_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);
        conn.execute(
            "INSERT INTO categories (name, parent_id, sort_order, icon) VALUES (?1, ?2, ?3, ?4)",
            params![name, parent_id, max_order + 1, icon],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Category {
            id,
            name: name.to_string(),
            parent_id,
            sort_order: max_order + 1,
            icon: icon.map(String::from),
            folder_path: None,
            created_at: String::new(),
            updated_at: String::new(),
        })
    }

    fn update(
        &self,
        id: i64,
        name: Option<&str>,
        icon: Option<&str>,
        parent_id: Option<Option<i64>>,
        folder_path: Option<&str>,
    ) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        let mut updates = Vec::new();
        let mut param_idx = 1usize;
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(n) = name {
            updates.push(format!("name = ?{param_idx}"));
            param_values.push(Box::new(n.to_string()));
            param_idx += 1;
        }
        if let Some(i) = icon {
            updates.push(format!("icon = ?{param_idx}"));
            param_values.push(Box::new(i.to_string()));
            param_idx += 1;
        }
        if let Some(p) = parent_id {
            updates.push(format!("parent_id = ?{param_idx}"));
            param_values.push(Box::new(p));
            param_idx += 1;
        }
        if let Some(fp) = folder_path {
            updates.push(format!("folder_path = ?{param_idx}"));
            param_values.push(Box::new(fp.to_string()));
            param_idx += 1;
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = datetime('now')".to_string());
        let sql = format!(
            "UPDATE categories SET {} WHERE id = ?{param_idx}",
            updates.join(", ")
        );
        param_values.push(Box::new(id));
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, refs.as_slice())?;
        Ok(())
    }

    fn delete(&self, id: i64) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        conn.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn reorder(&self, orders: &[(i64, i32)]) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        for (id, sort_order) in orders {
            conn.execute(
                "UPDATE categories SET sort_order = ?1, updated_at = datetime('now') \
                 WHERE id = ?2",
                params![sort_order, id],
            )?;
        }
        Ok(())
    }

    fn link_folder(&self, id: i64, folder_path: &str) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        conn.execute(
            "UPDATE categories SET folder_path = ?1 WHERE id = ?2",
            params![folder_path, id],
        )?;
        Ok(())
    }

    fn unlink_folder(&self, id: i64) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        conn.execute(
            "UPDATE categories SET folder_path = NULL WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
}

// ===========================================================================
// Pinyin helper
// ===========================================================================

fn compute_pinyin(name: &str) -> String {
    use pinyin::ToPinyinMulti;
    let mut result = String::new();
    for pinyin_result in name.to_pinyin_multi() {
        if let Some(py) = pinyin_result {
            if let Some(first_letter) = py.get(0).plain().chars().next() {
                result.push(first_letter.to_ascii_lowercase());
            }
        }
    }
    result
}

// ===========================================================================
// LNK parsing helpers
// ===========================================================================

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

// ===========================================================================
// Scan functions
// ===========================================================================

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

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(apps)
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

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(apps)
}

pub fn scan_uwp_apps() -> Result<Vec<ScannedApp>, AppError> {
    use windows::Management::Deployment::PackageManager;
    use windows_core::HSTRING;

    let mut apps = Vec::new();
    let manager = PackageManager::new().map_err(|e| AppError::Scan(e.to_string()))?;
    let packages = manager
        .FindPackages()
        .map_err(|e| AppError::Scan(e.to_string()))?;

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

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
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

/// Import scanned apps into a category using ItemRepo trait methods.
/// Returns the number of newly imported apps.
pub fn import_scanned_apps(
    item_repo: &dyn ItemRepo,
    category_id: i64,
    apps: &[ScannedApp],
) -> Result<usize, AppError> {
    let mut imported = 0usize;

    // Determine current max sort_order via get_by_category
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

/// Auto-scan on application start using CategoryRepo + ItemRepo traits.
/// Returns (number of imported apps, default category id).
pub fn auto_scan_on_start(
    item_repo: &dyn ItemRepo,
    category_repo: &dyn CategoryRepo,
    config: &desk_core::config::AppConfig,
) -> Result<(u32, i64), AppError> {
    if !config.scan.auto_scan_on_start {
        return Ok((0, 0));
    }

    // Find or create the default category
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

// ===========================================================================
// FolderWatcher
// ===========================================================================

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct FolderWatcher {
    _watcher: RecommendedWatcher,
    _rx_thread: std::thread::JoinHandle<()>,
}

impl FolderWatcher {
    pub fn start(app: AppHandle, db: DbState) -> Result<Self, AppError> {
        let (tx, rx) = mpsc::channel::<Event>();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        )
        .map_err(|e| AppError::Scan(e.to_string()))?;

        // Query categories with folder_path using CategoryRepo
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

        let rx_thread = std::thread::spawn(move || {
            let debounce = Duration::from_millis(500);
            let mut last_event_time = std::time::Instant::now();

            while let Ok(event) = rx.recv_timeout(Duration::from_secs(30)) {
                match event.kind {
                    EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => {
                        let now = std::time::Instant::now();
                        if now.duration_since(last_event_time) < debounce {
                            continue;
                        }
                        last_event_time = now;

                        for path in &event.paths {
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if !["exe", "lnk", "url", "bat"].contains(&ext) {
                                continue;
                            }

                            for (cat_id, folder) in &categories {
                                let folder_path = PathBuf::from(folder);
                                if path.starts_with(&folder_path) {
                                    let apps = scan_folder(folder).unwrap_or_default();
                                    // Create a local ItemRepo for the import
                                    let item_repo = SqliteItemRepo::new(db.clone());
                                    import_scanned_apps(&item_repo, *cat_id, &apps).ok();
                                    let _ = app.emit("folder-changed", *cat_id);
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            _rx_thread: rx_thread,
        })
    }
}

/// Link a folder to a category using CategoryRepo + ItemRepo trait methods.
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

/// Unlink a folder from a category using CategoryRepo trait method.
pub fn unlink_folder(
    category_repo: &dyn CategoryRepo,
    category_id: i64,
) -> Result<(), AppError> {
    category_repo.unlink_folder(category_id)
}

// ===========================================================================
// Tauri Commands (in a submodule to avoid __cmd__ macro name collisions)
// ===========================================================================

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

// ===========================================================================
// Plugin init
// ===========================================================================

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
