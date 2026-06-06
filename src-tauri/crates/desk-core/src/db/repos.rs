//! Unified SQLite repository implementations.
//!
//! All concrete repo implementations live here so that plugins can reuse them
//! without duplicating code. Each plugin simply imports the struct it needs
//! and wires it up in its Tauri `setup` callback.

use crate::db::DbState;
use crate::domain::category::{Category, CategoryRepo};
use crate::domain::item::{Item, ItemRepo};
use crate::domain::search::{SearchPort, SearchResult};
use crate::error::AppError;
use rusqlite::params;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

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

/// Compute the first-letter pinyin abbreviation for a Chinese/mixed string.
pub fn compute_pinyin(name: &str) -> String {
    use pinyin::ToPinyinMulti;
    let mut result = String::new();
    for py in name.to_pinyin_multi().flatten() {
        if let Some(first_letter) = py.get(0).plain().chars().next() {
            result.push(first_letter.to_ascii_lowercase());
        }
    }
    result
}

// --- SqliteCategoryRepo ---

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
        let mut stmt = conn.prepare_cached(
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
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = conn.prepare_cached(
                "UPDATE categories SET sort_order = ?1, updated_at = datetime('now') WHERE id = ?2",
            )?;
            for (id, sort_order) in orders {
                stmt.execute(params![sort_order, id])?;
            }
        }
        tx.commit()?;
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

// --- SqliteItemRepo ---

pub struct SqliteItemRepo {
    db: DbState,
}

impl SqliteItemRepo {
    pub fn new(db: DbState) -> Self {
        Self { db }
    }
}

impl ItemRepo for SqliteItemRepo {
    fn get_by_category(&self, category_id: i64) -> Result<Vec<Item>, AppError> {
        let conn = self.db.lock()?;
        let mut stmt = conn.prepare_cached(&format!(
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
            "UPDATE items SET category_id = ?1, sort_order = ?2, updated_at = datetime('now') \
             WHERE id = ?3",
            params![category_id, max_order + 1, id],
        )?;
        Ok(())
    }

    fn reorder(&self, orders: &[(i64, i32)]) -> Result<(), AppError> {
        let conn = self.db.lock()?;
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = conn.prepare_cached(
                "UPDATE items SET sort_order = ?1, updated_at = datetime('now') WHERE id = ?2",
            )?;
            for (id, sort_order) in orders {
                stmt.execute(params![sort_order, id])?;
            }
        }
        tx.commit()?;
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
        let tx = conn.unchecked_transaction()?;
        let deleted = {
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
            conn.execute(&sql, refs.as_slice())?
        };
        tx.commit()?;
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

// --- SqliteSearchRepo ---

/// 基于 SQLite 的搜索仓库
pub struct SqliteSearchRepo {
    db: DbState,
}

impl SqliteSearchRepo {
    pub fn new(db: DbState) -> Self {
        Self { db }
    }
}

impl SearchPort for SqliteSearchRepo {
    fn search(&self, query: &str) -> Result<Vec<SearchResult>, AppError> {
        let conn = self.db.lock()?;

        // 1. FTS5 prefix match with rank scoring
        let fts_query = format!("\"{}*\"", query.replace('"', ""));
        let mut stmt = conn.prepare_cached(
            "SELECT i.id, i.category_id, i.name, i.pinyin_name, i.item_type, i.path, \
             i.icon_path, i.arguments, i.working_dir, i.sort_order, i.is_pinned, \
             i.created_at, i.updated_at, c.name as category_name, f.rank \
             FROM items_fts f \
             JOIN items i ON f.rowid = i.id \
             JOIN categories c ON i.category_id = c.id \
             WHERE items_fts MATCH ?1 \
             ORDER BY rank \
             LIMIT 20",
        )?;

        let rows = stmt.query_map(params![fts_query], |row| {
            Ok(SearchResult {
                item: Item {
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
                },
                category_name: row.get(13)?,
                match_type: "fts".to_string(),
                score: row.get::<_, f64>(14).unwrap_or(0.0),
            })
        })?;

        let mut results: Vec<SearchResult> = rows.filter_map(|r| r.ok()).collect();

        // FTS5 无结果时的模糊匹配回退
        if results.is_empty() {
            let query_lower = query.to_lowercase();
            let mut stmt = conn.prepare_cached(
                "SELECT i.id, i.category_id, i.name, i.pinyin_name, i.item_type, i.path, \
                 i.icon_path, i.arguments, i.working_dir, i.sort_order, i.is_pinned, \
                 i.created_at, i.updated_at, c.name as category_name \
                 FROM items i \
                 JOIN categories c ON i.category_id = c.id \
                 ORDER BY i.is_pinned DESC, i.sort_order",
            )?;

            let all_items = stmt.query_map([], |row| {
                Ok(SearchResult {
                    item: Item {
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
                    },
                    category_name: row.get(13)?,
                    match_type: "fuzzy".to_string(),
                    score: 0.0,
                })
            })?;

            use fuzzy_matcher::FuzzyMatcher;
            let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

            let mut fuzzy_results: Vec<SearchResult> = all_items
                .filter_map(|r| r.ok())
                .filter_map(|mut r| {
                    let name = &r.item.name;
                    let pinyin = r.item.pinyin_name.as_deref().unwrap_or("");
                    let score = matcher
                        .fuzzy_match(name, &query_lower)
                        .or_else(|| matcher.fuzzy_match(pinyin, &query_lower));
                    match score {
                        Some(s) => {
                            r.score = s as f64;
                            Some(r)
                        }
                        None => None,
                    }
                })
                .collect();

            fuzzy_results.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            fuzzy_results.truncate(20);
            results = fuzzy_results;
        }

        Ok(results)
    }
}
