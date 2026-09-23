mod db;
mod models;
mod scanner;
mod thumbnails;

use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use models::{MediaPage, MediaQuery, ScanProgress, Source};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::db::AppState;

#[tauri::command]
fn health() -> &'static str {
    "ok"
}

#[tauri::command]
fn get_ui_preferences(state: State<'_, AppState>) -> Result<models::UiPreferences, String> {
    db::get_ui_preferences(&state.db_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_ui_preferences(
    preferences: models::UiPreferences,
    state: State<'_, AppState>,
) -> Result<models::UiPreferences, String> {
    db::save_ui_preferences(&state.db_path, preferences).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_sources(state: State<'_, AppState>) -> Result<Vec<Source>, String> {
    db::refresh_source_availability(&state.db_path).map_err(|error| error.to_string())?;
    db::list_sources(&state.db_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn add_source(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Source, String> {
    let requested = PathBuf::from(path);

    if !requested.is_absolute() {
        return Err("source path must be absolute".into());
    }

    let metadata = fs::metadata(&requested)
        .map_err(|error| format!("cannot access source: {error}"))?;

    if !metadata.is_dir() {
        return Err("selected source is not a directory".into());
    }

    let root_path = requested.to_string_lossy().into_owned();
    let display_name = requested
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| root_path.clone());

    app.asset_protocol_scope()
        .allow_directory(&requested, true)
        .map_err(|error| format!("failed to expose selected source to viewer: {error}"))?;

    db::insert_or_get_source(&state.db_path, &root_path, &display_name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn start_scan(
    source_id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let cancel = Arc::new(AtomicBool::new(false));

    {
        let mut scanning = state
            .scanning
            .lock()
            .map_err(|_| "scanner state is unavailable".to_string())?;

        if scanning.contains_key(&source_id) {
            return Ok(false);
        }

        scanning.insert(source_id, cancel.clone());
    }

    let state = state.inner().clone();
    let app_for_task = app.clone();

    std::thread::spawn(move || {
        if let Err(error) = scanner::run_scan(
            app_for_task.clone(),
            state.clone(),
            source_id,
            cancel,
        ) {
            let _ = db::mark_scan_error(&state.db_path, source_id);
            let _ = app_for_task.emit(
                "scan-progress",
                ScanProgress {
                    source_id,
                    discovered: 0,
                    supported: 0,
                    errors: 1,
                    done: true,
                    cancelled: false,
                    message: Some(error.to_string()),
                },
            );
        }

        if let Ok(mut scanning) = state.scanning.lock() {
            scanning.remove(&source_id);
        }
    });

    Ok(true)
}

#[tauri::command]
fn cancel_scan(source_id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    let scanning = state
        .scanning
        .lock()
        .map_err(|_| "scanner state is unavailable".to_string())?;

    let Some(cancel) = scanning.get(&source_id) else {
        return Ok(false);
    };

    cancel.store(true, Ordering::Relaxed);
    Ok(true)
}

#[tauri::command]
fn remove_source(source_id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    {
        let scanning = state
            .scanning
            .lock()
            .map_err(|_| "scanner state is unavailable".to_string())?;

        if scanning.contains_key(&source_id) {
            return Err("Cancele a varredura antes de remover esta fonte.".into());
        }
    }

    db::remove_source(&state.db_path, source_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn query_media(
    query: MediaQuery,
    state: State<'_, AppState>,
) -> Result<MediaPage, String> {
    db::query_media_filtered(&state.db_path, &query).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_media_item(
    media_id: i64,
    state: State<'_, AppState>,
) -> Result<models::MediaItem, String> {
    db::get_media_item(&state.db_path, media_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_extensions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    db::list_extensions(&state.db_path).map_err(|error| error.to_string())
}

#[tauri::command]
async fn ensure_thumbnail(
    media_id: i64,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let state = state.inner().clone();

    tauri::async_runtime::spawn_blocking(move || thumbnails::ensure_thumbnail(&state, media_id))
        .await
        .map_err(|error| format!("thumbnail task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_media_external(
    media_id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let media = db::media_path(&state.db_path, media_id).map_err(|error| error.to_string())?;
    let root = fs::canonicalize(&media.root_path)
        .map_err(|_| "A fonte desta mídia está offline ou indisponível.".to_string())?;
    let full = fs::canonicalize(root.join(&media.relative_path))
        .map_err(|_| "O arquivo original não está disponível.".to_string())?;

    if !full.starts_with(&root) {
        return Err("O caminho da mídia saiu da fonte cadastrada.".into());
    }

    app.opener()
        .open_path(full.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|error| format!("Não foi possível abrir no aplicativo padrão: {error}"))
}

#[tauri::command]
fn media_asset_path(
    media_id: i64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let media = db::media_path(&state.db_path, media_id).map_err(|error| error.to_string())?;
    let root = fs::canonicalize(&media.root_path)
        .map_err(|_| "A fonte desta mídia está offline ou indisponível.".to_string())?;
    let full = fs::canonicalize(root.join(&media.relative_path))
        .map_err(|error| format!("media is unavailable: {error}"))?;

    if !full.starts_with(&root) {
        return Err("media resolved outside its registered source".into());
    }

    app.asset_protocol_scope()
        .allow_file(&full)
        .map_err(|error| format!("failed to expose media to viewer: {error}"))?;

    Ok(full.to_string_lossy().into_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let data_dir = app.path().app_local_data_dir()?;
            let cache_dir = app.path().app_cache_dir()?;

            fs::create_dir_all(&data_dir)?;
            fs::create_dir_all(&cache_dir)?;

            let state = AppState::new(data_dir.join("library.db"), cache_dir);
            db::init_database(&state.db_path)?;
            db::refresh_source_availability(&state.db_path)?;

            for source in db::list_sources(&state.db_path)? {
                let root = PathBuf::from(source.root_path);
                if root.is_dir() {
                    let _ = app.asset_protocol_scope().allow_directory(root, true);
                }
            }

            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health,
            get_ui_preferences,
            save_ui_preferences,
            list_sources,
            add_source,
            start_scan,
            cancel_scan,
            remove_source,
            query_media,
            get_media_item,
            list_extensions,
            ensure_thumbnail,
            open_media_external,
            media_asset_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lume");
}
