use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::db::AppState;

const MAX_LOG_BYTES: u64 = 1024 * 1024;

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn rotate_if_needed(path: &Path) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if metadata.len() < MAX_LOG_BYTES {
        return;
    }

    let backup = path.with_extension("log.1");
    let _ = fs::remove_file(&backup);
    let _ = fs::rename(path, backup);
}

pub fn log(state: &AppState, level: &str, message: impl AsRef<str>) {
    let path = &state.log_path;
    let Some(parent) = path.parent() else {
        return;
    };

    if fs::create_dir_all(parent).is_err() {
        return;
    }

    rotate_if_needed(path);

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };

    let message = message.as_ref().replace(['\r', '\n'], " ");
    let _ = writeln!(file, "{} [{}] {}", timestamp(), level, message);
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    #[test]
    fn log_is_local_and_single_line() {
        let root = std::env::temp_dir().join(format!(
            "lume-log-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let state = AppState::new(root.join("library.db"), root.join("cache"));
        log(&state, "WARN", "one\ntwo\rthree");

        let content = fs::read_to_string(&state.log_path).unwrap();
        assert!(content.contains("[WARN] one two three"));
        assert_eq!(content.lines().count(), 1);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn log_path_is_derived_from_app_data_not_media_sources() {
        let data = PathBuf::from(r"C:\Users\Test\AppData\Local\Lume\library.db");
        let state = AppState::new(data.clone(), PathBuf::from(r"C:\Temp\cache"));
        assert_eq!(
            state.log_path,
            data.parent().unwrap().join("logs").join("lume.log")
        );
    }
}
