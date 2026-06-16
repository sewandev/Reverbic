use std::path::Path;

use serde::Deserialize;
use tokio::process::Command;

use super::resolve::requires_youtube_sign_in;
use super::search::parse_video_entries;
use super::{YoutubeError, YoutubePlaylist, YoutubeVideo};

const LIKED_VIDEOS_URL: &str = "https://www.youtube.com/playlist?list=LL";
const PLAYLISTS_URL: &str = "https://www.youtube.com/feed/playlists";

// YouTube `sp` filter that restricts search results to playlists only.
// It is the percent-encoded base64 token YouTube uses internally; isolated here
// because it is opaque and may need updating if YouTube changes the encoding.
const SEARCH_PLAYLISTS_FILTER: &str = "EgIQAw%3D%3D";

pub async fn search_playlists(
    binary: &Path,
    query: &str,
    cookies_path: Option<&Path>,
    deno_path: &Path,
    limit: usize,
) -> Result<Vec<YoutubePlaylist>, YoutubeError> {
    let url = format!(
        "https://www.youtube.com/results?search_query={}&sp={SEARCH_PLAYLISTS_FILTER}",
        encode_query(query)
    );

    let output = Command::new(binary)
        .args(build_flat_playlist_args(
            &url,
            limit,
            cookies_path,
            deno_path,
        ))
        .output()
        .await
        .map_err(|e| {
            YoutubeError::Search(format!(
                "{}: {e}",
                crate::i18n::t("modal.youtube.search_failed")
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let raw = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout.trim().to_string()
        };
        tracing::error!(query, "yt-dlp playlist search failed: {raw}");
        return Err(YoutubeError::Search(format!(
            "{}: {}",
            crate::i18n::t("modal.youtube.search_failed"),
            super::summarize_ytdlp_error(&raw)
        )));
    }

    parse_playlists_output(&output.stdout)
}

fn encode_query(query: &str) -> String {
    let mut out = String::with_capacity(query.len());
    for byte in query.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

pub async fn fetch_liked_videos(
    binary: &Path,
    cookies_path: Option<&Path>,
    deno_path: &Path,
    limit: usize,
) -> Result<Vec<YoutubeVideo>, YoutubeError> {
    let bytes = run_flat_playlist(binary, LIKED_VIDEOS_URL, limit, cookies_path, deno_path).await?;
    parse_video_entries(&bytes)
}

pub async fn fetch_playlists(
    binary: &Path,
    cookies_path: Option<&Path>,
    deno_path: &Path,
    limit: usize,
) -> Result<Vec<YoutubePlaylist>, YoutubeError> {
    let bytes = run_flat_playlist(binary, PLAYLISTS_URL, limit, cookies_path, deno_path).await?;
    parse_playlists_output(&bytes)
}

pub async fn fetch_mix_videos(
    binary: &Path,
    seed_video_id: &str,
    deno_path: &Path,
    limit: usize,
) -> Result<Vec<YoutubeVideo>, YoutubeError> {
    if !is_valid_video_id(seed_video_id) {
        return Err(YoutubeError::Library(crate::i18n::t(
            "modal.youtube.library_failed",
        )));
    }
    let url = format!("https://www.youtube.com/watch?v={seed_video_id}&list=RD{seed_video_id}");
    let bytes = run_flat_playlist(binary, &url, limit, None, deno_path).await?;
    parse_video_entries(&bytes)
}

fn is_valid_video_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 16
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub async fn fetch_playlist_videos(
    binary: &Path,
    cookies_path: Option<&Path>,
    deno_path: &Path,
    playlist_id: &str,
    limit: usize,
) -> Result<Vec<YoutubeVideo>, YoutubeError> {
    let url = format!("https://www.youtube.com/playlist?list={playlist_id}");
    let bytes = run_flat_playlist(binary, &url, limit, cookies_path, deno_path).await?;
    parse_video_entries(&bytes)
}

async fn run_flat_playlist(
    binary: &Path,
    url: &str,
    limit: usize,
    cookies_path: Option<&Path>,
    deno_path: &Path,
) -> Result<Vec<u8>, YoutubeError> {
    let output = Command::new(binary)
        .args(build_flat_playlist_args(
            url,
            limit,
            cookies_path,
            deno_path,
        ))
        .output()
        .await
        .map_err(|e| {
            YoutubeError::Library(format!(
                "{}: {e}",
                crate::i18n::t("modal.youtube.library_failed")
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let raw = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout.trim().to_string()
        };
        tracing::error!(url, "yt-dlp playlist fetch failed: {raw}");

        if requires_youtube_sign_in(&raw) {
            let key = if cookies_path.is_some() {
                "modal.youtube.cookies_expired"
            } else {
                "modal.youtube.auth_required"
            };
            return Err(YoutubeError::Library(crate::i18n::t(key)));
        }

        return Err(YoutubeError::Library(format!(
            "{}: {}",
            crate::i18n::t("modal.youtube.library_failed"),
            super::summarize_ytdlp_error(&raw)
        )));
    }

    Ok(output.stdout)
}

pub(crate) fn build_flat_playlist_args(
    url: &str,
    limit: usize,
    cookies_path: Option<&Path>,
    deno_path: &Path,
) -> Vec<String> {
    let mut args = super::base_ytdlp_args(super::EXTRACTOR_ARGS_FLAT, deno_path, cookies_path);
    args.push("--dump-single-json".to_string());
    args.push("--flat-playlist".to_string());
    args.push("--playlist-end".to_string());
    args.push(limit.max(1).to_string());
    args.push(url.to_string());
    args
}

fn parse_playlists_output(bytes: &[u8]) -> Result<Vec<YoutubePlaylist>, YoutubeError> {
    let payload: PlaylistsPayload = serde_json::from_slice(bytes).map_err(|e| {
        YoutubeError::Library(format!(
            "{}: {e}",
            crate::i18n::t("modal.youtube.library_failed")
        ))
    })?;

    Ok(payload
        .entries
        .unwrap_or_default()
        .into_iter()
        .filter(|entry| !entry.id.trim().is_empty())
        .map(|entry| YoutubePlaylist {
            id: entry.id,
            title: entry.title.unwrap_or_else(|| "YouTube".to_string()),
            video_count: entry.playlist_count.unwrap_or(0),
        })
        .collect())
}

#[derive(Deserialize)]
struct PlaylistsPayload {
    entries: Option<Vec<PlaylistEntry>>,
}

#[derive(Deserialize)]
struct PlaylistEntry {
    id: String,
    title: Option<String>,
    playlist_count: Option<u32>,
}
