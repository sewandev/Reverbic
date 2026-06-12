use std::path::{Path, PathBuf};

use super::YoutubeError;

const MAX_COOKIES_FILE_SIZE: u64 = 1024 * 1024;

pub fn validate_cookies_path(path: &Path) -> Result<PathBuf, YoutubeError> {
    let canonical = dunce::canonicalize(path)
        .map_err(|_| YoutubeError::Cookies(crate::i18n::t("modal.youtube.cookies_invalid_path")))?;

    let metadata = std::fs::metadata(&canonical)
        .map_err(|_| YoutubeError::Cookies(crate::i18n::t("modal.youtube.cookies_invalid_path")))?;

    if !metadata.is_file() {
        return Err(YoutubeError::Cookies(crate::i18n::t(
            "modal.youtube.cookies_not_a_file",
        )));
    }

    if metadata.len() > MAX_COOKIES_FILE_SIZE {
        return Err(YoutubeError::Cookies(crate::i18n::t(
            "modal.youtube.cookies_too_large",
        )));
    }

    warn_if_permissive(&canonical, &metadata);

    Ok(canonical)
}

pub fn configured_cookies_path(cookies_path: Option<&Path>) -> Option<PathBuf> {
    let path = cookies_path?;
    match validate_cookies_path(path) {
        Ok(valid) => Some(valid),
        Err(err) => {
            tracing::warn!("ignoring configured YouTube cookies file: {err}");
            None
        }
    }
}

#[cfg(unix)]
fn warn_if_permissive(path: &Path, metadata: &std::fs::Metadata) {
    use std::os::unix::fs::PermissionsExt;

    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        tracing::warn!(
            "cookies file at {} is readable by other users (mode {mode:o}); run chmod 600 on it",
            path.display(),
        );
    }
}

#[cfg(not(unix))]
fn warn_if_permissive(_path: &Path, _metadata: &std::fs::Metadata) {}

#[cfg(test)]
mod tests {
    use super::validate_cookies_path;
    use std::io::Write;

    #[test]
    fn rejects_missing_path() {
        let path = std::env::temp_dir().join("reverbic_cookies_does_not_exist.txt");
        let err = validate_cookies_path(&path).expect_err("missing file should be rejected");
        assert!(matches!(err, super::YoutubeError::Cookies(_)));
    }

    #[test]
    fn rejects_directory() {
        let dir = std::env::temp_dir();
        let err = validate_cookies_path(&dir).expect_err("directory should be rejected");
        assert!(matches!(err, super::YoutubeError::Cookies(_)));
    }

    #[test]
    fn accepts_small_regular_file() {
        let path = std::env::temp_dir().join("reverbic_cookies_valid_test.txt");
        let mut file = std::fs::File::create(&path).expect("temp file should be created");
        file.write_all(b"# Netscape HTTP Cookie File\n")
            .expect("temp file should be writable");
        drop(file);

        let result = validate_cookies_path(&path);
        let _ = std::fs::remove_file(&path);

        result.expect("small regular file should be accepted");
    }

    #[test]
    fn rejects_oversized_file() {
        let path = std::env::temp_dir().join("reverbic_cookies_oversized_test.txt");
        let file = std::fs::File::create(&path).expect("temp file should be created");
        file.set_len(super::MAX_COOKIES_FILE_SIZE + 1)
            .expect("temp file should be resizable");
        drop(file);

        let result = validate_cookies_path(&path);
        let _ = std::fs::remove_file(&path);

        let err = result.expect_err("oversized file should be rejected");
        assert!(matches!(err, super::YoutubeError::Cookies(_)));
    }
}
