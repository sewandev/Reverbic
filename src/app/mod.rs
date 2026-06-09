mod favorites;
mod input;
mod integrations;
mod metadata;
mod modal;
mod on_demand;
mod player_ctrl;
mod search;
mod spotify_state;
mod update_ctrl;
mod youtube;
mod youtube_state;

pub use modal::{
    settings_items, AppFocus, RadioSubTab, SearchMode, SettingItem, SpotifyAuthStatus,
    SpotifyPlayerStatus, SpotifySubTab,
};
use spotify_state::SpotifyPlaybackBackend;
pub use spotify_state::SpotifyState;
pub use youtube_state::{YoutubeState, YoutubeStatus};

use std::collections::HashSet;
use std::time::Instant;

use crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::audio::{AudioPlayer, PlayerState};
use crate::config::{Config, SpotifyPlaybackMode};
use crate::favorites::{self as fav_store, FavoriteStation};
use crate::station::on_demand::OnDemandShow;
use crate::station::{DynamicStation, Station, StationDetails};

pub(super) fn cycle_prev(sel: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        sel.checked_sub(1).unwrap_or(len - 1)
    }
}

pub(super) fn cycle_next(sel: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (sel + 1) % len
    }
}

pub(super) fn handle_filter_list_key(
    key: KeyCode,
    filter: &mut String,
    selected: &mut usize,
    len: usize,
) -> bool {
    match key {
        KeyCode::Esc => {
            if !filter.is_empty() {
                filter.clear();
                *selected = 0;
            } else {
                return true;
            }
        }
        KeyCode::Up => *selected = cycle_prev(*selected, len),
        KeyCode::Down => *selected = cycle_next(*selected, len),
        KeyCode::Backspace => {
            filter.pop();
            *selected = 0;
        }
        KeyCode::Char(c) if !c.is_control() => {
            filter.push(c);
            *selected = 0;
        }
        _ => {}
    }
    false
}

pub(super) fn abort_task(task: &mut Option<tokio::task::JoinHandle<()>>) {
    if let Some(h) = task.take() {
        h.abort();
    }
}

pub(super) fn scroll_by(sel: usize, delta: i32, len: usize) -> usize {
    if delta > 0 {
        (sel + delta as usize).min(len.saturating_sub(1))
    } else {
        sel.saturating_sub((-delta) as usize)
    }
}

enum SpotifyControlTarget {
    Remote { token: String, device_id: String },
    Native,
    None,
}

pub struct App {
    pub stations: Vec<Station>,
    pub favorites: Vec<FavoriteStation>,
    pub selected: usize,
    pub player: AudioPlayer,
    pub should_quit: bool,
    pub replay_onboarding: bool,
    pub focus: AppFocus,
    pub recent_selected: usize,
    pub saved_tracks: Vec<String>,
    pub save_notice: Option<String>,
    pub save_notice_is_dup: bool,
    pub search_query: String,
    pub search_results: Vec<DynamicStation>,
    pub search_loading: bool,
    pub terminal_area: Rect,
    pub on_demand_shows: Vec<OnDemandShow>,
    pub on_demand_selected: usize,
    pub on_demand_loading: bool,
    pub selected_program: usize,
    pub seek_input: String,
    pub settings_selected: usize,
    pub show_search_modal: bool,
    pub modal_mode: SearchMode,
    pub modal_selected: usize,
    pub radio_sub_tab: RadioSubTab,
    pub radio_fav_selected: usize,
    pub genre_selected: usize,
    pub genre_filter: String,
    pub genre_query: String,
    pub country_selected: usize,
    pub country_filter: String,
    pub renaming_favorite: Option<usize>,
    pub rename_input: String,
    pub editing_client_id: bool,
    pub client_id_input: String,
    pub theme_picker_open: bool,
    pub theme_picker_selected: usize,
    pub click_flash: Option<(usize, Instant)>,
    pub last_activity: Instant,
    pub border_tick: u32,
    pub station_details: Option<StationDetails>,
    pub windows_tx: Option<tokio::sync::watch::Sender<crate::config::Config>>,
    pub config: Config,
    pub show_help: bool,
    pub spotify: SpotifyState,
    pub youtube: YoutubeState,
    pub radio_enriched_track: Option<crate::metadata::EnrichedTrack>,
    pub(super) radio_enriched_for: Option<String>,
    pub(super) radio_enrichment_task: Option<tokio::task::JoinHandle<()>>,
    pub(super) radio_enrichment_rx:
        Option<std::sync::mpsc::Receiver<Option<crate::metadata::EnrichedTrack>>>,
    metadata_task: Option<tokio::task::JoinHandle<()>>,
    search_task: Option<tokio::task::JoinHandle<()>>,
    search_result_rx: Option<std::sync::mpsc::Receiver<Vec<DynamicStation>>>,
    on_demand_task: Option<tokio::task::JoinHandle<()>>,
    on_demand_rx: Option<std::sync::mpsc::Receiver<Vec<OnDemandShow>>>,
    station_details_rx: Option<std::sync::mpsc::Receiver<StationDetails>>,
    last_details_uuid: Option<String>,
    pub notice_until: Option<std::time::Instant>,
    pub dead_urls: HashSet<String>,
    pub update_available: Option<String>,
    pub update_path: Option<std::path::PathBuf>,
    update_check_task: Option<tokio::task::JoinHandle<()>>,
    update_check_rx: Option<std::sync::mpsc::Receiver<Option<crate::update::UpdateAsset>>>,
    update_download_task: Option<tokio::task::JoinHandle<()>>,
    update_download_rx: Option<std::sync::mpsc::Receiver<Option<std::path::PathBuf>>>,
    fav_enrich_task: Option<tokio::task::JoinHandle<()>>,
    fav_enrich_rx: Option<std::sync::mpsc::Receiver<Vec<FavoriteStation>>>,
    next_preview_id: u64,
}

