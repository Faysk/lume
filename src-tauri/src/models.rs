use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: i64,
    pub root_path: String,
    pub display_name: String,
    pub status: String,
    pub last_scan_started_at: Option<String>,
    pub last_scan_finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: i64,
    pub source_id: i64,
    pub relative_path: String,
    pub file_name: String,
    pub extension: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub created_at_fs: Option<i64>,
    pub modified_at_fs: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub thumbnail_state: String,
    pub source_name: String,
    pub source_root: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaQuery {
    pub offset: u32,
    pub limit: u32,
    pub search: Option<String>,
    pub media_type: Option<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub source_ids: Vec<i64>,
    pub modified_from: Option<i64>,
    pub modified_to: Option<i64>,
    pub min_size_bytes: Option<i64>,
    pub max_size_bytes: Option<i64>,
    pub sort: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaPage {
    pub items: Vec<MediaItem>,
    pub total: i64,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub source_id: i64,
    pub discovered: u64,
    pub supported: u64,
    pub errors: u64,
    pub done: bool,
    pub message: Option<String>,
}
