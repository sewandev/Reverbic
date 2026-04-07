
use std::collections::HashSet;
use std::num::NonZero;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    mpsc as std_mpsc, Arc,
};

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use tokio::sync::{mpsc, watch};
use tracing::{error, info, warn};

use crate::audio::meter::rms_to_db;
use crate::audio::stream::StreamReader;
use crate::station::Station;

pub enum PlayerCommand {
    Play(Station),
    Pause,
    Resume,
    Stop,
    SetVolume(f32),
    ApiMetadata {
        title:  String,
        artist: String,
        show:   String,
        recent: Vec<String>,
    },
    PlayPreview { url: String, title: String, raw_track: String },
    StopPreview,
    SetPreviewSearching(bool),
    SetPreviewLoadingTrack(Option<String>),
    MarkPreviewUnavailable(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PlayerStatus {
    #[default]
    Idle,
    Connecting,
    Reconnecting(u32),
    Playing,
    Paused,
    Error(String),
}

#[derive(Clone)]
pub struct PlayerState {
    pub status:        PlayerStatus,
    pub station:       Option<Station>,
    pub title:         Option<String>,
    pub level_db:      f32,
    pub volume:        f32,
    pub recent_titles: Vec<String>,
    pub api_show:      Option<String>,
    pub preview_title: Option<String>,
    pub preview_searching: bool,
    pub preview_loading_track: Option<String>,
    pub preview_playing_track: Option<String>,
    pub preview_unavailable: HashSet<String>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            status:                PlayerStatus::Idle,
            station:               None,
            title:                 None,
            level_db:              -60.0,
            volume:                1.0,
            recent_titles:         Vec::new(),
            api_show:              None,
            preview_title:         None,
            preview_searching:     false,
            preview_loading_track: None,
            preview_playing_track: None,
            preview_unavailable:   HashSet::new(),
        }
    }
}

struct MeterSource<S> {
    inner:      S,
    level:      Arc<AtomicU32>,
    batch:      Vec<f32>,
    batch_size: usize,
}

impl<S: Source<Item = f32>> MeterSource<S> {
    fn new(inner: S, level: Arc<AtomicU32>) -> Self {
        Self {
            inner,
            level,
            batch: Vec::with_capacity(4096),
            batch_size: 4096,
        }
    }
}

impl<S: Source<Item = f32>> Iterator for MeterSource<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        self.batch.push(sample);

        if self.batch.len() >= self.batch_size {
            let db = rms_to_db(&self.batch);
            self.level.store(db.to_bits(), Ordering::Release);
            self.batch.clear();
        }
        Some(sample)
    }
}

impl<S: Source<Item = f32>> Source for MeterSource<S> {
    fn current_span_len(&self) -> Option<usize> { self.inner.current_span_len() }
    fn channels(&self) -> NonZero<u16>           { self.inner.channels() }
    fn sample_rate(&self) -> NonZero<u32>         { self.inner.sample_rate() }
    fn total_duration(&self) -> Option<std::time::Duration> { self.inner.total_duration() }
}

pub struct AudioPlayer {
    cmd_tx:   mpsc::Sender<PlayerCommand>,
    state_rx: watch::Receiver<PlayerState>,
}

impl AudioPlayer {
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<PlayerCommand>(32);
        let (state_tx, state_rx) = watch::channel(PlayerState::default());
        let handle = tokio::runtime::Handle::current();

        std::thread::spawn(move || {
            audio_loop(cmd_rx, state_tx, handle);
        });

        Self { cmd_tx, state_rx }
    }
    pub async fn send(&self, cmd: PlayerCommand) -> bool {
        if self.cmd_tx.send(cmd).await.is_err() {
            error!("AudioPlayer: canal cerrado — el audio thread puede haber fallado");
            false
        } else {
            true
        }
    }

    pub fn state(&self) -> PlayerState {
        self.state_rx.borrow().clone()
    }
    pub fn clone_sender(&self) -> mpsc::Sender<PlayerCommand> {
        self.cmd_tx.clone()
    }
}

