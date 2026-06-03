use crate::db::connection::DbState;
use crate::db::models::{AppError, Item};
use rusqlite::params;

pub fn get_items_by_category(db: &DbState, category_id: i64) -> Result<Vec<Item>, AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, category_id, name, pinyin_name, item_type, path, icon_path, arguments, working_dir, sort_order, is_pinned, created_at, updated_at FROM items WHERE category_id = ?1 ORDER BY is_pinned DESC, sort_order",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    let rows = stmt
        .query_map(params![category_id], |row| {
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
        })
        .map_err(|e| AppError::Database(e.to_string()))?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| AppError::Database(e.to_string()))?);
    }
    Ok(items)
}

pub fn create_item(
    db: &DbState,
    category_id: i64,
    name: &str,
    item_type: &str,
    path: &str,
) -> Result<Item, AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    let pinyin_name = compute_pinyin(name);
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM items WHERE category_id = ?1",
            params![category_id],
            |row| row.get(0),
        )
        .unwrap_or(-1);
    conn.execute(
        "INSERT INTO items (category_id, name, pinyin_name, item_type, path, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![category_id, name, pinyin_name, item_type, path, max_order + 1],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    let id = conn.last_insert_rowid();
    Ok(Item {
        id,
        category_id,
        name: name.to_string(),
        pinyin_name: Some(pinyin_name),
        item_type: item_type.to_string(),
        path: path.to_string(),
        icon_path: None,
        arguments: None,
        working_dir: None,
        sort_order: max_order + 1,
        is_pinned: false,
        created_at: String::new(),
        updated_at: String::new(),
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

pub fn launch_item(db: &DbState, id: i64) -> Result<(), AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    let (path, arguments, working_dir, item_type): (
        String,
        Option<String>,
        Option<String>,
        String,
    ) = conn
        .query_row(
            "SELECT path, arguments, working_dir, item_type FROM items WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    drop(conn);

    match item_type.as_str() {
        "Web" => open::that(&path).map_err(AppError::Io)?,
        _ => {
            let mut cmd = std::process::Command::new(&path);
            if let Some(args) = arguments {
                cmd.args(args.split_whitespace());
            }
            if let Some(dir) = working_dir {
                cmd.current_dir(dir);
            }
            cmd.spawn().map_err(AppError::Io)?;
        }
    }
    Ok(())
}

pub fn move_item(db: &DbState, id: i64, target_category_id: i64) -> Result<(), AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM items WHERE category_id = ?1",
            params![target_category_id],
            |row| row.get(0),
        )
        .unwrap_or(-1);
    conn.execute(
        "UPDATE items SET category_id = ?1, sort_order = ?2, updated_at = datetime('now') WHERE id = ?3",
        params![target_category_id, max_order + 1, id],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub fn reorder_items(db: &DbState, orders: &[(i64, i32)]) -> Result<(), AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    for (id, sort_order) in orders {
        conn.execute(
            "UPDATE items SET sort_order = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![sort_order, id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    Ok(())
}

pub fn delete_item(db: &DbState, id: i64) -> Result<(), AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    conn.execute("DELETE FROM items WHERE id = ?1", params![id])
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

pub fn toggle_pin_item(db: &DbState, id: i64) -> Result<(), AppError> {
    let conn = db.0.lock().map_err(|e| AppError::Database(e.to_string()))?;
    conn.execute(
        "UPDATE items SET is_pinned = CASE WHEN is_pinned = 0 THEN 1 ELSE 0 END, updated_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}
