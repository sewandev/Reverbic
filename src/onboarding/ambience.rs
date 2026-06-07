use crate::audio::{AudioPlayer, PlayerCommand};
use crate::integrations::youtube::{install, resolve};

const WATCH_URL: &str = "https://www.youtube.com/watch?v=TPi3wu8t4GE";
const TITLE: &str = "Milky - Just the Way You Are";

pub(super) const VOLUME: f32 = 0.12;

/// Starts a calm welcome track quietly in the background.
/// Best-effort: silently does nothing if `yt-dlp` isn't installed yet or the
/// resolve step fails, since this is meant to be a pleasant surprise,
/// not something a first-time user should ever wait on or see fail.
///
/// `SetVolume` is sent *before* `PlayPreview` on purpose: the preview attaches
/// at whatever `current_volume` is at that moment, so setting it first is what
/// actually makes the track quiet — sending it afterwards would instead overwrite
/// the duck-restore value and leave the user's real volume stuck at this level.
pub async fn play(player: &AudioPlayer) {
    if !install::is_installed() {
        return;
    }
    let binary = install::managed_binary_path();

    let stream_url = match resolve::resolve_audio_url(&binary, WATCH_URL).await {
        Ok(url) => url,
        Err(e) => {
            tracing::debug!("onboarding ambience: failed to resolve stream ({e}), skipping");
            return;
        }
    };

    player.send(PlayerCommand::SetVolume(VOLUME)).await;
    player
        .send(PlayerCommand::PlayPreview {
            url: stream_url,
            title: TITLE.to_string(),
            raw_track: TITLE.to_string(),
        })
        .await;
}

/// Stops the welcome track and restores the volume the player had before `play` lowered it.
pub async fn stop(player: &AudioPlayer, original_volume: f32) {
    player.send(PlayerCommand::StopPreview).await;
    player.send(PlayerCommand::SetVolume(original_volume)).await;
}
