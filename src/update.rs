use std::path::{Path, PathBuf};

const REPO: &str = "sewandev/Reverbic";
const CURRENT: &str = env!("CARGO_PKG_VERSION");

pub async fn fetch_latest_version() -> Option<String> {
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
    let json: serde_json::Value = resp.json().await.ok()?;
    let tag = json.get("tag_name")?.as_str()?;
    let version = tag.trim_start_matches('v');
    if is_newer(version, CURRENT) {
        Some(version.to_owned())
    } else {
        None
    }
}

pub async fn download_update(version: &str) -> Option<PathBuf> {
    let name = format!("reverbic-v{version}-x86_64-windows.exe");
    let url = format!("https://github.com/{REPO}/releases/download/v{version}/{name}");
    let path = std::env::temp_dir().join(format!("reverbic-update-v{version}.exe"));

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
    let resp = client.get(&url).send().await.ok()?;
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
