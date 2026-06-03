use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

pub struct DbState(pub Mutex<Connection>);

impl DbState {
    pub fn new(app_data_dir: &std::path::Path) -> Result<Self, String> {
        std::fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
        let db_path = app_data_dir.join("data.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| e.to_string())?;
        Ok(Self(Mutex::new(conn)))
    }
}

pub fn init_db(app: &tauri::App) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_state = DbState::new(&app_data_dir)?;
    {
        let mut conn = db_state.0.lock().map_err(|e| e.to_string())?;
        crate::db::migration::run_migrations(&mut conn)?;
    }
    app.manage(db_state);
    Ok(())
}
