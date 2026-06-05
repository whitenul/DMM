use desk_core::db::{DbState, SqliteSearchRepo};
use desk_core::domain::search::{SearchPort, SearchResult};
use desk_core::error::AppError;
use tauri::{plugin::TauriPlugin, Manager, Runtime};

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
