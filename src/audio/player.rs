use std::collections::HashSet;
use std::num::NonZero;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
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
    PlayWithDuration {
        station: Station,
        duration_secs: f32,
    },
    CrossfadeTo {
        station: Station,
        secs: u8,
    },
    Pause,
    Resume,
    Stop,
    SetVolume(f32),
    Seek(f32),
    ApiMetadata {
        title: String,
        artist: String,
        show: String,
        recent: Vec<String>,
    },
    PlayPreview {
        url: String,
        title: String,
        raw_track: String,
        start_at_secs: f32,
    },
    StopPreview,
    SetPreviewSearching(bool),
    SetPreviewLoadingTrack(Option<String>),
    MarkPreviewUnavailable(String),
    SetPrebuffer(f32),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PlayerStatus {
    #[default]
    Idle,
    Connecting,
    Buffering(f32),
    Reconnecting(u32),
    Playing,
    Paused,
    Error(String),
}

#[derive(Clone)]
pub struct PlayerState {
    pub status: PlayerStatus,
    pub station: Option<Station>,
    pub title: Option<String>,
    pub level_db: f32,
    pub volume: f32,
    pub recent_titles: Vec<String>,
    pub api_show: Option<String>,
    pub preview_title: Option<String>,
    pub preview_searching: bool,
    pub preview_loading_track: Option<String>,
    pub preview_playing_track: Option<String>,
    pub preview_unavailable: HashSet<String>,
    pub playback_pos_secs: Option<f32>,
    pub playback_duration_secs: Option<f32>,
    pub is_dead_url: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            status: PlayerStatus::Idle,
            station: None,
            title: None,
            level_db: -60.0,
            volume: 1.0,
            recent_titles: Vec::new(),
            api_show: None,
            preview_title: None,
            preview_searching: false,
            preview_loading_track: None,
            preview_playing_track: None,
            preview_unavailable: HashSet::new(),
            playback_pos_secs: None,
            playback_duration_secs: None,
            is_dead_url: false,
        }
    }
}

struct MeterSource<S> {
    inner: S,
    level: Arc<AtomicU32>,
    batch: Vec<f32>,
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
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }
    fn channels(&self) -> NonZero<u16> {
        self.inner.channels()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }
}

struct OnDemandTracker {
    active: bool,
    play_start: Option<std::time::Instant>,
    seek_base_secs: f32,
    pause_elapsed: Option<f32>,
}

impl OnDemandTracker {
    fn inactive() -> Self {
        Self {
            active: false,
            play_start: None,
            seek_base_secs: 0.0,
            pause_elapsed: None,
        }
    }

    fn current_pos(&self) -> f32 {
        self.pause_elapsed.unwrap_or_else(|| {
            self.play_start
                .map(|s| self.seek_base_secs + s.elapsed().as_secs_f32())
                .unwrap_or(self.seek_base_secs)
        })
    }

    fn start_playback(&mut self, is_on_demand: bool) {
        self.active = is_on_demand;
        if is_on_demand {
            self.play_start = Some(std::time::Instant::now());
            self.seek_base_secs = 0.0;
            self.pause_elapsed = None;
        }
    }

    fn on_pause(&mut self) {
        if self.active {
            if let Some(start) = self.play_start.take() {
                self.pause_elapsed = Some(self.seek_base_secs + start.elapsed().as_secs_f32());
            }
        }
    }

    fn on_resume(&mut self) {
        if self.active {
            if let Some(elapsed) = self.pause_elapsed.take() {
                self.seek_base_secs = elapsed;
                self.play_start = Some(std::time::Instant::now());
            }
        }
    }

    fn on_seek(&mut self, target_secs: f32) {
        self.seek_base_secs = target_secs;
        self.play_start = Some(std::time::Instant::now());
        self.pause_elapsed = None;
    }

    fn reset(&mut self) {
        *self = Self::inactive();
    }
}

pub struct AudioPlayer {
    cmd_tx: mpsc::Sender<PlayerCommand>,
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
            error!("AudioPlayer: channel closed; the audio thread may have failed");
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

    #[cfg(target_os = "windows")]
    pub fn subscribe(&self) -> watch::Receiver<PlayerState> {
        self.state_rx.clone()
    }
}

