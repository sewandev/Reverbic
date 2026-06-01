use std::time::Instant;

use crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::audio::{AudioPlayer, PlayerCommand, PlayerState, PlayerStatus};
use crate::config::Config;
use crate::favorites::{self, FavoriteStation};
use crate::library::{self, SaveResult};
use crate::preview::{deezer_preview, parse_seek_input};
use crate::schedule::poll_metadata_loop;
use crate::station::on_demand::OnDemandShow;
use crate::i18n::{self, t};
use crate::station::{enrich, fetch_station_details, fetch_station_details_by_name, is_uuid, find_enrichment, on_demand, search_stations, search_stations_by_tag, search_stations_by_country, filter_items, DynamicStation, Station, StationDetails, GENRES, COUNTRIES};

fn cycle_prev(sel: usize, len: usize) -> usize {
    if len == 0 { 0 } else { sel.checked_sub(1).unwrap_or(len - 1) }
}

fn cycle_next(sel: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (sel + 1) % len }
}

fn abort_task(task: &mut Option<tokio::task::JoinHandle<()>>) {
    if let Some(h) = task.take() { h.abort(); }
}

fn scroll_by(sel: usize, delta: i32, len: usize) -> usize {
    if delta > 0 {
        (sel + delta as usize).min(len.saturating_sub(1))
    } else {
        sel.saturating_sub((-delta) as usize)
    }
}

pub enum SearchMode {
    Name,
    Genre,
    Country,
    Settings,
    Integrations,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SpotifyField {
    Username,
    Password,
}

pub enum SpotifyAuthStatus {
    Idle,
    Connecting,
    LoggedIn,
    Error(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum IntegrationView {
    ServiceList,
    SpotifyDetail,
    SpotifyUserPass,
    SpotifyWebBrowser,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SettingItem {
    Autoplay,
    RestoreVolume,
    Crossfade,
    OverlayMode,
    OverlayAlpha,
    OverlayPosition,
    Screensaver,
    DuckEnabled,
    DuckVolume,
    MediaKeys,
    TrayIcon,
    Notifications,
    Language,
}

impl SettingItem {
    pub(crate) fn label(self) -> String {
        use crate::i18n::t;
        match self {
            Self::Autoplay        => t("config.setting.autoplay"),
            Self::RestoreVolume   => t("config.setting.restore_volume"),
            Self::Crossfade       => t("config.setting.crossfade"),
            Self::OverlayMode     => t("config.setting.overlay"),
            Self::OverlayAlpha    => t("config.setting.overlay_alpha"),
            Self::OverlayPosition => t("config.setting.overlay_position"),
            Self::Screensaver     => t("config.setting.screensaver"),
            Self::DuckEnabled     => t("config.setting.duck"),
            Self::DuckVolume      => t("config.setting.duck_volume"),
            Self::MediaKeys     => t("config.setting.media_keys"),
            Self::TrayIcon      => t("config.setting.tray"),
            Self::Notifications => t("config.setting.notifications"),
            Self::Language      => t("config.setting.language"),
        }
    }

    pub(crate) fn tooltip_key(self) -> &'static str {
        match self {
            Self::Autoplay        => "config.tooltip.autoplay",
            Self::RestoreVolume   => "config.tooltip.restore_volume",
            Self::Crossfade       => "config.tooltip.crossfade",
            Self::OverlayMode     => "config.tooltip.overlay",
            Self::OverlayAlpha    => "config.tooltip.overlay_alpha",
            Self::OverlayPosition => "config.tooltip.overlay_position",
            Self::Screensaver     => "config.tooltip.screensaver",
            Self::DuckEnabled     => "config.tooltip.duck",
            Self::DuckVolume      => "config.tooltip.duck_volume",
            Self::MediaKeys       => "config.tooltip.media_keys",
            Self::TrayIcon        => "config.tooltip.tray",
            Self::Notifications   => "config.tooltip.notifications",
            Self::Language        => "config.tooltip.language",
        }
    }

    pub(crate) fn group_key(self) -> &'static str {
        match self {
            Self::Autoplay | Self::RestoreVolume | Self::Crossfade
                => "config.group.playback",
            Self::OverlayMode | Self::OverlayAlpha | Self::OverlayPosition | Self::Screensaver
                => "config.group.overlay",
            Self::DuckEnabled | Self::DuckVolume
                => "config.group.game",
            Self::MediaKeys | Self::TrayIcon | Self::Notifications
                => "config.group.system",
            Self::Language
                => "config.group.appearance",
        }
    }
}

pub(crate) fn settings_items(duck_enabled: bool) -> Vec<SettingItem> {
    let mut items = vec![
        SettingItem::Autoplay,
        SettingItem::RestoreVolume,
        SettingItem::Crossfade,
        SettingItem::OverlayMode,
        SettingItem::OverlayAlpha,
        SettingItem::OverlayPosition,
        SettingItem::Screensaver,
        SettingItem::DuckEnabled,
    ];
    if duck_enabled {
        items.push(SettingItem::DuckVolume);
    }
    items.extend([
        SettingItem::MediaKeys,
        SettingItem::TrayIcon,
        SettingItem::Notifications,
        SettingItem::Language,
    ]);
    items
}

pub enum AppFocus {
    Stations,
    RecentTracks,
    StationSearch,
    OnDemandList,
}

pub struct App {
    pub stations:            Vec<Station>,
    pub favorites:           Vec<FavoriteStation>,
    pub selected:            usize,
    pub player:              AudioPlayer,
    pub should_quit:         bool,
    pub focus:               AppFocus,
    pub recent_selected:     usize,
    pub saved_tracks:        Vec<String>,
    pub save_notice:         Option<String>,
    pub save_notice_is_dup:  bool,
    pub search_query:        String,
    pub search_results:      Vec<DynamicStation>,
    pub search_loading:      bool,
    pub terminal_area:       Rect,
    pub on_demand_shows:     Vec<OnDemandShow>,
    pub on_demand_selected:  usize,
    pub on_demand_loading:   bool,
    pub selected_program:    usize,
    pub seek_input:          String,
    pub show_settings:       bool,
    pub settings_selected:   usize,
    pub show_search_modal:   bool,
    pub modal_mode:          SearchMode,
    pub modal_selected:      usize,
    pub genre_selected:      usize,
    pub genre_filter:        String,
    pub genre_query:         String,
    pub country_selected:    usize,
    pub country_filter:      String,
    pub renaming_favorite:   Option<usize>,
    pub rename_input:        String,
    pub click_flash:         Option<(usize, Instant)>,
    pub last_activity:       Instant,
    pub station_details:     Option<StationDetails>,
    pub windows_tx:          Option<tokio::sync::watch::Sender<crate::config::Config>>,
    pub config:              Config,
    pub integration_view:       IntegrationView,
    pub integration_selected:   usize,
    pub spotify_auth_selected:  usize,
    pub spotify_username_input: String,
    pub spotify_password_input: String,
    pub spotify_field:          SpotifyField,
    pub spotify_status:         SpotifyAuthStatus,
    metadata_task:           Option<tokio::task::JoinHandle<()>>,
    spotify_auth_task:       Option<tokio::task::JoinHandle<()>>,
    spotify_auth_rx:         Option<std::sync::mpsc::Receiver<crate::integrations::spotify::AuthResult>>,
    search_task:             Option<tokio::task::JoinHandle<()>>,
    search_result_rx:        Option<std::sync::mpsc::Receiver<Vec<DynamicStation>>>,
    on_demand_task:          Option<tokio::task::JoinHandle<()>>,
    on_demand_rx:            Option<std::sync::mpsc::Receiver<Vec<OnDemandShow>>>,
    station_details_rx:      Option<std::sync::mpsc::Receiver<StationDetails>>,
    last_details_uuid:       Option<String>,
}

impl App {
    pub async fn new() -> Self {
        let config = Config::load();
        let player = AudioPlayer::spawn();
        let initial_vol = if config.restore_volume { config.volume } else { 1.0 };
        player.send(PlayerCommand::SetVolume(initial_vol)).await;

        let favorites = favorites::load();
        let spotify_username_input = config.spotify.display_name.clone().unwrap_or_default();
        Self {
            stations:           Vec::new(),
            favorites,
            selected:           0,
            player,
            should_quit:        false,
            focus:              AppFocus::Stations,
            recent_selected:    0,
            saved_tracks:       Vec::new(),
            save_notice:        None,
            save_notice_is_dup: false,
            search_query:       String::new(),
            search_results:     Vec::new(),
            search_loading:     false,
            terminal_area:      Rect::default(),
            on_demand_shows:    Vec::new(),
            on_demand_selected: 0,
            on_demand_loading:  false,
            selected_program:   0,
            seek_input:         String::new(),
            show_settings:      false,
            settings_selected:  0,
            show_search_modal:  true,
            modal_mode:         SearchMode::Name,
            modal_selected:     0,
            genre_selected:     0,
            genre_filter:       String::new(),
            genre_query:        String::new(),
            country_selected:   0,
            country_filter:     String::new(),
            renaming_favorite:  None,
            rename_input:       String::new(),
            click_flash:        None,
            last_activity:      Instant::now(),
            station_details:    None,
            windows_tx:         None,
            config,
            integration_view:       IntegrationView::ServiceList,
            integration_selected:   0,
            spotify_auth_selected:  0,
            spotify_username_input,
            spotify_password_input: String::new(),
            spotify_field:          SpotifyField::Username,
            spotify_status:         SpotifyAuthStatus::Idle,
            metadata_task:      None,
            spotify_auth_task:  None,
            spotify_auth_rx:    None,
            search_task:        None,
            search_result_rx:   None,
            on_demand_task:     None,
            on_demand_rx:       None,
            station_details_rx: None,
            last_details_uuid:  None,
        }
    }