impl App {
    pub async fn new() -> Self {
        let config = Config::load();
        let player = AudioPlayer::spawn();
        let initial_vol = if config.restore_volume {
            config.volume
        } else {
            1.0
        };
        player
            .send(crate::audio::PlayerCommand::SetVolume(initial_vol))
            .await;
        player
            .send(crate::audio::PlayerCommand::SetPrebuffer(
                config.prebuffer_secs as f32,
            ))
            .await;

        let favorites = fav_store::load();
        let mut app = Self {
            stations: Vec::new(),
            favorites,
            selected: 0,
            player,
            should_quit: false,
            replay_onboarding: false,
            focus: AppFocus::Stations,
            recent_selected: 0,
            saved_tracks: Vec::new(),
            save_notice: None,
            save_notice_is_dup: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_loading: false,
            terminal_area: Rect::default(),
            on_demand_shows: Vec::new(),
            on_demand_selected: 0,
            on_demand_loading: false,
            selected_program: 0,
            seek_input: String::new(),
            settings_selected: 0,
            show_search_modal: true,
            modal_mode: SearchMode::Name,
            modal_selected: 0,
            radio_sub_tab: RadioSubTab::default(),
            radio_fav_selected: 0,
            genre_selected: 0,
            genre_filter: String::new(),
            genre_query: String::new(),
            country_selected: 0,
            country_filter: String::new(),
            renaming_favorite: None,
            rename_input: String::new(),
            editing_client_id: false,
            client_id_input: String::new(),
            theme_picker_open: false,
            theme_picker_selected: 0,
            click_flash: None,
            last_activity: Instant::now(),
            border_tick: 0,
            station_details: None,
            windows_tx: None,
            config,
            show_help: false,
            spotify: SpotifyState::default(),
            youtube: YoutubeState::default(),
            radio_enriched_track: None,
            radio_enriched_for: None,
            radio_enrichment_task: None,
            radio_enrichment_rx: None,
            metadata_task: None,
            search_task: None,
            search_result_rx: None,
            on_demand_task: None,
            on_demand_rx: None,
            station_details_rx: None,
            last_details_uuid: None,
            notice_until: None,
            dead_urls: HashSet::new(),
            update_available: None,
            update_path: None,
            update_check_task: None,
            update_check_rx: None,
            update_download_task: None,
            update_download_rx: None,
            fav_enrich_task: None,
            fav_enrich_rx: None,
            next_preview_id: 1,
        };
        app.start_favorites_enrichment();
        app
    }

