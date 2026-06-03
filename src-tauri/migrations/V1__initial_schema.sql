CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    icon TEXT,
    folder_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    pinyin_name TEXT,
    item_type TEXT NOT NULL CHECK(item_type IN ('App', 'File', 'Folder', 'Web')),
    path TEXT NOT NULL,
    icon_path TEXT,
    arguments TEXT,
    working_dir TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE VIRTUAL TABLE items_fts USING fts5(
    name,
    pinyin_name,
    path,
    content='items',
    content_rowid='id'
);

CREATE TRIGGER items_ai AFTER INSERT ON items BEGIN
    INSERT INTO items_fts(rowid, name, pinyin_name, path)
    VALUES (new.id, new.name, new.pinyin_name, new.path);
END;

CREATE TRIGGER items_ad AFTER DELETE ON items BEGIN
    INSERT INTO items_fts(items_fts, rowid, name, pinyin_name, path)
    VALUES ('delete', old.id, old.name, old.pinyin_name, old.path);
END;

CREATE TRIGGER items_au AFTER UPDATE ON items BEGIN
    INSERT INTO items_fts(items_fts, rowid, name, pinyin_name, path)
    VALUES ('delete', old.id, old.name, old.pinyin_name, old.path);
    INSERT INTO items_fts(rowid, name, pinyin_name, path)
    VALUES (new.id, new.name, new.pinyin_name, new.path);
END;

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX idx_items_category ON items(category_id);
CREATE INDEX idx_items_type ON items(item_type);
CREATE INDEX idx_categories_parent ON categories(parent_id);
CREATE INDEX idx_categories_sort ON categories(sort_order);
CREATE INDEX idx_items_sort ON items(sort_order);
CREATE INDEX idx_items_pinned ON items(is_pinned);
