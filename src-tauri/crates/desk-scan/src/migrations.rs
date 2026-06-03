use desk_core::db::MigrationSource;

/// desk-scan 的数据库 migration
///
/// 提供 scan_history 表记录扫描历史（用于"上次扫描时间"显示和"是否需要重新扫描"判断）
pub struct DeskScanMigrations;

impl MigrationSource for DeskScanMigrations {
    fn plugin_name(&self) -> &str {
        "desk-scan"
    }

    fn migration_names(&self) -> &[&str] {
        &["V1__scan_history"]
    }

    fn migration_sqls(&self) -> &[&str] {
        &[SCAN_HISTORY_SCHEMA_SQL]
    }
}

const SCAN_HISTORY_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS scan_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scan_type TEXT NOT NULL,
    apps_count INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'success',
    error_message TEXT,
    scanned_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_scan_history_scan_type ON scan_history(scan_type);
CREATE INDEX IF NOT EXISTS idx_scan_history_scanned_at ON scan_history(scanned_at);
"#;