    pub fn screensaver_active(&self) -> bool {
        let secs = self.config.screensaver_secs;
        secs > 0
            && self.show_search_modal
            && self.last_activity.elapsed().as_secs() >= secs as u64
    }

    fn total_stations(&self) -> usize {
        self.favorites.len() + self.stations.len() + self.search_results.len()
    }

    fn is_favorite_selected(&self) -> bool {
        self.selected < self.favorites.len()
    }

    fn favorite_index(&self) -> Option<usize> {
        if self.is_favorite_selected() { Some(self.selected) } else { None }
    }

    fn is_hardcoded_selected(&self) -> bool {
        let f = self.favorites.len();
        self.selected >= f && self.selected < f + self.stations.len()
    }

    fn hardcoded_index(&self) -> Option<usize> {
        if self.is_hardcoded_selected() {
            Some(self.selected - self.favorites.len())
        } else {
            None
        }
    }

    fn is_search_result_selected(&self) -> bool {
        self.selected >= self.favorites.len() + self.stations.len()
    }

    fn search_result_index(&self) -> Option<usize> {
        if self.is_search_result_selected() {
            Some(self.selected - self.favorites.len() - self.stations.len())
        } else {
            None
        }
    }

    fn build_favorite_from_selected(&self) -> Option<FavoriteStation> {
        let make = |key: &str, name: &str, url: &str, bitrate_kbps| FavoriteStation {
            key: key.to_owned(), name: name.to_owned(), url: url.to_owned(), bitrate_kbps,
        };
        if let Some(i) = self.favorite_index() {
            let f = &self.favorites[i];
            Some(make(&f.key, &f.name, &f.url, f.bitrate_kbps))
        } else if let Some(i) = self.hardcoded_index() {
            let s = &self.stations[i];
            Some(make(&s.key, &s.name, &s.url, s.bitrate_kbps))
        } else {
            self.search_result_index()
                .and_then(|i| self.search_results.get(i))
                .map(|ds| make(&ds.key, &ds.name, &ds.url, ds.bitrate_kbps))
        }
    }

    fn toggle_selected_favorite(&mut self) {
        if let Some(fav) = self.build_favorite_from_selected() {
            let added = favorites::toggle(&mut self.favorites, fav);
            favorites::save(&self.favorites);
            let max = self.total_stations().saturating_sub(1);
            self.selected = self.selected.min(max);
            self.save_notice_is_dup = false;
            self.save_notice = Some(if added {
                t("notice.fav_added")
            } else {
                t("notice.fav_removed")
            });
        }
    }

    fn start_metadata_polling(
        &mut self,
        url: &'static str,
        history_url: Option<&'static str>,
        schedule_url: Option<&'static str>,
    ) {
        self.stop_metadata_polling();
        let cmd_tx = self.player.clone_sender();
        self.metadata_task = Some(tokio::spawn(poll_metadata_loop(
            url.to_string(),
            history_url.map(str::to_string),
            schedule_url.map(str::to_string),
            cmd_tx,
        )));
    }

    fn save_config(&mut self) {
        self.config.volume = self.player.state().volume;
        self.config.save();
    }

    pub fn init_integrations(&mut self) {}

