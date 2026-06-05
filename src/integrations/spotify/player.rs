use std::sync::mpsc::SyncSender;
use std::sync::Arc;

use librespot_connect::{ConnectConfig, LoadRequest, LoadRequestOptions, Spirc};
use librespot_core::config::DeviceType;
use librespot_core::{authentication::Credentials, cache::Cache, Session, SessionConfig};
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
    pub fn play(&self, track: SpotifyTrack) {
        let _ = self.cmd_tx.send(SpotifyPlayerCmd::Play(track));
    }

    pub fn pause(&self) {
        let _ = self.cmd_tx.send(SpotifyPlayerCmd::Pause);
    }

    pub fn resume(&self) {
        let _ = self.cmd_tx.send(SpotifyPlayerCmd::Resume);
    }
}

pub fn spawn_player(
    audio_token: String,
    event_tx: SyncSender<SpotifyPlayerEvent>,
) -> SpotifyPlayerHandle {
    let (cmd_tx, cmd_rx) = unbounded_channel::<SpotifyPlayerCmd>();
    tokio::spawn(run_player(audio_token, cmd_rx, event_tx));
    SpotifyPlayerHandle { cmd_tx }
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
    let cache_dir = std::env::var("APPDATA")
        .map(|p| {
            std::path::PathBuf::from(p)
                .join(".reverbic")
                .join("librespot")
        })
        .unwrap_or_else(|_| std::path::PathBuf::from(".reverbic").join("librespot"));
    let cache = Cache::new(
        Some(&cache_dir),
        None::<&std::path::PathBuf>,
        None::<&std::path::PathBuf>,
        None,
    )
    .ok();
    let credentials = match cache.as_ref().and_then(|c| c.credentials()) {
        Some(cached) => {
            tracing::info!("librespot: using cached credentials");
            cached
        }
        None => {
            tracing::info!("librespot: using OAuth token for first login");
            Credentials::with_access_token(&audio_token)
        }
    };

    let session = Session::new(session_config, cache);
    let mixer: Arc<dyn Mixer> = Arc::new(match SoftMixer::open(MixerConfig::default()) {
        Ok(m) => m,
        Err(e) => {
            let _ = event_tx.try_send(SpotifyPlayerEvent::Error(format!("Mixer: {e}")));
            return;
        }
    });

    let backend = match audio_backend::find(None) {
        Some(b) => b,
        None => {
            let _ = event_tx.try_send(SpotifyPlayerEvent::Error(
                "No compatible audio backend was found".to_string(),
            ));
            return;
        }
    };
    let format = AudioFormat::default();
    let volume_fn = mixer.get_soft_volume();

    let player = Player::new(
        PlayerConfig::default(),
        session.clone(),
        volume_fn,
        move || (backend)(None, format),
    );

    let mut librespot_events = player.get_player_event_channel();

    let connect_config = ConnectConfig {
        name: "Reverbic".to_string(),
        device_type: DeviceType::Computer,
        initial_volume: 65535 / 2,
        is_group: false,
        disable_volume: false,
        volume_steps: 64,
    };

    let (spirc, spirc_task) = match Spirc::new(
        connect_config,
        session.clone(),
        credentials,
        player.clone(),
        mixer.clone(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            let _ = event_tx.try_send(SpotifyPlayerEvent::Error(format!("Spirc init: {e}")));
            return;
        }
    };

    tokio::spawn(spirc_task);
    let _ = spirc.activate();

    loop {
        tokio::select! {
            Some(evt) = librespot_events.recv() => {
                use librespot_playback::player::PlayerEvent;
                let mapped = match evt {
                    PlayerEvent::Playing { .. }    => Some(SpotifyPlayerEvent::Playing),
                    PlayerEvent::Paused { .. }     => Some(SpotifyPlayerEvent::Paused),
                    PlayerEvent::Stopped { .. }    => Some(SpotifyPlayerEvent::Stopped),
                    PlayerEvent::EndOfTrack { .. } => Some(SpotifyPlayerEvent::EndOfTrack),
                    PlayerEvent::Unavailable { track_id, .. } => Some(SpotifyPlayerEvent::Error(
                        format!("Pista no disponible: {track_id}"),
                    )),
                    _ => None,
                };
                if let Some(e) = mapped {
                    let _ = event_tx.try_send(e);
                }
            }
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(SpotifyPlayerCmd::Play(track)) => {
                        let request = LoadRequest::from_tracks(
                            vec![track.uri.clone()],
                            LoadRequestOptions { start_playing: true, ..Default::default() },
                        );
                        if let Err(e) = spirc.load(request) {
                            let _ = event_tx.try_send(SpotifyPlayerEvent::Error(
                                format!("Spirc load: {e}"),
                            ));
                        }
                    }
                    Some(SpotifyPlayerCmd::Pause)  => { let _ = spirc.pause(); }
                    Some(SpotifyPlayerCmd::Resume) => { let _ = spirc.play(); }
                    None => break,
                }
            }
        }
    }
}
