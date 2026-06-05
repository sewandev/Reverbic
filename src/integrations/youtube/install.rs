use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

use super::YoutubeError;

#[cfg(target_os = "windows")]
const YT_DLP_WINDOWS_URL: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
#[cfg(not(target_os = "windows"))]
const YT_DLP_UNIX_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";

pub fn managed_binary_path() -> PathBuf {
    crate::config::reverbic_dir()
        .join("bin")
        .join(binary_name())
}

pub fn is_installed() -> bool {
    managed_binary_path().exists()
}

pub async fn ensure_installed() -> Result<PathBuf, YoutubeError> {
    let path = managed_binary_path();
    if path.exists() {
        return Ok(path);
    }

    let Some(parent) = path.parent() else {
        return Err(YoutubeError::Install(crate::i18n::t(
            "modal.youtube.install_failed",
        )));
    };

    tokio::fs::create_dir_all(parent).await.map_err(|e| {
        YoutubeError::Install(format!(
            "{}: {e}",
            crate::i18n::t("modal.youtube.install_failed")
        ))
    })?;

    let client = crate::http::http_client_timeout(120)
        .ok_or_else(|| YoutubeError::Install(crate::i18n::t("modal.youtube.install_failed")))?;
    let response = client.get(download_url()).send().await.map_err(|e| {
        YoutubeError::Install(format!(
            "{}: {e}",
            crate::i18n::t("modal.youtube.install_failed")
        ))
    })?;

    if !response.status().is_success() {
        return Err(YoutubeError::Install(format!(
            "{}: HTTP {}",
            crate::i18n::t("modal.youtube.install_failed"),
            response.status()
        )));
    }

    let tmp_path = path.with_extension("tmp");
    let mut file = tokio::fs::File::create(&tmp_path).await.map_err(|e| {
        YoutubeError::Install(format!(
            "{}: {e}",
            crate::i18n::t("modal.youtube.install_failed")
        ))
    })?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| {
            YoutubeError::Install(format!(
                "{}: {e}",
                crate::i18n::t("modal.youtube.install_failed")
            ))
        })?;
        file.write_all(&bytes).await.map_err(|e| {
            YoutubeError::Install(format!(
                "{}: {e}",
                crate::i18n::t("modal.youtube.install_failed")
            ))
        })?;
    }
    file.flush().await.map_err(|e| {
        YoutubeError::Install(format!(
            "{}: {e}",
            crate::i18n::t("modal.youtube.install_failed")
        ))
    })?;
    drop(file);

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = tokio::fs::metadata(&tmp_path).await.map_err(|e| {
            YoutubeError::Install(format!(
                "{}: {e}",
                crate::i18n::t("modal.youtube.install_failed")
            ))
        })?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&tmp_path, perms)
            .await
            .map_err(|e| {
                YoutubeError::Install(format!(
                    "{}: {e}",
                    crate::i18n::t("modal.youtube.install_failed")
                ))
            })?;
    }

    tokio::fs::rename(&tmp_path, &path).await.map_err(|e| {
        YoutubeError::Install(format!(
            "{}: {e}",
            crate::i18n::t("modal.youtube.install_failed")
        ))
    })?;

    Ok(path)
}

fn binary_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "yt-dlp.exe"
    }

    #[cfg(not(target_os = "windows"))]
    {
        "yt-dlp"
    }
}

fn download_url() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        YT_DLP_WINDOWS_URL
    }

    #[cfg(not(target_os = "windows"))]
    {
        YT_DLP_UNIX_URL
    }
}

#[cfg(test)]
mod tests {
    use super::{download_url, managed_binary_path};

    #[test]
    fn managed_binary_path_uses_reverbic_bin_directory() {
        let path = managed_binary_path();
        let rendered = path.to_string_lossy();
        assert!(rendered.contains(".reverbic"));
        assert!(rendered.contains("bin"));
        assert!(path.file_name().is_some());
    }

    #[test]
    fn download_url_points_to_latest_release_asset() {
        let url = download_url();
        assert!(url.contains("yt-dlp/releases/latest/download/"));
    }
}