    pub(super) fn next_preview_id(&mut self) -> u64 {
        let preview_id = self.next_preview_id;
        self.next_preview_id = self.next_preview_id.wrapping_add(1).max(1);
        preview_id
    }

    pub fn screensaver_active(&self) -> bool {
        let secs = self.config.screensaver_secs;
        secs > 0 && self.show_search_modal && self.last_activity.elapsed().as_secs() >= secs as u64
    }

    fn spotify_remote_control_target(&self) -> Option<(String, String)> {
        if self.config.spotify.playback_mode == SpotifyPlaybackMode::Native {
            return None;
        }
        Some((
            self.spotify.access_token.clone()?,
            self.spotify.active_device_id.clone()?,
        ))
    }

    fn spotify_native_controls_enabled(&self) -> bool {
        self.config.spotify.playback_mode != SpotifyPlaybackMode::Remote
            && self.spotify.native_available
            && self.spotify.player_tx.is_some()
    }

    fn spotify_control_target(&self) -> SpotifyControlTarget {
        match self.config.spotify.playback_mode {
            SpotifyPlaybackMode::Remote => self
                .spotify_remote_control_target()
                .map(|(token, device_id)| SpotifyControlTarget::Remote { token, device_id })
                .unwrap_or(SpotifyControlTarget::None),
            SpotifyPlaybackMode::Native => {
                if self.spotify_native_controls_enabled() {
                    SpotifyControlTarget::Native
                } else {
                    SpotifyControlTarget::None
                }
            }
            SpotifyPlaybackMode::Auto => match self.spotify.active_backend {
                Some(SpotifyPlaybackBackend::Native) => {
                    if self.spotify_native_controls_enabled() {
                        SpotifyControlTarget::Native
                    } else {
                        SpotifyControlTarget::None
                    }
                }
                Some(SpotifyPlaybackBackend::Remote) | None => self
                    .spotify_remote_control_target()
                    .map(|(token, device_id)| SpotifyControlTarget::Remote { token, device_id })
                    .unwrap_or_else(|| {
                        if self.spotify_native_controls_enabled() {
                            SpotifyControlTarget::Native
                        } else {
                            SpotifyControlTarget::None
                        }
                    }),
            },
        }
    }

    fn spotify_native_status_is_active(&self) -> bool {
        self.spotify.active_backend == Some(SpotifyPlaybackBackend::Native)
            && matches!(
                self.spotify.player_status,
                SpotifyPlayerStatus::Playing
                    | SpotifyPlayerStatus::Paused
                    | SpotifyPlayerStatus::Loading
            )
            && self.spotify_native_controls_enabled()
    }

    pub(super) async fn toggle_spotify_playback(&mut self) {
        match self.spotify_control_target() {
            SpotifyControlTarget::Remote { token, device_id } => match self.spotify.player_status {
                SpotifyPlayerStatus::Playing => {
                    self.spotify.player_status = SpotifyPlayerStatus::Paused;
                    tokio::spawn(async move {
                        if let Err(e) =
                            crate::integrations::spotify::devices::pause_device(&token, &device_id)
                                .await
                        {
                            tracing::warn!("spotify pause error: {e}");
                        }
                    });
                }
                SpotifyPlayerStatus::Paused => {
                    self.spotify.player_status = SpotifyPlayerStatus::Playing;
                    tokio::spawn(async move {
                        if let Err(e) =
                            crate::integrations::spotify::devices::resume_device(&token, &device_id)
                                .await
                        {
                            tracing::warn!("spotify resume error: {e}");
                        }
                    });
                }
                _ => {}
            },
            SpotifyControlTarget::Native => {
                if let Some(handle) = &self.spotify.player_tx {
                    match self.spotify.player_status {
                        SpotifyPlayerStatus::Playing => {
                            handle.pause();
                            self.spotify.player_status = SpotifyPlayerStatus::Paused;
                        }
                        SpotifyPlayerStatus::Paused => {
                            handle.resume();
                            self.spotify.player_status = SpotifyPlayerStatus::Playing;
                        }
                        _ => {}
                    }
                }
            }
            SpotifyControlTarget::None => {}
        }
    }