    async fn adjust_volume(&mut self, delta: f32) {
        let new_vol = (self.player.state().volume + delta).clamp(0.0, 1.0);
        self.player.send(PlayerCommand::SetVolume(new_vol)).await;
        self.config.volume = new_vol;
        self.config.save();
    }

    fn stop_metadata_polling(&mut self) {
        abort_task(&mut self.metadata_task);
    }

    fn start_on_demand_fetch(&mut self) {
        abort_task(&mut self.on_demand_task);
        self.on_demand_rx = None;
        self.on_demand_loading = true;
        self.on_demand_shows.clear();

        let playlist_id = crate::station::on_demand::PROGRAMS
            .get(self.selected_program)
            .map(|p| p.playlist_id)
            .unwrap_or(crate::station::on_demand::PROGRAMS[0].playlist_id);

        let (tx, rx) = std::sync::mpsc::channel();
        self.on_demand_rx = Some(rx);

        let handle = tokio::spawn(async move {
            let shows = on_demand::fetch_shows_for_playlist(playlist_id)
                .await
                .unwrap_or_default();
            let _ = tx.send(shows);
        });
        self.on_demand_task = Some(handle);
    }

    pub fn poll_on_demand_results(&mut self) {
        if let Some(rx) = self.on_demand_rx.take() {
            match rx.try_recv() {
                Ok(shows) => {
                    self.on_demand_shows = shows;
                    self.on_demand_loading = false;
                    self.on_demand_selected = 0;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.on_demand_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.on_demand_loading = false;
                }
            }
        }
    }

    async fn play_station(&mut self, station: Station) {
        self.config.last_station = Some(crate::config::LastStation {
            key:          station.key.clone(),
            name:         station.name.clone(),
            url:          station.url.clone(),
            bitrate_kbps: station.bitrate_kbps,
        });
        self.save_config();
        self.stop_metadata_polling();

        let fade = self.config.crossfade_secs;
        let is_active = matches!(
            self.player.state().status,
            PlayerStatus::Playing | PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_)
        );

        if fade > 0 && is_active {
            self.player.send(PlayerCommand::CrossfadeTo { station: station.clone(), secs: fade }).await;
        } else {
            self.player.send(PlayerCommand::Play(station.clone())).await;
        }

        self.saved_tracks = library::load_saved_tracks(&station.key);
        if let Some(api_url) = station.metadata_api_url {
            self.start_metadata_polling(api_url, station.history_api_url, station.schedule_url);
        }
        if station.show_countdown {
            self.start_on_demand_fetch();
        } else {
            self.on_demand_shows.clear();
            self.on_demand_loading = false;
        }
    }

    async fn play_favorite_station(&mut self, index: usize) {
        if index >= self.favorites.len() { return; }
        let station = self.favorites[index].to_station();
        self.play_station(station).await;
    }

    async fn play_dynamic_station(&mut self, index: usize) {
        if index >= self.search_results.len() { return; }
        let ds = &self.search_results[index];

        let mut station = Station {
            key:              ds.key.clone(),
            name:             ds.name.clone(),
            url:              ds.url.clone(),
            metadata_api_url: None,
            history_api_url:  None,
            schedule_url:     None,
            show_countdown:   false,
            bitrate_kbps:     ds.bitrate_kbps,
        };

        if let Some(enrichment) = find_enrichment(&station.name) {
            enrich(&mut station, enrichment);
            tracing::info!("Enriquecimiento activado para '{}'", station.name);
        }

        self.play_station(station).await;
    }

    pub async fn on_key_event(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyModifiers;

        if event.modifiers.contains(KeyModifiers::SHIFT)
            && !self.show_search_modal
            && !self.show_settings
            && matches!(self.focus, AppFocus::Stations)
        {
            if let Some(idx) = self.favorite_index() {
                match event.code {
                    KeyCode::Up => {
                        crate::favorites::move_up(&mut self.favorites, idx);
                        crate::favorites::save(&self.favorites);
                        if idx > 0 { self.selected -= 1; }
                        return;
                    }
                    KeyCode::Down => {
                        crate::favorites::move_down(&mut self.favorites, idx);
                        crate::favorites::save(&self.favorites);
                        if idx + 1 < self.favorites.len() { self.selected += 1; }
                        return;
                    }
                    _ => {}
                }
            }
        }

        if self.renaming_favorite.is_some() {
            self.on_key_rename(event.code);
            return;
        }

        self.on_key(event.code).await;
    }

