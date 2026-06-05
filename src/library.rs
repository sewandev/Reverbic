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
        .join(format!("{}.txt", library_filename_stem(station_key)))
}

fn library_filename_stem(station_key: &str) -> String {
    let mut stem = String::new();
    for byte in station_key.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' => stem.push(byte as char),
            _ => stem.push_str(&format!("%{byte:02X}")),
        }
    }

    if stem.is_empty() {
        stem.push('_');
    }

    if is_windows_reserved_filename_stem(&stem) {
        stem.insert(0, '_');
    }

    stem
}

fn is_windows_reserved_filename_stem(stem: &str) -> bool {
    let lower = stem.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "con"
            | "prn"
            | "aux"
            | "nul"
            | "com1"
            | "com2"
            | "com3"
            | "com4"
            | "com5"
            | "com6"
            | "com7"
            | "com8"
            | "com9"
            | "lpt1"
            | "lpt2"
            | "lpt3"
            | "lpt4"
            | "lpt5"
            | "lpt6"
            | "lpt7"
            | "lpt8"
            | "lpt9"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_filename_stem_keeps_existing_safe_keys() {
        assert_eq!(
            library_filename_stem("tomorrowland_owr"),
            "tomorrowland_owr"
        );
        assert_eq!(
            library_filename_stem("ondemand_123-abc"),
            "ondemand_123-abc"
        );
    }

    #[test]
    fn library_filename_stem_escapes_path_and_filesystem_separators() {
        assert_eq!(
            library_filename_stem("../foo\\bar:baz"),
            "%2E%2E%2Ffoo%5Cbar%3Abaz"
        );
    }

    #[test]
    fn library_filename_stem_escapes_percent_and_unicode_bytes() {
        assert_eq!(library_filename_stem("rock%ñ"), "rock%25%C3%B1");
    }

    #[test]
    fn library_filename_stem_handles_empty_and_windows_reserved_names() {
        assert_eq!(library_filename_stem(""), "_");
        assert_eq!(library_filename_stem("CON"), "_CON");
        assert_eq!(library_filename_stem("lpt1"), "_lpt1");
    }
}
