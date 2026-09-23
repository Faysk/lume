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

    let thumbnail_dir = state.cache_dir.join("thumbnails");
    fs::create_dir_all(&thumbnail_dir)?;

    let version = media.modified_at_fs.unwrap_or(0);
    let output = thumbnail_dir.join(format!("{media_id}-{version}.png"));

    if output.is_file() {
        return Ok(output.to_string_lossy().into_owned());
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
