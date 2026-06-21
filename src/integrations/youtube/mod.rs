pub mod cookies;
pub mod deno;
pub mod error;
pub mod install;
pub mod playlists;
pub mod resolve;
pub mod search;
pub mod sponsorblock;

pub use error::YoutubeError;

pub fn runtime_installed() -> bool {
    install::is_installed() && deno::is_installed()
}

#[derive(serde::Deserialize)]
pub(crate) struct GithubRelease {
    pub tag_name: String,
    pub assets: Vec<GithubAsset>,
}

#[derive(serde::Deserialize)]
pub(crate) struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub digest: String,
}

pub(crate) async fn fetch_latest_release(api_url: &str) -> Result<GithubRelease, String> {
    let client = crate::http::http_client_timeout(120).ok_or("could not build HTTP client")?;
    client
        .get(api_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;

    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

pub(crate) const EXTRACTOR_ARGS_DEFAULT: &str = "youtube:player_client=android_vr,tv,web";

pub(crate) fn summarize_ytdlp_error(raw: &str) -> String {
    const MAX_CHARS: usize = 200;
    let trimmed = raw.trim();
    let line = trimmed
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| line.starts_with("ERROR:"))
        .map(|line| line.trim_start_matches("ERROR:").trim())
        .or_else(|| trimmed.lines().map(str::trim).find(|line| !line.is_empty()))
        .unwrap_or(trimmed);

    if line.chars().count() > MAX_CHARS {
        let cut: String = line.chars().take(MAX_CHARS).collect();
        format!("{cut}…")
    } else {
        line.to_string()
    }
}
pub(crate) const YTDLP_NETWORK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(45);
pub(crate) const YTDLP_LOCAL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

pub(crate) async fn run_ytdlp_output(
    mut cmd: tokio::process::Command,
    timeout: std::time::Duration,
    log_ctx: &str,
) -> std::io::Result<std::process::Output> {
    cmd.kill_on_drop(true);
    match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(result) => result,
        Err(_) => {
            tracing::error!(
                ctx = log_ctx,
                "yt-dlp command timed out after {}s",
                timeout.as_secs()
            );
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "yt-dlp command timed out",
            ))
        }
    }
}

pub(crate) const EXTRACTOR_ARGS_FLAT: &str =
    "youtube:player_client=android,web;player_skip=configs,webpage";

pub(crate) fn base_ytdlp_args(
    extractor_args: &str,
    deno_path: &std::path::Path,
    cookies_path: Option<&std::path::Path>,
) -> Vec<String> {
    let mut args = vec![
        "--quiet".to_string(),
        "--no-warnings".to_string(),
        "--force-ipv4".to_string(),
        "--extractor-args".to_string(),
        extractor_args.to_string(),
        "--js-runtimes".to_string(),
        format!("deno:{}", deno_path.to_string_lossy()),
    ];

    if let Some(path) = cookies_path {
        args.push("--cookies".to_string());
        args.push(path.to_string_lossy().into_owned());
    }

    args
}

#[derive(Clone, Debug, PartialEq)]
pub struct YoutubeVideo {
    pub id: String,
    pub title: String,
    pub channel: String,
    pub duration_secs: u32,
    pub watch_url: String,
    pub thumbnail: Option<String>,
    pub is_live: bool,
}

#[derive(Clone, Debug)]
pub struct ResolvedYoutubePlayback {
    pub video: YoutubeVideo,
    pub stream_url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub chapters: Vec<YoutubeChapter>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct YoutubeChapter {
    pub title: String,
    pub start_secs: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct YoutubePlaylist {
    pub id: String,
    pub title: String,
    pub video_count: u32,
}
