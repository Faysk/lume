use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

use crate::{
    db::{self, AppState, DiscoveredMedia},
    models::ScanProgress,
};

const BATCH_SIZE: usize = 128;

fn unix_seconds(value: Result<SystemTime, std::io::Error>) -> Option<i64> {
    value
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
}

fn media_type_for_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "jpg" | "jpeg" | "png" | "webp" | "gif" => Some("image"),
        "mp4" | "mov" | "webm" => Some("video"),
        _ => None,
    }
}

fn emit_progress(app: &AppHandle, progress: ScanProgress) {
    let _ = app.emit("scan-progress", progress);
}

fn cancelled(
    app: &AppHandle,
    state: &AppState,
    source_id: i64,
    cancel: &AtomicBool,
    discovered: u64,
    supported: u64,
    errors: u64,
) -> Result<bool> {
    if !cancel.load(Ordering::Relaxed) {
        return Ok(false);
    }

    db::mark_scan_cancelled(&state.db_path, source_id)?;
    emit_progress(
        app,
        ScanProgress {
            source_id,
            discovered,
            supported,
            errors,
            done: true,
            cancelled: true,
            message: Some("Varredura cancelada. O catálogo anterior foi preservado.".into()),
        },
    );
    Ok(true)
}

pub fn run_scan(
    app: AppHandle,
    state: AppState,
    source_id: i64,
    cancel: Arc<AtomicBool>,
) -> Result<()> {
    let source = db::get_source(&state.db_path, source_id)?;
    let root = Path::new(&source.root_path);

    if !root.is_dir() {
        db::mark_source_offline(&state.db_path, source_id)?;
        emit_progress(
            &app,
            ScanProgress {
                source_id,
                discovered: 0,
                supported: 0,
                errors: 0,
                done: true,
                cancelled: false,
                message: Some("A fonte não está disponível neste momento.".into()),
            },
        );
        return Ok(());
    }

    let generation = db::mark_scan_started(&state.db_path, source_id)?;

    let mut discovered = 0_u64;
    let mut supported = 0_u64;
    let mut errors = 0_u64;
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    for entry in WalkDir::new(root)
        .follow_links(false)
        .same_file_system(false)
        .into_iter()
    {
        if cancelled(
            &app,
            &state,
            source_id,
            &cancel,
            discovered,
            supported,
            errors,
        )? {
            return Ok(());
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                errors += 1;
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        discovered += 1;

        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let Some(media_type) = media_type_for_extension(&extension) else {
            continue;
        };

        let relative_path = match path.strip_prefix(root) {
            Ok(relative) => relative.to_string_lossy().into_owned(),
            Err(_) => {
                errors += 1;
                continue;
            }
        };

        let file_name = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| relative_path.clone());

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                errors += 1;
                continue;
            }
        };

        supported += 1;
        batch.push(DiscoveredMedia {
            relative_path,
            file_name,
            extension,
            media_type: media_type.to_owned(),
            size_bytes: metadata.len().min(i64::MAX as u64) as i64,
            created_at_fs: unix_seconds(metadata.created()),
            modified_at_fs: unix_seconds(metadata.modified()),
        });

        if batch.len() >= BATCH_SIZE {
            db::upsert_media_batch_for_scan(
                &state.db_path,
                source_id,
                generation,
                &batch,
            )?;
            batch.clear();

            emit_progress(
                &app,
                ScanProgress {
                    source_id,
                    discovered,
                    supported,
                    errors,
                    done: false,
                    cancelled: false,
                    message: None,
                },
            );
        }
    }

    if cancelled(
        &app,
        &state,
        source_id,
        &cancel,
        discovered,
        supported,
        errors,
    )? {
        return Ok(());
    }

    db::upsert_media_batch_for_scan(
        &state.db_path,
        source_id,
        generation,
        &batch,
    )
    .context("failed to persist final scan batch")?;

    if cancelled(
        &app,
        &state,
        source_id,
        &cancel,
        discovered,
        supported,
        errors,
    )? {
        return Ok(());
    }

    db::finish_scan_and_reconcile(&state.db_path, source_id, generation)?;

    emit_progress(
        &app,
        ScanProgress {
            source_id,
            discovered,
            supported,
            errors,
            done: true,
            cancelled: false,
            message: None,
        },
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::media_type_for_extension;

    #[test]
    fn supported_extensions_are_intentionally_small_for_v01() {
        for extension in ["jpg", "jpeg", "png", "webp", "gif"] {
            assert_eq!(media_type_for_extension(extension), Some("image"));
        }

        for extension in ["mp4", "mov", "webm"] {
            assert_eq!(media_type_for_extension(extension), Some("video"));
        }

        for extension in ["txt", "zip", "mkv", "exe", "raw"] {
            assert_eq!(media_type_for_extension(extension), None);
        }
    }
}
