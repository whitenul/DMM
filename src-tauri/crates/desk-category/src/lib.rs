use desk_core::db::DbState;
use desk_core::domain::category::{Category, CategoryRepo};
use desk_core::error::AppError;
use rusqlite::params;
use serde::Deserialize;
use tauri::{plugin::TauriPlugin, Manager, Runtime, State};

// ---------------------------------------------------------------------------
// ReorderEntry — used by the reorder_categories command
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ReorderEntry {
    pub id: i64,
    pub sort_order: i32,
}

// ---------------------------------------------------------------------------
// SqliteCategoryRepo — concrete CategoryRepo backed by SQLite
// ---------------------------------------------------------------------------

pub struct SqliteCategoryRepo {
    db: DbState,
}

impl SqliteCategoryRepo {
    pub fn new(db: DbState) -> Self {
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
        let params: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        conn.execute(&sql, params.as_slice())?;
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
                "UPDATE categories SET sort_order = ?1, updated_at = datetime('now') WHERE id = ?2",
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

// ---------------------------------------------------------------------------
// CategoryState — managed Tauri state holding the repo
// ---------------------------------------------------------------------------

pub struct CategoryState(pub Box<dyn CategoryRepo>);

// ---------------------------------------------------------------------------
// Tauri Commands (in a separate module to avoid __cmd__ macro name conflicts)
// ---------------------------------------------------------------------------

mod commands {
    use super::*;

    #[tauri::command]
    pub fn get_categories(state: State<'_, CategoryState>) -> Result<Vec<Category>, AppError> {
        state.0.get_all()
    }

    #[tauri::command]
    pub fn create_category(
        state: State<'_, CategoryState>,
        name: String,
        parent_id: Option<i64>,
        icon: Option<String>,
    ) -> Result<Category, AppError> {
        state.0.create(&name, parent_id, icon.as_deref())
    }

    #[tauri::command]
    pub fn update_category(
        state: State<'_, CategoryState>,
        id: i64,
        name: Option<String>,
        icon: Option<String>,
        parent_id: Option<Option<i64>>,
        folder_path: Option<String>,
    ) -> Result<(), AppError> {
        state
            .0
            .update(id, name.as_deref(), icon.as_deref(), parent_id, folder_path.as_deref())
    }

    #[tauri::command]
    pub fn delete_category(state: State<'_, CategoryState>, id: i64) -> Result<(), AppError> {
        state.0.delete(id)
    }

    #[tauri::command]
    pub fn reorder_categories(
        state: State<'_, CategoryState>,
        orders: Vec<ReorderEntry>,
    ) -> Result<(), AppError> {
        let orders: Vec<(i64, i32)> = orders.into_iter().map(|e| (e.id, e.sort_order)).collect();
        state.0.reorder(&orders)
    }

    #[tauri::command]
    pub fn link_folder(
        state: State<'_, CategoryState>,
        category_id: i64,
        folder_path: String,
    ) -> Result<(), AppError> {
        state.0.link_folder(category_id, &folder_path)
    }

    #[tauri::command]
    pub fn unlink_folder(
        state: State<'_, CategoryState>,
        category_id: i64,
    ) -> Result<(), AppError> {
        state.0.unlink_folder(category_id)
    }
}

// ---------------------------------------------------------------------------
// Plugin Init
// ---------------------------------------------------------------------------

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("desk-category")
        .invoke_handler(tauri::generate_handler![
            commands::get_categories,
            commands::create_category,
            commands::update_category,
            commands::delete_category,
            commands::reorder_categories,
            commands::link_folder,
            commands::unlink_folder,
        ])
        .setup(|app, _api| {
            let db = app.state::<DbState>().inner().clone();
            let repo = Box::new(SqliteCategoryRepo::new(db)) as Box<dyn CategoryRepo>;
            app.manage(CategoryState(repo));
            Ok(())
        })
        .build()
}
