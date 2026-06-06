use crate::error::AppError;
use rusqlite::Connection;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Compute a lightweight checksum for migration SQL content.
/// Uses DefaultHasher for speed — not cryptographic, but sufficient for
/// detecting accidental edits to already-applied migrations.
fn compute_checksum(s: &str) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// 每个 Plugin 通过此 trait 贡献自己的 migrations
pub trait MigrationSource: Send + Sync {
    /// 当前 Plugin 的标识符（如 "desk-core", "desk-icon"）
    fn plugin_name(&self) -> &str;

    /// 返回当前 Plugin 的所有 migration 名称
    fn migration_names(&self) -> &[&str];

    /// 返回当前 Plugin 的所有 migration SQL
    fn migration_sqls(&self) -> &[&str];
}

/// 统一管理所有 Plugin 的 migrations
///
/// 设计原则：
/// - 每个 Plugin 自治：自己提供 MigrationSource
/// - 不使用 refinery 哈希校验：使用 (plugin, name) 唯一性去重
/// - SQL 不嵌入二进制：保留为可读文件，便于调试和运维
pub struct MigrationAggregator {
    sources: Vec<Box<dyn MigrationSource>>,
}

impl MigrationAggregator {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn register(mut self, source: impl MigrationSource + 'static) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    /// 执行所有未应用的迁移
    pub fn run_all(&self, conn: &mut Connection) -> Result<(), AppError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS desk_migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                plugin TEXT NOT NULL,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(plugin, name)
            )",
        )?;

        for source in &self.sources {
            let names = source.migration_names();
            let sqls = source.migration_sqls();
            for (name, sql) in names.iter().zip(sqls.iter()) {
                let already_applied: bool = conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM desk_migrations WHERE plugin = ?1 AND name = ?2",
                        rusqlite::params![source.plugin_name(), name],
                        |row| row.get(0),
                    )
                    .unwrap_or(false);

                if already_applied {
                    let current_checksum = compute_checksum(sql);
                    tracing::debug!(
                        "Migration {}/{} already applied, skipping (checksum: {})",
                        source.plugin_name(),
                        name,
                        current_checksum
                    );
                    continue;
                }

                let checksum = compute_checksum(sql);
                tracing::info!(
                    "Applying migration: {}/{} (checksum: {})",
                    source.plugin_name(),
                    name,
                    checksum
                );
                conn.execute_batch(sql)?;
                conn.execute(
                    "INSERT INTO desk_migrations (plugin, name) VALUES (?1, ?2)",
                    rusqlite::params![source.plugin_name(), name],
                )?;
            }
        }
        Ok(())
    }

    /// 查看已应用的所有迁移
    pub fn list_applied(&self, conn: &Connection) -> Result<Vec<(String, String, String)>, AppError> {
        let mut stmt = conn.prepare_cached(
            "SELECT plugin, name, applied_at FROM desk_migrations ORDER BY id",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

impl Default for MigrationAggregator {
    fn default() -> Self {
        Self::new()
    }
}
