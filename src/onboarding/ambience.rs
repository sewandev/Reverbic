use crate::audio::{AudioPlayer, PlayerCommand};
use crate::integrations::youtube::{install, resolve};

const WATCH_URL: &str = "https://www.youtube.com/watch?v=eAKKa4zMoAE";
const TITLE: &str = "Milky x PNAU - Just the Way You Are";
const START_AT_SECS: f32 = 13.0;
pub enum AmbienceTrack {
    Pending,
    Ready(String),
    Failed,
}
pub async fn play(player: &AudioPlayer, track: &mut AmbienceTrack) {
    let stream_url = match track {
        AmbienceTrack::Ready(url) => url.clone(),
        AmbienceTrack::Failed => return,
        AmbienceTrack::Pending => {
            if !install::is_installed() {
                return;
            }
            let binary = install::managed_binary_path();
            match resolve::resolve_audio_url(&binary, WATCH_URL).await {
                Ok(url) => {
                    *track = AmbienceTrack::Ready(url.clone());
                    url
                }
                Err(e) => {
                    tracing::debug!(
                        "onboarding ambience: failed to resolve stream ({e}), skipping"
                    );
                    *track = AmbienceTrack::Failed;
                    return;
                }
            }
        }
    };

    player
        .send(PlayerCommand::PlayPreview {
            url: stream_url,
            title: TITLE.to_string(),
            raw_track: TITLE.to_string(),
            start_at_secs: START_AT_SECS,
        })
        .await;
}

pub async fn stop(player: &AudioPlayer) {
    player.send(PlayerCommand::StopPreview).await;
}
