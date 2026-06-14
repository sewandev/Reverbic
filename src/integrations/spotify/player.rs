use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::time::{Duration, Instant};

use librespot_core::{
    authentication::Credentials, cache::Cache, Session, SessionConfig, SpotifyUri,
};
use librespot_metadata::audio::{AudioItem, UniqueFields};
use librespot_playback::{
    audio_backend::{self, SinkBuilder},
    config::{AudioFormat, PlayerConfig, VolumeCtrl},
    mixer::{softmixer::SoftMixer, Mixer, MixerConfig},
    player::{Player, PlayerEvent},
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::{SpotifyPlayerCmd, SpotifyPlayerEvent, SpotifyTrack};

// Matches the 0.5 attenuation the SoftMixer applies by default, so adding
// per-player linear mixers does not change the perceived loudness.
const NATIVE_TARGET_VOLUME: u16 = u16::MAX / 2;
const POSITION_UPDATE_INTERVAL: Duration = Duration::from_millis(500);
const FADE_TICK: Duration = Duration::from_millis(100);
// If the incoming track never starts (slow network, unavailable), force the
// fade anyway so the outgoing player does not keep playing forever.
const FADE_START_TIMEOUT: Duration = Duration::from_secs(5);

pub struct SpotifyPlayerHandle {
    cmd_tx: UnboundedSender<SpotifyPlayerCmd>,
}

impl SpotifyPlayerHandle {
    pub fn play(&self, uris: Vec<String>) {
        let _ = self.cmd_tx.send(SpotifyPlayerCmd::Play { uris });
    }

    pub fn pause(&self) {
        let _ = self.cmd_tx.send(SpotifyPlayerCmd::Pause);
    }

    pub fn resume(&self) {
        let _ = self.cmd_tx.send(SpotifyPlayerCmd::Resume);
    }

    pub fn set_crossfade(&self, secs: u8) {
        let _ = self.cmd_tx.send(SpotifyPlayerCmd::SetCrossfade { secs });
    }

    pub fn crossfade_to(&self, uri: String) {
        let _ = self.cmd_tx.send(SpotifyPlayerCmd::CrossfadeTo { uri });
    }
}

fn track_from_audio_item(item: &AudioItem) -> Option<SpotifyTrack> {
    let UniqueFields::Track { artists, album, .. } = &item.unique_fields else {
        return None;
    };
    let first_artist = artists.first();
    Some(SpotifyTrack {
        name: item.name.clone(),
        artist: first_artist.map(|a| a.name.clone()).unwrap_or_default(),
        album: album.clone(),
        duration_ms: item.duration_ms,
        uri: item.uri.clone(),
    })
}

pub fn spawn_player(
    audio_token: String,
    event_tx: SyncSender<SpotifyPlayerEvent>,
) -> SpotifyPlayerHandle {
    let (cmd_tx, cmd_rx) = unbounded_channel::<SpotifyPlayerCmd>();
    tokio::spawn(run_player(audio_token, cmd_rx, event_tx));
    SpotifyPlayerHandle { cmd_tx }
}

pub fn has_cached_credentials() -> bool {
    open_cache()
        .as_ref()
        .and_then(|cache| cache.credentials())
        .is_some()
}

fn open_cache() -> Option<Cache> {
    Cache::new(
        Some(&cache_dir()),
        None::<&std::path::PathBuf>,
        None::<&std::path::PathBuf>,
        None,
    )
    .ok()
}

fn cache_dir() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(|p| {
                std::path::PathBuf::from(p)
                    .join(".reverbic")
                    .join("librespot")
            })
            .unwrap_or_else(|_| crate::config::reverbic_dir().join("librespot"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::config::reverbic_dir().join("librespot")
    }
}

struct NativePlayer {
    player: Arc<Player>,
    mixer: SoftMixer,
    events: UnboundedReceiver<PlayerEvent>,
}

fn build_native_player(
    session: &Session,
    backend: SinkBuilder,
    format: AudioFormat,
    initial_volume: u16,
) -> Result<NativePlayer, librespot_core::Error> {
    let mixer = SoftMixer::open(MixerConfig {
        volume_ctrl: VolumeCtrl::Linear,
        ..MixerConfig::default()
    })?;
    mixer.set_volume(initial_volume);
    let volume_fn = mixer.get_soft_volume();
    let player_config = PlayerConfig {
        position_update_interval: Some(POSITION_UPDATE_INTERVAL),
        ..PlayerConfig::default()
    };
    let player = Player::new(player_config, session.clone(), volume_fn, move || {
        (backend)(None, format)
    });
    let events = player.get_player_event_channel();
    Ok(NativePlayer {
        player,
        mixer,
        events,
    })
}

struct CrossfadeOut {
    player: Arc<Player>,
    mixer: SoftMixer,
    requested_at: Instant,
    fade_started: Option<Instant>,
    duration_secs: f32,
}

fn finish_crossfade_out(fading: &mut Option<CrossfadeOut>) {
    if let Some(out) = fading.take() {
        out.player.stop();
    }
}

async fn run_player(
    audio_token: String,
    mut cmd_rx: UnboundedReceiver<SpotifyPlayerCmd>,
    event_tx: SyncSender<SpotifyPlayerEvent>,
) {
    let session_config = SessionConfig {
        client_id: "65b708073fc0480ea92a077233ca87bd".to_string(),
        ..Default::default()
    };
    let cache = open_cache();
    let credentials = match cache.as_ref().and_then(|c| c.credentials()) {
        Some(cached) => {
            tracing::info!("librespot: using cached credentials");
            cached
        }
        None => {
            if audio_token.trim().is_empty() {
                let _ = event_tx.try_send(SpotifyPlayerEvent::Error(
                    "native_missing_credentials".into(),
                ));
                return;
            }
            tracing::info!("librespot: using OAuth token for first login");
            Credentials::with_access_token(&audio_token)
        }
    };

    let session = Session::new(session_config, cache);
    if let Err(e) = session.connect(credentials, true).await {
        let _ = event_tx.try_send(SpotifyPlayerEvent::Error(format!(
            "native_session_connect: {e}"
        )));
        return;
    }

    let backend = match audio_backend::find(None) {
        Some(b) => b,
        None => {
            let _ = event_tx.try_send(SpotifyPlayerEvent::Error(
                "native_audio_backend_missing".to_string(),
            ));
            return;
        }
    };
    let format = AudioFormat::default();

    let mut active = match build_native_player(&session, backend, format, NATIVE_TARGET_VOLUME) {
        Ok(p) => p,
        Err(e) => {
            let _ = event_tx.try_send(SpotifyPlayerEvent::Error(format!("native_mixer: {e}")));
            return;
        }
    };

    let mut crossfade_secs: u8 = 0;
    let mut track_duration_ms: Option<u32> = None;
    let mut last_position: Option<(u32, Instant)> = None;
    let mut paused = false;
    let mut near_end_sent = false;
    let mut fading: Option<CrossfadeOut> = None;
    let mut fade_tick = tokio::time::interval(FADE_TICK);
    fade_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            Some(evt) = active.events.recv() => {
                let mapped = match evt {
                    PlayerEvent::Playing { position_ms, .. } => {
                        tracing::debug!("librespot: Playing fired");
                        paused = false;
                        last_position = Some((position_ms, Instant::now()));
                        if let Some(out) = fading.as_mut() {
                            out.fade_started.get_or_insert_with(Instant::now);
                        }
                        Some(SpotifyPlayerEvent::Playing)
                    }
                    PlayerEvent::Paused { position_ms, .. } => {
                        paused = true;
                        last_position = Some((position_ms, Instant::now()));
                        Some(SpotifyPlayerEvent::Paused)
                    }
                    PlayerEvent::PositionChanged { position_ms, .. }
                    | PlayerEvent::PositionCorrection { position_ms, .. }
                    | PlayerEvent::Seeked { position_ms, .. } => {
                        last_position = Some((position_ms, Instant::now()));
                        None
                    }
                    PlayerEvent::Stopped { .. } => {
                        tracing::debug!("librespot: Stopped fired");
                        Some(SpotifyPlayerEvent::Stopped)
                    }
                    PlayerEvent::EndOfTrack { .. } => {
                        tracing::debug!("librespot: EndOfTrack fired");
                        Some(SpotifyPlayerEvent::EndOfTrack)
                    }
                    PlayerEvent::TrackChanged { audio_item } => {
                        track_duration_ms = Some(audio_item.duration_ms);
                        near_end_sent = false;
                        track_from_audio_item(&audio_item).map(SpotifyPlayerEvent::TrackChanged)
                    }
                    PlayerEvent::Unavailable { track_id, .. } => {
                        finish_crossfade_out(&mut fading);
                        active.mixer.set_volume(NATIVE_TARGET_VOLUME);
                        Some(SpotifyPlayerEvent::Error(
                            format!("native_track_unavailable: {track_id}"),
                        ))
                    }
                    _ => None,
                };
                if let Some(e) = mapped {
                    let _ = event_tx.try_send(e);
                }
            }
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(SpotifyPlayerCmd::Play { uris }) => {
                        finish_crossfade_out(&mut fading);
                        active.mixer.set_volume(NATIVE_TARGET_VOLUME);
                        track_duration_ms = None;
                        last_position = None;
                        near_end_sent = false;
                        paused = false;
                        if let Some(uri) = uris.into_iter().next() {
                            match SpotifyUri::from_uri(&uri) {
                                Ok(spotify_uri) => {
                                    active.player.load(spotify_uri, true, 0);
                                }
                                Err(e) => {
                                    let _ = event_tx.try_send(SpotifyPlayerEvent::Error(
                                        format!("native_uri_parse: {e}"),
                                    ));
                                }
                            }
                        }
                    }
                    Some(SpotifyPlayerCmd::Pause) => {
                        finish_crossfade_out(&mut fading);
                        active.mixer.set_volume(NATIVE_TARGET_VOLUME);
                        active.player.pause();
                    }
                    Some(SpotifyPlayerCmd::Resume) => { active.player.play(); }
                    Some(SpotifyPlayerCmd::SetCrossfade { secs }) => {
                        crossfade_secs = secs;
                    }
                    Some(SpotifyPlayerCmd::CrossfadeTo { uri }) => {
                        match SpotifyUri::from_uri(&uri) {
                            Err(e) => {
                                let _ = event_tx.try_send(SpotifyPlayerEvent::Error(
                                    format!("native_uri_parse: {e}"),
                                ));
                            }
                            Ok(spotify_uri) => {
                                finish_crossfade_out(&mut fading);
                                match build_native_player(&session, backend, format, 0) {
                                    Err(e) => {
                                        tracing::warn!(
                                            "librespot: crossfade player failed, hard cut: {e}"
                                        );
                                        active.mixer.set_volume(NATIVE_TARGET_VOLUME);
                                        active.player.load(spotify_uri, true, 0);
                                    }
                                    Ok(incoming) => {
                                        incoming.player.load(spotify_uri, true, 0);
                                        let outgoing = std::mem::replace(&mut active, incoming);
                                        tracing::info!(
                                            "librespot: crossfade started ({crossfade_secs}s)"
                                        );
                                        fading = Some(CrossfadeOut {
                                            player: outgoing.player,
                                            mixer: outgoing.mixer,
                                            requested_at: Instant::now(),
                                            fade_started: None,
                                            duration_secs: f32::from(crossfade_secs.max(1)),
                                        });
                                    }
                                }
                                track_duration_ms = None;
                                last_position = None;
                                near_end_sent = false;
                                paused = false;
                            }
                        }
                    }
                    None => break,
                }
            }
            _ = fade_tick.tick() => {
                let mut fade_done = false;
                if let Some(out) = fading.as_mut() {
                    if out.fade_started.is_none()
                        && out.requested_at.elapsed() >= FADE_START_TIMEOUT
                    {
                        out.fade_started = Some(Instant::now());
                    }
                    if let Some(started) = out.fade_started {
                        let progress =
                            (started.elapsed().as_secs_f32() / out.duration_secs).clamp(0.0, 1.0);
                        let target = f32::from(NATIVE_TARGET_VOLUME);
                        out.mixer.set_volume((target * (1.0 - progress)) as u16);
                        active.mixer.set_volume((target * progress) as u16);
                        fade_done = progress >= 1.0;
                    }
                }
                if fade_done {
                    finish_crossfade_out(&mut fading);
                    active.mixer.set_volume(NATIVE_TARGET_VOLUME);
                }
                if crossfade_secs > 0 && !near_end_sent && !paused && fading.is_none() {
                    if let (Some(duration_ms), Some((pos_ms, at))) =
                        (track_duration_ms, last_position)
                    {
                        let fade_ms = u32::from(crossfade_secs) * 1000;
                        let estimated_ms =
                            pos_ms.saturating_add(at.elapsed().as_millis() as u32);
                        if duration_ms > fade_ms * 2
                            && estimated_ms >= duration_ms.saturating_sub(fade_ms)
                        {
                            near_end_sent = true;
                            tracing::debug!("librespot: TrackNearEnd fired");
                            let _ = event_tx.try_send(SpotifyPlayerEvent::TrackNearEnd);
                        }
                    }
                }
            }
        }
    }
}
