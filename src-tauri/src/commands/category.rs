use crate::db::connection::DbState;
use crate::db::models::{AppError, Category};
use tauri::State;

#[tauri::command]
pub fn get_categories(db: State<'_, DbState>) -> Result<Vec<Category>, AppError> {
    crate::services::category_service::get_categories(&db)
}

#[tauri::command]
pub fn create_category(
    db: State<'_, DbState>,
    name: String,
    parent_id: Option<i64>,
    icon: Option<String>,
) -> Result<Category, AppError> {
    crate::services::category_service::create_category(&db, &name, parent_id, icon.as_deref())
}

#[tauri::command]
pub fn update_category(
    db: State<'_, DbState>,
    id: i64,
    name: Option<String>,
    icon: Option<String>,
    parent_id: Option<Option<i64>>,
) -> Result<(), AppError> {
    crate::services::category_service::update_category(
        &db,
        id,
        name.as_deref(),
        icon.as_deref(),
        parent_id,
    )
}

#[tauri::command]
pub fn delete_category(db: State<'_, DbState>, id: i64) -> Result<(), AppError> {
    crate::services::category_service::delete_category(&db, id)
}

#[tauri::command]
pub fn reorder_categories(
    db: State<'_, DbState>,
    orders: Vec<(i64, i32)>,
) -> Result<(), AppError> {
    crate::services::category_service::reorder_categories(&db, &orders)
}
