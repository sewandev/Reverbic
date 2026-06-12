use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;

use super::YoutubeError;

const YT_DLP_RELEASES_URL: &str = "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest";
const UPDATE_CHECK_INTERVAL_SECS: u64 = 24 * 3600;

pub fn managed_binary_path() -> PathBuf {
    crate::config::reverbic_dir()
        .join("bin")
        .join(binary_name())
}

fn version_file_path() -> PathBuf {
    crate::config::reverbic_dir()
        .join("bin")
        .join("yt-dlp.version")
}

fn last_check_file_path() -> PathBuf {
    crate::config::reverbic_dir()
        .join("bin")
        .join("yt-dlp.last-check")
}

pub fn is_installed() -> bool {
    managed_binary_path().exists()
}

pub async fn ensure_installed() -> Result<PathBuf, YoutubeError> {
    let path = managed_binary_path();
    if path.exists() {
        return Ok(path);
    }
    install_latest(&path).await?;
    Ok(path)
}

pub async fn update_if_outdated() {
    let path = managed_binary_path();
    if !path.exists() || !should_check_now() {
        return;
    }
    record_check_time();

    let Some(installed) = installed_version(&path).await else {
        tracing::warn!("yt-dlp update: could not determine installed version, skipping check");
        return;
    };

    let release = match super::fetch_latest_release(YT_DLP_RELEASES_URL).await {
        Ok(release) => release,
        Err(e) => {
            tracing::warn!("yt-dlp update: could not query latest release: {e}");
            return;
        }
    };

    if release.tag_name == installed {
        tracing::info!(version = %installed, "yt-dlp is up to date");
        return;
    }

    tracing::info!(
        installed = %installed,
        latest = %release.tag_name,
        "yt-dlp update available, downloading"
    );
    match install_latest(&path).await {
        Ok(version) => tracing::info!(%version, "yt-dlp updated successfully"),
        Err(e) => tracing::warn!("yt-dlp update failed, keeping current binary: {e}"),
    }
}

async fn installed_version(binary: &Path) -> Option<String> {
    if let Ok(version) = tokio::fs::read_to_string(version_file_path()).await {
        let version = version.trim().to_string();
        if !version.is_empty() {
            return Some(version);
        }
    }

    let output = tokio::process::Command::new(binary)
        .arg("--version")
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return None;
    }
    let _ = tokio::fs::write(version_file_path(), &version).await;
    Some(version)
}

fn should_check_now() -> bool {
    let now = unix_now_secs();
    match std::fs::read_to_string(last_check_file_path()) {
        Ok(stamp) => match stamp.trim().parse::<u64>() {
            Ok(last) => now.saturating_sub(last) >= UPDATE_CHECK_INTERVAL_SECS,
            Err(_) => true,
        },
        Err(_) => true,
    }
}

fn record_check_time() {
    if let Err(e) = std::fs::write(last_check_file_path(), unix_now_secs().to_string()) {
        tracing::debug!("yt-dlp update: could not record check time: {e}");
    }
}

fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

async fn install_latest(path: &Path) -> Result<String, YoutubeError> {
    let Some(parent) = path.parent() else {
        return Err(install_error("missing parent directory"));
    };
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(install_error)?;

    let release = super::fetch_latest_release(YT_DLP_RELEASES_URL)
        .await
        .map_err(install_error)?;
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == binary_name())
        .ok_or_else(|| install_error("no matching yt-dlp asset for this platform"))?;

    let expected_sha256 = asset
        .digest
        .strip_prefix("sha256:")
        .ok_or_else(|| install_error("release asset has no SHA256 digest"))?
        .to_lowercase();

    let client = crate::http::http_client_timeout(120)
        .ok_or_else(|| install_error("could not build HTTP client"))?;
    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(install_error)?
        .error_for_status()
        .map_err(install_error)?
        .bytes()
        .await
        .map_err(install_error)?;

    if super::sha256_hex(bytes.as_ref()) != expected_sha256 {
        return Err(YoutubeError::Install(crate::i18n::t(
            "modal.youtube.install_hash_mismatch",
        )));
    }

    let tmp_path = path.with_extension("tmp");
    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(install_error)?;
    file.write_all(&bytes).await.map_err(install_error)?;
    file.flush().await.map_err(install_error)?;
    drop(file);

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = tokio::fs::metadata(&tmp_path)
            .await
            .map_err(install_error)?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&tmp_path, perms)
            .await
            .map_err(install_error)?;
    }

    tokio::fs::rename(&tmp_path, &path)
        .await
        .map_err(install_error)?;

    if let Err(e) = tokio::fs::write(version_file_path(), &release.tag_name).await {
        tracing::debug!("yt-dlp install: could not record version: {e}");
    }
    tracing::info!(version = %release.tag_name, "yt-dlp installed");

    Ok(release.tag_name)
}

fn install_error(e: impl std::fmt::Display) -> YoutubeError {
    YoutubeError::Install(format!(
        "{}: {e}",
        crate::i18n::t("modal.youtube.install_failed")
    ))
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

#[cfg(test)]
mod tests {
    use super::{managed_binary_path, YT_DLP_RELEASES_URL};

    #[test]
    fn managed_binary_path_uses_reverbic_bin_directory() {
        let path = managed_binary_path();
        let rendered = path.to_string_lossy();
        assert!(rendered.contains(".reverbic"));
        assert!(rendered.contains("bin"));
        assert!(path.file_name().is_some());
    }

    #[test]
    fn releases_url_points_to_official_repo_api() {
        assert!(YT_DLP_RELEASES_URL.contains("api.github.com/repos/yt-dlp/yt-dlp"));
    }
}
