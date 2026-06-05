use desk_core::db::{DbState, SqliteItemRepo};
use desk_core::domain::item::{Item, ItemRepo};
use desk_core::error::AppError;
use serde::Deserialize;
use tauri::{plugin::TauriPlugin, Manager, Runtime};

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
                // URL: 用 cmd /c start 在默认浏览器中打开，避免 ShellExecuteW COM 死锁
                std::process::Command::new("cmd")
                    .args(["/c", "start", "", &item.path])
                    .spawn()
                    .map_err(|e| {
                        tracing::error!("Failed to open URL {}: {}", item.path, e);
                        AppError::Io(e)
                    })?;
            }
            "Folder" => {
                // 文件夹: 用 explorer 打开，不会阻塞
                std::process::Command::new("explorer")
                    .arg(&item.path)
                    .spawn()
                    .map_err(|e| {
                        tracing::error!("Failed to open folder {}: {}", item.path, e);
                        AppError::Io(e)
                    })?;
            }
            "Uwp" => {
                // UWP 应用: 用 cmd /c start 启动
                // 完整方案需要 IApplicationActivationManager COM 接口 + AUMID
                // 当前存储的 path 是可执行文件路径，用 cmd start 作为兼容方案
                std::process::Command::new("cmd")
                    .args(["/c", "start", "", &item.path])
                    .spawn()
                    .map_err(|e| {
                        tracing::error!("Failed to launch UWP app {}: {}", item.path, e);
                        AppError::Io(e)
                    })?;
            }
            _ => {
                // App / File: 有命令行参数时作为可执行文件启动
                if item.arguments.is_some() {
                    let mut cmd = std::process::Command::new(&item.path);
                    cmd.args(item.arguments.as_deref().unwrap().split_whitespace());
                    if let Some(dir) = &item.working_dir {
                        cmd.current_dir(dir);
                    }
                    match cmd.spawn() {
                        Ok(_) => {}
                        Err(_) => {
                            // 回退: 用 cmd /c start 打开（支持文档等非可执行文件）
                            std::process::Command::new("cmd")
                                .args(["/c", "start", "", &item.path])
                                .spawn()
                                .map_err(|e| {
                                    tracing::error!("Failed to open {}: {}", item.path, e);
                                    AppError::Io(e)
                                })?;
                        }
                    }
                } else {
                    // 无参数: 用 cmd /c start 打开（支持 exe/文档/URL）
                    // 这比 ShellExecuteW 更安全，不会导致 COM 死锁
                    std::process::Command::new("cmd")
                        .args(["/c", "start", "", &item.path])
                        .spawn()
                        .map_err(|e| {
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
