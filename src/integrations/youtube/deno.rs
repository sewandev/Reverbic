use std::io::{Cursor, Read};
use std::path::PathBuf;

use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

use super::YoutubeError;

const DENO_RELEASES_URL: &str = "https://api.github.com/repos/denoland/deno/releases/latest";

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
    use super::{asset_name, binary_name, managed_binary_path};

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
}