fn audio_loop(
    mut cmd_rx: mpsc::Receiver<PlayerCommand>,
    state_tx: watch::Sender<PlayerState>,
    handle: tokio::runtime::Handle,
) {
    let device_sink: MixerDeviceSink = match DeviceSinkBuilder::open_default_sink() {
        Ok(s) => s,
        Err(e) => {
            error!("No se pudo abrir dispositivo de audio: {e}");
            let _ = state_tx.send(PlayerState {
                status: PlayerStatus::Error(format!("Audio device: {e}")),
                ..Default::default()
            });
            return;
        }
    };

    let level = Arc::new(AtomicU32::new((-60.0f32).to_bits()));
    let mut player: Option<Player> = None;
    let mut preview_player: Option<Player> = None;
    let mut current_volume: f32 = 1.0;
    let mut volume_before_duck: Option<f32> = None;
    let mut title_rx: Option<std_mpsc::Receiver<String>> = None;
    let mut api_last_success: Option<std::time::Instant> = None;
    // true cuando la API ya proveyó historial para la estación actual.
    // Se resetea al cambiar de estación. Controla si ICY actúa como fallback.
    let mut api_has_recent: bool = false;
    let mut current_station: Option<Station> = None;
    let mut reconnect_at: Option<std::time::Instant> = None;
    let mut stream_retry_at: Option<(u32, std::time::Instant)> = None;
    const MAX_STREAM_RETRIES: u32 = 5;
    const BASE_RETRY_DELAY_SECS: u64 = 3;
    const MAX_RETRY_DELAY_SECS: u64 = 30;

    loop {
        if let Some(ref rx) = title_rx {
            let api_fresh = api_last_success
                .map(|t| t.elapsed().as_secs() < 60)
                .unwrap_or(false);

            loop {
                match rx.try_recv() {
                    Ok(title) => {
                        // Usar ICY si:
                        // - la API no está fresca (o no hay API), O
                        // - la API está fresca pero nunca proveyó historial
                        let use_icy = !api_fresh || !api_has_recent;
                        if use_icy {
                            let mut state = state_tx.borrow().clone();
                            if state.recent_titles.first().map(String::as_str) != Some(title.as_str()) {
                                state.recent_titles.insert(0, title.clone());
                                state.recent_titles.truncate(10);
                            }
                            state.title = Some(title);
                            let _ = state_tx.send(state);
                        }
                    }
                    Err(std_mpsc::TryRecvError::Empty) => break,
                    Err(std_mpsc::TryRecvError::Disconnected) => {
                        title_rx = None;
                        if current_station.is_some() && reconnect_at.is_none() {
                            warn!("Stream terminado inesperadamente — reconectando en 3s");
                            reconnect_at = Some(
                                std::time::Instant::now()
                                    + std::time::Duration::from_secs(3),
                            );
                        }
                        break;
                    }
                }
            }
        }
        let db = f32::from_bits(level.load(Ordering::Acquire));
        {
            let mut state = state_tx.borrow().clone();
            if (state.level_db - db).abs() > 0.5 {
                state.level_db = db;
                let _ = state_tx.send(state);
            }
        }
        let cmd = if reconnect_at.map(|t| std::time::Instant::now() >= t).unwrap_or(false) {
            reconnect_at = None;
            current_station.clone().map(PlayerCommand::Play)
        } else if stream_retry_at.map(|(_, t)| std::time::Instant::now() >= t).unwrap_or(false) {
            current_station.clone().map(PlayerCommand::Play)
        } else {
            let result = handle.block_on(async {
                tokio::time::timeout(
                    std::time::Duration::from_millis(50),
                    cmd_rx.recv(),
                )
                .await
            });
            match result {
                Ok(Some(c)) => Some(c),
                Ok(None)    => break,
                Err(_)      => None,
            }
        };

        let cmd = match cmd {
            Some(c) => c,
            None    => continue,
        };

        match cmd {
            PlayerCommand::ApiMetadata { title, artist, show, recent } => {
                api_last_success = Some(std::time::Instant::now());
                let mut state = state_tx.borrow().clone();
                state.title = Some(if artist.is_empty() {
                    title
                } else {
                    format!("{artist} - {title}")
                });
                state.api_show = Some(show);
                if !recent.is_empty() {
                    state.recent_titles = recent;
                    api_has_recent = true;
                }
                let _ = state_tx.send(state);
            }

            PlayerCommand::Play(station) => {
                current_station    = Some(station.clone());
                reconnect_at       = None;
                stream_retry_at    = None;
                api_last_success   = None;
                api_has_recent     = false;
                volume_before_duck = None;
                if let Some(p) = player.take() {
                    p.stop();
                }

                info!("Conectando a: {}", station.name);
                let _ = state_tx.send(PlayerState {
                    status:  PlayerStatus::Connecting,
                    station: Some(station.clone()),
                    volume:  current_volume,
                    ..Default::default()
                });

                let url = station.url.to_string();
                let (stream_reader, new_title_rx) = StreamReader::connect(url, handle.clone());
                title_rx = Some(new_title_rx);

                let reader = std::io::BufReader::new(stream_reader);

                match Decoder::try_from(reader) {
                    Ok(decoder) => {
                        stream_retry_at = None;
                        let metered = MeterSource::new(decoder, Arc::clone(&level));
                        let new_player = Player::connect_new(&device_sink.mixer());
                        new_player.set_volume(current_volume);
                        new_player.append(metered);
                        new_player.play();
                        player = Some(new_player);

                        let _ = state_tx.send(PlayerState {
                            status:                PlayerStatus::Playing,
                            station:               Some(station.clone()),
                            level_db:              -60.0,
                            volume:                current_volume,
                            title:                 None,
                            recent_titles:         Vec::new(),
                            api_show:              None,
                            preview_title:         None,
                            preview_searching:     false,
                            preview_loading_track: None,
                            preview_playing_track: None,
                            preview_unavailable:   HashSet::new(),
                        });
                        info!("Reproduciendo: {}", station.name);
                    }
                    Err(e) => {
                        let retry_count = stream_retry_at.take().map(|(count, _)| count + 1).unwrap_or(1);
                        title_rx = None;
                        if retry_count <= MAX_STREAM_RETRIES {
                            let delay_secs = (BASE_RETRY_DELAY_SECS * (2_u64.pow(retry_count - 1)))
                                .min(MAX_RETRY_DELAY_SECS);
                            warn!(
                                "Error al decodificar stream (intento {}/{}): {e}. Reintentando en {delay_secs}s",
                                retry_count, MAX_STREAM_RETRIES
                            );
                            stream_retry_at = Some((
                                retry_count,
                                std::time::Instant::now() + std::time::Duration::from_secs(delay_secs),
                            ));
                            let _ = state_tx.send(PlayerState {
                                status:  PlayerStatus::Reconnecting(retry_count),
                                station: Some(station),
                                volume:  current_volume,
                                ..Default::default()
                            });
                        } else {
                            error!("Stream falló después de {} intentos: {e}", retry_count);
                            let _ = state_tx.send(PlayerState {
                                status:  PlayerStatus::Error(format!("Stream: {} intentos fallidos", retry_count)),
                                station: Some(station),
                                volume:  current_volume,
                                ..Default::default()
                            });
                        }
                    }
                }
            }

            PlayerCommand::Pause => {
                if let Some(ref p) = player {
                    p.pause();
                    let mut state = state_tx.borrow().clone();
                    state.status = PlayerStatus::Paused;
                    let _ = state_tx.send(state);
                }
            }

            PlayerCommand::Resume => {
                if let Some(ref p) = player {
                    p.play();
                    let mut state = state_tx.borrow().clone();
                    state.status = PlayerStatus::Playing;
                    let _ = state_tx.send(state);
                }
            }

            PlayerCommand::SetVolume(v) => {
                current_volume = v.clamp(0.0, 1.0);
                if volume_before_duck.is_some() {
                    volume_before_duck = Some(current_volume);
                } else {
                    if let Some(ref p) = player {
                        p.set_volume(current_volume);
                    }
                }
                let mut state = state_tx.borrow().clone();
                state.volume = current_volume;
                let _ = state_tx.send(state);
            }

            PlayerCommand::SetPreviewSearching(searching) => {
                let mut state = state_tx.borrow().clone();
                state.preview_searching = searching;
                let _ = state_tx.send(state);
            }

            PlayerCommand::PlayPreview { url, title, raw_track } => {
                if let Some(p) = preview_player.take() { p.stop(); }
                let preview_reader = StreamReader::connect_preview(url, handle.clone());
                let reader = std::io::BufReader::new(preview_reader);
                match Decoder::try_from(reader) {
                    Ok(decoder) => {
                        volume_before_duck = Some(current_volume);
                        if let Some(ref p) = player {
                            p.set_volume(0.05);
                        }

                        let p = Player::connect_new(&device_sink.mixer());
                        p.set_volume(current_volume);
                        p.append(decoder);
                        p.play();
                        preview_player = Some(p);
                        let mut state = state_tx.borrow().clone();
                        state.preview_title         = Some(title);
                        state.preview_searching     = false;
                        state.preview_playing_track = Some(raw_track);
                        let _ = state_tx.send(state);
                        info!("Preview streaming iniciado (volumen radio → 5%)");
                    }
                    Err(e) => {
                        error!("Error iniciando preview streaming: {e}");
                        let mut state = state_tx.borrow().clone();
                        state.preview_searching     = false;
                        state.preview_playing_track = None;
                        let _ = state_tx.send(state);
                    }
                }
            }

            PlayerCommand::StopPreview => {
                if let Some(p) = preview_player.take() { p.stop(); }
                if let Some(pre_duck) = volume_before_duck.take() {
                    if let Some(ref p) = player {
                        p.set_volume(pre_duck);
                    }
                    info!("Preview detenido (volumen radio restaurado → {:.0}%)", pre_duck * 100.0);
                }
                let mut state = state_tx.borrow().clone();
                state.preview_title         = None;
                state.preview_searching     = false;
                state.preview_loading_track = None;
                state.preview_playing_track = None;
                let _ = state_tx.send(state);
            }

            PlayerCommand::SetPreviewLoadingTrack(track) => {
                let mut state = state_tx.borrow().clone();
                state.preview_loading_track = track;
                let _ = state_tx.send(state);
            }

            PlayerCommand::MarkPreviewUnavailable(track) => {
                let mut state = state_tx.borrow().clone();
                state.preview_unavailable.insert(track);
                let _ = state_tx.send(state);
            }

            PlayerCommand::Stop => {
                if let Some(p) = player.take() {
                    p.stop();
                }
                if let Some(p) = preview_player.take() { p.stop(); }
                title_rx            = None;
                api_last_success    = None;
                current_station     = None;
                reconnect_at        = None;
                stream_retry_at     = None;
                volume_before_duck = None;
                level.store((-60.0f32).to_bits(), Ordering::Release);
                let _ = state_tx.send(PlayerState {
                    volume: current_volume,
                    ..Default::default()
                });
                info!("Reproducción detenida");
            }
        }
    }
}
