use crate::audio::{AudioPlayer, PlayerCommand};
use crate::integrations::youtube::{deno, install, resolve, runtime_installed};

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const WATCH_URL: &str = "https://www.youtube.com/watch?v=eAKKa4zMoAE";
pub(super) const TITLE: &str = "Milky x PNAU - Just the Way You Are";
const START_AT_SECS: f32 = 13.0;
#[derive(Debug, PartialEq, Eq)]
pub enum AmbienceTrack {
    Pending,
    Resolving,
    Ready(String),
    Failed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AmbienceResolution {
    Ready(String),
    Failed,
}

pub fn start_resolution(
    track: &mut AmbienceTrack,
    tx: mpsc::Sender<AmbienceResolution>,
) -> Option<JoinHandle<()>> {
    if !matches!(track, AmbienceTrack::Pending) {
        return None;
    }

    *track = AmbienceTrack::Resolving;
    Some(tokio::spawn(async move {
        let resolution = resolve_stream_url().await;
        if tx.send(resolution).await.is_err() {
            tracing::debug!("onboarding ambience: resolved after onboarding finished");
        }
    }))
}

async fn resolve_stream_url() -> AmbienceResolution {
    if !runtime_installed() {
        return AmbienceResolution::Failed;
    }

    let binary = install::managed_binary_path();
    let deno_path = deno::managed_binary_path();
    match resolve::resolve_audio_url(&binary, WATCH_URL, None, &deno_path).await {
        Ok((url, _, _)) => AmbienceResolution::Ready(url),
        Err(e) => {
            tracing::debug!("onboarding ambience: failed to resolve stream ({e}), skipping");
            AmbienceResolution::Failed
        }
    }
}

pub fn finish_resolution(track: &mut AmbienceTrack, resolution: Option<AmbienceResolution>) {
    match resolution {
        Some(AmbienceResolution::Ready(url)) => *track = AmbienceTrack::Ready(url),
        Some(AmbienceResolution::Failed) | None => *track = AmbienceTrack::Failed,
    }
}

pub async fn play(player: &AudioPlayer, track: &AmbienceTrack) -> bool {
    let stream_url = match track {
        AmbienceTrack::Ready(url) => url.clone(),
        AmbienceTrack::Pending | AmbienceTrack::Resolving | AmbienceTrack::Failed => return false,
    };

    player
        .send(PlayerCommand::PlayPreview {
            url: stream_url,
            title: TITLE.to_string(),
            raw_track: TITLE.to_string(),
            preview_id: None,
            start_at_secs: START_AT_SECS,
        })
        .await
}

pub async fn stop(player: &AudioPlayer) {
    player.send(PlayerCommand::StopPreview).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finish_resolution_caches_ready_stream_url() {
        let mut track = AmbienceTrack::Resolving;

        finish_resolution(
            &mut track,
            Some(AmbienceResolution::Ready(
                "https://stream.example/audio".into(),
            )),
        );

        assert_eq!(
            track,
            AmbienceTrack::Ready("https://stream.example/audio".into())
        );
    }

    #[test]
    fn finish_resolution_marks_track_failed_when_result_is_missing() {
        let mut track = AmbienceTrack::Resolving;

        finish_resolution(&mut track, None);

        assert_eq!(track, AmbienceTrack::Failed);
    }
}
