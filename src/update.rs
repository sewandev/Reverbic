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
    MacOs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpdateArch {
    X86_64,
    Aarch64,
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
            "macos" | "darwin" => UpdateOs::MacOs,
            _ => return Err(AssetSelectionError::UnsupportedPlatform),
        };

        let arch = match arch {
            "x86_64" => UpdateArch::X86_64,
            "aarch64" => UpdateArch::Aarch64,
            _ => return Err(AssetSelectionError::NoCompatibleAsset),
        };

        match (os, arch) {
            (UpdateOs::Windows, UpdateArch::X86_64) => Ok(Self { os, arch }),
            (UpdateOs::MacOs, _) => Ok(Self { os, arch }),
            _ => Err(AssetSelectionError::NoCompatibleAsset),
        }
    }

    fn asset_name(self, version: &str) -> String {
        match (self.os, self.arch) {
            (UpdateOs::Windows, UpdateArch::X86_64) => {
                format!("reverbic-v{version}-x86_64-windows.exe")
            }
            (UpdateOs::MacOs, UpdateArch::X86_64) => {
                format!("reverbic-v{version}-x86_64-macos.tar.gz")
            }
            (UpdateOs::MacOs, UpdateArch::Aarch64) => {
                format!("reverbic-v{version}-aarch64-macos.tar.gz")
            }
            _ => unreachable!(),
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
    use std::fmt::Write;
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

    Ok(hasher
        .finalize()
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        }))
}

pub fn apply_update(new_exe: &Path) {
    #[cfg(target_os = "windows")]
    {
        if let Err(err) = spawn_windows_update_helper(new_exe) {
            tracing::error!("Failed to prepare update application: {err}");
        }
    }

    #[cfg(target_os = "macos")]
    {
        apply_macos_update(new_exe);
    }
}

#[cfg(target_os = "macos")]
fn apply_macos_update(update_payload: &Path) {
    let prepared = prepare_macos_update_payload(update_payload);
    let _ = std::fs::remove_file(update_payload);

    match prepared {
        Ok(candidate) => {
            if let Err(err) = replace_current_executable(&candidate.binary_path) {
                tracing::error!("Failed to apply macOS update: {err}");
            }
            if let Err(err) = std::fs::remove_dir_all(&candidate.extract_dir) {
                tracing::debug!(?err, "Failed to clean macOS update extraction directory");
            }
        }
        Err(err) => {
            tracing::debug!(
                ?err,
                payload = %update_payload.display(),
                "Could not prepare macOS update payload"
            );
        }
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
struct PreparedMacosUpdate {
    binary_path: PathBuf,
    extract_dir: PathBuf,
}

#[cfg(target_os = "macos")]
fn prepare_macos_update_payload(update_payload: &Path) -> std::io::Result<PreparedMacosUpdate> {
    let extract_dir = unique_macos_extract_dir(update_payload);
    std::fs::create_dir(&extract_dir)?;

    match extract_macos_update_archive(update_payload, &extract_dir) {
        Ok(binary_path) => Ok(PreparedMacosUpdate {
            binary_path,
            extract_dir,
        }),
        Err(err) => {
            let _ = std::fs::remove_dir_all(&extract_dir);
            Err(err)
        }
    }
}

#[cfg(target_os = "macos")]
fn unique_macos_extract_dir(update_payload: &Path) -> PathBuf {
    let payload_name = update_payload
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "reverbic-update".into());
    let base = std::env::temp_dir().join(format!("{payload_name}.{}.extract", std::process::id()));
    unique_part_path(&base)
}

#[cfg(target_os = "macos")]
fn extract_macos_update_archive(
    update_payload: &Path,
    extract_dir: &Path,
) -> std::io::Result<PathBuf> {
    use flate2::read::GzDecoder;
    use std::ffi::OsStr;
    use std::io;
    use tar::EntryType;

    let archive_file = std::fs::File::open(update_payload)?;
    let decoder = GzDecoder::new(archive_file);
    let mut archive = tar::Archive::new(decoder);
    let mut candidate = None;

    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let entry_path = safe_macos_archive_path(entry.path()?.as_ref())?;
        let entry_type = entry.header().entry_type();

        if entry_type == EntryType::Directory {
            std::fs::create_dir_all(extract_dir.join(&entry_path))?;
            continue;
        }

        if !entry_type.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unsupported archive entry type for {}",
                    entry_path.display()
                ),
            ));
        }

        let destination = extract_dir.join(&entry_path);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        entry.unpack(&destination)?;

        if entry_path.file_name() == Some(OsStr::new("reverbic"))
            && candidate.replace(destination).is_some()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "macOS update archive contains multiple reverbic binaries",
            ));
        }
    }

    let candidate = candidate.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "macOS update archive does not contain a reverbic binary",
        )
    })?;
    prepare_macos_update_binary(&candidate)?;
    Ok(candidate)
}

