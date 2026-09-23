CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_path TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'online',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_scan_started_at TEXT,
    last_scan_finished_at TEXT
);

CREATE TABLE IF NOT EXISTS media (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    relative_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    extension TEXT NOT NULL,
    media_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    created_at_fs INTEGER,
    modified_at_fs INTEGER,
    width INTEGER,
    height INTEGER,
    thumbnail_state TEXT NOT NULL DEFAULT 'pending',
    first_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(source_id, relative_path)
);

CREATE INDEX IF NOT EXISTS idx_media_source_id_id
    ON media(source_id, id);

CREATE INDEX IF NOT EXISTS idx_media_type_id
    ON media(media_type, id);

PRAGMA user_version = 1;