    pub fn on_paste(&mut self, text: String) {
        if self.show_search_modal {
            match self.modal_mode {
                SearchMode::Name => {
                    for c in text.chars().filter(|c| !c.is_control()) {
                        self.search_query.push(c);
                    }
                    self.modal_selected = 0;
                    self.perform_search();
                }
                SearchMode::Genre => {
                    for c in text.chars().filter(|c| !c.is_control()) {
                        self.genre_filter.push(c);
                    }
                    self.genre_selected = 0;
                }
                SearchMode::Country => {
                    for c in text.chars().filter(|c| !c.is_control()) {
                        self.country_filter.push(c);
                    }
                    self.country_selected = 0;
                }
                SearchMode::Settings     => {}
                SearchMode::Integrations => {
                    if matches!(self.integration_view, IntegrationView::SpotifyUserPass)
                        && !matches!(self.spotify_status, SpotifyAuthStatus::Connecting)
                    {
                        match self.spotify_field {
                            SpotifyField::Username => {
                                for c in text.chars().filter(|c| !c.is_control()) {
                                    self.spotify_username_input.push(c);
                                }
                            }
                            SpotifyField::Password => {
                                for c in text.chars().filter(|c| !c.is_control()) {
                                    self.spotify_password_input.push(c);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn on_key_rename(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.renaming_favorite = None;
                self.rename_input.clear();
            }
            KeyCode::Enter => {
                if let Some(idx) = self.renaming_favorite {
                    if !self.rename_input.is_empty() {
                        if let Some(fav) = self.favorites.get_mut(idx) {
                            fav.name = self.rename_input.clone();
                        }
                        crate::favorites::save(&self.favorites);
                    }
                }
                self.renaming_favorite = None;
                self.rename_input.clear();
            }
            KeyCode::Backspace => { self.rename_input.pop(); }
            KeyCode::Char(c) if !c.is_control() => { self.rename_input.push(c); }
            _ => {}
        }
    }

    pub async fn on_key(&mut self, key: KeyCode) {
        if self.screensaver_active() {
            match key {
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    self.adjust_volume(0.05).await;
                    return;
                }
                KeyCode::Char('-') => {
                    self.adjust_volume(-0.05).await;
                    return;
                }
                _ => {}
            }
            self.last_activity = Instant::now();
            if key == KeyCode::Char('o') || key == KeyCode::Char('O') {
                if let Some(ref d) = self.station_details {
                    if !d.homepage.is_empty() {
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd")
                            .args(["/c", "start", "", &d.homepage])
                            .spawn();
                    }
                }
                return;
            }
            if !matches!(key, KeyCode::Enter | KeyCode::Up | KeyCode::Down) {
                return;
            }
        }
        self.last_activity = Instant::now();
        if self.show_search_modal {
            self.on_key_search_modal(key).await;
            return;
        }

        if self.show_settings {
            self.on_key_settings(key);
            return;
        }

        self.save_notice = None;
        match key {
            KeyCode::Char(' ') => {
                match self.player.state().status {
                    PlayerStatus::Playing => { self.player.send(PlayerCommand::Pause).await; }
                    PlayerStatus::Paused  => { self.player.send(PlayerCommand::Resume).await; }
                    _ => {}
                }
                return;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.adjust_volume(0.05).await;
                return;
            }
            KeyCode::Char('-') => {
                self.adjust_volume(-0.05).await;
                return;
            }
            KeyCode::Char('q') => {
                self.config.last_selected = self.selected;
                self.save_config();
                self.stop_metadata_polling();
                self.player.send(PlayerCommand::Stop).await;
                self.should_quit = true;
                return;
            }
            KeyCode::Char('o') if !matches!(self.focus, AppFocus::StationSearch) => {
                self.show_search_modal = true;
                self.modal_mode = SearchMode::Settings;
                self.settings_selected = 0;
                return;
            }
            KeyCode::Tab => {
                let has_recent    = !self.player.state().recent_titles.is_empty();
                let has_on_demand = !self.on_demand_shows.is_empty() || self.on_demand_loading;

                self.focus = match self.focus {
                    AppFocus::Stations | AppFocus::StationSearch => {
                        if has_on_demand {
                            self.on_demand_selected = 0;
                            AppFocus::OnDemandList
                        } else if has_recent {
                            self.recent_selected = 0;
                            AppFocus::RecentTracks
                        } else {
                            AppFocus::Stations
                        }
                    }
                    AppFocus::OnDemandList => {
                        if has_recent {
                            self.recent_selected = 0;
                            AppFocus::RecentTracks
                        } else {
                            AppFocus::Stations
                        }
                    }
                    AppFocus::RecentTracks => AppFocus::Stations,
                };
                return;
            }
            _ => {}
        }

        match self.focus {
            AppFocus::Stations      => self.on_key_stations(key).await,
            AppFocus::RecentTracks  => self.on_key_recent(key).await,
            AppFocus::StationSearch => self.on_key_station_search(key).await,
            AppFocus::OnDemandList  => self.on_key_on_demand(key).await,
        }
    }

    async fn on_key_search_modal(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.adjust_volume(0.05).await;
                return;
            }
            KeyCode::Char('-') => {
                self.adjust_volume(-0.05).await;
                return;
            }
            _ => {}
        }

        if key == KeyCode::Tab {
            self.modal_mode = match self.modal_mode {
                SearchMode::Name         => SearchMode::Genre,
                SearchMode::Genre        => SearchMode::Country,
                SearchMode::Country      => SearchMode::Settings,
                SearchMode::Settings     => SearchMode::Integrations,
                SearchMode::Integrations => SearchMode::Name,
            };
            self.modal_selected = 0;
            self.search_results.clear();
            self.search_query.clear();
            self.genre_filter.clear();
            self.genre_query.clear();
            self.genre_selected = 0;
            self.country_filter.clear();
            self.country_selected = 0;
            abort_task(&mut self.search_task);
            self.search_loading = false;
            self.integration_view = IntegrationView::ServiceList;
            return;
        }

        match self.modal_mode {
            SearchMode::Name         => self.on_key_modal_name(key).await,
            SearchMode::Genre        => self.on_key_modal_genre(key).await,
            SearchMode::Country      => self.on_key_modal_country(key).await,
            SearchMode::Settings     => self.on_key_modal_settings(key),
            SearchMode::Integrations => self.on_key_modal_integrations(key),
        }
    }

    async fn on_key_modal_name(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                if self.search_query.is_empty() && self.search_results.is_empty() {
                    self.should_quit = true;
                } else {
                    self.search_query.clear();
                    self.search_results.clear();
                    self.modal_selected = 0;
                }
            }
            KeyCode::Enter => {
                if !self.search_results.is_empty() {
                    let idx = self.modal_selected.min(self.search_results.len() - 1);
                    if !self.search_query.is_empty() {
                        self.config.add_to_history(self.search_query.clone());
                        self.save_config();
                    }
                    self.play_dynamic_station(idx).await;
                }
            }
            KeyCode::Up => {
                self.modal_selected = cycle_prev(self.modal_selected, self.search_results.len());
            }
            KeyCode::Down => {
                self.modal_selected = cycle_next(self.modal_selected, self.search_results.len());
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.modal_selected = 0;
                self.perform_search();
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.search_query.push(c);
                self.modal_selected = 0;
                self.perform_search();
            }
            _ => {}
        }
    }

    fn play_random_result(&mut self) -> Option<usize> {
        if self.search_results.is_empty() { return None; }
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        Some((ms as usize) % self.search_results.len())
    }

    async fn on_key_modal_results(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.search_results.clear();
                self.genre_query.clear();
                self.modal_selected = 0;
            }
            KeyCode::Enter => {
                let idx = self.modal_selected.min(self.search_results.len() - 1);
                self.play_dynamic_station(idx).await;
            }
            KeyCode::Char('R') => {
                if let Some(idx) = self.play_random_result() {
                    self.play_dynamic_station(idx).await;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.modal_selected = cycle_prev(self.modal_selected, self.search_results.len());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.modal_selected = cycle_next(self.modal_selected, self.search_results.len());
            }
            _ => {}
        }
    }

    async fn on_key_modal_genre(&mut self, key: KeyCode) {
        if !self.search_results.is_empty() {
            self.on_key_modal_results(key).await;
            return;
        }

        let filtered = filter_items(GENRES, &self.genre_filter);
        match key {
            KeyCode::Esc => {
                if !self.genre_filter.is_empty() {
                    self.genre_filter.clear();
                } else {
                    self.modal_mode = SearchMode::Name;
                }
                self.genre_selected = 0;
            }
            KeyCode::Up => {
                self.genre_selected = cycle_prev(self.genre_selected, filtered.len());
            }
            KeyCode::Down => {
                self.genre_selected = cycle_next(self.genre_selected, filtered.len());
            }
            KeyCode::Enter => {
                if let Some(&(tag, label)) = filtered.get(self.genre_selected) {
                    self.genre_query = label.to_string();
                    self.modal_selected = 0;
                    self.perform_genre_search(tag);
                }
            }
            KeyCode::Backspace => {
                self.genre_filter.pop();
                self.genre_selected = 0;
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.genre_filter.push(c);
                self.genre_selected = 0;
            }
            _ => {}
        }
    }