    pub(super) fn set_spotify_playback_mode(&mut self, mode: SpotifyPlaybackMode) {
        let previous = self.config.spotify.playback_mode;
        if previous == mode {
            return;
        }

        match mode {
            SpotifyPlaybackMode::Native => {
                self.pause_remote_spotify();
                self.stop_playback_polling();
            }
            SpotifyPlaybackMode::Remote => {
                self.pause_native_spotify();
                if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Native) {
                    self.spotify.active_backend = None;
                }
                self.spotify.now_playing = None;
                if !matches!(self.spotify.player_status, SpotifyPlayerStatus::Error(_)) {
                    self.spotify.player_status = SpotifyPlayerStatus::Idle;
                }
                if self.spotify.playback_task.is_none() && self.spotify.access_token.is_some() {
                    self.start_playback_polling();
                }
            }
            SpotifyPlaybackMode::Auto => {
                if self.spotify.playback_task.is_none() && self.spotify.access_token.is_some() {
                    self.start_playback_polling();
                }
            }
        }

        self.config.spotify.playback_mode = mode;
    }

    pub(super) async fn pause_spotify_for_radio(&mut self) {
        match self.config.spotify.playback_mode {
            SpotifyPlaybackMode::Remote => self.pause_remote_spotify(),
            SpotifyPlaybackMode::Native => self.pause_native_spotify(),
            SpotifyPlaybackMode::Auto => {
                self.pause_remote_spotify();
                self.pause_native_spotify();
            }
        }
    }

    fn pause_remote_spotify(&mut self) {
        let Some((token, device_id)) = self.spotify_remote_control_target() else {
            return;
        };
        if matches!(self.spotify.player_status, SpotifyPlayerStatus::Playing) {
            self.spotify.player_status = SpotifyPlayerStatus::Paused;
        }
        tokio::spawn(async move {
            if let Err(e) =
                crate::integrations::spotify::devices::pause_device(&token, &device_id).await
            {
                tracing::warn!("spotify pause error: {e}");
            }
        });
    }

    fn pause_native_spotify(&mut self) {
        if !self.spotify_native_controls_enabled() {
            return;
        }
        if let Some(handle) = &self.spotify.player_tx {
            handle.pause();
        }
        if matches!(self.spotify.player_status, SpotifyPlayerStatus::Playing) {
            self.spotify.player_status = SpotifyPlayerStatus::Paused;
        }
    }

    pub fn active_source_is_spotify(&self) -> bool {
        use crate::audio::PlayerStatus;
        let radio_active = matches!(
            self.player.state().status,
            PlayerStatus::Playing
                | PlayerStatus::Paused
                | PlayerStatus::Buffering(_)
                | PlayerStatus::Reconnecting(_)
        );
        if radio_active {
            return false;
        }
        match self.config.spotify.playback_mode {
            SpotifyPlaybackMode::Remote => self.spotify.playback.is_some(),
            SpotifyPlaybackMode::Native => self.spotify_native_status_is_active(),
            SpotifyPlaybackMode::Auto => {
                self.spotify.playback.is_some() || self.spotify_native_status_is_active()
            }
        }
    }

    pub(super) fn total_stations(&self) -> usize {
        self.favorites.len() + self.stations.len() + self.search_results.len()
    }

    pub(super) fn is_favorite_selected(&self) -> bool {
        self.selected < self.favorites.len()
    }

    pub(super) fn favorite_index(&self) -> Option<usize> {
        if self.is_favorite_selected() {
            Some(self.selected)
        } else {
            None
        }
    }

    pub(super) fn is_hardcoded_selected(&self) -> bool {
        let f = self.favorites.len();
        self.selected >= f && self.selected < f + self.stations.len()
    }

    pub(super) fn hardcoded_index(&self) -> Option<usize> {
        if self.is_hardcoded_selected() {
            Some(self.selected - self.favorites.len())
        } else {
            None
        }
    }

    pub(super) fn is_search_result_selected(&self) -> bool {
        self.selected >= self.favorites.len() + self.stations.len()
    }

    pub(super) fn search_result_index(&self) -> Option<usize> {
        if self.is_search_result_selected() {
            Some(self.selected - self.favorites.len() - self.stations.len())
        } else {
            None
        }
    }

    pub(super) fn build_favorite_from_selected(&self) -> Option<FavoriteStation> {
        if let Some(i) = self.favorite_index() {
            let f = &self.favorites[i];
            Some(f.clone())
        } else if let Some(i) = self.hardcoded_index() {
            let s = &self.stations[i];
            Some(FavoriteStation {
                key: s.key.clone(),
                name: s.name.clone(),
                url: s.url.clone(),
                bitrate_kbps: s.bitrate_kbps,
                country: String::new(),
                tags: Vec::new(),
                homepage: String::new(),
            })
        } else {
            self.search_result_index()
                .and_then(|i| self.search_results.get(i))
                .map(|ds| FavoriteStation {
                    key: ds.key.clone(),
                    name: ds.name.clone(),
                    url: ds.url.clone(),
                    bitrate_kbps: ds.bitrate_kbps,
                    country: ds.country.clone(),
                    tags: ds.tags.clone(),
                    homepage: ds.homepage.clone(),
                })
        }
    }

    pub(super) fn save_config(&mut self) {
        self.config.volume = self.player.state().volume;
        let config = self.config.clone();
        tokio::spawn(async move {
            tokio::task::spawn_blocking(move || config.save())
                .await
                .ok();
        });
    }

    pub fn player_state(&self) -> PlayerState {
        self.player.state()
    }

    pub fn abort_all_tasks(&mut self) {
        abort_task(&mut self.metadata_task);
        abort_task(&mut self.search_task);
        abort_task(&mut self.on_demand_task);
        abort_task(&mut self.radio_enrichment_task);
        self.spotify.cleanup();
        self.youtube.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::spotify::{SpotifyPlaybackState, SpotifyTrack};

    fn test_app() -> App {
        App {
            stations: Vec::new(),
            favorites: Vec::new(),
            selected: 0,
            player: AudioPlayer::spawn(),
            should_quit: false,
            replay_onboarding: false,
            focus: AppFocus::Stations,
            recent_selected: 0,
            saved_tracks: Vec::new(),
            save_notice: None,
            save_notice_is_dup: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_loading: false,
            terminal_area: Rect::default(),
            on_demand_shows: Vec::new(),
            on_demand_selected: 0,
            on_demand_loading: false,
            selected_program: 0,
            seek_input: String::new(),
            settings_selected: 0,
            show_search_modal: true,
            modal_mode: SearchMode::Name,
            modal_selected: 0,
            radio_sub_tab: RadioSubTab::default(),
            radio_fav_selected: 0,
            genre_selected: 0,
            genre_filter: String::new(),
            genre_query: String::new(),
            country_selected: 0,
            country_filter: String::new(),
            renaming_favorite: None,
            rename_input: String::new(),
            editing_client_id: false,
            client_id_input: String::new(),
            theme_picker_open: false,
            theme_picker_selected: 0,
            click_flash: None,
            last_activity: Instant::now(),
            border_tick: 0,
            station_details: None,
            windows_tx: None,
            config: Config::default(),
            show_help: false,
            spotify: SpotifyState::default(),
            youtube: YoutubeState::default(),
            radio_enriched_track: None,
            radio_enriched_for: None,
            radio_enrichment_task: None,
            radio_enrichment_rx: None,
            metadata_task: None,
            search_task: None,
            search_result_rx: None,
            on_demand_task: None,
            on_demand_rx: None,
            station_details_rx: None,
            last_details_uuid: None,
            notice_until: None,
            dead_urls: HashSet::new(),
            update_available: None,
            update_path: None,
            update_check_task: None,
            update_check_rx: None,
            update_download_task: None,
            update_download_rx: None,
            fav_enrich_task: None,
            fav_enrich_rx: None,
            next_preview_id: 1,
        }
    }

    fn spotify_track(name: &str) -> SpotifyTrack {
        SpotifyTrack {
            name: name.to_string(),
            artist: "artist".to_string(),
            album: "album".to_string(),
            duration_ms: 180_000,
            uri: format!("spotify:track:{name}"),
        }
    }

    #[tokio::test]
    async fn switching_spotify_playback_from_remote_to_native_clears_remote_state() {
        let mut app = test_app();
        app.config.spotify.playback_mode = SpotifyPlaybackMode::Remote;
        app.spotify.active_backend = Some(SpotifyPlaybackBackend::Remote);
        app.spotify.playback = Some(SpotifyPlaybackState {
            is_playing: true,
            progress_ms: 1_000,
            duration_ms: 200_000,
            track_name: "Remote track".to_string(),
            artist: "Remote artist".to_string(),
            album: "Remote album".to_string(),
            device_name: "Remote device".to_string(),
            volume_pct: 50,
        });
        app.spotify.playback_rx = Some(std::sync::mpsc::channel().1);
        app.spotify.playback_task = Some(tokio::spawn(async {
            std::future::pending::<()>().await;
        }));

        app.set_spotify_playback_mode(SpotifyPlaybackMode::Native);

        assert_eq!(
            app.config.spotify.playback_mode,
            SpotifyPlaybackMode::Native
        );
        assert_eq!(app.spotify.active_backend, None);
        assert!(app.spotify.playback.is_none());
        assert!(app.spotify.playback_rx.is_none());
        assert!(app.spotify.playback_task.is_none());
    }

    #[tokio::test]
    async fn spotify_logout_clears_persisted_spotify_fields_before_save() {
        let mut app = test_app();
        app.config.spotify.display_name = Some("listener".to_string());
        app.config.spotify.search_token = Some("search-token".to_string());
        app.config.spotify.refresh_token = Some("refresh-token".to_string());
        app.config.spotify.is_premium = Some(true);
        app.config.spotify.country = Some("CL".to_string());
        app.config.spotify.followers = Some(42);

        app.spotify_logout_with_save(|config| {
            assert!(config.spotify.display_name.is_none());
            assert!(config.spotify.search_token.is_none());
            assert!(config.spotify.refresh_token.is_none());
            assert!(config.spotify.is_premium.is_none());
            assert!(config.spotify.country.is_none());
            assert!(config.spotify.followers.is_none());
        });
    }

    #[tokio::test]
    async fn spotify_logout_drops_pending_spotify_receivers() {
        let mut app = test_app();
        app.spotify.auth_rx = Some(std::sync::mpsc::channel().1);
        app.spotify.search_rx = Some(std::sync::mpsc::channel().1);
        app.spotify.search_more_rx = Some(std::sync::mpsc::channel().1);
        app.spotify.token_refresh_rx = Some(std::sync::mpsc::channel().1);

        app.spotify_logout_with_save(|_| {});

        assert!(app.spotify.auth_rx.is_none());
        assert!(app.spotify.search_rx.is_none());
        assert!(app.spotify.search_more_rx.is_none());
        assert!(app.spotify.token_refresh_rx.is_none());
    }

    #[tokio::test]
    async fn spotify_search_clears_pending_load_more_when_query_changes() {
        let mut app = test_app();
        app.spotify.access_token = Some("access-token".to_string());
        app.spotify.search_query = "daft punk".to_string();
        app.spotify.search_offset = 10;
        app.spotify.search_has_more = true;
        app.spotify.search_loading_more = true;
        app.spotify.search_more_rx = Some(std::sync::mpsc::channel().1);

        app.perform_spotify_search();

        assert!(app.spotify.search_more_rx.is_none());
        assert!(!app.spotify.search_loading_more);
        assert!(!app.spotify.search_has_more);
        assert_eq!(app.spotify.search_offset, 0);
    }

    #[tokio::test]
    async fn spotify_search_more_discards_stale_query_results() {
        let mut app = test_app();
        app.spotify.search_generation = 2;
        app.spotify.search_query = "daft punk".to_string();
        app.spotify.search_results = vec![spotify_track("current")];
        app.spotify.search_offset = 0;
        app.spotify.search_has_more = true;
        app.spotify.search_loading_more = true;
        let (tx, rx) = std::sync::mpsc::channel();
        assert!(tx
            .send(spotify_state::SpotifySearchPage {
                generation: 1,
                query: "radiohead".to_string(),
                offset: 10,
                results: vec![spotify_track("stale")],
                has_more: true,
                rate_limit_secs: None,
            })
            .is_ok());
        app.spotify.search_more_rx = Some(rx);

        app.poll_spotify_search_more();

        assert_eq!(app.spotify.search_results.len(), 1);
        assert_eq!(app.spotify.search_results[0].name, "current");
        assert_eq!(app.spotify.search_offset, 0);
        assert!(app.spotify.search_has_more);
        assert!(!app.spotify.search_loading_more);
    }
}
