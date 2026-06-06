use desk_core::db::MigrationSource;

/// desk-web 的数据库 migration
///
/// web_metadata 表，缓存网页元数据
pub struct DeskWebMigrations;

impl MigrationSource for DeskWebMigrations {
    fn plugin_name(&self) -> &str {
        "desk-web"
    }

    fn migration_names(&self) -> &[&str] {
        &["V1__web_metadata_cache"]
    }

    fn migration_sqls(&self) -> &[&str] {
        &[WEB_METADATA_SCHEMA_SQL]
    }
}

const WEB_METADATA_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS web_metadata (
    url TEXT PRIMARY KEY,
    title TEXT,
    favicon_url TEXT,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_web_metadata_fetched_at ON web_metadata(fetched_at);
"#;
