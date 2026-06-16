use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use super::{YoutubeChapter, YoutubeError};

type ResolvedStream = (String, HashMap<String, String>, Vec<YoutubeChapter>);

const CACHE_TTL_SECS: u64 = 4 * 3600;
const CACHE_MAX_ENTRIES: usize = 200;
const CACHE_MAX_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
struct CacheEntry {
    url: String,
    headers: HashMap<String, String>,
    expires_at_unix: u64,
    #[serde(default)]
    persist: bool,
    #[serde(default)]
    used_cookies: bool,
    #[serde(default)]
    chapters: Vec<YoutubeChapter>,
}

fn get_url_cache() -> &'static Mutex<HashMap<String, CacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(load_cache_from_disk()))
}

fn cache_file_path() -> PathBuf {
    crate::paths::youtube_url_cache_file()
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn load_cache_from_disk() -> HashMap<String, CacheEntry> {
    let path = cache_file_path();
    let Ok(metadata) = std::fs::metadata(&path) else {
        return HashMap::new();
    };
    if metadata.len() > CACHE_MAX_FILE_BYTES {
        tracing::warn!("youtube url cache file is suspiciously large, ignoring it");
        return HashMap::new();
    }
    let Ok(data) = std::fs::read_to_string(&path) else {
        return HashMap::new();
    };
    let Ok(entries) = serde_json::from_str::<HashMap<String, CacheEntry>>(&data) else {
        tracing::warn!("youtube url cache file is malformed, ignoring it");
        return HashMap::new();
    };

    let now = unix_now();
    let valid: HashMap<String, CacheEntry> = entries
        .into_iter()
        .filter(|(watch_url, entry)| {
            watch_url.starts_with("https://")
                && entry.url.starts_with("https://")
                && !entry.url.contains("/api/manifest/")
                && entry.expires_at_unix > now
                && !contains_sensitive_header(&entry.headers)
        })
        .collect();
    tracing::info!(entries = valid.len(), "youtube url cache loaded from disk");
    valid
}

fn save_cache_to_disk(cache: &HashMap<String, CacheEntry>) {
    let now = unix_now();
    let mut persistable: Vec<(&String, &CacheEntry)> = cache
        .iter()
        .filter(|(_, entry)| entry.persist && entry.expires_at_unix > now)
        .collect();
    persistable.sort_by_key(|(_, entry)| std::cmp::Reverse(entry.expires_at_unix));
    persistable.truncate(CACHE_MAX_ENTRIES);
    let map: HashMap<&String, &CacheEntry> = persistable.into_iter().collect();

    match serde_json::to_string(&map) {
        Ok(json) => {
            if let Err(e) = std::fs::write(cache_file_path(), json) {
                tracing::debug!("could not persist youtube url cache: {e}");
            }
        }
        Err(e) => tracing::debug!("could not serialize youtube url cache: {e}"),
    }
}

fn contains_sensitive_header(headers: &HashMap<String, String>) -> bool {
    headers.keys().any(|key| {
        let key = key.to_lowercase();
        key == "cookie" || key == "authorization"
    })
}

pub async fn resolve_audio_url(
    binary: &Path,
    watch_url: &str,
    cookies_path: Option<&Path>,
    deno_path: &Path,
) -> Result<ResolvedStream, YoutubeError> {
    if let Ok(mut cache) = get_url_cache().lock() {
        if let Some(entry) = cache.get(watch_url) {
            let cookies_revoked = entry.used_cookies && cookies_path.is_none();
            if cookies_revoked || entry.expires_at_unix <= unix_now() {
                cache.remove(watch_url);
            } else {
                return Ok((
                    entry.url.clone(),
                    entry.headers.clone(),
                    entry.chapters.clone(),
                ));
            }
        }
    } else {
        tracing::warn!("youtube url cache lock poisoned during lookup, skipping cache");
    }

    let (resolved, used_cookies) = match run_yt_dlp_resolve(binary, watch_url, None, deno_path)
        .await
    {
        Ok(resolved) => (resolved, false),
        Err(anonymous_err) => {
            if cookies_path.is_none() {
                return Err(anonymous_err);
            }
            tracing::info!(watch_url, "anonymous resolve failed, retrying with cookies");
            let resolved = run_yt_dlp_resolve(binary, watch_url, cookies_path, deno_path).await?;
            (resolved, true)
        }
    };
    let (resolved_url, headers, chapters) = resolved;

    let persist = !used_cookies && !contains_sensitive_header(&headers);
    if let Ok(mut cache) = get_url_cache().lock() {
        cache.insert(
            watch_url.to_string(),
            CacheEntry {
                url: resolved_url.clone(),
                headers: headers.clone(),
                expires_at_unix: unix_now() + CACHE_TTL_SECS,
                persist,
                used_cookies,
                chapters: chapters.clone(),
            },
        );
        if persist {
            save_cache_to_disk(&cache);
        }
    } else {
        tracing::warn!("youtube url cache lock poisoned during insert, result not cached");
    }

    Ok((resolved_url, headers, chapters))
}

async fn run_yt_dlp_resolve(
    binary: &Path,
    watch_url: &str,
    cookies_path: Option<&Path>,
    deno_path: &Path,
) -> Result<ResolvedStream, YoutubeError> {
    let output = Command::new(binary)
        .args(build_resolve_args(watch_url, cookies_path, deno_path))
        .output()
        .await
        .map_err(|e| {
            YoutubeError::Resolve(format!(
                "{}: {e}",
                crate::i18n::t("modal.youtube.resolve_failed")
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
        tracing::error!(
            watch_url,
            with_cookies = cookies_path.is_some(),
            "yt-dlp resolve failed: {raw}"
        );

        if requires_youtube_sign_in(&raw) {
            let key = if cookies_path.is_some() {
                "modal.youtube.cookies_expired"
            } else {
                "modal.youtube.auth_required"
            };
            return Err(YoutubeError::Resolve(crate::i18n::t(key)));
        }

        if no_compatible_formats(&raw) {
            let key = match probe_live_status(binary, watch_url, cookies_path, deno_path)
                .await
                .as_deref()
            {
                Some("is_live") => {
                    tracing::info!(watch_url, "video is a live stream, reporting it clearly");
                    "modal.youtube.live_not_supported"
                }
                Some("post_live") => "modal.youtube.post_live_not_ready",
                _ => "modal.youtube.no_audio_formats",
            };
            return Err(YoutubeError::Resolve(crate::i18n::t(key)));
        }

        return Err(YoutubeError::Resolve(format!(
            "{}: {}",
            crate::i18n::t("modal.youtube.resolve_failed"),
            super::summarize_ytdlp_error(&raw)
        )));
    }

    parse_resolve_output(&output.stdout)
}

async fn probe_live_status(
    binary: &Path,
    watch_url: &str,
    cookies_path: Option<&Path>,
    deno_path: &Path,
) -> Option<String> {
    let mut args = super::base_ytdlp_args(super::EXTRACTOR_ARGS_DEFAULT, deno_path, cookies_path);
    args.push("--no-playlist".to_string());
    args.push("-j".to_string());
    args.push(watch_url.to_string());

    let output = match Command::new(binary).args(args).output().await {
        Ok(output) => output,
        Err(e) => {
            tracing::debug!(watch_url, "live status probe failed to spawn: {e}");
            return None;
        }
    };
    if !output.status.success() {
        tracing::debug!(watch_url, "live status probe failed");
        return None;
    }
    let json: serde_json::Value =
        match serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim()) {
            Ok(json) => json,
            Err(e) => {
                tracing::debug!(watch_url, "live status probe returned invalid json: {e}");
                return None;
            }
        };
    json["live_status"].as_str().map(str::to_string)
}

pub fn invalidate_cached_url(watch_url: &str) {
    if let Ok(mut cache) = get_url_cache().lock() {
        if cache.remove(watch_url).is_some() {
            save_cache_to_disk(&cache);
        }
    }
}

pub fn is_cached(watch_url: &str, cookies_path: Option<&Path>) -> bool {
    get_url_cache()
        .lock()
        .map(|cache| {
            cache.get(watch_url).is_some_and(|entry| {
                let cookies_revoked = entry.used_cookies && cookies_path.is_none();
                !cookies_revoked && entry.expires_at_unix > unix_now()
            })
        })
        .unwrap_or(false)
}

pub(crate) fn requires_youtube_sign_in(message: &str) -> bool {
    let message = message.to_lowercase();
    message.contains("sign in to confirm")
        || message.contains("not a bot")
        || message.contains("confirm your age")
}

pub(crate) fn no_compatible_formats(message: &str) -> bool {
    let message = message.to_lowercase();
    message.contains("requested format is not available")
        || message.contains("only images are available")
        || message.contains("po token")
}

pub fn build_resolve_args(
    watch_url: &str,
    cookies_path: Option<&Path>,
    deno_path: &Path,
) -> Vec<String> {
    let mut args = super::base_ytdlp_args(super::EXTRACTOR_ARGS_DEFAULT, deno_path, cookies_path);
    args.push("--no-playlist".to_string());
    args.push("-f".to_string());
    args.push("bestaudio[acodec^=mp4a.40.2][protocol!=m3u8_native]/bestaudio[ext=m4a][protocol!=m3u8_native]/bestaudio[acodec^=mp4a][protocol!=m3u8_native]/best[ext=m4a][protocol!=m3u8_native]/best[ext=mp4][protocol!=m3u8_native]".to_string());
    args.push("-j".to_string());
    args.push(watch_url.to_string());

    args
}

fn parse_resolve_output(bytes: &[u8]) -> Result<ResolvedStream, YoutubeError> {
    let output = String::from_utf8(bytes.to_vec()).map_err(|e| {
        YoutubeError::Resolve(format!(
            "{}: {e}",
            crate::i18n::t("modal.youtube.resolve_failed")
        ))
    })?;

    let json: serde_json::Value = serde_json::from_str(output.trim()).map_err(|e| {
        YoutubeError::Resolve(format!(
            "{}: Failed to parse JSON: {}",
            crate::i18n::t("modal.youtube.resolve_failed"),
            e
        ))
    })?;

    let url = json["url"]
        .as_str()
        .ok_or_else(|| {
            YoutubeError::Resolve(format!(
                "{}: Missing url field",
                crate::i18n::t("modal.youtube.resolve_failed")
            ))
        })?
        .to_string();

    let protocol = json["protocol"].as_str().unwrap_or("");
    if is_manifest_protocol(protocol) {
        let live_status = json["live_status"].as_str().unwrap_or("unknown");
        tracing::warn!(
            protocol,
            live_status,
            "yt-dlp resolved a manifest-based format the decoder cannot play"
        );
        let key = if live_status == "post_live" {
            "modal.youtube.post_live_not_ready"
        } else {
            "modal.youtube.no_audio_formats"
        };
        return Err(YoutubeError::Resolve(crate::i18n::t(key)));
    }

    let mut headers = HashMap::new();
    if let Some(h) = json["http_headers"].as_object() {
        for (k, v) in h {
            if let Some(vs) = v.as_str() {
                headers.insert(k.clone(), vs.to_string());
            }
        }
    }

    let chapters = parse_chapters(&json);

    log_resolved_format(&json);

    Ok((url, headers, chapters))
}

fn is_manifest_protocol(protocol: &str) -> bool {
    protocol.contains("dash") || protocol.contains("m3u8")
}

fn parse_chapters(json: &serde_json::Value) -> Vec<YoutubeChapter> {
    let Some(raw) = json["chapters"].as_array() else {
        return Vec::new();
    };
    let mut chapters: Vec<YoutubeChapter> = raw
        .iter()
        .filter_map(|chapter| {
            let title = chapter["title"].as_str()?.trim();
            let start = chapter["start_time"].as_f64()?;
            if title.is_empty() || start < 0.0 {
                return None;
            }
            Some(YoutubeChapter {
                title: title.to_string(),
                start_secs: start as f32,
            })
        })
        .collect();
    chapters.sort_by(|a, b| a.start_secs.total_cmp(&b.start_secs));
    chapters
}

fn log_resolved_format(json: &serde_json::Value) {
    let format_id = json["format_id"].as_str().unwrap_or("unknown");
    let acodec = json["acodec"].as_str().unwrap_or("unknown");
    let abr = json["abr"].as_f64().unwrap_or(0.0);
    let asr = json["asr"].as_u64().unwrap_or(0);
    tracing::info!(
        format_id,
        acodec,
        abr,
        asr,
        "yt-dlp: resolved YouTube audio format"
    );

    if format_id == "18" || acodec.starts_with("mp4a.40.5") {
        tracing::warn!(
            format_id,
            acodec,
            "yt-dlp: fell back to a combined or HE-AAC format; the decoder may fail (audio-only AAC-LC unavailable, possibly PO token enforcement)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{build_resolve_args, parse_resolve_output};
    use std::path::Path;

    #[test]
    fn build_resolve_args_requests_audio_url() {
        let deno = Path::new("/home/user/.reverbic/bin/deno");
        let args = build_resolve_args("https://www.youtube.com/watch?v=abc123", None, deno);
        assert!(args.contains(&"-j".to_string()));
        assert!(args.contains(&"--no-playlist".to_string()));
        assert!(args.contains(&"--force-ipv4".to_string()));
        assert!(args.contains(
            &"bestaudio[acodec^=mp4a.40.2][protocol!=m3u8_native]/bestaudio[ext=m4a][protocol!=m3u8_native]/bestaudio[acodec^=mp4a][protocol!=m3u8_native]/best[ext=m4a][protocol!=m3u8_native]/best[ext=mp4][protocol!=m3u8_native]".to_string()
        ));
        assert!(!args.contains(&"--cookies".to_string()));
    }

    #[test]
    fn build_resolve_args_includes_cookies_when_configured() {
        let cookies = Path::new("/home/user/.reverbic/cookies.txt");
        let deno = Path::new("/home/user/.reverbic/bin/deno");
        let args = build_resolve_args(
            "https://www.youtube.com/watch?v=abc123",
            Some(cookies),
            deno,
        );
        let cookies_idx = args
            .iter()
            .position(|arg| arg == "--cookies")
            .expect("--cookies flag should be present");
        assert_eq!(args[cookies_idx + 1], cookies.to_string_lossy());
    }

    #[test]
    fn build_resolve_args_includes_deno_runtime() {
        let deno = Path::new("/home/user/.reverbic/bin/deno");
        let args = build_resolve_args("https://www.youtube.com/watch?v=abc123", None, deno);
        let runtime_idx = args
            .iter()
            .position(|arg| arg == "--js-runtimes")
            .expect("--js-runtimes flag should be present");
        assert_eq!(args[runtime_idx + 1], "deno:/home/user/.reverbic/bin/deno");
    }

    #[test]
    fn parse_resolve_output_reads_json() {
        let json = r#"{"url": "https://stream.example/audio.m4a", "http_headers": {"User-Agent": "test"}, "chapters": [{"title": "Intro", "start_time": 0.0}, {"title": "Drop", "start_time": 62.5}]}"#;
        let (parsed_url, headers, chapters) =
            parse_resolve_output(json.as_bytes()).expect("url is present");
        assert_eq!(parsed_url, "https://stream.example/audio.m4a");
        assert_eq!(
            headers.get("User-Agent").expect("User-Agent header"),
            "test"
        );
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[1].title, "Drop");
        assert_eq!(chapters[1].start_secs, 62.5);
    }
}