#[cfg(target_os = "macos")]
fn safe_macos_archive_path(path: &Path) -> std::io::Result<PathBuf> {
    use std::path::Component;

    let mut safe_path = PathBuf::new();
    let mut has_component = false;

    for component in path.components() {
        match component {
            Component::Normal(part) => {
                safe_path.push(part);
                has_component = true;
            }
            Component::CurDir => {}
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unsafe archive path {}", path.display()),
                ));
            }
        }
    }

    if !has_component {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "empty archive path",
        ));
    }

    Ok(safe_path)
}

#[cfg(target_os = "macos")]
fn prepare_macos_update_binary(candidate: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::symlink_metadata(candidate)?;
    if !metadata.file_type().is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "macOS update candidate is not a regular file",
        ));
    }

    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    std::fs::set_permissions(candidate, permissions)?;

    let mode = std::fs::metadata(candidate)?.permissions().mode();
    if mode & 0o111 == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "macOS update candidate is not executable",
        ));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn replace_current_executable(new_exe: &Path) -> std::io::Result<()> {
    let Ok(current) = std::env::current_exe() else {
        return Ok(());
    };
    if is_managed_macos_installation(&current) {
        let reason = managed_macos_installation_reason(&current).unwrap_or("managed");
        tracing::debug!(
            current = %current.display(),
            reason,
            "Skipping self-update for managed macOS installation"
        );
        return Ok(());
    };

    replace_executable_at(&current, new_exe)
}

#[cfg(target_os = "macos")]
fn replace_executable_at(current: &Path, new_exe: &Path) -> std::io::Result<()> {
    let Some(file_name) = current.file_name() else {
        return Ok(());
    };
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(current)
        .map(|m| m.permissions())
        .unwrap_or_else(|_| std::fs::Permissions::from_mode(0o755));
    perms.set_mode(perms.mode() | 0o111);
    let _ = std::fs::set_permissions(new_exe, perms.clone());

    let old_name = format!("{}.old", file_name.to_string_lossy());
    let old = current.with_file_name(old_name);

    remove_file_if_exists(&old)?;
    std::fs::rename(current, &old)?;

    let replace_result = std::fs::rename(new_exe, current).or_else(|_| {
        std::fs::copy(new_exe, current)?;
        let _ = std::fs::remove_file(new_exe);
        Ok(())
    });

    if let Err(err) = replace_result {
        let _ = std::fs::remove_file(current);
        let _ = std::fs::rename(&old, current);
        return Err(err);
    }

    let _ = std::fs::set_permissions(current, perms);
    let _ = std::fs::remove_file(old);
    Ok(())
}

#[cfg(target_os = "macos")]
fn is_managed_macos_installation(current: &Path) -> bool {
    managed_macos_installation_reason(current).is_some()
}

