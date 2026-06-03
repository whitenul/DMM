use desk_core::db::DbState;
use desk_core::domain::item::{Item, ItemRepo};
use desk_core::error::AppError;
use rusqlite::params;
use serde::Deserialize;
use tauri::{plugin::TauriPlugin, Manager, Runtime};

// ---------------------------------------------------------------------------
// SqliteItemRepo — ItemRepo backed by SQLite via DbState
// ---------------------------------------------------------------------------

pub struct SqliteItemRepo {
    db: DbState,
}

impl SqliteItemRepo {
    pub fn new(db: DbState) -> Self {
        Self { db }
    }
}

// ---------------------------------------------------------------------------
// ItemState — managed Tauri state holding a dyn ItemRepo
// ---------------------------------------------------------------------------

pub struct ItemState(pub Box<dyn ItemRepo>);

// ---------------------------------------------------------------------------
// ReorderEntry — deserialization target for the reorder_items command
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ReorderEntry {
    pub id: i64,
    pub sort_order: i32,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const ITEM_COLUMNS: &str =
    "id, category_id, name, pinyin_name, item_type, path, icon_path, arguments, working_dir, sort_order, is_pinned, created_at, updated_at";

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

// ---------------------------------------------------------------------------
// ItemRepo implementation
// ---------------------------------------------------------------------------

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
            "INSERT INTO items (category_id, name, pinyin_name, item_type, path, arguments, working_dir, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![category_id, name, pinyin_name, item_type, path, arguments, working_dir, max_order + 1],
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
        let params: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, params.as_slice())?;
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
            "UPDATE items SET category_id = ?1, sort_order = ?2, updated_at = datetime('now') WHERE id = ?3",
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
            "UPDATE items SET is_pinned = CASE WHEN is_pinned = 0 THEN 1 ELSE 0 END, updated_at = datetime('now') WHERE id = ?1",
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
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let deleted = conn.execute(&sql, params_refs.as_slice())?;
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
            "SELECT id FROM items WHERE path = ?1 AND category_id = ?2 AND icon_path IS NULL LIMIT 1"
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
            "INSERT INTO items (category_id, name, pinyin_name, item_type, path, arguments, working_dir, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![category_id, name, pinyin_name, item_type, path, arguments, working_dir, sort_order],
        )?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tauri Commands (in a submodule to avoid __cmd__ macro name collisions)
// ---------------------------------------------------------------------------

mod commands {
    use super::{AppError, Item, ItemState, ReorderEntry};
    use tauri::State;

    #[tauri::command]
    pub fn get_items_by_category(
        state: State<'_, ItemState>,
        category_id: i64,
    ) -> Result<Vec<Item>, AppError> {
        state.0.get_by_category(category_id)
    }

    #[tauri::command]
    pub fn create_item(
        state: State<'_, ItemState>,
        category_id: i64,
        name: String,
        item_type: String,
        path: String,
        arguments: Option<String>,
        working_dir: Option<String>,
    ) -> Result<Item, AppError> {
        // TODO: Icon extraction will be handled by desk-icon plugin after item creation.
        // The original create_item command extracted and saved the item icon here.
        // When desk-icon is integrated, emit an event or call the icon plugin after creation.
        state.0.create(
            category_id,
            &name,
            &item_type,
            &path,
            arguments.as_deref(),
            working_dir.as_deref(),
        )
    }

    #[tauri::command]
    pub fn update_item(
        state: State<'_, ItemState>,
        id: i64,
        name: Option<String>,
        item_type: Option<String>,
        path: Option<String>,
        arguments: Option<String>,
        working_dir: Option<String>,
    ) -> Result<(), AppError> {
        state.0.update(
            id,
            name.as_deref(),
            item_type.as_deref(),
            path.as_deref(),
            arguments.as_deref(),
            working_dir.as_deref(),
        )
    }

    #[tauri::command]
    pub fn launch_item(state: State<'_, ItemState>, id: i64) -> Result<(), AppError> {
        let item = state
            .0
            .get_by_id(id)?
            .ok_or_else(|| AppError::NotFound(format!("Item with id {} not found", id)))?;

        tracing::info!(
            "Launching item {}: type={}, path={}",
            id,
            item.item_type,
            item.path
        );

        match item.item_type.as_str() {
            "Web" => {
                open::that_detached(&item.path).map_err(|e| {
                    tracing::error!("Failed to open URL {}: {}", item.path, e);
                    AppError::Io(e)
                })?;
            }
            _ => {
                // 有命令行参数时尝试作为可执行文件直接启动
                if item.arguments.is_some() {
                    let mut cmd = std::process::Command::new(&item.path);
                    cmd.args(item.arguments.as_deref().unwrap().split_whitespace());
                    if let Some(dir) = &item.working_dir {
                        cmd.current_dir(dir);
                    }
                    match cmd.spawn() {
                        Ok(_) => {}
                        Err(_) => {
                            // 回退：用 ShellExecute 打开（支持文件夹、文档等）
                            open::that_detached(&item.path).map_err(|e| {
                                tracing::error!("Failed to open {}: {}", item.path, e);
                                AppError::Io(e)
                            })?;
                        }
                    }
                } else {
                    // 无参数：使用 ShellExecute 打开（支持 exe/文件夹/文档/URL）
                    open::that_detached(&item.path).map_err(|e| {
                        tracing::error!("Failed to open {}: {}", item.path, e);
                        AppError::Io(e)
                    })?;
                }
            }
        }
        Ok(())
    }

    #[tauri::command]
    pub fn move_item(
        state: State<'_, ItemState>,
        id: i64,
        target_category_id: i64,
    ) -> Result<(), AppError> {
        state.0.move_to_category(id, target_category_id)
    }

    #[tauri::command]
    pub fn reorder_items(
        state: State<'_, ItemState>,
        orders: Vec<ReorderEntry>,
    ) -> Result<(), AppError> {
        let orders: Vec<(i64, i32)> = orders.into_iter().map(|e| (e.id, e.sort_order)).collect();
        state.0.reorder(&orders)
    }

    #[tauri::command]
    pub fn delete_item(state: State<'_, ItemState>, id: i64) -> Result<(), AppError> {
        state.0.delete(id)
    }

    #[tauri::command]
    pub fn toggle_pin_item(state: State<'_, ItemState>, id: i64) -> Result<(), AppError> {
        state.0.toggle_pin(id)
    }

    #[tauri::command]
    pub fn batch_delete_items(
        state: State<'_, ItemState>,
        ids: Vec<i64>,
    ) -> Result<usize, AppError> {
        state.0.batch_delete(&ids)
    }
}

// ---------------------------------------------------------------------------
// Plugin init
// ---------------------------------------------------------------------------

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("desk-item")
        .invoke_handler(tauri::generate_handler![
            commands::get_items_by_category,
            commands::create_item,
            commands::update_item,
            commands::launch_item,
            commands::move_item,
            commands::reorder_items,
            commands::delete_item,
            commands::toggle_pin_item,
            commands::batch_delete_items,
        ])
        .setup(|app, _api| {
            let db_state = app.state::<DbState>().inner().clone();
            let repo = SqliteItemRepo::new(db_state);
            app.manage(ItemState(Box::new(repo)));
            Ok(())
        })
        .build()
}
