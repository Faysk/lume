use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use rusqlite::{params, params_from_iter, types::Value, Connection};

use crate::models::{MediaItem, MediaPage, MediaQuery, Source};

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_path: PathBuf,
    pub cache_dir: PathBuf,
    pub scanning: Arc<
        Mutex<
            std::collections::HashMap<
                i64,
                Arc<std::sync::atomic::AtomicBool>,
            >,
        >,
    >,
}

impl AppState {
    pub fn new(db_path: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            db_path,
            cache_dir,
            scanning: Arc::new(Mutex::new(std::collections::HashMap::new())),
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

    if current_version < 2 {
        connection
            .execute_batch(include_str!("../migrations/0002_scan_reconciliation.sql"))
            .context("failed to apply migration 0002_scan_reconciliation")?;
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

pub fn refresh_source_availability(db_path: &Path) -> Result<()> {
    let connection = open(db_path)?;
    let sources = {
        let mut statement = connection.prepare(
            "SELECT id, root_path, status FROM sources ORDER BY id ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    for (id, root_path, status) in sources {
        if status == "scanning" {
            continue;
        }

        let available = Path::new(&root_path).is_dir();

        if !available && status != "offline" {
            connection.execute(
                "UPDATE sources SET status = 'offline' WHERE id = ?1",
                params![id],
            )?;
        } else if available && status == "offline" {
            connection.execute(
                "UPDATE sources SET status = 'online' WHERE id = ?1",
                params![id],
            )?;
        }
    }

    Ok(())
}

pub fn remove_source(db_path: &Path, source_id: i64) -> Result<bool> {
    let connection = open(db_path)?;
    let affected = connection.execute(
        "DELETE FROM sources WHERE id = ?1",
        params![source_id],
    )?;
    Ok(affected > 0)
}

pub fn mark_scan_started(db_path: &Path, source_id: i64) -> Result<i64> {
    let connection = open(db_path)?;
    let affected = connection.execute(
        "
        UPDATE sources
        SET status = 'scanning',
            scan_generation = scan_generation + 1,
            last_scan_started_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        ",
        params![source_id],
    )?;

    if affected == 0 {
        anyhow::bail!("source {source_id} not found");
    }

    connection
        .query_row(
            "SELECT scan_generation FROM sources WHERE id = ?1",
            params![source_id],
            |row| row.get(0),
        )
        .context("failed to read source scan generation")
}

pub fn finish_scan_and_reconcile(
    db_path: &Path,
    source_id: i64,
    generation: i64,
) -> Result<()> {
    let mut connection = open(db_path)?;
    let transaction = connection.transaction()?;

    transaction.execute(
        "
        UPDATE media
        SET is_present = 0,
            updated_at = CURRENT_TIMESTAMP
        WHERE source_id = ?1
          AND last_seen_generation <> ?2
          AND is_present = 1
        ",
        params![source_id, generation],
    )?;

    transaction.execute(
        "
        UPDATE sources
        SET status = 'online',
            last_scan_finished_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        ",
        params![source_id],
    )?;

    transaction.commit()?;
    Ok(())
}

pub fn mark_scan_cancelled(db_path: &Path, source_id: i64) -> Result<()> {
    let connection = open(db_path)?;
    connection.execute(
        "UPDATE sources SET status = 'online' WHERE id = ?1",
        params![source_id],
    )?;
    Ok(())
}

pub fn mark_scan_partial(db_path: &Path, source_id: i64) -> Result<()> {
    let connection = open(db_path)?;
    connection.execute(
        "UPDATE sources SET status = 'error' WHERE id = ?1",
        params![source_id],
    )?;
    Ok(())
}

pub fn mark_source_offline(db_path: &Path, source_id: i64) -> Result<()> {
    let connection = open(db_path)?;
    connection.execute(
        "UPDATE sources SET status = 'offline' WHERE id = ?1",
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
    upsert_media_batch_for_scan(db_path, source_id, 0, items)
}

pub fn upsert_media_batch_for_scan(
    db_path: &Path,
    source_id: i64,
    generation: i64,
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
                size_bytes, created_at_fs, modified_at_fs,
                is_present, last_seen_generation
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)
            ON CONFLICT(source_id, relative_path) DO UPDATE SET
                file_name = excluded.file_name,
                extension = excluded.extension,
                media_type = excluded.media_type,
                size_bytes = excluded.size_bytes,
                created_at_fs = excluded.created_at_fs,
                modified_at_fs = excluded.modified_at_fs,
                is_present = 1,
                last_seen_generation = excluded.last_seen_generation,
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
                generation,
            ])?;
        }
    }

