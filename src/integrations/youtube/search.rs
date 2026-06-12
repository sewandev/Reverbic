use std::path::Path;

use serde::Deserialize;
use tokio::process::Command;

use super::{YoutubeError, YoutubeVideo};

pub async fn search_videos(
    binary: &Path,
    query: &str,
    limit: usize,
    cookies_path: Option<&Path>,
    deno_path: &Path,
) -> Result<Vec<YoutubeVideo>, YoutubeError> {
    let output = Command::new(binary)
        .args(build_search_args(query, limit, cookies_path, deno_path))
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
        tracing::error!(query, "yt-dlp search failed: {raw}");
        return Err(YoutubeError::Search(format!(
            "{}: {}",
            crate::i18n::t("modal.youtube.search_failed"),
            super::summarize_ytdlp_error(&raw)
        )));
    }

    parse_video_entries(&output.stdout)
}

pub fn build_search_args(
    query: &str,
    limit: usize,
    cookies_path: Option<&Path>,
    deno_path: &Path,
) -> Vec<String> {
    let mut args = super::base_ytdlp_args(super::EXTRACTOR_ARGS_FLAT, deno_path, cookies_path);
    args.push("--dump-single-json".to_string());
    args.push("--flat-playlist".to_string());
    args.push(format!("ytsearch{}:{}", limit.max(1), query));
    args
}

pub(crate) fn parse_video_entries(bytes: &[u8]) -> Result<Vec<YoutubeVideo>, YoutubeError> {
    let payload: VideoListPayload = serde_json::from_slice(bytes).map_err(|e| {
        YoutubeError::Search(format!(
            "{}: {e}",
            crate::i18n::t("modal.youtube.search_failed")
        ))
    })?;

    Ok(payload
        .entries
        .unwrap_or_default()
        .into_iter()
        .filter(|entry| !entry.id.trim().is_empty())
        .map(|entry| {
            let id = entry.id;
            let title = entry.title.unwrap_or_else(|| "YouTube".to_string());
            let channel = entry
                .channel
                .or(entry.uploader)
                .unwrap_or_else(|| "YouTube".to_string());
            let watch_url = entry
                .webpage_url
                .or_else(|| entry.url.map(|url| normalize_watch_url(&id, &url)))
                .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={id}"));
            let thumbnail = entry.thumbnail.or_else(|| {
                entry
                    .thumbnails
                    .and_then(|thumbs| thumbs.into_iter().last().map(|t| t.url))
            });

            YoutubeVideo {
                id,
                title,
                channel,
                duration_secs: entry.duration.map(|value| value.as_u32()).unwrap_or(0),
                watch_url,
                thumbnail,
            }
        })
        .collect())
}

fn normalize_watch_url(id: &str, value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("https://www.youtube.com/watch?v={id}")
    }
}

#[derive(Deserialize)]
struct VideoListPayload {
    entries: Option<Vec<VideoEntry>>,
}

#[derive(Deserialize)]
struct VideoEntry {
    id: String,
    title: Option<String>,
    channel: Option<String>,
    uploader: Option<String>,
    duration: Option<YoutubeDuration>,
    webpage_url: Option<String>,
    url: Option<String>,
    thumbnail: Option<String>,
    thumbnails: Option<Vec<Thumbnail>>,
}

#[derive(Deserialize)]
struct Thumbnail {
    url: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum YoutubeDuration {
    Integer(u64),
    Float(f64),
}

impl YoutubeDuration {
    fn as_u32(&self) -> u32 {
        match self {
            Self::Integer(value) => (*value).min(u32::MAX as u64) as u32,
            Self::Float(value) => {
                if !value.is_finite() || *value <= 0.0 {
                    0
                } else if *value >= u32::MAX as f64 {
                    u32::MAX
                } else {
                    value.round() as u32
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_search_args, parse_video_entries};
    use std::path::Path;

    #[test]
    fn build_search_args_uses_flat_json_search() {
        let deno = Path::new("/home/user/.reverbic/bin/deno");
        let args = build_search_args("lofi hip hop", 7, None, deno);
        assert!(args.contains(&"--dump-single-json".to_string()));
        assert!(args.contains(&"--flat-playlist".to_string()));
        assert!(args.contains(&"--force-ipv4".to_string()));
        assert_eq!(args.last(), Some(&"ytsearch7:lofi hip hop".to_string()));
        assert!(!args.contains(&"--cookies".to_string()));
    }

    #[test]
    fn build_search_args_includes_cookies_when_configured() {
        let cookies = Path::new("/home/user/.reverbic/cookies.txt");
        let deno = Path::new("/home/user/.reverbic/bin/deno");
        let args = build_search_args("lofi hip hop", 7, Some(cookies), deno);
        let cookies_idx = args
            .iter()
            .position(|arg| arg == "--cookies")
            .expect("--cookies flag should be present");
        assert_eq!(args[cookies_idx + 1], cookies.to_string_lossy());
        assert_eq!(args.last(), Some(&"ytsearch7:lofi hip hop".to_string()));
    }

    #[test]
    fn build_search_args_includes_deno_runtime() {
        let deno = Path::new("/home/user/.reverbic/bin/deno");
        let args = build_search_args("lofi hip hop", 7, None, deno);
        let runtime_idx = args
            .iter()
            .position(|arg| arg == "--js-runtimes")
            .expect("--js-runtimes flag should be present");
        assert_eq!(args[runtime_idx + 1], "deno:/home/user/.reverbic/bin/deno");
    }

    #[test]
    fn parse_search_output_maps_entries_into_videos() {
        let json = br#"{
          "entries": [
            {
              "id": "abc123",
              "title": "Lofi Mix",
              "channel": "Reverbic",
              "duration": 321,
              "webpage_url": "https://www.youtube.com/watch?v=abc123",
              "thumbnail": "https://img.example/thumb.jpg"
            }
          ]
        }"#;

        let videos = parse_video_entries(json).expect("search json should parse");
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].id, "abc123");
        assert_eq!(videos[0].title, "Lofi Mix");
        assert_eq!(videos[0].channel, "Reverbic");
        assert_eq!(videos[0].duration_secs, 321);
    }

    #[test]
    fn parse_search_output_accepts_float_duration() {
        let json = br#"{
          "entries": [
            {
              "id": "def456",
              "title": "Pendora Mix",
              "uploader": "Uploader",
              "duration": 6107.0
            }
          ]
        }"#;

        let videos = parse_video_entries(json).expect("float duration should parse");
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].duration_secs, 6107);
        assert_eq!(
            videos[0].watch_url,
            "https://www.youtube.com/watch?v=def456"
        );
    }
}