fn process_icy_titles(
    st: &mut AudioLoopState,
    state_tx: &watch::Sender<PlayerState>,
    api_fresh: bool,
    download_done: bool,
) {
    let rx = match st.title_rx {
        Some(ref mut r) => r,
        None => return,
    };
    loop {
        match rx.try_recv() {
            Ok(title) => {
                let use_icy = !api_fresh || !st.api_has_recent;
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
                st.title_rx = None;
                if st.current_station.is_some() && st.reconnect_at.is_none() {
                    let delay = backoff_duration(st.reconnect_count, 1, 30);
                    if st.od.active {
                        if download_done {
                            info!("On-demand: download complete, playing from buffer");
                        } else {
                            let pos = st.od.current_pos();
                            let duration =
                                state_tx.borrow().playback_duration_secs.unwrap_or(f32::MAX);
                            if pos >= duration * 0.97 {
                                info!("On-demand: end of file, stopping");
                            } else {
                                warn!(
                                    "On-demand: stream cut at {pos:.0}s, reconnecting in {:.1}s (attempt {})",
                                    delay.as_secs_f32(), st.reconnect_count + 1
                                );
                                st.reconnect_at = Some(std::time::Instant::now() + delay);
                                st.reconnect_count += 1;
                            }
                        }
                    } else {
                        warn!(
                            "Stream ended unexpectedly; reconnecting in {:.1}s (attempt {})",
                            delay.as_secs_f32(),
                            st.reconnect_count + 1
                        );
                        st.reconnect_at = Some(std::time::Instant::now() + delay);
                        st.reconnect_count += 1;
                    }
                }
                break;
            }
        }
    }
}

fn update_level_and_position(state_tx: &watch::Sender<PlayerState>, st: &AudioLoopState) {
    let db = f32::from_bits(st.level.load(Ordering::Acquire));
    let mut state = state_tx.borrow().clone();
    let mut changed = false;
    if (state.level_db - db).abs() > 0.5 {
        state.level_db = db;
        changed = true;
    }
    if st.od.active {
        let pos = st.od.current_pos();
        if state
            .playback_pos_secs
            .map(|p| (p - pos).abs() > 0.5)
            .unwrap_or(true)
        {
            state.playback_pos_secs = Some(pos);
            changed = true;
        }
    }
    if changed {
        let _ = state_tx.send(state);
    }
}

fn backoff_duration(attempt: u32, base_secs: u64, max_secs: u64) -> std::time::Duration {
    let exp = (base_secs * (1u64 << attempt.min(6))).min(max_secs);
    let jitter_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 % 500)
        .unwrap_or(0);
    std::time::Duration::from_millis(exp * 1000 + jitter_ms)
}

struct CrossfadeOut {
    player: Player,
    from_vol: f32,
    start: std::time::Instant,
    duration_secs: f32,
}

struct AudioLoopState {
    level: Arc<AtomicU32>,
    player: Option<Player>,
    preview_player: Option<Player>,
    current_volume: f32,
    volume_before_duck: Option<f32>,
    title_rx: Option<std_mpsc::Receiver<String>>,
    api_last_success: Option<std::time::Instant>,
    api_has_recent: bool,
    current_station: Option<Station>,
    reconnect_at: Option<std::time::Instant>,
    reconnect_count: u32,
    stream_retry_at: Option<(u32, std::time::Instant)>,
    od: OnDemandTracker,
    stream_last_chunk: Option<Arc<AtomicU64>>,
    stream_download_done: Option<Arc<AtomicBool>>,
    crossfade_out: Option<CrossfadeOut>,
    pre_buffer_secs: f32,
}

impl AudioLoopState {
    fn new() -> Self {
        Self {
            level: Arc::new(AtomicU32::new((-60.0f32).to_bits())),
            player: None,
            preview_player: None,
            current_volume: 1.0,
            volume_before_duck: None,
            title_rx: None,
            api_last_success: None,
            api_has_recent: false,
            current_station: None,
            reconnect_at: None,
            reconnect_count: 0,
            stream_retry_at: None,
            od: OnDemandTracker::inactive(),
            stream_last_chunk: None,
            stream_download_done: None,
            crossfade_out: None,
            pre_buffer_secs: 30.0,
        }
    }
}

struct StreamConnection {
    player: Player,
    duration_secs: Option<f32>,
    title_rx: std_mpsc::Receiver<String>,
    last_chunk: Arc<AtomicU64>,
    download_done: Arc<AtomicBool>,
}

const STALL_SECS_LIVE: u64 = 30;
const STALL_SECS_ON_DEMAND: u64 = 60;
const MAX_STREAM_RETRIES: u32 = 6;
const BASE_RETRY_DELAY_SECS: u64 = 2;
const BASE_RECONNECT_DELAY_SECS: u64 = 1;
const MAX_RETRY_DELAY_SECS: u64 = 30;
const ONDEMAND_BYTES_PER_SEC: f32 = 16_000.0;
const YOUTUBE_PREBUFFER_SECS: f32 = 7.0;

