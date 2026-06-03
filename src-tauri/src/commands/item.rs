use crate::db::connection::DbState;
use crate::db::models::{AppError, Item};
use tauri::State;

#[tauri::command]
pub fn get_items_by_category(
    db: State<'_, DbState>,
    category_id: i64,
) -> Result<Vec<Item>, AppError> {
    crate::services::item_service::get_items_by_category(&db, category_id)
}

#[tauri::command]
pub fn create_item(
    db: State<'_, DbState>,
    category_id: i64,
    name: String,
    item_type: String,
    path: String,
) -> Result<Item, AppError> {
    crate::services::item_service::create_item(&db, category_id, &name, &item_type, &path)
}

#[tauri::command]
pub fn launch_item(db: State<'_, DbState>, id: i64) -> Result<(), AppError> {
    crate::services::item_service::launch_item(&db, id)
}

#[tauri::command]
pub fn move_item(
    db: State<'_, DbState>,
    id: i64,
    target_category_id: i64,
) -> Result<(), AppError> {
    crate::services::item_service::move_item(&db, id, target_category_id)
}

#[tauri::command]
pub fn reorder_items(
    db: State<'_, DbState>,
    orders: Vec<(i64, i32)>,
) -> Result<(), AppError> {
    crate::services::item_service::reorder_items(&db, &orders)
}

#[tauri::command]
pub fn delete_item(db: State<'_, DbState>, id: i64) -> Result<(), AppError> {
    crate::services::item_service::delete_item(&db, id)
}

#[tauri::command]
pub fn toggle_pin_item(db: State<'_, DbState>, id: i64) -> Result<(), AppError> {
    crate::services::item_service::toggle_pin_item(&db, id)
}
