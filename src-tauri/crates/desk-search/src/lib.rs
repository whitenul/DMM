use desk_core::db::DbState;
use desk_core::domain::item::Item;
use desk_core::domain::search::{SearchPort, SearchResult};
use desk_core::error::AppError;
use rusqlite::params;
use tauri::{plugin::TauriPlugin, Manager, Runtime};

// ---------------------------------------------------------------------------
// SqliteSearchRepo — concrete SearchPort backed by SQLite FTS
// ---------------------------------------------------------------------------

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
        let mut stmt = conn.prepare(
            "SELECT i.id, i.category_id, i.name, i.pinyin_name, i.item_type, i.path, \
             i.icon_path, i.arguments, i.working_dir, i.sort_order, i.is_pinned, \
             i.created_at, i.updated_at, c.name as category_name \
             FROM items_fts f \
             JOIN items i ON f.rowid = i.id \
             JOIN categories c ON i.category_id = c.id \
             WHERE items_fts MATCH ?1 \
             ORDER BY rank \
             LIMIT 20",
        )?;

        let fts_query = format!("\"{}*\"", query.replace('"', ""));
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
                score: 0.0,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// SearchState — managed Tauri state holding a dyn SearchPort
// ---------------------------------------------------------------------------

pub struct SearchState(pub Box<dyn SearchPort>);

// ---------------------------------------------------------------------------
// Tauri Commands (in a submodule to avoid __cmd__ macro name collisions)
// ---------------------------------------------------------------------------

mod commands {
    use super::{AppError, SearchResult, SearchState};
    use tauri::State;

    #[tauri::command]
    pub fn search_items(
        state: State<'_, SearchState>,
        query: String,
    ) -> Result<Vec<SearchResult>, AppError> {
        state.0.search(&query)
    }
}

// ---------------------------------------------------------------------------
// Plugin Init
// ---------------------------------------------------------------------------

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("desk-search")
        .invoke_handler(tauri::generate_handler![commands::search_items])
        .setup(|app, _api| {
            let db = app.state::<DbState>().inner().clone();
            let repo = Box::new(SqliteSearchRepo::new(db)) as Box<dyn SearchPort>;
            app.manage(SearchState(repo));
            Ok(())
        })
        .build()
}
