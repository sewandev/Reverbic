use std::path::{Path, PathBuf};

const REPO: &str = "sewandev/Reverbic";
const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug)]
pub struct UpdateAsset {
    pub version: String,
    pub name: String,
    pub download_url: String,
    pub size: u64,
    pub digest: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, serde::Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}

impl GitHubAsset {
    fn into_update_asset(self, version: &str) -> UpdateAsset {
        UpdateAsset {
            version: version.to_owned(),
            name: self.name,
            download_url: self.browser_download_url,
            size: self.size,
            digest: self.digest,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpdateOs {
    Windows,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpdateArch {
    X86_64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UpdateTarget {
    os: UpdateOs,
    arch: UpdateArch,
}

#[derive(Debug, PartialEq, Eq)]
enum AssetSelectionError {
    UnsupportedPlatform,
    NoCompatibleAsset,
}

impl UpdateTarget {
    fn current() -> Result<Self, AssetSelectionError> {
        Self::from_parts(std::env::consts::OS, std::env::consts::ARCH)
    }

    fn from_parts(os: &str, arch: &str) -> Result<Self, AssetSelectionError> {
        let os = match os {
            "windows" => UpdateOs::Windows,
            _ => return Err(AssetSelectionError::UnsupportedPlatform),
        };

        let arch = match arch {
            "x86_64" => UpdateArch::X86_64,
            _ => return Err(AssetSelectionError::NoCompatibleAsset),
        };

        match (os, arch) {
            (UpdateOs::Windows, UpdateArch::X86_64) => Ok(Self { os, arch }),
        }
    }

    fn asset_name(self, version: &str) -> String {
        match (self.os, self.arch) {
            (UpdateOs::Windows, UpdateArch::X86_64) => {
                format!("reverbic-v{version}-x86_64-windows.exe")
            }
        }
    }
}

pub async fn fetch_latest_update() -> Option<UpdateAsset> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let client = reqwest::Client::builder()
        .user_agent(concat!("reverbic/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .ok()?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let release: GitHubRelease = resp.json().await.ok()?;
    let version = release.tag_name.trim_start_matches('v');
    if is_newer(version, CURRENT) {
        let target = match UpdateTarget::current() {
            Ok(target) => target,
            Err(err) => {
                tracing::debug!(?err, "No compatible updater target");
                return None;
            }
        };
        match select_compatible_asset(release.assets, version, target) {
            Ok(asset) => Some(asset),
            Err(err) => {
                tracing::debug!(?err, "No compatible update asset");
                None
            }
        }
    } else {
        None
    }
}

fn select_compatible_asset(
    assets: Vec<GitHubAsset>,
    version: &str,
    target: UpdateTarget,
) -> Result<UpdateAsset, AssetSelectionError> {
    let name = target.asset_name(version);
    assets
        .into_iter()
        .find(|asset| asset.name == name)
        .map(|asset| asset.into_update_asset(version))
        .ok_or(AssetSelectionError::NoCompatibleAsset)
}

pub async fn download_update(asset: &UpdateAsset) -> Option<PathBuf> {
    let path = update_download_path(asset);
    let part_path = unique_part_path(&path);
    tracing::debug!(
        asset = %asset.name,
        size = asset.size,
        digest = asset.digest.as_deref().unwrap_or(""),
        "Downloading update asset"
    );

    let client = reqwest::Client::builder()
        .user_agent(concat!("reverbic/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .ok()?;
    let resp = client.get(&asset.download_url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }

    if !stream_to_file(resp, &part_path).await {
        let _ = tokio::fs::remove_file(&part_path).await;
        return None;
    }

    if let Err(err) = validate_update_payload(&part_path, asset) {
        tracing::warn!(?err, asset = %asset.name, "Rejected update payload");
        let _ = tokio::fs::remove_file(&part_path).await;
        return None;
    }

    let _ = tokio::fs::remove_file(&path).await;
    if tokio::fs::rename(&part_path, &path).await.is_err() {
        let _ = tokio::fs::remove_file(&part_path).await;
        return None;
    }

    Some(path)
}

fn update_download_path(asset: &UpdateAsset) -> PathBuf {
    std::env::temp_dir().join(format!("reverbic-update-{}", asset.name))
}

fn unique_part_path(path: &Path) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static PART_COUNTER: AtomicU64 = AtomicU64::new(0);

    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "reverbic-update".into());
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let counter = PART_COUNTER.fetch_add(1, Ordering::Relaxed);

    path.with_file_name(format!(
        "{file_name}.{}.{}.{}.part",
        std::process::id(),
        timestamp,
        counter
    ))
}

#[derive(Debug, PartialEq, Eq)]
enum PayloadValidationError {
    MissingDigest,
    UnsupportedDigest,
    SizeMismatch,
    HashMismatch,
    ReadFailed,
}

fn validate_update_payload(path: &Path, asset: &UpdateAsset) -> Result<(), PayloadValidationError> {
    let metadata = std::fs::metadata(path).map_err(|_| PayloadValidationError::ReadFailed)?;
    if metadata.len() != asset.size {
        return Err(PayloadValidationError::SizeMismatch);
    }

    let expected = expected_sha256(asset.digest.as_deref())?;
    let actual = sha256_file(path)?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(PayloadValidationError::HashMismatch)
    }
}

fn expected_sha256(digest: Option<&str>) -> Result<&str, PayloadValidationError> {
    let digest = digest.ok_or(PayloadValidationError::MissingDigest)?;
    let hash = digest
        .strip_prefix("sha256:")
        .ok_or(PayloadValidationError::UnsupportedDigest)?;

    if hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(hash)
    } else {
        Err(PayloadValidationError::UnsupportedDigest)
    }
}

fn sha256_file(path: &Path) -> Result<String, PayloadValidationError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|_| PayloadValidationError::ReadFailed)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 16 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| PayloadValidationError::ReadFailed)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn apply_update(new_exe: &Path) {
    #[cfg(target_os = "windows")]
    {
        if let Err(err) = spawn_windows_update_helper(new_exe) {
            tracing::error!("Failed to prepare update application: {err}");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        apply_update_in_place(new_exe);
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_update_in_place(new_exe: &Path) {
    let Ok(current) = std::env::current_exe() else {
        return;
    };
    let Some(file_name) = current.file_name() else {
        return;
    };
    let old_name = format!("{}.old", file_name.to_string_lossy());
    let old = current.with_file_name(old_name);
    if std::fs::rename(&current, &old).is_ok() && std::fs::copy(new_exe, &current).is_err() {
        let _ = std::fs::rename(&old, &current);
    }
}

#[cfg(target_os = "windows")]
fn spawn_windows_update_helper(new_exe: &Path) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let current = std::env::current_exe()?;
    let script_path = helper_script_path();
    let script = build_update_script(
        new_exe,
        &current,
        &current.with_file_name(format!(
            "{}.old",
            current
                .file_name()
                .map(|name| name.to_string_lossy())
                .unwrap_or_default()
        )),
    );
    std::fs::write(&script_path, script)?;

    std::process::Command::new("cmd")
        .args(["/C", script_path.to_string_lossy().as_ref()])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn helper_script_path() -> PathBuf {
    std::env::temp_dir().join(format!("reverbic-update-{}.cmd", std::process::id()))
}

#[cfg(target_os = "windows")]
fn build_update_script(new_exe: &Path, current_exe: &Path, backup_exe: &Path) -> String {
    let src = escape_batch_path(new_exe);
    let dst = escape_batch_path(current_exe);
    let old = escape_batch_path(backup_exe);

    format!(
        "@echo off\r\n\
setlocal\r\n\
set \"SRC={src}\"\r\n\
set \"DST={dst}\"\r\n\
set \"OLD={old}\"\r\n\
for /L %%I in (1,1,30) do (\r\n\
  move /Y \"%DST%\" \"%OLD%\" >nul 2>nul && goto copy_new\r\n\
  timeout /t 1 /nobreak >nul\r\n\
)\r\n\
goto end\r\n\
:copy_new\r\n\
copy /Y \"%SRC%\" \"%DST%\" >nul 2>nul || (\r\n\
  move /Y \"%OLD%\" \"%DST%\" >nul 2>nul\r\n\
  goto end\r\n\
)\r\n\
del /F /Q \"%OLD%\" >nul 2>nul\r\n\
del /F /Q \"%SRC%\" >nul 2>nul\r\n\
:end\r\n\
del /F /Q \"%~f0\" >nul 2>nul\r\n"
    )
}

#[cfg(target_os = "windows")]
fn escape_batch_path(path: &Path) -> String {
    path.to_string_lossy().replace('%', "%%")
}

pub fn cleanup_stale() {
    let Ok(current) = std::env::current_exe() else {
        return;
    };
    let Some(parent) = current.parent() else {
        return;
    };
    let Some(file_name) = current.file_name() else {
        return;
    };
    let old = parent.join(format!("{}.old", file_name.to_string_lossy()));
    if old.exists() {
        let _ = std::fs::remove_file(old);
    }
}

async fn stream_to_file(resp: reqwest::Response, path: &Path) -> bool {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    let Ok(mut file) = tokio::fs::File::create(path).await else {
        return false;
    };
    let mut stream = resp.bytes_stream();
    loop {
        match stream.next().await {
            Some(Ok(chunk)) => {
                if file.write_all(&chunk).await.is_err() {
                    return false;
                }
            }
            Some(Err(_)) => return false,
            None => break,
        }
    }
    file.flush().await.is_ok()
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_semver(latest), parse_semver(current)) {
        (Some(l), Some(c)) => l > c,
        _ => latest != current,
    }
}

fn parse_semver(v: &str) -> Option<(u32, u32, u32)> {
    let mut it = v.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next()?.parse().ok()?;
    let patch = it.next()?.parse().ok()?;
    Some((major, minor, patch))
}

#[cfg(test)]
mod payload_validation_tests {
    use super::{
        download_update, update_download_path, validate_update_payload, PayloadValidationError,
        UpdateAsset,
    };
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    fn next_id() -> u64 {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    fn test_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "reverbic-update-test-{}-{}-{label}",
            std::process::id(),
            next_id()
        ))
    }

    fn asset_with(size: u64, digest: Option<String>) -> UpdateAsset {
        UpdateAsset {
            version: "2.0.0".to_owned(),
            name: format!("reverbic-v2.0.0-test-{}.exe", next_id()),
            download_url: "http://127.0.0.1:1/reverbic.exe".to_owned(),
            size,
            digest,
        }
    }

    #[test]
    fn validates_payload_size_and_sha256() {
        let path = test_path("valid");
        std::fs::write(&path, b"hello").expect("test payload should be written");
        let asset = asset_with(5, Some(format!("sha256:{HELLO_SHA256}")));

        let result = validate_update_payload(&path, &asset);
        let _ = std::fs::remove_file(path);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn rejects_payload_with_hash_mismatch() {
        let path = test_path("hash-mismatch");
        std::fs::write(&path, b"hello").expect("test payload should be written");
        let asset = asset_with(
            5,
            Some(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_owned(),
            ),
        );

        let result = validate_update_payload(&path, &asset);
        let _ = std::fs::remove_file(path);

        assert_eq!(result, Err(PayloadValidationError::HashMismatch));
    }

    #[test]
    fn rejects_payload_with_size_mismatch() {
        let path = test_path("size-mismatch");
        std::fs::write(&path, b"hello").expect("test payload should be written");
        let asset = asset_with(6, Some(format!("sha256:{HELLO_SHA256}")));

        let result = validate_update_payload(&path, &asset);
        let _ = std::fs::remove_file(path);

        assert_eq!(result, Err(PayloadValidationError::SizeMismatch));
    }

    #[tokio::test]
    async fn download_update_does_not_reuse_existing_temp_payload() {
        let asset = asset_with(
            1_000_001,
            Some(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_owned(),
            ),
        );
        let path = update_download_path(&asset);
        let file = std::fs::File::create(&path).expect("stale test payload should be created");
        file.set_len(1_000_001)
            .expect("stale test payload size should be set");

        let result = download_update(&asset).await;
        let _ = std::fs::remove_file(path);

        assert!(result.is_none());
    }
}

#[cfg(test)]
mod download_path_tests {
    use super::{unique_part_path, update_download_path, UpdateAsset};

    fn update_asset() -> UpdateAsset {
        UpdateAsset {
            version: "2.0.0".to_owned(),
            name: "reverbic-v2.0.0-x86_64-windows.exe".to_owned(),
            download_url: "https://example.com/reverbic.exe".to_owned(),
            size: 42,
            digest: Some("sha256:abc".to_owned()),
        }
    }

    #[test]
    fn update_download_path_uses_asset_name_without_part_suffix() {
        let path = update_download_path(&update_asset());
        let file_name = path
            .file_name()
            .expect("update path should include a file name")
            .to_string_lossy();

        assert_eq!(
            file_name,
            "reverbic-update-reverbic-v2.0.0-x86_64-windows.exe"
        );
        assert!(!file_name.ends_with(".part"));
    }

    #[test]
    fn unique_part_path_appends_part_suffix_and_changes_each_call() {
        let path = update_download_path(&update_asset());
        let first = unique_part_path(&path);
        let second = unique_part_path(&path);
        let first_name = first
            .file_name()
            .expect("first part path should include a file name")
            .to_string_lossy();
        let second_name = second
            .file_name()
            .expect("second part path should include a file name")
            .to_string_lossy();

        assert!(first_name.starts_with("reverbic-update-reverbic-v2.0.0-x86_64-windows.exe."));
        assert!(first_name.ends_with(".part"));
        assert!(second_name.ends_with(".part"));
        assert_ne!(first, second);
    }
}

#[cfg(test)]
mod asset_selection_tests {
    use super::{select_compatible_asset, AssetSelectionError, GitHubAsset, UpdateTarget};

    fn asset(name: &str) -> GitHubAsset {
        GitHubAsset {
            name: name.to_owned(),
            browser_download_url: format!("https://example.com/{name}"),
            size: 42,
            digest: Some("sha256:abc".to_owned()),
        }
    }

    #[test]
    fn selects_update_asset_for_windows_x86_64() {
        let target = UpdateTarget::from_parts("windows", "x86_64")
            .expect("windows x86_64 should be a supported update target");
        let selected = select_compatible_asset(
            vec![
                asset("reverbic-v2.0.0-x86_64-macos.tar.gz"),
                asset("reverbic-v2.0.0-x86_64-windows.exe"),
            ],
            "2.0.0",
            target,
        )
        .expect("windows release asset should be selected");

        assert_eq!(selected.name, "reverbic-v2.0.0-x86_64-windows.exe");
    }

    #[test]
    fn rejects_update_asset_for_linux() {
        let err = UpdateTarget::from_parts("linux", "x86_64")
            .expect_err("linux update target should be unsupported");

        assert_eq!(err, AssetSelectionError::UnsupportedPlatform);
    }

    #[test]
    fn rejects_update_asset_for_macos_until_archives_are_installable() {
        let err = UpdateTarget::from_parts("macos", "aarch64")
            .expect_err("macos update target should be unsupported until archives are installable");

        assert_eq!(err, AssetSelectionError::UnsupportedPlatform);
    }

    #[test]
    fn rejects_update_asset_when_compatible_asset_is_missing() {
        let target = UpdateTarget::from_parts("windows", "x86_64")
            .expect("windows x86_64 should be a supported update target");
        let err = select_compatible_asset(
            vec![asset("reverbic-v2.0.0-aarch64-macos.tar.gz")],
            "2.0.0",
            target,
        )
        .expect_err("missing compatible asset should be rejected");

        assert_eq!(err, AssetSelectionError::NoCompatibleAsset);
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    #[cfg(target_os = "windows")]
    use super::{build_update_script, escape_batch_path};
    #[cfg(target_os = "windows")]
    use std::path::Path;

    #[test]
    fn build_update_script_contains_replace_and_cleanup_steps() {
        let script = build_update_script(
            Path::new(r"C:\Temp\reverbic-update.exe"),
            Path::new(r"C:\Apps\reverbic.exe"),
            Path::new(r"C:\Apps\reverbic.exe.old"),
        );

        assert!(script.contains("move /Y \"%DST%\" \"%OLD%\""));
        assert!(script.contains("copy /Y \"%SRC%\" \"%DST%\""));
        assert!(script.contains("del /F /Q \"%OLD%\""));
        assert!(script.contains("del /F /Q \"%SRC%\""));
        assert!(script.contains("del /F /Q \"%~f0\""));
    }

    #[test]
    fn escape_batch_path_escapes_percent() {
        let escaped = escape_batch_path(Path::new(r"C:\Temp\100%\reverbic.exe"));
        assert_eq!(escaped, r"C:\Temp\100%%\reverbic.exe");
    }
}
