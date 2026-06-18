use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

use super::YoutubeError;

const DENO_RELEASES_URL: &str = "https://api.github.com/repos/denoland/deno/releases/latest";

const MIN_DENO_VERSION: (u32, u32, u32) = (2, 3, 0);

pub fn managed_binary_path() -> PathBuf {
    crate::paths::bin_dir().join(binary_name())
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
    if !path.exists() {
        return;
    }

    match installed_version(&path).await {
        Some(version) if version >= MIN_DENO_VERSION => {
            tracing::info!(
                ?version,
                "deno meets the minimum version required by yt-dlp"
            );
        }
        Some(version) => {
            tracing::warn!(
                ?version,
                minimum = ?MIN_DENO_VERSION,
                "deno is older than the minimum required by yt-dlp, reinstalling"
            );
            reinstall(&path).await;
        }
        None => {
            tracing::warn!("deno version could not be determined, reinstalling to be safe");
            reinstall(&path).await;
        }
    }
}

pub async fn ensure_runtime_ready() {
    let path = managed_binary_path();
    if !path.exists() {
        tracing::warn!("deno runtime is missing, reinstalling before the next YouTube attempt");
        reinstall(&path).await;
        return;
    }
    update_if_outdated().await;
}

async fn reinstall(path: &Path) {
    match install_latest(path).await {
        Ok(version) => tracing::info!(%version, "deno updated successfully"),
        Err(e) => tracing::warn!("deno update failed, keeping current binary: {e}"),
    }
}

async fn installed_version(binary: &Path) -> Option<(u32, u32, u32)> {
    let mut command = tokio::process::Command::new(binary);
    command.arg("--version");
    let output = super::run_ytdlp_output(command, super::YTDLP_LOCAL_TIMEOUT, "deno_version")
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_version(&String::from_utf8_lossy(&output.stdout))
}

fn parse_version(output: &str) -> Option<(u32, u32, u32)> {
    let token = output.lines().next()?.split_whitespace().nth(1)?;
    let core = token.split(['+', '-']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

async fn install_latest(path: &Path) -> Result<String, YoutubeError> {
    let Some(parent) = path.parent() else {
        return Err(install_error("missing parent directory"));
    };

    tokio::fs::create_dir_all(parent)
        .await
        .map_err(install_error)?;

    let client = crate::http::http_client_timeout(120)
        .ok_or_else(|| install_error("could not build HTTP client"))?;

    let release = super::fetch_latest_release(DENO_RELEASES_URL)
        .await
        .map_err(install_error)?;

    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == asset_name())
        .ok_or_else(|| install_error("no matching Deno asset for this platform"))?;

    let expected_sha256 = asset
        .digest
        .strip_prefix("sha256:")
        .ok_or_else(|| install_error("release asset has no SHA256 digest"))?
        .to_lowercase();

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

    let binary = {
        let mut archive = ZipArchive::new(Cursor::new(bytes.as_ref())).map_err(install_error)?;
        let mut entry = archive.by_name(binary_name()).map_err(install_error)?;
        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf).map_err(install_error)?;
        buf
    };

    let tmp_path = path.with_extension("tmp");
    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(install_error)?;
    file.write_all(&binary).await.map_err(install_error)?;
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

    tokio::fs::rename(&tmp_path, path)
        .await
        .map_err(install_error)?;

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
        "deno.exe"
    }

    #[cfg(not(target_os = "windows"))]
    {
        "deno"
    }
}

fn asset_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "deno-x86_64-pc-windows-msvc.zip"
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "deno-aarch64-apple-darwin.zip"
    }

    #[cfg(all(target_os = "macos", not(target_arch = "aarch64")))]
    {
        "deno-x86_64-apple-darwin.zip"
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "deno-aarch64-unknown-linux-gnu.zip"
    }

    #[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
    {
        "deno-x86_64-unknown-linux-gnu.zip"
    }
}

#[cfg(test)]
mod tests {
    use super::{asset_name, binary_name, managed_binary_path, parse_version, MIN_DENO_VERSION};

    #[test]
    fn managed_binary_path_uses_bin_directory() {
        let path = managed_binary_path();
        let rendered = path.to_string_lossy();
        assert!(rendered.contains("bin"));
        assert!(path.starts_with(crate::paths::bin_dir()));
        assert_eq!(path.file_name().expect("file name"), binary_name());
    }

    #[test]
    fn asset_name_targets_a_deno_release_zip() {
        assert!(asset_name().starts_with("deno-"));
        assert!(asset_name().ends_with(".zip"));
    }

    #[test]
    fn parse_version_reads_first_line() {
        assert_eq!(
            parse_version("deno 2.8.2 (stable, release, x86_64-pc-windows-msvc)\nv8 14.2.0"),
            Some((2, 8, 2))
        );
    }

    #[test]
    fn parse_version_strips_prerelease_and_build() {
        assert_eq!(parse_version("deno 2.3.0+abcdef"), Some((2, 3, 0)));
        assert_eq!(parse_version("deno 2.4.1-rc.1"), Some((2, 4, 1)));
    }

    #[test]
    fn parse_version_rejects_garbage() {
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("not a version"), None);
    }

    #[test]
    fn outdated_version_is_below_minimum() {
        assert!((2, 2, 9) < MIN_DENO_VERSION);
        assert!((2, 3, 0) >= MIN_DENO_VERSION);
        assert!((2, 8, 2) >= MIN_DENO_VERSION);
    }
}
