use std::path::PathBuf;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use super::YoutubeError;

const QUICKJS_NG_RELEASES_URL: &str =
    "https://api.github.com/repos/quickjs-ng/quickjs/releases/latest";

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
        return Err(install_error("missing parent directory"));
    };

    tokio::fs::create_dir_all(parent)
        .await
        .map_err(install_error)?;

    let client = crate::http::http_client_timeout(120)
        .ok_or_else(|| install_error("could not build HTTP client"))?;

    let release: Release = client
        .get(QUICKJS_NG_RELEASES_URL)
        .send()
        .await
        .map_err(install_error)?
        .error_for_status()
        .map_err(install_error)?
        .json()
        .await
        .map_err(install_error)?;

    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == asset_name())
        .ok_or_else(|| install_error("no matching QuickJS-NG asset for this platform"))?;

    let expected_sha256 = asset
        .digest
        .strip_prefix("sha256:")
        .ok_or_else(|| install_error("release asset has no SHA256 digest"))?
        .to_lowercase();

    let response = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(install_error)?
        .error_for_status()
        .map_err(install_error)?;

    let tmp_path = path.with_extension("tmp");
    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(install_error)?;

    let mut hasher = Sha256::new();
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(install_error)?;
        hasher.update(&bytes);
        file.write_all(&bytes).await.map_err(install_error)?;
    }
    file.flush().await.map_err(install_error)?;
    drop(file);

    use std::fmt::Write;
    let actual_sha256 = hasher
        .finalize()
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        });
    if actual_sha256 != expected_sha256 {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(YoutubeError::Install(crate::i18n::t(
            "modal.youtube.install_hash_mismatch",
        )));
    }

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

    Ok(path)
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
        "qjs.exe"
    }

    #[cfg(not(target_os = "windows"))]
    {
        "qjs"
    }
}

fn asset_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "qjs-windows-x86_64.exe"
    }

    #[cfg(target_os = "macos")]
    {
        "qjs-darwin"
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "qjs-linux-aarch64"
    }

    #[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
    {
        "qjs-linux-x86_64"
    }
}

#[derive(Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    digest: String,
}

#[cfg(test)]
mod tests {
    use super::{asset_name, binary_name, managed_binary_path};

    #[test]
    fn managed_binary_path_uses_reverbic_bin_directory() {
        let path = managed_binary_path();
        let rendered = path.to_string_lossy();
        assert!(rendered.contains(".reverbic"));
        assert!(rendered.contains("bin"));
        assert_eq!(path.file_name().expect("file name"), binary_name());
    }

    #[test]
    fn asset_name_targets_a_quickjs_ng_release_binary() {
        assert!(asset_name().starts_with("qjs-"));
    }
}
