use std::path::Path;

use tokio::process::Command;

use super::YoutubeError;

pub async fn resolve_audio_url(
    binary: &Path,
    watch_url: &str,
    cookies_path: Option<&Path>,
    quickjs_path: &Path,
) -> Result<String, YoutubeError> {
    let output = Command::new(binary)
        .args(build_resolve_args(watch_url, cookies_path, quickjs_path))
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

        if requires_youtube_sign_in(&msg) {
            let key = if cookies_path.is_some() {
                "modal.youtube.cookies_expired"
            } else {
                "modal.youtube.auth_required"
            };
            return Err(YoutubeError::Resolve(crate::i18n::t(key)));
        }

        return Err(YoutubeError::Resolve(format!(
            "{}: {}",
            crate::i18n::t("modal.youtube.resolve_failed"),
            msg
        )));
    }

    parse_resolve_output(&output.stdout)
}

pub(crate) fn requires_youtube_sign_in(message: &str) -> bool {
    let message = message.to_lowercase();
    message.contains("sign in to confirm")
        || message.contains("not a bot")
        || message.contains("confirm your age")
}

pub fn build_resolve_args(
    watch_url: &str,
    cookies_path: Option<&Path>,
    quickjs_path: &Path,
) -> Vec<String> {
    let mut args = vec![
        "--quiet".to_string(),
        "--no-warnings".to_string(),
        "--no-playlist".to_string(),
        "--js-runtimes".to_string(),
        format!("quickjs:{}", quickjs_path.to_string_lossy()),
    ];

    if let Some(path) = cookies_path {
        args.push("--cookies".to_string());
        args.push(path.to_string_lossy().into_owned());
    }

    args.push("-f".to_string());
    args.push("bestaudio[ext=m4a][protocol!=m3u8_native]/bestaudio[acodec^=mp4a][protocol!=m3u8_native]/best[ext=m4a][protocol!=m3u8_native]/best[ext=mp4][protocol!=m3u8_native]".to_string());
    args.push("-g".to_string());
    args.push(watch_url.to_string());

    args
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
    use std::path::Path;

    #[test]
    fn build_resolve_args_requests_audio_url() {
        let quickjs = Path::new("/home/user/.reverbic/bin/qjs");
        let args = build_resolve_args("https://www.youtube.com/watch?v=abc123", None, quickjs);
        assert!(args.contains(&"-g".to_string()));
        assert!(args.contains(&"--no-playlist".to_string()));
        assert!(args.contains(
            &"bestaudio[ext=m4a][protocol!=m3u8_native]/bestaudio[acodec^=mp4a][protocol!=m3u8_native]/best[ext=m4a][protocol!=m3u8_native]/best[ext=mp4][protocol!=m3u8_native]".to_string()
        ));
        assert!(!args.contains(&"--cookies".to_string()));
    }

    #[test]
    fn build_resolve_args_includes_cookies_when_configured() {
        let cookies = Path::new("/home/user/.reverbic/cookies.txt");
        let quickjs = Path::new("/home/user/.reverbic/bin/qjs");
        let args = build_resolve_args(
            "https://www.youtube.com/watch?v=abc123",
            Some(cookies),
            quickjs,
        );
        let cookies_idx = args
            .iter()
            .position(|arg| arg == "--cookies")
            .expect("--cookies flag should be present");
        assert_eq!(args[cookies_idx + 1], cookies.to_string_lossy());
    }

    #[test]
    fn build_resolve_args_includes_quickjs_runtime() {
        let quickjs = Path::new("/home/user/.reverbic/bin/qjs");
        let args = build_resolve_args("https://www.youtube.com/watch?v=abc123", None, quickjs);
        let runtime_idx = args
            .iter()
            .position(|arg| arg == "--js-runtimes")
            .expect("--js-runtimes flag should be present");
        assert_eq!(
            args[runtime_idx + 1],
            "quickjs:/home/user/.reverbic/bin/qjs"
        );
    }

    #[test]
    fn parse_resolve_output_reads_first_non_empty_line() {
        let parsed =
            parse_resolve_output(b"\nhttps://stream.example/audio.m4a\n").expect("url is present");
        assert_eq!(parsed, "https://stream.example/audio.m4a");
    }
}