fn on_demand_byte_offset(target_secs: f32, station: &Station) -> u64 {
    let bytes_per_sec = station
        .bitrate_kbps
        .map(|kbps| kbps as f32 * 1_000.0 / 8.0)
        .unwrap_or(ONDEMAND_BYTES_PER_SEC);

    (target_secs * bytes_per_sec) as u64
}

fn is_on_demand_station_key(key: &str) -> bool {
    key.starts_with("ondemand_") || key.starts_with("youtube:")
}

fn prebuffer_secs_for_station_key(key: &str, configured_secs: f32) -> f32 {
    if key.starts_with("youtube:") {
        YOUTUBE_PREBUFFER_SECS
    } else {
        configured_secs
    }
}

#[cfg(test)]
#[test]
fn classifies_on_demand_station_keys() {
    assert!(is_on_demand_station_key("ondemand_123"));
    assert!(is_on_demand_station_key("youtube:abc123"));
    assert!(!is_on_demand_station_key("radio-browser:abc123"));
}

#[cfg(test)]
#[test]
fn uses_shorter_youtube_prebuffer() {
    assert_eq!(prebuffer_secs_for_station_key("youtube:abc123", 30.0), 7.0);
    assert_eq!(prebuffer_secs_for_station_key("ondemand_123", 30.0), 30.0);
}

#[cfg(test)]
#[test]
fn uses_playback_duration_fallback_when_decoder_has_none() {
    assert_eq!(
        playback_duration_or_fallback(None, Some(300.0)),
        Some(300.0)
    );
    assert_eq!(
        playback_duration_or_fallback(Some(120.0), Some(300.0)),
        Some(120.0)
    );
    assert_eq!(playback_duration_or_fallback(None, Some(0.0)), None);
}

fn update_state(tx: &watch::Sender<PlayerState>, f: impl FnOnce(&mut PlayerState)) {
    let mut s = tx.borrow().clone();
    f(&mut s);
    let _ = tx.send(s);
}

fn attach_player<S>(source: S, volume: f32, sink: &MixerDeviceSink) -> Player
where
    S: Source<Item = f32> + Send + 'static,
{
    let p = Player::connect_new(sink.mixer());
    p.set_volume(volume);
    p.append(source);
    p.play();
    p
}

fn playing_state(
    station: Station,
    volume: f32,
    is_on_demand: bool,
    duration_secs: Option<f32>,
) -> PlayerState {
    PlayerState {
        status: PlayerStatus::Playing,
        station: Some(station),
        level_db: -60.0,
        volume,
        playback_pos_secs: if is_on_demand { Some(0.0) } else { None },
        playback_duration_secs: duration_secs,
        ..Default::default()
    }
}

fn playback_duration_or_fallback(
    decoded_duration_secs: Option<f32>,
    fallback_duration_secs: Option<f32>,
) -> Option<f32> {
    decoded_duration_secs
        .filter(|d| *d > 0.0)
        .or_else(|| fallback_duration_secs.filter(|d| *d > 0.0))
}

fn tick_crossfade(st: &mut AudioLoopState) {
    let cf_done = if let Some(ref cf) = st.crossfade_out {
        let progress = (cf.start.elapsed().as_secs_f32() / cf.duration_secs).clamp(0.0, 1.0);
        cf.player.set_volume(cf.from_vol * (1.0 - progress));
        if let Some(ref p) = st.player {
            p.set_volume(st.current_volume * progress);
        }
        progress >= 1.0
    } else {
        false
    };
    if cf_done {
        if let Some(cf) = st.crossfade_out.take() {
            cf.player.stop();
        }
    }
}

fn check_download_done(st: &mut AudioLoopState) -> bool {
    if let Some(ref arc) = st.stream_download_done {
        if arc.load(Ordering::Acquire) {
            st.stream_last_chunk = None;
            st.stream_download_done = None;
            return true;
        }
    }
    false
}

