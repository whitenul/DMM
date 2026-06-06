mod logging;

use desk_core::config::ConfigState;
use desk_core::db::{
    resolve_config_path, resolve_logs_dir, DbState, DeskCoreMigrations, MigrationAggregator,
};
use desk_scan::DeskScanMigrations;
use desk_web::DeskWebMigrations;
use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, WindowEvent,
};

/// 全局 AppHandle 缓存
static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

/// 真正强制退出整个进程
#[tauri::command]
fn quit_app(state: tauri::State<'_, desk_core::db::DbState>) {
    // 退出前执行 WAL checkpoint
    if let Ok(conn) = state.lock() {
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    }
    std::process::exit(0);
}

/// 重置窗口背景色为完全透明
#[tauri::command]
fn reset_window_background(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_background_color(Some(tauri::window::Color(0, 0, 0, 0)))
            .map_err(|e| format!("Failed to set background color: {e}"))?;
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        // .plugin(tauri_plugin_updater::Builder::new().build()) // TODO: 启用前需配置 pubkey 签名公钥
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .invoke_handler(tauri::generate_handler![quit_app, reset_window_background])
        .on_window_event(|window, event| {
            // 拦截窗口关闭请求，交由前端决定行为
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.app_handle().emit("window-close-requested", ());
            }
        })
        .setup(|app| {
            // COM 生命周期守卫，drop 时自动 CoUninitialize
            struct ComGuard;
            impl ComGuard {
                fn init() -> Result<Self, String> {
                    unsafe {
                        windows::Win32::System::Com::CoInitializeEx(
                            None,
                            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
                        )
                        .ok()
                        .map_err(|e| format!("COM init failed: {e}"))?;
                    }
                    Ok(Self)
                }
            }
            impl Drop for ComGuard {
                fn drop(&mut self) {
                    unsafe { windows::Win32::System::Com::CoUninitialize(); }
                }
            }
            let _com_guard = ComGuard::init().map_err(|e| e.to_string())?;

            *APP_HANDLE.lock().unwrap() = Some(app.handle().clone());

            let logs_dir = resolve_logs_dir(app.handle()).map_err(|e| e.to_string())?;
            logging::init(&logs_dir);

            let aggregator = MigrationAggregator::new()
                .register(DeskCoreMigrations)
                .register(DeskScanMigrations)
                .register(DeskWebMigrations);
            let _db_state = desk_core::db::init_db(app, aggregator)?;

            let config_path = resolve_config_path(app.handle()).map_err(|e| e.to_string())?;
            let app_data_dir = config_path
                .parent()
                .ok_or_else(|| "Cannot resolve app data dir".to_string())?;
            let config_state = ConfigState::new(app_data_dir).map_err(|e| e.to_string())?;
            app.manage(config_state);

            app.handle().plugin(desk_category::init())?;
            app.handle().plugin(desk_item::init())?;
            app.handle().plugin(desk_search::init())?;
            app.handle().plugin(desk_scan::init())?;
            app.handle().plugin(desk_icon::init())?;
            app.handle().plugin(desk_settings::init())?;
            app.handle().plugin(desk_web::init())?;

            if let Some(main_window) = app.get_webview_window("main") {
                // 设置 WebView 背景为透明
                let _ = main_window.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));

                let config_state = app.state::<ConfigState>();
                if let Ok(config) = config_state.get() {
                    let _ = main_window.set_size(tauri::LogicalSize::new(
                        config.window.width as f64,
                        config.window.height as f64,
                    ));
                    let _ = main_window.set_position(tauri::LogicalPosition::new(
                        config.window.x as f64,
                        config.window.y as f64,
                    ));
                }

                // 不启用 DWM 效果，视觉由 CSS 控制
                let _ = main_window.set_effects(tauri::utils::config::WindowEffectsConfig {
                    effects: vec![],
                    ..Default::default()
                });

                // 编译时内嵌图标，确保开发和生产环境都能正确设置任务栏图标
                let icon = tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/icon.png"
                ))
                .expect("icon.png should be valid");
                let _ = main_window.set_icon(icon);
            }

            let show_item = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(
                    tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))
                        .expect("icon.png should be valid"),
                )
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        // 托盘菜单"退出"是用户明确意图，直接强制退出整个进程
                        if let Some(db_state) = app.try_state::<desk_core::db::DbState>() {
                            if let Ok(conn) = db_state.lock() {
                                let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
                            }
                        }
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let db_state = app.state::<DbState>().inner().clone();
            let folder_watcher = desk_scan::FolderWatcher::start(app.handle().clone(), db_state);
            if let Ok(_fw) = folder_watcher {
                app.manage(_fw);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
