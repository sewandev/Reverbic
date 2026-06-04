use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use crate::config::reverbic_dir;

pub enum SaveResult {
    Saved,
    AlreadySaved,
}
pub fn save_track(title: &str, station_key: &str) -> SaveResult {
    let path = library_path(station_key);

    if let Ok(content) = std::fs::read_to_string(&path) {
        if content.lines().any(|line| line == title) {
            tracing::debug!("Track ya guardado, ignorando duplicado: {title}");
            return SaveResult::AlreadySaved;
        }
    }

    let Some(dir) = path.parent() else {
        return SaveResult::AlreadySaved;
    };
    if let Err(e) = std::fs::create_dir_all(dir) {
        tracing::error!("save_track: no se pudo crear directorio: {e}");
        return SaveResult::AlreadySaved;
    }

    let line = format!("{title}\n");
    match OpenOptions::new().append(true).create(true).open(&path) {
        Ok(mut f) => {
            if let Err(e) = f.write_all(line.as_bytes()) {
                tracing::error!("save_track: error escribiendo: {e}");
                SaveResult::AlreadySaved
            } else {
                tracing::info!("Track guardado en {:?}: {}", path, title);
                SaveResult::Saved
            }
        }
        Err(e) => {
            tracing::error!("save_track: error abriendo archivo: {e}");
            SaveResult::AlreadySaved
        }
    }
}
pub fn load_saved_tracks(station_key: &str) -> Vec<String> {
    match std::fs::read_to_string(library_path(station_key)) {
        Ok(content) => {
            let mut tracks: Vec<String> = content
                .lines()
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect();
            tracks.reverse();
            tracks
        }
        Err(_) => Vec::new(),
    }
}

fn library_path(station_key: &str) -> PathBuf {
    reverbic_dir()
        .join("library")
        .join(format!("{station_key}.txt"))
}