    fn perform_country_search(&mut self, country: &str) {
        let c = country.to_string();
        self.spawn_search(move || async move { search_stations_by_country(&c, 30).await });
    }

    async fn on_key_modal_country(&mut self, key: KeyCode) {
        if !self.search_results.is_empty() {
            self.on_key_modal_results(key).await;
            return;
        }

        let filtered = filter_items(COUNTRIES, &self.country_filter);
        match key {
            KeyCode::Esc => {
                if !self.country_filter.is_empty() {
                    self.country_filter.clear();
                } else {
                    self.modal_mode = SearchMode::Name;
                }
                self.country_selected = 0;
            }
            KeyCode::Up => {
                self.country_selected = cycle_prev(self.country_selected, filtered.len());
            }
            KeyCode::Down => {
                self.country_selected = cycle_next(self.country_selected, filtered.len());
            }
            KeyCode::Enter => {
                if let Some(&(tag, label)) = filtered.get(self.country_selected) {
                    self.genre_query = label.to_string();
                    self.modal_selected = 0;
                    self.perform_country_search(tag);
                }
            }
            KeyCode::Backspace => {
                self.country_filter.pop();
                self.country_selected = 0;
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.country_filter.push(c);
                self.country_selected = 0;
            }
            _ => {}
        }
    }

    fn on_key_modal_settings(&mut self, key: KeyCode) {
        let count = settings_items(self.config.duck_enabled).len();
        match key {
            KeyCode::Esc => {
                self.modal_mode = SearchMode::Name;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.settings_selected = cycle_prev(self.settings_selected, count);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings_selected = cycle_next(self.settings_selected, count);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.apply_settings_toggle(self.settings_selected);
            }
            _ => {}
        }
    }

    fn on_key_modal_integrations(&mut self, key: KeyCode) {
        match self.integration_view {
            IntegrationView::ServiceList       => self.on_key_integration_list(key),
            IntegrationView::SpotifyDetail     => self.on_key_integration_spotify_detail(key),
            IntegrationView::SpotifyUserPass   => self.on_key_integration_spotify_userpass(key),
            IntegrationView::SpotifyWebBrowser => self.on_key_integration_spotify_web(key),
        }
    }

    fn on_key_integration_list(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.modal_mode = SearchMode::Name,
            KeyCode::Up  => self.integration_selected = cycle_prev(self.integration_selected, 1),
            KeyCode::Down => self.integration_selected = cycle_next(self.integration_selected, 1),
            KeyCode::Enter if self.integration_selected == 0 => {
                self.integration_view = IntegrationView::SpotifyDetail;
            }
            _ => {}
        }
    }

    fn on_key_integration_spotify_detail(&mut self, key: KeyCode) {
        if matches!(self.spotify_status, SpotifyAuthStatus::LoggedIn) {
            match key {
                KeyCode::Char('d') | KeyCode::Char('D') => self.spotify_logout(),
                KeyCode::Esc => self.integration_view = IntegrationView::ServiceList,
                _ => {}
            }
            return;
        }

        match key {
            KeyCode::Esc => self.integration_view = IntegrationView::ServiceList,
            KeyCode::Char('d') | KeyCode::Char('D')
                if self.config.spotify.display_name.is_some() =>
            {
                self.spotify_logout();
            }
            KeyCode::Up   => self.spotify_auth_selected = cycle_prev(self.spotify_auth_selected, 2),
            KeyCode::Down => self.spotify_auth_selected = cycle_next(self.spotify_auth_selected, 2),
            KeyCode::Enter => match self.spotify_auth_selected {
                0 => self.integration_view = IntegrationView::SpotifyUserPass,
                _ => self.integration_view = IntegrationView::SpotifyWebBrowser,
            },
            _ => {}
        }
    }

    fn on_key_integration_spotify_userpass(&mut self, key: KeyCode) {
        if matches!(self.spotify_status, SpotifyAuthStatus::Connecting) {
            if key == KeyCode::Esc {
                abort_task(&mut self.spotify_auth_task);
                self.spotify_auth_rx = None;
                self.spotify_status  = SpotifyAuthStatus::Idle;
                self.integration_view = IntegrationView::SpotifyDetail;
            }
            return;
        }

        match key {
            KeyCode::Esc => {
                if matches!(self.spotify_status, SpotifyAuthStatus::Error(_)) {
                    self.spotify_status = SpotifyAuthStatus::Idle;
                }
                self.integration_view = IntegrationView::SpotifyDetail;
            }
            KeyCode::Up | KeyCode::Down => {
                self.spotify_field = match self.spotify_field {
                    SpotifyField::Username => SpotifyField::Password,
                    SpotifyField::Password => SpotifyField::Username,
                };
            }
            KeyCode::Enter => {
                if !self.spotify_username_input.is_empty() && !self.spotify_password_input.is_empty() {
                    self.start_spotify_login();
                }
            }
            KeyCode::Backspace => match self.spotify_field {
                SpotifyField::Username => { self.spotify_username_input.pop(); }
                SpotifyField::Password => { self.spotify_password_input.pop(); }
            },
            KeyCode::Char(c) if !c.is_control() => match self.spotify_field {
                SpotifyField::Username => self.spotify_username_input.push(c),
                SpotifyField::Password => self.spotify_password_input.push(c),
            },
            _ => {}
        }
    }

    fn on_key_integration_spotify_web(&mut self, key: KeyCode) {
        if matches!(self.spotify_status, SpotifyAuthStatus::Connecting) {
            if key == KeyCode::Esc {
                abort_task(&mut self.spotify_auth_task);
                self.spotify_auth_rx = None;
                self.spotify_status  = SpotifyAuthStatus::Idle;
            }
            return;
        }
        match key {
            KeyCode::Enter => self.start_oauth_flow(),
            KeyCode::Esc   => self.integration_view = IntegrationView::SpotifyDetail,
            _ => {}
        }
    }

    fn start_oauth_flow(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify_auth_rx = Some(rx);
        self.spotify_status  = SpotifyAuthStatus::Connecting;
        let handle = tokio::spawn(async move {
            let result = crate::integrations::spotify::oauth::start_flow().await;
            let _ = tx.send(result);
        });
        self.spotify_auth_task = Some(handle);
    }

