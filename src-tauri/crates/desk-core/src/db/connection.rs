use crate::db::path_resolver::{resolve_db_path, AppDataPath};
use crate::db::MigrationAggregator;
use crate::error::AppError;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[derive(Clone)]
pub struct DbState(pub Arc<Mutex<Connection>>);

impl DbState {
    pub fn new(app_data_dir: &Path) -> Result<Self, AppError> {
        std::fs::create_dir_all(app_data_dir)?;
        let db_path = app_data_dir.join("data.db");

        let conn = match Self::open_with_pragma(&db_path) {
            Ok(c) => c,
            Err(_) => {
                tracing::warn!("Database open failed, attempting recovery");
                let _ = std::fs::remove_file(format!("{}-shm", db_path.display()));
                let _ = std::fs::remove_file(format!("{}-wal", db_path.display()));
                let _ = std::fs::remove_file(&db_path);
                Self::open_with_pragma(&db_path)?
            }
        };

        Ok(Self(Arc::new(Mutex::new(conn))))
    }

    fn open_with_pragma(db_path: &Path) -> Result<Connection, AppError> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;
             PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA busy_timeout=5000;
             PRAGMA cache_size=-64000;
             PRAGMA temp_store=MEMORY;
             PRAGMA mmap_size=268435456;
             PRAGMA wal_autocheckpoint=1000;"
        )?;
        Ok(conn)
    }

    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.0.lock().map_err(|e| AppError::Database(e.to_string()))
    }
}

/// 初始化数据库，解析数据目录、打开连接、运行迁移并注册为 Tauri 状态
pub fn init_db<R: tauri::Runtime>(
    app: &tauri::App<R>,
    aggregator: MigrationAggregator,
) -> Result<DbState, AppError> {
    let db_path = resolve_db_path(app.handle())?;
    tracing::info!(
        "Initializing database at {:?} (env: {:?})",
        db_path,
        AppDataPath::detect()
    );

    let db_state = DbState::new(db_path.parent().unwrap())?;
    {
        let mut conn = db_state.lock()?;
        aggregator.run_all(&mut conn)?;
    }
    app.manage(db_state.clone());
    Ok(db_state)
}
