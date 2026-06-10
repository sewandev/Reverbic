use std::sync::mpsc::SyncSender;
use std::sync::Arc;

use librespot_core::{
    authentication::Credentials, cache::Cache, Session, SessionConfig, SpotifyUri,
};
use librespot_metadata::audio::{AudioItem, UniqueFields};
use librespot_playback::{
    audio_backend,
    config::{AudioFormat, PlayerConfig},
    mixer::{softmixer::SoftMixer, Mixer, MixerConfig},
    player::Player,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::{SpotifyPlayerCmd, SpotifyPlayerEvent, SpotifyTrack};

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

    let mixer: Arc<dyn Mixer> = Arc::new(match SoftMixer::open(MixerConfig::default()) {
        Ok(m) => m,
        Err(e) => {
            let _ = event_tx.try_send(SpotifyPlayerEvent::Error(format!("native_mixer: {e}")));
            return;
        }
    });

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
    let volume_fn = mixer.get_soft_volume();

    let player = Player::new(PlayerConfig::default(), session, volume_fn, move || {
        (backend)(None, format)
    });

    let mut librespot_events = player.get_player_event_channel();

    loop {
        tokio::select! {
            Some(evt) = librespot_events.recv() => {
                use librespot_playback::player::PlayerEvent;
                let mapped = match evt {
                    PlayerEvent::Playing { .. }    => {
                        tracing::debug!("librespot: Playing fired");
                        Some(SpotifyPlayerEvent::Playing)
                    }
                    PlayerEvent::Paused { .. }     => Some(SpotifyPlayerEvent::Paused),
                    PlayerEvent::Stopped { .. }    => {
                        tracing::debug!("librespot: Stopped fired");
                        Some(SpotifyPlayerEvent::Stopped)
                    }
                    PlayerEvent::EndOfTrack { .. } => {
                        tracing::debug!("librespot: EndOfTrack fired");
                        Some(SpotifyPlayerEvent::EndOfTrack)
                    }
                    PlayerEvent::TrackChanged { audio_item } => {
                        track_from_audio_item(&audio_item).map(SpotifyPlayerEvent::TrackChanged)
                    }
                    PlayerEvent::Unavailable { track_id, .. } => Some(SpotifyPlayerEvent::Error(
                        format!("native_track_unavailable: {track_id}"),
                    )),
                    _ => None,
                };
                if let Some(e) = mapped {
                    let _ = event_tx.try_send(e);
                }
            }
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(SpotifyPlayerCmd::Play { uris }) => {
                        if let Some(uri) = uris.into_iter().next() {
                            match SpotifyUri::from_uri(&uri) {
                                Ok(spotify_uri) => {
                                    player.load(spotify_uri, true, 0);
                                }
                                Err(e) => {
                                    let _ = event_tx.try_send(SpotifyPlayerEvent::Error(
                                        format!("native_uri_parse: {e}"),
                                    ));
                                }
                            }
                        }
                    }
                    Some(SpotifyPlayerCmd::Pause)  => { player.pause(); }
                    Some(SpotifyPlayerCmd::Resume) => { player.play(); }
                    None => break,
                }
            }
        }
    }
}
