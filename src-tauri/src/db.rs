use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::models::{MediaItem, MediaPage, Source};

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_path: PathBuf,
    pub cache_dir: PathBuf,
    pub scanning: Arc<Mutex<std::collections::HashSet<i64>>>,
}

impl AppState {
    pub fn new(db_path: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            db_path,
            cache_dir,
            scanning: Arc::new(Mutex::new(std::collections::HashSet::new())),
        }
    }
}

#[derive(Debug)]
pub struct DiscoveredMedia {
    pub relative_path: String,
    pub file_name: String,
    pub extension: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub created_at_fs: Option<i64>,
    pub modified_at_fs: Option<i64>,
}

#[derive(Debug)]
pub struct MediaPath {
    pub root_path: String,
    pub relative_path: String,
    pub modified_at_fs: Option<i64>,
}

pub fn open(path: &Path) -> Result<Connection> {
    let connection = Connection::open(path)
        .with_context(|| format!("failed to open catalog at {}", path.display()))?;

    connection.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;
        ",
    )?;

    Ok(connection)
}

pub fn init_database(path: &Path) -> Result<()> {
    let connection = open(path)?;
    let current_version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if current_version < 1 {
        connection
            .execute_batch(include_str!("../migrations/0001_initial.sql"))
            .context("failed to apply migration 0001_initial")?;
    }

    Ok(())
}

pub fn insert_or_get_source(db_path: &Path, root_path: &str, display_name: &str) -> Result<Source> {
    let connection = open(db_path)?;

    connection.execute(
        "
        INSERT INTO sources(root_path, display_name, status)
        VALUES (?1, ?2, 'online')
        ON CONFLICT(root_path) DO UPDATE SET
            display_name = excluded.display_name
        ",
        params![root_path, display_name],
    )?;

    source_by_root(&connection, root_path)
}

fn source_by_root(connection: &Connection, root_path: &str) -> Result<Source> {
    connection
        .query_row(
            "
            SELECT id, root_path, display_name, status,
                   last_scan_started_at, last_scan_finished_at
            FROM sources
            WHERE root_path = ?1
            ",
            params![root_path],
            |row| {
                Ok(Source {
                    id: row.get(0)?,
                    root_path: row.get(1)?,
                    display_name: row.get(2)?,
                    status: row.get(3)?,
                    last_scan_started_at: row.get(4)?,
                    last_scan_finished_at: row.get(5)?,
                })
            },
        )
        .context("source was not found after insert")
}

pub fn get_source(db_path: &Path, source_id: i64) -> Result<Source> {
    let connection = open(db_path)?;
    connection
        .query_row(
            "
            SELECT id, root_path, display_name, status,
                   last_scan_started_at, last_scan_finished_at
            FROM sources
            WHERE id = ?1
            ",
            params![source_id],
            |row| {
                Ok(Source {
                    id: row.get(0)?,
                    root_path: row.get(1)?,
                    display_name: row.get(2)?,
                    status: row.get(3)?,
                    last_scan_started_at: row.get(4)?,
                    last_scan_finished_at: row.get(5)?,
                })
            },
        )
        .with_context(|| format!("source {source_id} not found"))
}