fn check_stream_stall(st: &mut AudioLoopState) {
    if st.reconnect_at.is_some() || st.stream_retry_at.is_some() {
        return;
    }
    let Some(ref arc) = st.stream_last_chunk else {
        return;
    };
    if st.current_station.is_none() {
        return;
    }
    let last_ms = arc.load(Ordering::Acquire);
    if last_ms == 0 {
        return;
    }
    let stall_threshold = if st.od.active {
        STALL_SECS_ON_DEMAND
    } else {
        STALL_SECS_LIVE
    };
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    if now_ms.saturating_sub(last_ms) > stall_threshold * 1000 {
        let delay = backoff_duration(
            st.reconnect_count,
            BASE_RECONNECT_DELAY_SECS,
            MAX_RETRY_DELAY_SECS,
        );
        warn!(
            "Stream had no data for {}s; reconnecting in {:.1}s (attempt {})",
            stall_threshold,
            delay.as_secs_f32(),
            st.reconnect_count + 1
        );
        st.stream_last_chunk = None;
        st.reconnect_at = Some(std::time::Instant::now() + delay);
        st.reconnect_count += 1;
    }
}

fn open_stream(
    station: &Station,
    player_volume: f32,
    handle: &tokio::runtime::Handle,
    device_sink: &MixerDeviceSink,
    state_tx: &watch::Sender<PlayerState>,
    st: &mut AudioLoopState,
) -> Result<StreamConnection, Option<String>> {
    let is_on_demand = is_on_demand_station_key(&station.key);
    let url = station.url.to_string();
    let ch_size = if is_on_demand { 4096 } else { 64 };
    let (mut stream_reader, title_rx) = StreamReader::connect(url, 0, ch_size, handle.clone());
    let last_chunk = stream_reader.last_chunk_arc();
    let download_done = stream_reader.download_done_arc();
    let dead_url_arc = stream_reader.dead_url_arc();

    if is_on_demand {
        let _ = state_tx.send(PlayerState {
            status: PlayerStatus::Buffering(0.0),
            station: Some(station.clone()),
            volume: st.current_volume,
            ..Default::default()
        });
        let prebuffer_secs = prebuffer_secs_for_station_key(&station.key, st.pre_buffer_secs);
        let target = (prebuffer_secs * ONDEMAND_BYTES_PER_SEC) as usize;
        stream_reader.pre_buffer(target, |pct| {
            update_state(state_tx, |s| s.status = PlayerStatus::Buffering(pct));
        });
    }

    let buf_cap = if is_on_demand { 512 * 1024 } else { 8 * 1024 };
    let reader = std::io::BufReader::with_capacity(buf_cap, stream_reader);

    match Decoder::try_from(reader) {
        Ok(decoder) => {
            let duration_secs = decoder
                .total_duration()
                .map(|d| d.as_secs_f32())
                .filter(|&d| d > 0.0);
            let metered = MeterSource::new(decoder, Arc::clone(&st.level));
            let player = attach_player(metered, player_volume, device_sink);
            st.od.start_playback(is_on_demand);
            Ok(StreamConnection {
                player,
                duration_secs,
                title_rx,
                last_chunk,
                download_done,
            })
        }
        Err(e) => {
            if dead_url_arc.load(Ordering::Acquire) {
                let _ = state_tx.send(PlayerState {
                    status: PlayerStatus::Error(crate::i18n::t("status.dead_url")),
                    station: Some(station.clone()),
                    volume: st.current_volume,
                    is_dead_url: true,
                    ..Default::default()
                });
                Err(None)
            } else {
                Err(Some(e.to_string()))
            }
        }
    }
}

fn schedule_retry(
    st: &mut AudioLoopState,
    error_msg: &str,
    station: Station,
    state_tx: &watch::Sender<PlayerState>,
) {
    if let Some(cf) = st.crossfade_out.take() {
        cf.player.stop();
    }
    st.title_rx = None;
    let retry_count = st
        .stream_retry_at
        .take()
        .map(|(count, _)| count + 1)
        .unwrap_or(1);
    if retry_count <= MAX_STREAM_RETRIES {
        let delay = backoff_duration(retry_count - 1, BASE_RETRY_DELAY_SECS, MAX_RETRY_DELAY_SECS);
        warn!(
            "Error decoding stream (attempt {}/{}): {error_msg}. Retrying in {:.1}s",
            retry_count,
            MAX_STREAM_RETRIES,
            delay.as_secs_f32()
        );
        st.stream_retry_at = Some((retry_count, std::time::Instant::now() + delay));
        let _ = state_tx.send(PlayerState {
            status: PlayerStatus::Reconnecting(retry_count),
            station: Some(station),
            volume: st.current_volume,
            ..Default::default()
        });
    } else {
        error!("Stream failed after {} attempts: {error_msg}", retry_count);
        let _ = state_tx.send(PlayerState {
            status: PlayerStatus::Error(format!("Stream: {} failed attempts", retry_count)),
            station: Some(station),
            volume: st.current_volume,
            ..Default::default()
        });
    }
}