    fn start_spotify_login(&mut self) {
        let username = self.spotify_username_input.clone();
        let password = self.spotify_password_input.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify_auth_rx = Some(rx);
        self.spotify_status  = SpotifyAuthStatus::Connecting;
        let handle = tokio::spawn(async move {
            let result = crate::integrations::spotify::authenticate(username, password).await;
            let _ = tx.send(result);
        });
        self.spotify_auth_task = Some(handle);
    }

    fn spotify_logout(&mut self) {
        self.config.spotify.display_name = None;
        self.config.save();
        self.spotify_status         = SpotifyAuthStatus::Idle;
        self.spotify_username_input.clear();
        self.spotify_password_input.clear();
        self.spotify_field          = SpotifyField::Username;
    }

    pub fn poll_spotify_auth(&mut self) {
        use crate::integrations::spotify::AuthResult;
        if let Some(rx) = self.spotify_auth_rx.take() {
            match rx.try_recv() {
                Ok(AuthResult::Success { username }) => {
                    self.config.spotify.display_name = Some(username);
                    self.config.save();
                    self.spotify_status   = SpotifyAuthStatus::LoggedIn;
                    self.integration_view = IntegrationView::SpotifyDetail;
                    self.spotify_password_input.clear();
                }
                Ok(AuthResult::Failure(msg)) => {
                    self.spotify_status = SpotifyAuthStatus::Error(msg);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.spotify_auth_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.spotify_status = SpotifyAuthStatus::Idle;
                }
            }
        }
    }

