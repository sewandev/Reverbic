
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

// ---------------------------------------------------------------------------
// Tipos públicos
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// MeterSource — intercepta muestras para calcular el nivel RMS
// ---------------------------------------------------------------------------

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
            // Release: garantiza que el valor es visible antes de cualquier
            // lectura posterior en audio_loop (que usa Acquire).
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

// ---------------------------------------------------------------------------
// AudioPlayer — handle público para enviar comandos y leer estado
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Audio loop — OS thread dedicado
// ---------------------------------------------------------------------------

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
    // Volumen guardado antes del ducking. `Some` mientras hay preview activo.
    let mut volume_before_duck: Option<f32> = None;
    let mut title_rx: Option<std_mpsc::Receiver<String>> = None;
    // Última vez que la API oficial respondió correctamente.
    // Si han pasado >60s sin respuesta, ICY retoma el control.
    let mut api_last_success: Option<std::time::Instant> = None;
    // Estación activa — necesaria para auto-reconexión.
    let mut current_station: Option<Station> = None;
    // Momento en que se debe intentar la reconexión (None = sin reconexión pendiente).
    let mut reconnect_at: Option<std::time::Instant> = None;

    loop {
        // Verificar si llegó un nuevo título ICY; detectar fin del stream (Disconnected)
        if let Some(ref rx) = title_rx {
            let api_fresh = api_last_success
                .map(|t| t.elapsed().as_secs() < 60)
                .unwrap_or(false);

            loop {
                match rx.try_recv() {
                    Ok(title) => {
                        if !api_fresh {
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
                        // El task de descarga terminó (error o EOF): programar reconexión
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

        // Actualizar nivel de audio
        let db = f32::from_bits(level.load(Ordering::Acquire));
        {
            let mut state = state_tx.borrow().clone();
            if (state.level_db - db).abs() > 0.5 {
                state.level_db = db;
                let _ = state_tx.send(state);
            }
        }

        // Si hay una reconexión pendiente y ya venció el timer, inyectarla como Play
        let cmd = if reconnect_at.map(|t| std::time::Instant::now() >= t).unwrap_or(false) {
            reconnect_at = None;
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
                Ok(None)    => break,   // canal cerrado
                Err(_)      => None,    // timeout
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
                }
                let _ = state_tx.send(state);
            }

            PlayerCommand::Play(station) => {
                current_station    = Some(station.clone());
                reconnect_at       = None;
                api_last_success   = None;
                volume_before_duck = None; // nueva estación: duck ya no aplica
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
                        error!("Error al decodificar stream: {e}");
                        title_rx = None;
                        let _ = state_tx.send(PlayerState {
                            status:  PlayerStatus::Error(format!("Decoder: {e}")),
                            station: Some(station),
                            volume:  current_volume,
                            ..Default::default()
                        });
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
                    // Durante el preview: actualizar el volumen target (el que se restaurará)
                    // pero mantener la radio duckeada al 5%
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
                // Detener preview anterior si lo hubiera
                if let Some(p) = preview_player.take() { p.stop(); }

                // Streaming: descarga en paralelo mientras rodio reproduce.
                // connect_preview stripea el ID3v2 al vuelo y retorna EOF al terminar.
                let preview_reader = StreamReader::connect_preview(url, handle.clone());
                let reader = std::io::BufReader::new(preview_reader);
                match Decoder::try_from(reader) {
                    Ok(decoder) => {
                        // Duck: bajar el stream principal al 5% mientras suena el preview
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
                // Restaurar volumen original de la radio
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
                title_rx           = None;
                api_last_success   = None;
                current_station    = None;
                reconnect_at       = None;
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
