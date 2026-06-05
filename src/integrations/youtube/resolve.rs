use std::path::Path;

use tokio::process::Command;

use super::YoutubeError;

pub async fn resolve_audio_url(binary: &Path, watch_url: &str) -> Result<String, YoutubeError> {
    let output = Command::new(binary)
        .args(build_resolve_args(watch_url))
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
        let msg = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout.trim().to_string()
        };
        return Err(YoutubeError::Resolve(format!(
            "{}: {}",
            crate::i18n::t("modal.youtube.resolve_failed"),
            msg
        )));
    }

    parse_resolve_output(&output.stdout)
}

pub fn build_resolve_args(watch_url: &str) -> Vec<String> {
    vec![
        "--quiet".to_string(),
        "--no-warnings".to_string(),
        "--no-playlist".to_string(),
        "-f".to_string(),
        "bestaudio[ext=m4a][protocol!=m3u8_native]/bestaudio[acodec^=mp4a][protocol!=m3u8_native]/best[ext=m4a][protocol!=m3u8_native]/best[ext=mp4][protocol!=m3u8_native]".to_string(),
        "-g".to_string(),
        watch_url.to_string(),
    ]
}

fn parse_resolve_output(bytes: &[u8]) -> Result<String, YoutubeError> {
    let output = String::from_utf8(bytes.to_vec()).map_err(|e| {
        YoutubeError::Resolve(format!(
            "{}: {e}",
            crate::i18n::t("modal.youtube.resolve_failed")
        ))
    })?;

    output
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .ok_or_else(|| YoutubeError::Resolve(crate::i18n::t("modal.youtube.resolve_failed")))
}

#[cfg(test)]
mod tests {
    use super::{build_resolve_args, parse_resolve_output};

    #[test]
    fn build_resolve_args_requests_audio_url() {
        let args = build_resolve_args("https://www.youtube.com/watch?v=abc123");
        assert!(args.contains(&"-g".to_string()));
        assert!(args.contains(&"--no-playlist".to_string()));
        assert!(args.contains(
            &"bestaudio[ext=m4a][protocol!=m3u8_native]/bestaudio[acodec^=mp4a][protocol!=m3u8_native]/best[ext=m4a][protocol!=m3u8_native]/best[ext=mp4][protocol!=m3u8_native]".to_string()
        ));
    }

    #[test]
    fn parse_resolve_output_reads_first_non_empty_line() {
        let parsed =
            parse_resolve_output(b"\nhttps://stream.example/audio.m4a\n").expect("url is present");
        assert_eq!(parsed, "https://stream.example/audio.m4a");
    }
}