fn handle_play_cmd(
    st: &mut AudioLoopState,
    station: Station,
    is_auto_reconnect: bool,
    fallback_duration_secs: Option<f32>,
    handle: &tokio::runtime::Handle,
    device_sink: &MixerDeviceSink,
    state_tx: &watch::Sender<PlayerState>,
) {
    if !is_auto_reconnect {
        st.reconnect_count = 0;
    }
    st.current_station = Some(station.clone());
    st.reconnect_at = None;
    st.stream_retry_at = None;
    st.api_last_success = None;
    st.api_has_recent = false;
    st.volume_before_duck = None;
    st.od.reset();
    if let Some(cf) = st.crossfade_out.take() {
        cf.player.stop();
    }
    if let Some(p) = st.player.take() {
        p.stop();
    }

    info!("Connecting to: {}", station.name);
    let _ = state_tx.send(PlayerState {
        status: PlayerStatus::Connecting,
        station: Some(station.clone()),
        volume: st.current_volume,
        ..Default::default()
    });

    match open_stream(
        &station,
        st.current_volume,
        handle,
        device_sink,
        state_tx,
        st,
    ) {
        Ok(conn) => {
            st.stream_retry_at = None;
            st.title_rx = Some(conn.title_rx);
            st.stream_last_chunk = Some(conn.last_chunk);
            st.stream_download_done = Some(conn.download_done);
            info!(station = %station.name, url = %station.url, on_demand = st.od.active, "Playback started");
            let _ = state_tx.send(playing_state(
                station,
                st.current_volume,
                st.od.active,
                playback_duration_or_fallback(conn.duration_secs, fallback_duration_secs),
            ));
            st.player = Some(conn.player);
        }
        Err(None) => {}
        Err(Some(e)) => schedule_retry(st, &e, station, state_tx),
    }
}

fn handle_crossfade_cmd(
    st: &mut AudioLoopState,
    station: Station,
    secs: u8,
    handle: &tokio::runtime::Handle,
    device_sink: &MixerDeviceSink,
    state_tx: &watch::Sender<PlayerState>,
) {
    st.reconnect_count = 0;
    if let Some(cf) = st.crossfade_out.take() {
        cf.player.stop();
    }
    st.current_station = Some(station.clone());
    st.reconnect_at = None;
    st.stream_retry_at = None;
    st.api_last_success = None;
    st.api_has_recent = false;
    st.volume_before_duck = None;
    st.od.reset();

    let outgoing = st.player.take();

    info!("Crossfade → {}", station.name);
    let _ = state_tx.send(PlayerState {
        status: PlayerStatus::Connecting,
        station: Some(station.clone()),
        volume: st.current_volume,
        ..Default::default()
    });

    match open_stream(&station, 0.0, handle, device_sink, state_tx, st) {
        Ok(conn) => {
            st.stream_retry_at = None;
            st.title_rx = Some(conn.title_rx);
            st.stream_last_chunk = Some(conn.last_chunk);
            st.stream_download_done = Some(conn.download_done);
            if let Some(out) = outgoing {
                st.crossfade_out = Some(CrossfadeOut {
                    player: out,
                    from_vol: st.current_volume,
                    start: std::time::Instant::now(),
                    duration_secs: secs as f32,
                });
            }
            st.player = Some(conn.player);
            info!(station = %station.name, "Crossfade: playback started");
            let _ = state_tx.send(playing_state(
                station,
                st.current_volume,
                st.od.active,
                conn.duration_secs,
            ));
        }
        Err(None) => {
            if let Some(out) = outgoing {
                out.stop();
            }
        }
        Err(Some(e)) => {
            if let Some(out) = outgoing {
                out.stop();
            }
            schedule_retry(st, &e, station, state_tx);
        }
    }
}

struct PreviewSource {
    url: String,
    title: String,
    raw_track: String,
    start_at_secs: f32,
}

