use crate::db::MigrationSource;

/// desk-core 自身的基础 schema migration
///
/// 由 desk-core crate 拥有，初始化应用最基础的 categories / items / fts5 表
pub struct DeskCoreMigrations;

impl MigrationSource for DeskCoreMigrations {
    fn plugin_name(&self) -> &str {
        "desk-core"
    }

    fn migration_names(&self) -> &[&str] {
        &["V1__initial_schema"]
    }

    fn migration_sqls(&self) -> &[&str] {
        &[INITIAL_SCHEMA_SQL]
    }
}

const INITIAL_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    parent_id INTEGER,
    sort_order INTEGER NOT NULL DEFAULT 0,
    icon TEXT,
    folder_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    pinyin_name TEXT,
    item_type TEXT NOT NULL DEFAULT 'App',
    path TEXT NOT NULL,
    icon_path TEXT,
    arguments TEXT,
    working_dir TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE IF NOT EXISTS items_fts USING fts5(
    name,
    pinyin_name,
    content='items',
    content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS items_ai AFTER INSERT ON items BEGIN
    INSERT INTO items_fts(rowid, name, pinyin_name) VALUES (new.id, new.name, new.pinyin_name);
END;

CREATE TRIGGER IF NOT EXISTS items_ad AFTER DELETE ON items BEGIN
    INSERT INTO items_fts(items_fts, rowid, name, pinyin_name) VALUES ('delete', old.id, old.name, old.pinyin_name);
END;

CREATE TRIGGER IF NOT EXISTS items_au AFTER UPDATE ON items BEGIN
    INSERT INTO items_fts(items_fts, rowid, name, pinyin_name) VALUES ('delete', old.id, old.name, old.pinyin_name);
    INSERT INTO items_fts(rowid, name, pinyin_name) VALUES (new.id, new.name, new.pinyin_name);
END;

CREATE INDEX IF NOT EXISTS idx_items_category_id ON items(category_id);
CREATE INDEX IF NOT EXISTS idx_items_item_type ON items(item_type);
CREATE INDEX IF NOT EXISTS idx_items_is_pinned ON items(is_pinned);
CREATE INDEX IF NOT EXISTS idx_items_sort_order ON items(sort_order);
CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_sort_order ON categories(sort_order);
"#;