    async fn on_key_stations(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.total_stations() {
                    self.selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(idx) = self.favorite_index() {
                    self.play_favorite_station(idx).await;
                } else if let Some(idx) = self.search_result_index() {
                    self.play_dynamic_station(idx).await;
                } else if let Some(idx) = self.hardcoded_index() {
                    let station = self.stations[idx].clone();
                    self.play_station(station).await;
                }
            }
            KeyCode::Char('F') => {
                self.toggle_selected_favorite();
            }
            KeyCode::Char('e') => {
                if let Some(idx) = self.favorite_index() {
                    self.rename_input = self.favorites[idx].name.clone();
                    self.renaming_favorite = Some(idx);
                }
            }
            KeyCode::Char('r') => {
                let state = self.player.state();
                if let Some(station) = state.station {
                    self.play_station(station).await;
                }
            }
            KeyCode::Char('/') => {
                self.show_search_modal = true;
                self.modal_mode = SearchMode::Name;
                self.search_query.clear();
                self.search_results.clear();
                self.modal_selected = 0;
            }
            KeyCode::Right => {
                if !self.on_demand_shows.is_empty() || self.on_demand_loading {
                    self.on_demand_selected = 0;
                    self.focus = AppFocus::OnDemandList;
                }
            }
            KeyCode::Char('s') => {
                self.stop_metadata_polling();
                self.player.send(PlayerCommand::Stop).await;
                self.saved_tracks = Vec::new();
            }
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.search_results.clear();
                    self.selected = self.selected.min(self.stations.len().saturating_sub(1));
                } else {
                    self.config.last_selected = self.selected.min(self.stations.len().saturating_sub(1));
                    self.save_config();
                    self.stop_metadata_polling();
                    self.player.send(PlayerCommand::Stop).await;
                    self.should_quit = true;
                }
            }
            _ => {
                if let KeyCode::Char(c) = key {
                    if c.is_alphanumeric() || c == ' ' || c == '-' {
                        self.focus = AppFocus::StationSearch;
                        self.search_query.push(c);
                        self.selected = self.favorites.len() + self.stations.len();
                        self.perform_search();
                    }
                }
            }
        }
    }

    async fn on_key_station_search(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.search_query.clear();
                self.search_results.clear();
                self.selected = self.selected
                    .min(self.stations.len().saturating_sub(1));
                self.focus = AppFocus::Stations;
            }
            KeyCode::Enter => {
                if let Some(idx) = self.search_result_index() {
                    if idx < self.search_results.len() {
                        self.play_dynamic_station(idx).await;
                    }
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                if self.search_query.is_empty() {
                    self.search_results.clear();
                    self.selected = self.selected
                        .min(self.stations.len().saturating_sub(1));
                    self.focus = AppFocus::Stations;
                } else {
                    self.selected = self.favorites.len() + self.stations.len();
                    self.perform_search();
                }
            }
            KeyCode::Char(c) if c.is_alphanumeric() || c == ' ' || c == '-' => {
                self.search_query.push(c);
                self.selected = self.favorites.len() + self.stations.len();
                self.perform_search();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let min = self.favorites.len() + self.stations.len();
                if self.selected > min {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.total_stations() {
                    self.selected += 1;
                }
            }
            _ => {}
        }
    }

    fn perform_search(&mut self) {
        let query = self.search_query.clone();
        if query.trim().is_empty() {
            self.search_results.clear();
            self.search_loading = false;
            if let Some(t) = self.search_task.take() { t.abort(); }
            return;
        }
        self.spawn_search(move || async move { search_stations(&query, 20).await });
    }

    fn perform_genre_search(&mut self, tag: &str) {
        let tag = tag.to_string();
        self.spawn_search(move || async move { search_stations_by_tag(&tag, 20).await });
    }

    fn spawn_search<F, Fut>(&mut self, build: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Option<Vec<DynamicStation>>> + Send + 'static,
    {
        abort_task(&mut self.search_task);
        self.search_result_rx = None;
        self.search_loading = true;

        let existing_urls: std::collections::HashSet<String> =
            self.stations.iter().map(|s| s.url.clone()).collect();
        let (tx, rx) = std::sync::mpsc::channel();
        self.search_result_rx = Some(rx);

        self.search_task = Some(tokio::spawn(async move {
            let filtered: Vec<DynamicStation> = build().await
                .unwrap_or_default()
                .into_iter()
                .filter(|s| !existing_urls.contains(&s.url))
                .collect();
            let _ = tx.send(filtered);
        }));
    }

    pub fn poll_station_details(&mut self) {
        if let Some(rx) = self.station_details_rx.take() {
            match rx.try_recv() {
                Ok(details) => { self.station_details = Some(details); }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.station_details_rx = Some(rx);
                }
                Err(_) => {}
            }
        }

        let current_uuid = self.player.state().station.as_ref().map(|s| s.key.clone());
        if current_uuid == self.last_details_uuid { return; }

        self.last_details_uuid = current_uuid.clone();
        self.station_details   = None;

        if let Some(key) = current_uuid {
            if key.is_empty() || key.starts_with("ondemand_") { return; }

            let station_name = self.player.state().station
                .as_ref().map(|s| s.name.clone()).unwrap_or_default();

            let (tx, rx) = std::sync::mpsc::channel();
            self.station_details_rx = Some(rx);
            tokio::spawn(async move {
                let details = if is_uuid(&key) {
                    fetch_station_details(&key).await
                } else {
                    fetch_station_details_by_name(&station_name).await
                };
                if let Some(d) = details {
                    let _ = tx.send(d);
                }
            });
        }
    }

    pub fn poll_search_results(&mut self) {
        if let Some(rx) = self.search_result_rx.take() {
            match rx.try_recv() {
                Ok(results) => {
                    self.search_results = results;
                    self.search_loading = false;
                    let max = self.total_stations();
                    if self.selected >= max && max > 0 {
                        self.selected = max - 1;
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.search_result_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.search_loading = false;
                }
            }
        }
    }

    pub fn player_state(&self) -> PlayerState {
        self.player.state()
    }

    pub async fn on_click(&mut self, col: u16, row: u16) {
        let player_state = self.player.state();
        if let Some(duration) = player_state.playback_duration_secs {
            let has_recent    = !player_state.recent_titles.is_empty();
            let has_saved     = !self.saved_tracks.is_empty();
            let has_on_demand = !self.on_demand_shows.is_empty() || self.on_demand_loading;
            let show_countdown = player_state.station.as_ref().map(|s| s.show_countdown).unwrap_or(false);

            if let Some(np_area) = crate::ui::renderer::now_playing_rect(
                self.terminal_area,
                has_recent,
                has_saved,
                show_countdown,
                has_on_demand,
            ) {
                if row == np_area.y {
                    let inner_x     = np_area.x + 1;
                    let inner_width = np_area.width.saturating_sub(1);
                    let pos = player_state.playback_pos_secs.unwrap_or(0.0);
                    let time_str_len = format!(
                        " {} / {} ",
                        crate::ui::widgets::now_playing::fmt_duration(pos),
                        crate::ui::widgets::now_playing::fmt_duration(duration),
                    ).len();
                    let bar_width = (inner_width as usize).saturating_sub(time_str_len + 2);
                    if bar_width > 0 && col >= inner_x {
                        let fill_col = col.saturating_sub(inner_x + 1) as usize;
                        let ratio = (fill_col as f32 / bar_width as f32).clamp(0.0, 1.0);
                        self.player.send(PlayerCommand::Seek(ratio * duration)).await;
                    }
                    return;
                }
            }
        }

        let h = self.terminal_area.height;
        if h == 0 {
            return;
        }
        let content_start: u16 = if h >= 11 { 4 } else { 2 };
        let footer_rows:   u16 = if h >= 11 { 5 } else { 2 };
        let list_max_row = h.saturating_sub(footer_rows);
        if row < content_start || row >= list_max_row {
            return;
        }
        let item_idx = (row - content_start) as usize;

        match &self.focus {
            AppFocus::StationSearch => {
                let abs_idx = self.favorites.len() + self.stations.len() + item_idx;
                if abs_idx < self.total_stations() {
                    self.selected = abs_idx;
                }
            }
            AppFocus::Stations => {
                if item_idx < self.total_stations() {
                    self.selected = item_idx;
                }
            }
            AppFocus::RecentTracks | AppFocus::OnDemandList => {}
        }
    }

    pub async fn on_mouse_scroll(&mut self, delta: i32) {
        if self.show_search_modal {
            let (len, sel) = if self.search_results.is_empty() && matches!(self.modal_mode, SearchMode::Genre) {
                (filter_items(GENRES, &self.genre_filter).len(), &mut self.genre_selected)
            } else if self.search_results.is_empty() && matches!(self.modal_mode, SearchMode::Country) {
                (filter_items(COUNTRIES, &self.country_filter).len(), &mut self.country_selected)
            } else if matches!(self.modal_mode, SearchMode::Settings) {
                (settings_items(self.config.duck_enabled).len(), &mut self.settings_selected)
            } else {
                (self.search_results.len(), &mut self.modal_selected)
            };
            if len == 0 { return; }
            *sel = scroll_by(*sel, delta, len);
            return;
        }

        match self.focus {
            AppFocus::RecentTracks => {
                let len = self.player.state().recent_titles.len();
                if len == 0 { return; }
                self.recent_selected = scroll_by(self.recent_selected, delta, len);
            }
            AppFocus::OnDemandList => {
                let len = self.on_demand_shows.len();
                if len == 0 { return; }
                self.on_demand_selected = scroll_by(self.on_demand_selected, delta, len);
            }
            AppFocus::Stations | AppFocus::StationSearch => {
                self.selected = scroll_by(self.selected, delta, self.total_stations());
            }
        }
    }

    pub async fn on_double_click(&mut self) {
        match self.focus {
            AppFocus::Stations | AppFocus::StationSearch => {
                self.click_flash = Some((self.selected, Instant::now()));
                if let Some(idx) = self.favorite_index() {
                    self.play_favorite_station(idx).await;
                } else if let Some(idx) = self.search_result_index() {
                    self.play_dynamic_station(idx).await;
                } else if let Some(idx) = self.hardcoded_index() {
                    let station = self.stations[idx].clone();
                    self.play_station(station).await;
                }
            }
            _ => {}
        }
    }

    async fn on_key_on_demand(&mut self, key: KeyCode) {
        let len = self.on_demand_shows.len();
        if len == 0 && !self.on_demand_loading {
            self.focus = AppFocus::Stations;
            return;
        }

        match key {
            KeyCode::Char('p') => {
                let total_programs = crate::station::on_demand::PROGRAMS.len();
                self.selected_program = (self.selected_program + 1) % total_programs;
                self.on_demand_selected = 0;
                self.start_on_demand_fetch();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.seek_input.clear();
                if self.on_demand_selected > 0 {
                    self.on_demand_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.seek_input.clear();
                if len > 0 && self.on_demand_selected + 1 < len {
                    self.on_demand_selected += 1;
                }
            }
            KeyCode::Enter => {
                if !self.seek_input.is_empty() {
                    if let Some(target) = parse_seek_input(&self.seek_input) {
                        self.player.send(PlayerCommand::Seek(target)).await;
                    }
                    self.seek_input.clear();
                } else if let Some(show) = self.on_demand_shows.get(self.on_demand_selected) {
                    let station = Station {
                        key:              format!("ondemand_{}", show.id),
                        name:             show.title.clone(),
                        url:              show.audio_url.clone(),
                        metadata_api_url: None,
                        history_api_url:  None,
                        schedule_url:     None,
                        show_countdown:   false,
                        bitrate_kbps:     None,
                    };
                    self.play_station(station).await;
                }
            }
            KeyCode::Backspace => {
                self.seek_input.pop();
            }
            KeyCode::Char('[') => {
                let pos = self.player.state().playback_pos_secs.unwrap_or(0.0);
                self.player.send(PlayerCommand::Seek((pos - 60.0).max(0.0))).await;
            }
            KeyCode::Char(']') => {
                let state  = self.player.state();
                let pos    = state.playback_pos_secs.unwrap_or(0.0);
                let target = pos + 60.0;
                let target = state.playback_duration_secs.map(|d| target.min(d)).unwrap_or(target);
                self.player.send(PlayerCommand::Seek(target)).await;
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == ':' => {
                if self.seek_input.len() < 7 {
                    self.seek_input.push(c);
                }
            }
            KeyCode::Left | KeyCode::Esc => {
                if !self.seek_input.is_empty() {
                    self.seek_input.clear();
                } else {
                    self.focus = AppFocus::Stations;
                }
            }
            _ => {}
        }
    }

    fn on_key_settings(&mut self, key: KeyCode) {
        const SETTINGS_COUNT: usize = 2;
        match key {
            KeyCode::Esc | KeyCode::Char('o') => {
                self.show_settings = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_selected > 0 {
                    self.settings_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.settings_selected + 1 < SETTINGS_COUNT {
                    self.settings_selected += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.apply_settings_toggle(self.settings_selected);
            }
            _ => {}
        }
    }

    fn apply_settings_toggle(&mut self, idx: usize) {
        let Some(&item) = settings_items(self.config.duck_enabled).get(idx) else { return };
        match item {
            SettingItem::Autoplay        => self.config.autoplay_last  = !self.config.autoplay_last,
            SettingItem::RestoreVolume   => self.config.restore_volume = !self.config.restore_volume,
            SettingItem::Crossfade       => self.config.crossfade_next(),
            SettingItem::OverlayMode     => self.config.overlay_mode     = self.config.overlay_mode.next(),
            SettingItem::OverlayAlpha    => {
                self.config.overlay_alpha = match self.config.overlay_alpha {
                    v if v < 30 => 30,
                    v if v < 50 => 50,
                    v if v < 70 => 70,
                    v if v < 90 => 90,
                    _           => 20,
                };
            }
            SettingItem::OverlayPosition => self.config.overlay_position = self.config.overlay_position.next(),
            SettingItem::Screensaver     => self.config.screensaver_next(),
            SettingItem::DuckEnabled     => self.config.duck_enabled = !self.config.duck_enabled,
            SettingItem::DuckVolume      => {
                self.config.duck_volume = match self.config.duck_volume {
                    v if v < 20 => 20,
                    v if v < 30 => 30,
                    v if v < 40 => 40,
                    v if v < 50 => 50,
                    v if v < 60 => 60,
                    v if v < 70 => 70,
                    v if v < 80 => 80,
                    _           => 10,
                };
            }
            SettingItem::MediaKeys       => self.config.media_keys    = !self.config.media_keys,
            SettingItem::TrayIcon        => self.config.tray_icon     = !self.config.tray_icon,
            SettingItem::Notifications   => self.config.notifications = !self.config.notifications,
            SettingItem::Language => {
                self.config.language = self.config.language.next();
                i18n::set_language(self.config.language);
            }
        }
        self.save_config();
        if let Some(ref tx) = self.windows_tx {
            let _ = tx.send(self.config.clone());
        }
    }

    async fn on_key_recent(&mut self, key: KeyCode) {
        let titles = self.player.state().recent_titles;
        let len = titles.len();
        if len == 0 {
            self.focus = AppFocus::Stations;
            return;
        }
        self.recent_selected = self.recent_selected.min(len - 1);

        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.recent_selected = self.recent_selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.recent_selected + 1 < len {
                    self.recent_selected += 1;
                }
            }
            KeyCode::Enter => {
                let title = titles[self.recent_selected].clone();
                let state = self.player.state();
                let key_str = state.station.as_ref().map(|s| s.key.as_str()).unwrap_or("unknown");
                match library::save_track(&title, key_str) {
                    SaveResult::Saved => {
                        self.saved_tracks = library::load_saved_tracks(key_str);
                        self.save_notice_is_dup = false;
                        self.save_notice = Some(format!("{} {title}", t("notice.saved")));
                    }
                    SaveResult::AlreadySaved => {
                        self.save_notice_is_dup = true;
                        self.save_notice = Some(format!("{} {title}", t("notice.already_saved")));
                    }
                }
            }
            KeyCode::Char('p') => {
                let state = self.player.state();
                if state.preview_title.is_some() || state.preview_searching {
                    self.player.send(PlayerCommand::StopPreview).await;
                    self.player.send(PlayerCommand::SetPreviewSearching(false)).await;
                } else if !titles.is_empty() {
                    let raw = titles[self.recent_selected].clone();
                    let cmd_tx = self.player.clone_sender();
                    let _ = cmd_tx.send(PlayerCommand::SetPreviewSearching(true)).await;
                    let _ = cmd_tx.send(PlayerCommand::SetPreviewLoadingTrack(Some(raw.clone()))).await;
                    tokio::spawn(async move {
                        match deezer_preview(&raw).await {
                            Some((url, title)) => {
                                let _ = cmd_tx.send(PlayerCommand::SetPreviewLoadingTrack(None)).await;
                                if cmd_tx.send(PlayerCommand::PlayPreview { url, title, raw_track: raw }).await.is_err() {
                                    return;
                                }
                                tokio::time::sleep(std::time::Duration::from_secs(35)).await;
                                let _ = cmd_tx.send(PlayerCommand::StopPreview).await;
                            }
                            None => {
                                tracing::warn!("Deezer: sin resultado para '{raw}'");
                                let _ = cmd_tx.send(PlayerCommand::SetPreviewLoadingTrack(None)).await;
                                let _ = cmd_tx.send(PlayerCommand::SetPreviewSearching(false)).await;
                                let _ = cmd_tx.send(PlayerCommand::MarkPreviewUnavailable(raw)).await;
                            }
                        }
                    });
                }
            }
            KeyCode::Esc => {
                self.focus = AppFocus::Stations;
            }
            _ => {}
        }
    }
}