    transaction.commit()?;
    Ok(())
}

pub fn list_extensions(db_path: &Path) -> Result<Vec<String>> {
    let connection = open(db_path)?;
    let mut statement = connection.prepare(
        "SELECT DISTINCT extension FROM media WHERE is_present = 1 ORDER BY extension COLLATE NOCASE ASC",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .context("failed to read media extensions")
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn build_media_filter(query: &MediaQuery) -> (String, Vec<Value>) {
    let mut clauses = vec!["m.is_present = 1".to_string()];
    let mut values = Vec::new();

    if let Some(search) = query.search.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        clauses.push("m.file_name LIKE ? ESCAPE '\\' COLLATE NOCASE".to_string());
        values.push(Value::Text(format!("%{}%", escape_like(search))));
    }

    if let Some(media_type) = query
        .media_type
        .as_deref()
        .map(str::trim)
        .filter(|value| matches!(*value, "image" | "video"))
    {
        clauses.push("m.media_type = ?".to_string());
        values.push(Value::Text(media_type.to_string()));
    }

    let extensions = query
        .extensions
        .iter()
        .map(|value| value.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if !extensions.is_empty() {
        clauses.push(format!(
            "m.extension IN ({})",
            std::iter::repeat("?")
                .take(extensions.len())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        values.extend(extensions.into_iter().map(Value::Text));
    }

    if !query.source_ids.is_empty() {
        clauses.push(format!(
            "m.source_id IN ({})",
            std::iter::repeat("?")
                .take(query.source_ids.len())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        values.extend(query.source_ids.iter().copied().map(Value::Integer));
    }

    if let Some(value) = query.modified_from {
        clauses.push("COALESCE(m.modified_at_fs, m.created_at_fs, 0) >= ?".to_string());
        values.push(Value::Integer(value));
    }

    if let Some(value) = query.modified_to {
        clauses.push("COALESCE(m.modified_at_fs, m.created_at_fs, 0) <= ?".to_string());
        values.push(Value::Integer(value));
    }

    if let Some(value) = query.min_size_bytes.filter(|value| *value >= 0) {
        clauses.push("m.size_bytes >= ?".to_string());
        values.push(Value::Integer(value));
    }

    if let Some(value) = query.max_size_bytes.filter(|value| *value >= 0) {
        clauses.push("m.size_bytes <= ?".to_string());
        values.push(Value::Integer(value));
    }

    let sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };

    (sql, values)
}

fn media_order_by(sort: Option<&str>) -> &'static str {
    match sort {
        Some("date_desc") => "COALESCE(m.modified_at_fs, m.created_at_fs, 0) DESC, m.id DESC",
        Some("date_asc") => "COALESCE(m.modified_at_fs, m.created_at_fs, 0) ASC, m.id ASC",
        Some("name_asc") => "m.file_name COLLATE NOCASE ASC, m.id ASC",
        Some("name_desc") => "m.file_name COLLATE NOCASE DESC, m.id DESC",
        Some("size_asc") => "m.size_bytes ASC, m.id ASC",
        Some("size_desc") => "m.size_bytes DESC, m.id DESC",
        _ => "m.id ASC",
    }
}

pub fn query_media(db_path: &Path, offset: u32, limit: u32) -> Result<MediaPage> {
    query_media_filtered(
        db_path,
        &MediaQuery {
            offset,
            limit,
            ..MediaQuery::default()
        },
    )
}

pub fn query_media_filtered(db_path: &Path, query: &MediaQuery) -> Result<MediaPage> {
    let limit = query.limit.clamp(1, 500);
    let (filter_sql, filter_values) = build_media_filter(query);
    let connection = open(db_path)?;

    let count_sql = format!(
        "SELECT COUNT(*) FROM media m JOIN sources s ON s.id = m.source_id{}",
        filter_sql
    );
    let total: i64 = connection.query_row(
        &count_sql,
        params_from_iter(filter_values.iter()),
        |row| row.get(0),
    )?;

    let select_sql = format!(
        "
        SELECT
            m.id, m.source_id, m.relative_path, m.file_name, m.extension,
            m.media_type, m.size_bytes, m.created_at_fs, m.modified_at_fs,
            m.width, m.height, m.thumbnail_state,
            s.display_name, s.root_path
        FROM media m
        JOIN sources s ON s.id = m.source_id
        {filter_sql}
        ORDER BY {order_by}
        LIMIT ? OFFSET ?
        ",
        order_by = media_order_by(query.sort.as_deref()),
    );

    let mut values = filter_values;
    values.push(Value::Integer(i64::from(limit)));
    values.push(Value::Integer(i64::from(query.offset)));

    let mut statement = connection.prepare(&select_sql)?;
    let rows = statement.query_map(params_from_iter(values.iter()), |row| {
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
        offset: query.offset,
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
    fn version_one_catalog_upgrades_without_losing_media() {
        let (db_path, root) = temporary_catalog();
        let connection = open(&db_path).unwrap();
        connection
            .execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        connection.execute(
            "INSERT INTO sources(root_path, display_name) VALUES (?1, ?2)",
            params![r"C:\Legacy", "Legacy"],
        ).unwrap();
        let source_id = connection.last_insert_rowid();
        connection.execute(
            "
            INSERT INTO media(
                source_id, relative_path, file_name, extension, media_type,
                size_bytes, created_at_fs, modified_at_fs
            ) VALUES (?1, 'old.jpg', 'old.jpg', 'jpg', 'image', 10, 1, 1)
            ",
            params![source_id],
        ).unwrap();
        drop(connection);

        init_database(&db_path).expect("v1 catalog should upgrade to v2");
        let connection = open(&db_path).unwrap();
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        let present: i64 = connection
            .query_row(
                "SELECT is_present FROM media WHERE file_name = 'old.jpg'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(version, 2);
        assert_eq!(present, 1);
        drop(connection);
        fs::remove_dir_all(root).unwrap();
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

        assert_eq!(version, 2);
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
    fn completed_scan_reconciles_missing_items_but_cancelled_scan_does_not() {
        let (db_path, root) = temporary_catalog();
        init_database(&db_path).expect("migration should succeed");
        let source = insert_or_get_source(&db_path, r"C:\Reconcile", "Reconcile").unwrap();

        let generation_one = mark_scan_started(&db_path, source.id).unwrap();
        upsert_media_batch_for_scan(
            &db_path,
            source.id,
            generation_one,
            &[
                DiscoveredMedia {
                    relative_path: "keep.jpg".into(),
                    file_name: "keep.jpg".into(),
                    extension: "jpg".into(),
                    media_type: "image".into(),
                    size_bytes: 10,
                    created_at_fs: Some(1),
                    modified_at_fs: Some(1),
                },
                DiscoveredMedia {
                    relative_path: "gone.jpg".into(),
                    file_name: "gone.jpg".into(),
                    extension: "jpg".into(),
                    media_type: "image".into(),
                    size_bytes: 20,
                    created_at_fs: Some(1),
                    modified_at_fs: Some(1),
                },
            ],
        ).unwrap();
        finish_scan_and_reconcile(&db_path, source.id, generation_one).unwrap();
        assert_eq!(query_media(&db_path, 0, 50).unwrap().total, 2);

        let generation_two = mark_scan_started(&db_path, source.id).unwrap();
        upsert_media_batch_for_scan(
            &db_path,
            source.id,
            generation_two,
            &[DiscoveredMedia {
                relative_path: "keep.jpg".into(),
                file_name: "keep.jpg".into(),
                extension: "jpg".into(),
                media_type: "image".into(),
                size_bytes: 10,
                created_at_fs: Some(1),
                modified_at_fs: Some(1),
            }],
        ).unwrap();

        mark_scan_cancelled(&db_path, source.id).unwrap();
        assert_eq!(query_media(&db_path, 0, 50).unwrap().total, 2);

        let generation_three = mark_scan_started(&db_path, source.id).unwrap();
        upsert_media_batch_for_scan(
            &db_path,
            source.id,
            generation_three,
            &[DiscoveredMedia {
                relative_path: "keep.jpg".into(),
                file_name: "keep.jpg".into(),
                extension: "jpg".into(),
                media_type: "image".into(),
                size_bytes: 10,
                created_at_fs: Some(1),
                modified_at_fs: Some(1),
            }],
        ).unwrap();
        finish_scan_and_reconcile(&db_path, source.id, generation_three).unwrap();

        let page = query_media(&db_path, 0, 50).unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].file_name, "keep.jpg");

        fs::remove_dir_all(root).expect("temporary catalog should be removable");
    }

    #[test]
    fn removing_source_only_removes_catalog_rows() {
        let (db_path, root) = temporary_catalog();
        init_database(&db_path).unwrap();
        let source = insert_or_get_source(&db_path, r"C:\Remove", "Remove").unwrap();
        upsert_media_batch(
            &db_path,
            source.id,
            &[DiscoveredMedia {
                relative_path: "keep-on-disk.jpg".into(),
                file_name: "keep-on-disk.jpg".into(),
                extension: "jpg".into(),
                media_type: "image".into(),
                size_bytes: 10,
                created_at_fs: Some(1),
                modified_at_fs: Some(1),
            }],
        ).unwrap();

        assert!(remove_source(&db_path, source.id).unwrap());
        assert_eq!(query_media(&db_path, 0, 50).unwrap().total, 0);
        assert!(list_sources(&db_path).unwrap().is_empty());

        fs::remove_dir_all(root).unwrap();
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
    fn filtered_query_combines_search_type_extension_source_and_sort() {
        let (db_path, root) = temporary_catalog();
        init_database(&db_path).expect("migration should succeed");

        let source_a = insert_or_get_source(&db_path, r"C:\A", "A").unwrap();
        let source_b = insert_or_get_source(&db_path, r"D:\B", "B").unwrap();

        upsert_media_batch(
            &db_path,
            source_a.id,
            &[
                DiscoveredMedia {
                    relative_path: "Sunset.JPG".into(),
                    file_name: "Sunset.JPG".into(),
                    extension: "jpg".into(),
                    media_type: "image".into(),
                    size_bytes: 50,
                    created_at_fs: Some(10),
                    modified_at_fs: Some(20),
                },
                DiscoveredMedia {
                    relative_path: "sunset.mp4".into(),
                    file_name: "sunset.mp4".into(),
                    extension: "mp4".into(),
                    media_type: "video".into(),
                    size_bytes: 500,
                    created_at_fs: Some(11),
                    modified_at_fs: Some(21),
                },
            ],
        ).unwrap();

        upsert_media_batch(
            &db_path,
            source_b.id,
            &[DiscoveredMedia {
                relative_path: "Sunset-Elsewhere.jpg".into(),
                file_name: "Sunset-Elsewhere.jpg".into(),
                extension: "jpg".into(),
                media_type: "image".into(),
                size_bytes: 700,
                created_at_fs: Some(12),
                modified_at_fs: Some(22),
            }],
        ).unwrap();

        let page = query_media_filtered(
            &db_path,
            &MediaQuery {
                offset: 0,
                limit: 50,
                search: Some("sunset".into()),
                media_type: Some("image".into()),
                extensions: vec!["JPG".into()],
                source_ids: vec![source_a.id],
                min_size_bytes: Some(1),
                max_size_bytes: Some(100),
                sort: Some("name_desc".into()),
                ..MediaQuery::default()
            },
        ).expect("filtered page should load");

        assert_eq!(page.total, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].file_name, "Sunset.JPG");

        fs::remove_dir_all(root).expect("temporary catalog should be removable");
    }

    #[test]
    fn filtered_query_keeps_sort_tie_breaker_deterministic() {
        let (db_path, root) = temporary_catalog();
        init_database(&db_path).expect("migration should succeed");
        let source = insert_or_get_source(&db_path, r"C:\Sort", "Sort").unwrap();

        for index in 0..3 {
            upsert_media_batch(
                &db_path,
                source.id,
                &[DiscoveredMedia {
                    relative_path: format!("folder-{index}/same.jpg"),
                    file_name: "same.jpg".into(),
                    extension: "jpg".into(),
                    media_type: "image".into(),
                    size_bytes: 100,
                    created_at_fs: Some(10),
                    modified_at_fs: Some(10),
                }],
            ).unwrap();
        }

        let page = query_media_filtered(
            &db_path,
            &MediaQuery {
                offset: 0,
                limit: 50,
                sort: Some("name_asc".into()),
                ..MediaQuery::default()
            },
        ).unwrap();

        let ids = page.items.iter().map(|item| item.id).collect::<Vec<_>>();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);

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