#[cfg(target_os = "macos")]
fn managed_macos_installation_reason(current: &Path) -> Option<&'static str> {
    if current.starts_with("/opt/homebrew/")
        || current.starts_with("/usr/local/Cellar/")
        || current.starts_with("/usr/local/Homebrew/")
    {
        return Some("homebrew");
    }

    if current.starts_with("/nix/store/") {
        return Some("nix");
    }

    if current.starts_with("/Applications/") && path_contains_app_bundle(current) {
        return Some("application bundle");
    }

    None
}

#[cfg(target_os = "macos")]
fn path_contains_app_bundle(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str().to_string_lossy().ends_with(".app"))
}

#[cfg(target_os = "macos")]
fn remove_file_if_exists(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
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
    fn selects_update_asset_for_macos_x86_64() {
        let target = UpdateTarget::from_parts("macos", "x86_64")
            .expect("macos x86_64 should be a supported update target");
        let selected = select_compatible_asset(
            vec![
                asset("reverbic-v2.0.0-x86_64-macos.tar.gz"),
                asset("reverbic-v2.0.0-x86_64-windows.exe"),
            ],
            "2.0.0",
            target,
        )
        .expect("macos release asset should be selected");

        assert_eq!(selected.name, "reverbic-v2.0.0-x86_64-macos.tar.gz");
    }

    #[test]
    fn selects_update_asset_for_macos_aarch64() {
        let target = UpdateTarget::from_parts("macos", "aarch64")
            .expect("macos aarch64 should be a supported update target");
        let selected = select_compatible_asset(
            vec![
                asset("reverbic-v2.0.0-aarch64-macos.tar.gz"),
                asset("reverbic-v2.0.0-x86_64-windows.exe"),
            ],
            "2.0.0",
            target,
        )
        .expect("macos release asset should be selected");

        assert_eq!(selected.name, "reverbic-v2.0.0-aarch64-macos.tar.gz");
    }

    #[test]
    fn rejects_update_asset_for_linux() {
        let err = UpdateTarget::from_parts("linux", "x86_64")
            .expect_err("linux update target should be unsupported");

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

#[cfg(all(test, target_os = "macos"))]
mod macos_payload_tests {
    use super::{
        is_managed_macos_installation, managed_macos_installation_reason,
        prepare_macos_update_payload, replace_executable_at, safe_macos_archive_path,
    };
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    fn next_id() -> u64 {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    fn test_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "reverbic-macos-update-test-{}-{}-{label}",
            std::process::id(),
            next_id()
        ))
    }

    fn test_dir(label: &str) -> PathBuf {
        let path = test_path(label);
        std::fs::create_dir(&path).expect("test directory should be created");
        path
    }

    fn write_archive(path: &Path, entries: &[(&str, &[u8])]) {
        let file = std::fs::File::create(path).expect("test archive should be created");
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = tar::Builder::new(encoder);

        for (entry_path, content) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, entry_path, *content)
                .expect("test archive entry should be appended");
        }

        builder.finish().expect("test archive should be finalized");
    }

    #[test]
    fn prepares_valid_macos_archive_candidate() {
        let archive_path = test_path("valid.tar.gz");
        write_archive(&archive_path, &[("reverbic", b"#!/bin/sh\n")]);

        let prepared = prepare_macos_update_payload(&archive_path)
            .expect("valid macOS archive should produce a candidate");
        let metadata =
            std::fs::metadata(&prepared.binary_path).expect("candidate should have metadata");

        assert!(metadata.is_file());
        assert_eq!(
            prepared
                .binary_path
                .file_name()
                .expect("candidate should include a file name"),
            "reverbic"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_ne!(metadata.permissions().mode() & 0o111, 0);
        }

        let _ = std::fs::remove_dir_all(prepared.extract_dir);
        let _ = std::fs::remove_file(archive_path);
    }

    #[test]
    fn rejects_archive_without_reverbic_binary() {
        let archive_path = test_path("missing.tar.gz");
        write_archive(&archive_path, &[("README.txt", b"not the binary")]);

        let err = prepare_macos_update_payload(&archive_path)
            .expect_err("archive without reverbic should be rejected");

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        let _ = std::fs::remove_file(archive_path);
    }

    #[test]
    fn rejects_archive_with_multiple_reverbic_binaries() {
        let archive_path = test_path("duplicate.tar.gz");
        write_archive(
            &archive_path,
            &[("reverbic", b"one"), ("nested/reverbic", b"two")],
        );

        let err = prepare_macos_update_payload(&archive_path)
            .expect_err("archive with duplicate reverbic binaries should be rejected");

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        let _ = std::fs::remove_file(archive_path);
    }

    #[test]
    fn rejects_suspicious_archive_paths() {
        let err = safe_macos_archive_path(Path::new("../reverbic"))
            .expect_err("parent directory path should be rejected");

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn replace_executable_at_installs_candidate_and_cleans_backup() {
        let dir = test_dir("replace-success");
        let current = dir.join("reverbic");
        let candidate = dir.join("candidate");
        let backup = dir.join("reverbic.old");
        std::fs::write(&current, b"old").expect("current binary should be written");
        std::fs::write(&candidate, b"new").expect("candidate binary should be written");

        replace_executable_at(&current, &candidate).expect("replacement should succeed");

        assert_eq!(
            std::fs::read(&current).expect("current binary should be readable"),
            b"new"
        );
        assert!(!candidate.exists());
        assert!(!backup.exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn replace_executable_at_removes_stale_backup_before_replace() {
        let dir = test_dir("replace-stale-backup");
        let current = dir.join("reverbic");
        let candidate = dir.join("candidate");
        let backup = dir.join("reverbic.old");
        std::fs::write(&current, b"old").expect("current binary should be written");
        std::fs::write(&candidate, b"new").expect("candidate binary should be written");
        std::fs::write(&backup, b"stale").expect("stale backup should be written");

        replace_executable_at(&current, &candidate)
            .expect("replacement should remove stale backup and succeed");

        assert_eq!(
            std::fs::read(&current).expect("current binary should be readable"),
            b"new"
        );
        assert!(!backup.exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn replace_executable_at_restores_backup_when_candidate_is_missing() {
        let dir = test_dir("replace-rollback");
        let current = dir.join("reverbic");
        let candidate = dir.join("missing-candidate");
        std::fs::write(&current, b"old").expect("current binary should be written");

        let err = replace_executable_at(&current, &candidate)
            .expect_err("missing candidate should fail replacement");

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        assert_eq!(
            std::fs::read(&current).expect("current binary should be restored"),
            b"old"
        );
        assert!(!dir.join("reverbic.old").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_managed_macos_installations() {
        assert_eq!(
            managed_macos_installation_reason(Path::new("/opt/homebrew/bin/reverbic")),
            Some("homebrew")
        );
        assert_eq!(
            managed_macos_installation_reason(Path::new(
                "/usr/local/Cellar/reverbic/1.0.0/bin/reverbic"
            )),
            Some("homebrew")
        );
        assert_eq!(
            managed_macos_installation_reason(Path::new("/usr/local/Homebrew/bin/reverbic")),
            Some("homebrew")
        );
        assert_eq!(
            managed_macos_installation_reason(Path::new("/nix/store/hash-reverbic/bin/reverbic")),
            Some("nix")
        );
        assert_eq!(
            managed_macos_installation_reason(Path::new(
                "/Applications/Reverbic.app/Contents/MacOS/reverbic"
            )),
            Some("application bundle")
        );
        assert!(is_managed_macos_installation(Path::new(
            "/Applications/Reverbic.app/Contents/MacOS/reverbic"
        )));
        assert!(!is_managed_macos_installation(Path::new(
            "/Users/example/bin/reverbic"
        )));
        assert!(!is_managed_macos_installation(Path::new(
            "/Applications/reverbic"
        )));
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