fn handle_play_preview(
    st: &mut AudioLoopState,
    source: PreviewSource,
    handle: &tokio::runtime::Handle,
    device_sink: &MixerDeviceSink,
    state_tx: &watch::Sender<PlayerState>,
) {
    if let Some(p) = st.preview_player.take() {
        p.stop();
    }
    let preview_reader = StreamReader::connect_preview(source.url, handle.clone());
    let reader = std::io::BufReader::new(preview_reader);
    match Decoder::try_from(reader) {
        Ok(decoder) => {
            st.volume_before_duck = Some(st.current_volume);
            if let Some(ref p) = st.player {
                p.set_volume(0.05);
            }
            let audio =
                decoder.skip_duration(std::time::Duration::from_secs_f32(source.start_at_secs));
            st.preview_player = Some(attach_player(audio, st.current_volume, device_sink));
            update_state(state_tx, |s| {
                s.preview_title = Some(source.title);
                s.preview_searching = false;
                s.preview_playing_track = Some(source.raw_track);
            });
            info!("Preview streaming iniciado (volumen radio → 5%)");
        }
        Err(e) => {
            error!("Error iniciando preview streaming: {e}");
            update_state(state_tx, |s| {
                s.preview_searching = false;
                s.preview_playing_track = None;
            });
        }
    }
}
fn check_preview_ended(st: &mut AudioLoopState, state_tx: &watch::Sender<PlayerState>) {
    if !st.preview_player.as_ref().is_some_and(|p| p.empty()) {
        return;
    }
    st.preview_player = None;
    if let Some(pre_duck) = st.volume_before_duck.take() {
        if let Some(ref p) = st.player {
            p.set_volume(pre_duck);
        }
    }
    update_state(state_tx, |s| {
        s.preview_title = None;
        s.preview_searching = false;
        s.preview_loading_track = None;
        s.preview_playing_track = None;
    });
}

fn handle_seek_cmd(
    st: &mut AudioLoopState,
    target_secs: f32,
    handle: &tokio::runtime::Handle,
    device_sink: &MixerDeviceSink,
    state_tx: &watch::Sender<PlayerState>,
) {
    if !st.od.active {
        return;
    }
    let (url, byte_offset) = match st.current_station.as_ref() {
        Some(station) => (
            station.url.clone(),
            on_demand_byte_offset(target_secs, station),
        ),
        None => return,
    };
    info!("Seek a {target_secs:.0}s → byte {byte_offset}");

    if let Some(p) = st.player.take() {
        p.stop();
    }
    update_state(state_tx, |s| {
        s.status = PlayerStatus::Buffering(0.0);
        s.playback_pos_secs = Some(target_secs);
    });

    let (mut stream_reader, new_title_rx) =
        StreamReader::connect(url, byte_offset, 4096, handle.clone());
    st.title_rx = Some(new_title_rx);
    st.stream_last_chunk = Some(stream_reader.last_chunk_arc());
    st.stream_download_done = Some(stream_reader.download_done_arc());
    let prebuffer_secs = st
        .current_station
        .as_ref()
        .map(|station| prebuffer_secs_for_station_key(&station.key, st.pre_buffer_secs))
        .unwrap_or(st.pre_buffer_secs);
    let target = (prebuffer_secs * ONDEMAND_BYTES_PER_SEC) as usize;
    stream_reader.pre_buffer(target, |pct| {
        update_state(state_tx, |s| s.status = PlayerStatus::Buffering(pct));
    });

    let reader = std::io::BufReader::with_capacity(512 * 1024, stream_reader);
    match Decoder::try_from(reader) {
        Ok(decoder) => {
            let metered = MeterSource::new(decoder, Arc::clone(&st.level));
            st.player = Some(attach_player(metered, st.current_volume, device_sink));
            st.od.on_seek(target_secs);
            update_state(state_tx, |s| {
                s.status = PlayerStatus::Playing;
                s.playback_pos_secs = Some(target_secs);
            });
        }
        Err(e) => warn!("Seek: error re-decoding from byte {byte_offset}: {e}"),
    }
}

#[cfg(target_os = "windows")]
fn handle_device_change(
    st: &mut AudioLoopState,
    device_sink: &mut MixerDeviceSink,
    state_tx: &watch::Sender<PlayerState>,
) {
    if let Some(p) = st.player.take() {
        p.stop();
    }
    if let Some(p) = st.preview_player.take() {
        p.stop();
    }
    if let Some(cf) = st.crossfade_out.take() {
        cf.player.stop();
    }
    st.title_rx = None;
    st.stream_last_chunk = None;
    st.stream_download_done = None;
    st.volume_before_duck = None;
    st.od.reset();

    std::thread::sleep(std::time::Duration::from_millis(500));

    for attempt in 0..5u8 {
        match DeviceSinkBuilder::open_default_sink() {
            Ok(new_sink) => {
                *device_sink = new_sink;
                info!("audio: audio device reconnected");
                if st.current_station.is_some() {
                    st.reconnect_at =
                        Some(std::time::Instant::now() + std::time::Duration::from_millis(200));
                    st.reconnect_count = 0;
                    update_state(state_tx, |s| s.status = PlayerStatus::Connecting);
                }
                return;
            }
            Err(e) => {
                if attempt < 4 {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                } else {
                    warn!("audio: failed to reconnect audio device: {e}");
                    update_state(state_tx, |s| {
                        s.status = PlayerStatus::Error(format!("Audio: {e}"));
                    });
                }
            }
        }
    }
}

