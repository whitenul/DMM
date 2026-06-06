use crate::error::AppError;
use rusqlite::Connection;

/// Schema 检查器：只读地查询 sqlite_master，返回表/索引/触发器/虚拟表清单
///
/// 调试和迁移验证时使用。与 "schema discovery" 不同，本工具不做任何
/// 主动探测或修改，纯粹是 introspect。
pub struct SchemaInspector;

impl SchemaInspector {
    /// 列出所有表名
    pub fn list_tables(conn: &Connection) -> Result<Vec<String>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 列出所有索引
    pub fn list_indexes(conn: &Connection) -> Result<Vec<(String, String)>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT name, tbl_name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY tbl_name, name",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 列出所有触发器
    pub fn list_triggers(conn: &Connection) -> Result<Vec<String>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='trigger' ORDER BY name",
        )?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 列出所有虚拟表
    pub fn list_virtual_tables(conn: &Connection) -> Result<Vec<String>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND sql LIKE '%VIRTUAL%USING%' ORDER BY name",
        )?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 导出完整 schema（用于调试）
    pub fn export_schema(conn: &Connection) -> Result<String, AppError> {
        let mut stmt = conn.prepare(
            "SELECT type, name, sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let typ: String = row.get(0)?;
                let name: String = row.get(1)?;
                let sql: Option<String> = row.get(2)?;
                Ok((typ, name, sql.unwrap_or_default()))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut out = String::new();
        let mut current_type = String::new();
        for (typ, name, sql) in rows {
            if typ != current_type {
                out.push_str(&format!("\n-- {typ}s --\n"));
                current_type = typ;
            }
            out.push_str(&format!("-- {name}\n{sql};\n\n"));
        }
        Ok(out)
    }
}
