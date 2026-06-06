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
    Macos,
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
        let arch = match arch {
            "x86_64" => UpdateArch::X86_64,
            "aarch64" => UpdateArch::Aarch64,
            _ => return Err(AssetSelectionError::NoCompatibleAsset),
        };

        let os = match os {
            "windows" => UpdateOs::Windows,
            "macos" => UpdateOs::Macos,
            _ => return Err(AssetSelectionError::UnsupportedPlatform),
        };

        match (os, arch) {
            (UpdateOs::Windows, UpdateArch::X86_64)
            | (UpdateOs::Macos, UpdateArch::X86_64 | UpdateArch::Aarch64) => Ok(Self { os, arch }),
            (UpdateOs::Windows, UpdateArch::Aarch64) => Err(AssetSelectionError::NoCompatibleAsset),
        }
    }

    fn asset_name(self, version: &str) -> String {
        match (self.os, self.arch) {
            (UpdateOs::Windows, UpdateArch::X86_64) => {
                format!("reverbic-v{version}-x86_64-windows.exe")
            }
            (UpdateOs::Macos, UpdateArch::X86_64) => {
                format!("reverbic-v{version}-x86_64-macos.tar.gz")
            }
            (UpdateOs::Macos, UpdateArch::Aarch64) => {
                format!("reverbic-v{version}-aarch64-macos.tar.gz")
            }
            (UpdateOs::Windows, UpdateArch::Aarch64) => unreachable!("unsupported target"),
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
    let path = std::env::temp_dir().join(format!("reverbic-update-{}", asset.name));
    tracing::debug!(
        asset = %asset.name,
        size = asset.size,
        digest = asset.digest.as_deref().unwrap_or(""),
        "Downloading update asset"
    );

    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > 1_000_000 {
            return Some(path);
        }
        let _ = std::fs::remove_file(&path);
    }

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

    if stream_to_file(resp, &path).await {
        Some(path)
    } else {
        let _ = tokio::fs::remove_file(&path).await;
        None
    }
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
        let target = UpdateTarget::from_parts("windows", "x86_64").unwrap();
        let selected = select_compatible_asset(
            vec![
                asset("reverbic-v2.0.0-x86_64-macos.tar.gz"),
                asset("reverbic-v2.0.0-x86_64-windows.exe"),
            ],
            "2.0.0",
            target,
        )
        .unwrap();

        assert_eq!(selected.name, "reverbic-v2.0.0-x86_64-windows.exe");
    }

    #[test]
    fn selects_update_asset_for_macos_arch() {
        let target = UpdateTarget::from_parts("macos", "aarch64").unwrap();
        let selected = select_compatible_asset(
            vec![
                asset("reverbic-v2.0.0-x86_64-windows.exe"),
                asset("reverbic-v2.0.0-aarch64-macos.tar.gz"),
            ],
            "2.0.0",
            target,
        )
        .unwrap();

        assert_eq!(selected.name, "reverbic-v2.0.0-aarch64-macos.tar.gz");
    }

    #[test]
    fn rejects_update_asset_for_linux() {
        let err = UpdateTarget::from_parts("linux", "x86_64").unwrap_err();

        assert_eq!(err, AssetSelectionError::UnsupportedPlatform);
    }

    #[test]
    fn rejects_update_asset_when_compatible_asset_is_missing() {
        let target = UpdateTarget::from_parts("macos", "x86_64").unwrap();
        let err = select_compatible_asset(
            vec![asset("reverbic-v2.0.0-x86_64-windows.exe")],
            "2.0.0",
            target,
        )
        .unwrap_err();

        assert_eq!(err, AssetSelectionError::NoCompatibleAsset);
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::{build_update_script, escape_batch_path};
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