fn audio_loop(
    mut cmd_rx: mpsc::Receiver<PlayerCommand>,
    state_tx: watch::Sender<PlayerState>,
    handle: tokio::runtime::Handle,
) {
    #[cfg(target_os = "windows")]
    let device_changed = {
        let flag = Arc::new(AtomicBool::new(false));
        crate::audio::device_monitor::spawn_monitor(Arc::clone(&flag));
        flag
    };

    let sink = match DeviceSinkBuilder::open_default_sink() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to open audio device: {e}");
            let _ = state_tx.send(PlayerState {
                status: PlayerStatus::Error(format!("Audio device: {e}")),
                ..Default::default()
            });
            return;
        }
    };
    #[cfg(target_os = "windows")]
    let mut device_sink = sink;
    #[cfg(not(target_os = "windows"))]
    let device_sink = sink;

    let mut st = AudioLoopState::new();

    loop {
        #[cfg(target_os = "windows")]
        if device_changed.swap(false, Ordering::AcqRel) {
            handle_device_change(&mut st, &mut device_sink, &state_tx);
        }

        tick_crossfade(&mut st);

        let api_fresh = st
            .api_last_success
            .map(|t| t.elapsed().as_secs() < 60)
            .unwrap_or(false);
        let download_done = check_download_done(&mut st);

        process_icy_titles(&mut st, &state_tx, api_fresh, download_done);
        update_level_and_position(&state_tx, &st);
        check_stream_stall(&mut st);
        check_preview_ended(&mut st, &state_tx);

        let mut is_auto_reconnect = false;
        let cmd = if let Ok(user_cmd) = cmd_rx.try_recv() {
            Some(user_cmd)
        } else if st
            .reconnect_at
            .map(|t| std::time::Instant::now() >= t)
            .unwrap_or(false)
        {
            st.reconnect_at = None;
            is_auto_reconnect = true;
            if st.od.active {
                Some(PlayerCommand::Seek(st.od.current_pos()))
            } else {
                st.current_station.clone().map(PlayerCommand::Play)
            }
        } else if st
            .stream_retry_at
            .map(|(_, t)| std::time::Instant::now() >= t)
            .unwrap_or(false)
        {
            is_auto_reconnect = true;
            if st.od.active {
                Some(PlayerCommand::Seek(st.od.current_pos()))
            } else {
                st.current_station.clone().map(PlayerCommand::Play)
            }
        } else {
            let result = handle.block_on(async {
                tokio::time::timeout(std::time::Duration::from_millis(50), cmd_rx.recv()).await
            });
            match result {
                Ok(Some(c)) => Some(c),
                Ok(None) => break,
                Err(_) => None,
            }
        };

        let cmd = match cmd {
            Some(c) => c,
            None => continue,
        };

        match cmd {
            PlayerCommand::Play(station) => {
                handle_play_cmd(
                    &mut st,
                    station,
                    is_auto_reconnect,
                    None,
                    &handle,
                    &device_sink,
                    &state_tx,
                );
            }

            PlayerCommand::PlayWithDuration {
                station,
                duration_secs,
            } => {
                handle_play_cmd(
                    &mut st,
                    station,
                    is_auto_reconnect,
                    Some(duration_secs),
                    &handle,
                    &device_sink,
                    &state_tx,
                );
            }

            PlayerCommand::CrossfadeTo { station, secs } => {
                handle_crossfade_cmd(&mut st, station, secs, &handle, &device_sink, &state_tx);
            }

            PlayerCommand::PlayPreview {
                url,
                title,
                raw_track,
                start_at_secs,
            } => {
                handle_play_preview(
                    &mut st,
                    PreviewSource {
                        url,
                        title,
                        raw_track,
                        start_at_secs,
                    },
                    &handle,
                    &device_sink,
                    &state_tx,
                );
            }

            PlayerCommand::Seek(target_secs) => {
                handle_seek_cmd(&mut st, target_secs, &handle, &device_sink, &state_tx);
            }

            PlayerCommand::ApiMetadata {
                title,
                artist,
                show,
                recent,
            } => {
                st.api_last_success = Some(std::time::Instant::now());
                let has_recent = !recent.is_empty();
                if has_recent {
                    st.api_has_recent = true;
                }
                let title_str = if artist.is_empty() {
                    title
                } else {
                    format!("{artist} - {title}")
                };
                update_state(&state_tx, |s| {
                    s.title = Some(title_str);
                    s.api_show = Some(show);
                    if has_recent {
                        s.recent_titles = recent;
                    }
                });
            }

            PlayerCommand::Pause => {
                if let Some(ref p) = st.player {
                    p.pause();
                    st.od.on_pause();
                    update_state(&state_tx, |s| s.status = PlayerStatus::Paused);
                }
            }

            PlayerCommand::Resume => {
                if let Some(ref p) = st.player {
                    p.play();
                    st.od.on_resume();
                    update_state(&state_tx, |s| s.status = PlayerStatus::Playing);
                }
            }

            PlayerCommand::SetVolume(v) => {
                st.current_volume = v.clamp(0.0, 1.0);
                if st.volume_before_duck.is_none() {
                    if let Some(ref p) = st.player {
                        p.set_volume(st.current_volume);
                    }
                } else {
                    st.volume_before_duck = Some(st.current_volume);
                }
                if let Some(ref p) = st.preview_player {
                    p.set_volume(st.current_volume);
                }
                update_state(&state_tx, |s| s.volume = st.current_volume);
            }

            PlayerCommand::SetPreviewSearching(searching) => {
                update_state(&state_tx, |s| s.preview_searching = searching);
            }

            PlayerCommand::StopPreview => {
                if let Some(p) = st.preview_player.take() {
                    p.stop();
                }
                if let Some(pre_duck) = st.volume_before_duck.take() {
                    if let Some(ref p) = st.player {
                        p.set_volume(pre_duck);
                    }
                    info!(
                        "Preview stopped (radio volume restored -> {:.0}%)",
                        pre_duck * 100.0
                    );
                }
                update_state(&state_tx, |s| {
                    s.preview_title = None;
                    s.preview_searching = false;
                    s.preview_loading_track = None;
                    s.preview_playing_track = None;
                });
            }

            PlayerCommand::SetPreviewLoadingTrack(track) => {
                update_state(&state_tx, |s| s.preview_loading_track = track);
            }

            PlayerCommand::MarkPreviewUnavailable(track) => {
                update_state(&state_tx, |s| {
                    s.preview_unavailable.insert(track);
                });
            }

            PlayerCommand::SetPrebuffer(secs) => {
                st.pre_buffer_secs = secs;
            }

            PlayerCommand::Stop => {
                if let Some(cf) = st.crossfade_out.take() {
                    cf.player.stop();
                }
                if let Some(p) = st.player.take() {
                    p.stop();
                }
                if let Some(p) = st.preview_player.take() {
                    p.stop();
                }
                st.title_rx = None;
                st.api_last_success = None;
                st.current_station = None;
                st.reconnect_at = None;
                st.reconnect_count = 0;
                st.stream_retry_at = None;
                st.stream_last_chunk = None;
                st.stream_download_done = None;
                st.volume_before_duck = None;
                st.od.reset();
                st.level.store((-60.0f32).to_bits(), Ordering::Release);
                let _ = state_tx.send(PlayerState {
                    volume: st.current_volume,
                    ..Default::default()
                });
                info!("Playback stopped");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn station_with_bitrate(bitrate_kbps: Option<u16>) -> Station {
        Station {
            key: "test".into(),
            name: "Test".into(),
            url: "https://example.com/audio.mp3".into(),
            metadata_api_url: None,
            history_api_url: None,
            schedule_url: None,
            show_countdown: false,
            bitrate_kbps,
        }
    }

    #[test]
    fn on_demand_seek_uses_station_bitrate_when_available() {
        let station = station_with_bitrate(Some(256));

        assert_eq!(on_demand_byte_offset(10.0, &station), 320_000);
    }

    #[test]
    fn on_demand_seek_keeps_128_kbps_behavior() {
        let station = station_with_bitrate(Some(128));

        assert_eq!(on_demand_byte_offset(10.0, &station), 160_000);
    }

    #[test]
    fn on_demand_seek_falls_back_to_default_when_bitrate_is_unknown() {
        let station = station_with_bitrate(None);

        assert_eq!(on_demand_byte_offset(10.0, &station), 160_000);
    }
}
