use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use image::{GenericImageView, ImageFormat};

use crate::db::{self, AppState};

const THUMBNAIL_EDGE: u32 = 512;

fn safe_media_path(root: &Path, relative: &Path) -> Result<PathBuf> {
    let root = fs::canonicalize(root)
        .with_context(|| format!("source is unavailable: {}", root.display()))?;
    let candidate = fs::canonicalize(root.join(relative))
        .with_context(|| format!("media is unavailable: {}", relative.display()))?;

    if !candidate.starts_with(&root) {
        anyhow::bail!("media resolved outside its registered source");
    }

    Ok(candidate)
}

pub fn ensure_thumbnail(state: &AppState, media_id: i64) -> Result<String> {
    let media = db::media_path(&state.db_path, media_id)?;
    let thumbnail_dir = state.cache_dir.join("thumbnails");
    fs::create_dir_all(&thumbnail_dir)?;

    let version = media.modified_at_fs.unwrap_or(0);
    let output = thumbnail_dir.join(format!("{media_id}-{version}.png"));

    if output.is_file() {
        return Ok(output.to_string_lossy().into_owned());
    }

    let root = Path::new(&media.root_path);
    let relative = Path::new(&media.relative_path);
    let source_path = safe_media_path(root, relative)?;

    let extension = source_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if !matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif") {
        anyhow::bail!("thumbnail generation is not available for this media type yet");
    }

    let result = (|| -> Result<(u32, u32)> {
        let image = image::ImageReader::open(&source_path)
            .with_context(|| format!("failed to open {}", source_path.display()))?
            .with_guessed_format()?
            .decode()
            .with_context(|| format!("failed to decode {}", source_path.display()))?;

        let dimensions = image.dimensions();
        let thumbnail = image.thumbnail(THUMBNAIL_EDGE, THUMBNAIL_EDGE);
        let temporary = output.with_extension("png.part");

        thumbnail
            .save_with_format(&temporary, ImageFormat::Png)
            .with_context(|| format!("failed to write thumbnail {}", temporary.display()))?;

        if output.exists() {
            let _ = fs::remove_file(&temporary);
        } else {
            fs::rename(&temporary, &output)
                .with_context(|| format!("failed to promote thumbnail {}", output.display()))?;
        }

        Ok(dimensions)
    })();

    match result {
        Ok((width, height)) => {
            db::mark_thumbnail_ready(
                &state.db_path,
                media_id,
                i64::from(width),
                i64::from(height),
            )?;
            Ok(output.to_string_lossy().into_owned())
        }
        Err(error) => {
            let _ = db::mark_thumbnail_failed(&state.db_path, media_id);
            Err(error)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use image::{ImageBuffer, Rgba};

    use crate::db::{self, DiscoveredMedia};

    use super::*;

    fn temporary_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "lume-thumbnail-test-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary root should be created");
        root
    }

    #[test]
    fn thumbnail_is_generated_outside_source_and_reused() {
        let root = temporary_root("reuse");
        let source_dir = root.join("source");
        let cache_dir = root.join("cache");
        let data_dir = root.join("data");
        fs::create_dir_all(&source_dir).expect("source should be created");
        fs::create_dir_all(&cache_dir).expect("cache should be created");
        fs::create_dir_all(&data_dir).expect("data should be created");

        let original = source_dir.join("photo.png");
        let image = ImageBuffer::from_pixel(900, 600, Rgba([240_u8, 180, 92, 255]));
        image.save(&original).expect("synthetic image should be written");

        let db_path = data_dir.join("library.db");
        db::init_database(&db_path).expect("catalog should initialize");
        let source = db::insert_or_get_source(
            &db_path,
            &source_dir.to_string_lossy(),
            "Synthetic",
        )
        .expect("source should be inserted");

        db::upsert_media_batch(
            &db_path,
            source.id,
            &[DiscoveredMedia {
                relative_path: "photo.png".into(),
                file_name: "photo.png".into(),
                extension: "png".into(),
                media_type: "image".into(),
                size_bytes: fs::metadata(&original).unwrap().len() as i64,
                created_at_fs: Some(1),
                modified_at_fs: Some(42),
            }],
        )
        .expect("media should be inserted");

        let page = db::query_media(&db_path, 0, 1).expect("media should load");
        let media_id = page.items[0].id;
        let state = AppState::new(db_path.clone(), cache_dir.clone());

        let first = PathBuf::from(ensure_thumbnail(&state, media_id).expect("thumbnail should build"));
        let first_modified = fs::metadata(&first)
            .and_then(|metadata| metadata.modified())
            .expect("thumbnail metadata should be readable");
        let second = PathBuf::from(ensure_thumbnail(&state, media_id).expect("thumbnail should reuse"));
        let second_modified = fs::metadata(&second)
            .and_then(|metadata| metadata.modified())
            .expect("thumbnail metadata should remain readable");

        assert_eq!(first, second);
        assert!(first.starts_with(&cache_dir));
        assert!(!first.starts_with(&source_dir));
        assert_eq!(first_modified, second_modified);

        let thumbnail = image::open(&first).expect("thumbnail should decode");
        assert!(thumbnail.width() <= THUMBNAIL_EDGE);
        assert!(thumbnail.height() <= THUMBNAIL_EDGE);
        assert_eq!(thumbnail.width(), 512);
        assert_eq!(thumbnail.height(), 341);

        let original_after = image::open(&original).expect("original should remain readable");
        assert_eq!(original_after.width(), 900);
        assert_eq!(original_after.height(), 600);

        fs::remove_dir_all(root).expect("temporary tree should be removable");
    }

    #[test]
    fn cached_thumbnail_remains_available_when_source_goes_offline() {
        let root = temporary_root("offline-cache");
        let source_dir = root.join("source");
        let cache_dir = root.join("cache");
        let data_dir = root.join("data");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&cache_dir).unwrap();
        fs::create_dir_all(&data_dir).unwrap();

        let original = source_dir.join("photo.png");
        ImageBuffer::from_pixel(320, 240, Rgba([1_u8, 2, 3, 255]))
            .save(&original)
            .unwrap();

        let db_path = data_dir.join("library.db");
        db::init_database(&db_path).unwrap();
        let source = db::insert_or_get_source(
            &db_path,
            &source_dir.to_string_lossy(),
            "Offline",
        )
        .unwrap();

        db::upsert_media_batch(
            &db_path,
            source.id,
            &[DiscoveredMedia {
                relative_path: "photo.png".into(),
                file_name: "photo.png".into(),
                extension: "png".into(),
                media_type: "image".into(),
                size_bytes: 1,
                created_at_fs: Some(1),
                modified_at_fs: Some(99),
            }],
        )
        .unwrap();

        let media_id = db::query_media(&db_path, 0, 1).unwrap().items[0].id;
        let state = AppState::new(db_path, cache_dir);
        let cached = ensure_thumbnail(&state, media_id).unwrap();
        assert!(Path::new(&cached).is_file());

        fs::remove_dir_all(&source_dir).unwrap();
        let reused = ensure_thumbnail(&state, media_id).unwrap();
        assert_eq!(cached, reused);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn safe_media_path_rejects_escape_from_source() {
        let root = temporary_root("escape");
        let source_dir = root.join("source");
        fs::create_dir_all(&source_dir).expect("source should be created");
        let outside = root.join("outside.png");
        fs::write(&outside, b"not-an-image").expect("outside file should exist");

        let error = safe_media_path(&source_dir, Path::new("../outside.png"))
            .expect_err("path traversal must be rejected");

        assert!(error.to_string().contains("outside its registered source"));
        fs::remove_dir_all(root).expect("temporary tree should be removable");
    }
}