pub fn list_sources(db_path: &Path) -> Result<Vec<Source>> {
    let connection = open(db_path)?;
    let mut statement = connection.prepare(
        "
        SELECT id, root_path, display_name, status,
               last_scan_started_at, last_scan_finished_at
        FROM sources
        ORDER BY id ASC
        ",
    )?;

    let rows = statement.query_map([], |row| {
        Ok(Source {
            id: row.get(0)?,
            root_path: row.get(1)?,
            display_name: row.get(2)?,
            status: row.get(3)?,
            last_scan_started_at: row.get(4)?,
            last_scan_finished_at: row.get(5)?,
        })
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to read sources")
}

pub fn mark_scan_started(db_path: &Path, source_id: i64) -> Result<()> {
    let connection = open(db_path)?;
    connection.execute(
        "
        UPDATE sources
        SET status = 'scanning',
            last_scan_started_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        ",
        params![source_id],
    )?;
    Ok(())
}

pub fn mark_scan_finished(db_path: &Path, source_id: i64) -> Result<()> {
    let connection = open(db_path)?;
    connection.execute(
        "
        UPDATE sources
        SET status = 'online',
            last_scan_finished_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        ",
        params![source_id],
    )?;
    Ok(())
}

pub fn mark_scan_error(db_path: &Path, source_id: i64) -> Result<()> {
    let connection = open(db_path)?;
    connection.execute(
        "UPDATE sources SET status = 'error' WHERE id = ?1",
        params![source_id],
    )?;
    Ok(())
}

pub fn upsert_media_batch(
    db_path: &Path,
    source_id: i64,
    items: &[DiscoveredMedia],
) -> Result<()> {
    if items.is_empty() {
        return Ok(());
    }

    let mut connection = open(db_path)?;
    let transaction = connection.transaction()?;

    {
        let mut statement = transaction.prepare_cached(
            "
            INSERT INTO media(
                source_id, relative_path, file_name, extension, media_type,
                size_bytes, created_at_fs, modified_at_fs
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(source_id, relative_path) DO UPDATE SET
                file_name = excluded.file_name,
                extension = excluded.extension,
                media_type = excluded.media_type,
                size_bytes = excluded.size_bytes,
                created_at_fs = excluded.created_at_fs,
                modified_at_fs = excluded.modified_at_fs,
                thumbnail_state = CASE
                    WHEN media.modified_at_fs IS excluded.modified_at_fs
                    THEN media.thumbnail_state
                    ELSE 'pending'
                END,
                updated_at = CURRENT_TIMESTAMP
            ",
        )?;

        for item in items {
            statement.execute(params![
                source_id,
                item.relative_path,
                item.file_name,
                item.extension,
                item.media_type,
                item.size_bytes,
                item.created_at_fs,
                item.modified_at_fs,
            ])?;
        }
    }

    transaction.commit()?;
    Ok(())
}

pub fn query_media(db_path: &Path, offset: u32, limit: u32) -> Result<MediaPage> {
    let limit = limit.clamp(1, 500);
    let connection = open(db_path)?;
    let total: i64 = connection.query_row("SELECT COUNT(*) FROM media", [], |row| row.get(0))?;

    let mut statement = connection.prepare(
        "
        SELECT
            m.id, m.source_id, m.relative_path, m.file_name, m.extension,
            m.media_type, m.size_bytes, m.created_at_fs, m.modified_at_fs,
            m.width, m.height, m.thumbnail_state,
            s.display_name, s.root_path
        FROM media m
        JOIN sources s ON s.id = m.source_id
        ORDER BY m.id ASC
        LIMIT ?1 OFFSET ?2
        ",
    )?;

    let rows = statement.query_map(params![i64::from(limit), i64::from(offset)], |row| {
        Ok(MediaItem {
            id: row.get(0)?,
            source_id: row.get(1)?,
            relative_path: row.get(2)?,
            file_name: row.get(3)?,
            extension: row.get(4)?,
            media_type: row.get(5)?,
            size_bytes: row.get(6)?,
            created_at_fs: row.get(7)?,
            modified_at_fs: row.get(8)?,
            width: row.get(9)?,
            height: row.get(10)?,
            thumbnail_state: row.get(11)?,
            source_name: row.get(12)?,
            source_root: row.get(13)?,
        })
    })?;

    let items = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to read media page")?;

    Ok(MediaPage {
        items,
        total,
        offset,
        limit,
    })
}

pub fn media_path(db_path: &Path, media_id: i64) -> Result<MediaPath> {
    let connection = open(db_path)?;
    connection
        .query_row(
            "
            SELECT s.root_path, m.relative_path, m.modified_at_fs
            FROM media m
            JOIN sources s ON s.id = m.source_id
            WHERE m.id = ?1
            ",
            params![media_id],
            |row| {
                Ok(MediaPath {
                    root_path: row.get(0)?,
                    relative_path: row.get(1)?,
                    modified_at_fs: row.get(2)?,
                })
            },
        )
        .with_context(|| format!("media {media_id} not found"))
}

pub fn mark_thumbnail_ready(
    db_path: &Path,
    media_id: i64,
    width: i64,
    height: i64,
) -> Result<()> {
    let connection = open(db_path)?;
    connection.execute(
        "
        UPDATE media
        SET thumbnail_state = 'ready',
            width = COALESCE(width, ?2),
            height = COALESCE(height, ?3),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        ",
        params![media_id, width, height],
    )?;
    Ok(())
}

pub fn mark_thumbnail_failed(db_path: &Path, media_id: i64) -> Result<()> {
    let connection = open(db_path)?;
    connection.execute(
        "
        UPDATE media
        SET thumbnail_state = 'failed',
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        ",
        params![media_id],
    )?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn temporary_catalog() -> (PathBuf, PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("lume-db-test-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).expect("temporary directory should be created");
        (root.join("library.db"), root)
    }

    #[test]
    fn migration_is_idempotent_and_sets_version() {
        let (db_path, root) = temporary_catalog();

        init_database(&db_path).expect("first migration should succeed");
        init_database(&db_path).expect("second migration should be idempotent");

        let connection = open(&db_path).expect("catalog should reopen");
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("user_version should be readable");
        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal mode should be readable");

        assert_eq!(version, 1);
        assert_eq!(journal_mode.to_ascii_lowercase(), "wal");

        drop(connection);
        fs::remove_dir_all(root).expect("temporary catalog should be removable");
    }

    #[test]
    fn media_query_is_bounded_and_deterministic() {
        let (db_path, root) = temporary_catalog();
        init_database(&db_path).expect("migration should succeed");

        let source = insert_or_get_source(&db_path, r"C:\Synthetic", "Synthetic")
            .expect("source should be inserted");

        let items = vec![
            DiscoveredMedia {
                relative_path: "one.jpg".into(),
                file_name: "one.jpg".into(),
                extension: "jpg".into(),
                media_type: "image".into(),
                size_bytes: 10,
                created_at_fs: Some(1),
                modified_at_fs: Some(1),
            },
            DiscoveredMedia {
                relative_path: "two.mp4".into(),
                file_name: "two.mp4".into(),
                extension: "mp4".into(),
                media_type: "video".into(),
                size_bytes: 20,
                created_at_fs: Some(2),
                modified_at_fs: Some(2),
            },
        ];

        upsert_media_batch(&db_path, source.id, &items).expect("batch should persist");

        let first = query_media(&db_path, 0, 1).expect("first page should load");
        let second = query_media(&db_path, 1, 1).expect("second page should load");

        assert_eq!(first.total, 2);
        assert_eq!(first.items.len(), 1);
        assert_eq!(first.items[0].file_name, "one.jpg");
        assert_eq!(second.items.len(), 1);
        assert_eq!(second.items[0].file_name, "two.mp4");

        fs::remove_dir_all(root).expect("temporary catalog should be removable");
    }

    #[test]
    fn duplicate_source_is_reused() {
        let (db_path, root) = temporary_catalog();
        init_database(&db_path).expect("migration should succeed");

        let first = insert_or_get_source(&db_path, r"D:\Media", "Media")
            .expect("source should be inserted");
        let second = insert_or_get_source(&db_path, r"D:\Media", "Renamed")
            .expect("source should be reused");

        assert_eq!(first.id, second.id);
        assert_eq!(second.display_name, "Renamed");

        fs::remove_dir_all(root).expect("temporary catalog should be removable");
    }

    #[test]
    #[ignore = "manual 100k catalog smoke fixture"]
    fn catalog_100k_fixture_stays_paginated() {
        use std::time::Instant;

        let (db_path, root) = temporary_catalog();
        init_database(&db_path).expect("migration should succeed");
        let source = insert_or_get_source(&db_path, r"E:\Synthetic-100k", "Synthetic 100k")
            .expect("source should be inserted");

        const TOTAL: usize = 100_000;
        const BATCH: usize = 1_000;

        for start in (0..TOTAL).step_by(BATCH) {
            let end = (start + BATCH).min(TOTAL);
            let items = (start..end)
                .map(|index| DiscoveredMedia {
                    relative_path: format!("folder/{index:06}.jpg"),
                    file_name: format!("{index:06}.jpg"),
                    extension: "jpg".into(),
                    media_type: "image".into(),
                    size_bytes: 1_024 + index as i64,
                    created_at_fs: Some(index as i64),
                    modified_at_fs: Some(index as i64),
                })
                .collect::<Vec<_>>();

            upsert_media_batch(&db_path, source.id, &items)
                .expect("synthetic batch should persist");
        }

        let started = Instant::now();
        let page = query_media(&db_path, 50_000, 240)
            .expect("bounded page should load from large fixture");
        let elapsed = started.elapsed();

        assert_eq!(page.total, TOTAL as i64);
        assert_eq!(page.items.len(), 240);
        assert_eq!(page.offset, 50_000);
        assert_eq!(page.limit, 240);

        println!(
            "100k catalog page: {} items in {:?}",
            page.items.len(),
            elapsed
        );

        fs::remove_dir_all(root).expect("temporary catalog should be removable");
    }

}
