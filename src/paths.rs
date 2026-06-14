use directories::ProjectDirs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

fn project_dirs() -> Option<&'static ProjectDirs> {
    static DIRS: OnceLock<Option<ProjectDirs>> = OnceLock::new();
    DIRS.get_or_init(|| ProjectDirs::from("", "", "Reverbic"))
        .as_ref()
}

fn home_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
}

pub(crate) fn config_dir() -> PathBuf {
    project_dirs()
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| legacy_root().join("config"))
}

pub(crate) fn data_dir() -> PathBuf {
    project_dirs()
        .map(|d| d.data_local_dir().to_path_buf())
        .unwrap_or_else(|| legacy_root().join("data"))
}

pub(crate) fn cache_dir() -> PathBuf {
    project_dirs()
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(|| legacy_root().join("cache"))
}

pub(crate) fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

pub(crate) fn favorites_file() -> PathBuf {
    data_dir().join("favorites.json")
}

pub(crate) fn playlists_file() -> PathBuf {
    data_dir().join("playlists.json")
}

pub(crate) fn youtube_bookmarks_file() -> PathBuf {
    data_dir().join("youtube_bookmarks.json")
}

pub(crate) fn games_file() -> PathBuf {
    data_dir().join("games.json")
}

pub(crate) fn library_dir() -> PathBuf {
    data_dir().join("library")
}

pub(crate) fn bin_dir() -> PathBuf {
    data_dir().join("bin")
}

pub(crate) fn logs_dir() -> PathBuf {
    cache_dir().join("logs")
}

pub(crate) fn youtube_media_cache_dir() -> PathBuf {
    cache_dir().join("youtube")
}

pub(crate) fn youtube_url_cache_file() -> PathBuf {
    cache_dir().join("youtube_url_cache.json")
}

pub(crate) fn librespot_cache_dir() -> PathBuf {
    cache_dir().join("librespot")
}

pub(crate) fn deezer_not_found_log() -> PathBuf {
    cache_dir().join("deezer_not_found.log")
}

/// Legacy single-folder layout used before the XDG migration (`~/.reverbic`).
fn legacy_root() -> PathBuf {
    home_dir().join(".reverbic")
}

/// On Windows the Spotify cache historically lived under `%APPDATA%\.reverbic`,
/// which differs from the `%USERPROFILE%\.reverbic` root used by everything else.
#[cfg(target_os = "windows")]
fn legacy_appdata_root() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(|p| PathBuf::from(p).join(".reverbic"))
}

/// Moves the pre-XDG `~/.reverbic` layout into the per-category directories.
///
/// Idempotent: each entry is only moved when the source exists and the
/// destination is still absent, so re-running on an already-migrated install is
/// a no-op. Failures are logged and never abort startup.
pub(crate) fn migrate_legacy() {
    let legacy = legacy_root();

    let mappings: [(PathBuf, PathBuf); 11] = [
        (legacy.join("config.json"), config_file()),
        (legacy.join("favorites.json"), favorites_file()),
        (legacy.join("playlists.json"), playlists_file()),
        (
            legacy.join("youtube_bookmarks.json"),
            youtube_bookmarks_file(),
        ),
        (legacy.join("games.json"), games_file()),
        (legacy.join("library"), library_dir()),
        (legacy.join("bin"), bin_dir()),
        (
            legacy.join("cache").join("youtube"),
            youtube_media_cache_dir(),
        ),
        (
            legacy.join("youtube_url_cache.json"),
            youtube_url_cache_file(),
        ),
        (legacy.join("logs"), logs_dir()),
        (legacy.join("deezer_not_found.log"), deezer_not_found_log()),
    ];

    for (src, dst) in &mappings {
        migrate_entry(src, dst);
    }

    migrate_entry(&legacy.join("librespot"), &librespot_cache_dir());

    #[cfg(target_os = "windows")]
    if let Some(appdata_legacy) = legacy_appdata_root() {
        migrate_entry(&appdata_legacy.join("librespot"), &librespot_cache_dir());
        cleanup_if_empty(&appdata_legacy);
    }

    cleanup_if_empty(&legacy.join("cache"));
    cleanup_if_empty(&legacy);
}

fn migrate_entry(src: &Path, dst: &Path) {
    if !src.exists() || dst.exists() {
        return;
    }

    if let Some(parent) = dst.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "Failed to create {} during migration: {e}",
                parent.display()
            );
            return;
        }
    }

    match std::fs::rename(src, dst) {
        Ok(()) => tracing::info!("Migrated {} to {}", src.display(), dst.display()),
        Err(_) => match move_across_devices(src, dst) {
            Ok(()) => tracing::info!("Migrated {} to {}", src.display(), dst.display()),
            Err(e) => tracing::warn!("Failed to migrate {}: {e}", src.display()),
        },
    }
}

/// `std::fs::rename` fails across filesystem boundaries (a real possibility when
/// moving from `%USERPROFILE%` to `%LOCALAPPDATA%` on a different drive), so fall
/// back to a recursive copy followed by removal of the source.
fn move_across_devices(src: &Path, dst: &Path) -> std::io::Result<()> {
    copy_recursive(src, dst)?;
    if src.is_dir() {
        std::fs::remove_dir_all(src)
    } else {
        std::fs::remove_file(src)
    }
}

fn copy_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            copy_recursive(&entry.path(), &dst.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        std::fs::copy(src, dst).map(|_| ())
    }
}

fn cleanup_if_empty(dir: &Path) {
    if dir.is_dir()
        && std::fs::read_dir(dir)
            .map(|mut it| it.next().is_none())
            .unwrap_or(false)
    {
        let _ = std::fs::remove_dir(dir);
    }
}
