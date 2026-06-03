use crate::db::connection::DbState;
use crate::db::models::{AppError, Category};
use rusqlite::params;

pub fn get_categories(db: &DbState) -> Result<Vec<Category>, AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    let mut stmt = conn
        .prepare("SELECT id, name, parent_id, sort_order, icon, folder_path, created_at, updated_at FROM categories ORDER BY sort_order")
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
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
        })
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut categories = Vec::new();
    for row in rows {
        categories.push(row.map_err(|e| AppError::Database(e.to_string()))?);
    }
    Ok(categories)
}

pub fn create_category(
    db: &DbState,
    name: &str,
    parent_id: Option<i64>,
    icon: Option<&str>,
) -> Result<Category, AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
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
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
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

pub fn update_category(
    db: &DbState,
    id: i64,
    name: Option<&str>,
    icon: Option<&str>,
    parent_id: Option<Option<i64>>,
) -> Result<(), AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
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
    conn.execute(&sql, params.as_slice())
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub fn delete_category(db: &DbState, id: i64) -> Result<(), AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    conn.execute("DELETE FROM categories WHERE id = ?1", params![id])
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub fn reorder_categories(db: &DbState, orders: &[(i64, i32)]) -> Result<(), AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    for (id, sort_order) in orders {
        conn.execute(
            "UPDATE categories SET sort_order = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![sort_order, id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    Ok(())
}
