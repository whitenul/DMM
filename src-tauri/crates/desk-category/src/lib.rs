use desk_core::db::{DbState, SqliteCategoryRepo};
use desk_core::domain::category::{Category, CategoryRepo};
use desk_core::error::AppError;
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
