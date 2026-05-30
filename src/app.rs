use crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::audio::{AudioPlayer, PlayerCommand, PlayerState, PlayerStatus};
use crate::config::Config;
use crate::favorites::{self, FavoriteStation};
use crate::library::{self, SaveResult};
use crate::preview::{deezer_preview, parse_seek_input};
use crate::schedule::poll_metadata_loop;
use crate::station::on_demand::OnDemandShow;
use crate::station::{enrich, find_enrichment, is_duplicate, on_demand, search_stations, search_stations_by_tag, search_stations_by_country, DynamicStation, Station};

pub enum SearchMode {
    Name,
    Genre,
    Country,
    Settings,
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
    pub overlay_mode_tx:     Option<tokio::sync::watch::Sender<crate::config::OverlayMode>>,
    pub config:              Config,
    metadata_task:           Option<tokio::task::JoinHandle<()>>,
    search_task:             Option<tokio::task::JoinHandle<()>>,
    search_result_rx:        Option<std::sync::mpsc::Receiver<Vec<DynamicStation>>>,
    on_demand_task:          Option<tokio::task::JoinHandle<()>>,
    on_demand_rx:            Option<std::sync::mpsc::Receiver<Vec<OnDemandShow>>>,
}

impl App {
    pub async fn new() -> Self {
        let config = Config::load();
        let player = AudioPlayer::spawn();
        player.send(PlayerCommand::SetVolume(config.volume)).await;

        let favorites = favorites::load();
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
            overlay_mode_tx:    None,
            config,
            metadata_task:      None,
            search_task:        None,
            search_result_rx:   None,
            on_demand_task:     None,
            on_demand_rx:       None,
        }
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
        if let Some(i) = self.favorite_index() {
            let f = &self.favorites[i];
            Some(FavoriteStation { key: f.key.clone(), name: f.name.clone(), url: f.url.clone(), bitrate_kbps: f.bitrate_kbps })
        } else if let Some(i) = self.hardcoded_index() {
            let s = &self.stations[i];
            Some(FavoriteStation { key: s.key.clone(), name: s.name.clone(), url: s.url.clone(), bitrate_kbps: s.bitrate_kbps })
        } else if let Some(i) = self.search_result_index() {
            self.search_results.get(i).map(|ds| FavoriteStation {
                key: ds.key.clone(), name: ds.name.clone(),
                url: ds.url.clone(), bitrate_kbps: ds.bitrate_kbps,
            })
        } else {
            None
        }
    }

    fn toggle_selected_favorite(&mut self) {
        if let Some(fav) = self.build_favorite_from_selected() {
            let added = favorites::toggle(&mut self.favorites, fav);
            favorites::save(&self.favorites);
            let max = self.total_stations().saturating_sub(1);
            self.selected = self.selected.min(max);
            self.save_notice = Some(if added {
                "★ Añadida a favoritas".to_string()
            } else {
                "☆ Quitada de favoritas".to_string()
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

    fn stop_metadata_polling(&mut self) {
        if let Some(handle) = self.metadata_task.take() {
            handle.abort();
        }
    }

    fn start_on_demand_fetch(&mut self) {
        if let Some(t) = self.on_demand_task.take() {
            t.abort();
        }
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
        self.config.save();
        self.stop_metadata_polling();
        if self.player.send(PlayerCommand::Play(station.clone())).await {
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
    }

    async fn play_favorite_station(&mut self, index: usize) {
        if index >= self.favorites.len() { return; }
        let station = self.favorites[index].to_station();
        self.play_station(station).await;
    }

    async fn play_dynamic_station(&mut self, index: usize) {
        if index >= self.search_results.len() { return; }
        let ds = self.search_results[index].clone();

        let mut station = Station {
            key:              ds.key,
            name:             ds.name.clone(),
            url:              ds.url,
            metadata_api_url: None,
            history_api_url:  None,
            schedule_url:     None,
            show_countdown:   false,
            bitrate_kbps:     ds.bitrate_kbps,
        };

        if let Some(enrichment) = find_enrichment(&ds.name) {
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
                SearchMode::Settings => {}
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
                let new_vol = (self.player.state().volume + 0.05).min(1.0);
                self.player.send(PlayerCommand::SetVolume(new_vol)).await;
                return;
            }
            KeyCode::Char('-') => {
                let new_vol = (self.player.state().volume - 0.05).max(0.0);
                self.player.send(PlayerCommand::SetVolume(new_vol)).await;
                return;
            }
            KeyCode::Char('q') => {
                let state = self.player.state();
                self.config.volume = state.volume;
                self.config.last_selected = self.selected;
                self.config.save();
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
        if key == KeyCode::Tab {
            self.modal_mode = match self.modal_mode {
                SearchMode::Name     => SearchMode::Genre,
                SearchMode::Genre    => SearchMode::Country,
                SearchMode::Country  => SearchMode::Settings,
                SearchMode::Settings => SearchMode::Name,
            };
            self.modal_selected = 0;
            self.search_results.clear();
            self.search_query.clear();
            self.genre_filter.clear();
            self.genre_query.clear();
            self.genre_selected = 0;
            self.country_filter.clear();
            self.country_selected = 0;
            if let Some(t) = self.search_task.take() { t.abort(); }
            self.search_loading = false;
            return;
        }

        match self.modal_mode {
            SearchMode::Name     => self.on_key_modal_name(key).await,
            SearchMode::Genre    => self.on_key_modal_genre(key).await,
            SearchMode::Country  => self.on_key_modal_country(key).await,
            SearchMode::Settings => self.on_key_modal_settings(key),
        }
    }

    async fn on_key_modal_name(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.show_search_modal = false;
                self.search_query.clear();
                self.search_results.clear();
                self.modal_selected = 0;
            }
            KeyCode::Enter => {
                if !self.search_results.is_empty() {
                    let idx = self.modal_selected.min(self.search_results.len() - 1);
                    self.show_search_modal = false;
                    if !self.search_query.is_empty() {
                        self.config.add_to_history(self.search_query.clone());
                        self.config.save();
                    }
                    self.play_dynamic_station(idx).await;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.modal_selected > 0 { self.modal_selected -= 1; }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.modal_selected + 1 < self.search_results.len() {
                    self.modal_selected += 1;
                }
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

    async fn on_key_modal_genre(&mut self, key: KeyCode) {
        if !self.search_results.is_empty() {
            match key {
                KeyCode::Esc => {
                    self.search_results.clear();
                    self.genre_query.clear();
                    self.modal_selected = 0;
                }
                KeyCode::Enter => {
                    let idx = self.modal_selected.min(self.search_results.len() - 1);
                    self.show_search_modal = false;
                    self.play_dynamic_station(idx).await;
                }
                KeyCode::Char('R') => {
                    if let Some(idx) = self.play_random_result() {
                        self.show_search_modal = false;
                        self.play_dynamic_station(idx).await;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.modal_selected > 0 { self.modal_selected -= 1; }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.modal_selected + 1 < self.search_results.len() {
                        self.modal_selected += 1;
                    }
                }
                _ => {}
            }
            return;
        }

        let filtered = Self::filter_genres(&self.genre_filter);
        match key {
            KeyCode::Esc => {
                if !self.genre_filter.is_empty() {
                    self.genre_filter.clear();
                    self.genre_selected = 0;
                } else {
                    self.show_search_modal = false;
                    self.modal_mode = SearchMode::Name;
                    self.genre_selected = 0;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.genre_selected > 0 { self.genre_selected -= 1; }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.genre_selected + 1 < filtered.len() {
                    self.genre_selected += 1;
                }
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

    fn filter_genres(filter: &str) -> Vec<(&'static str, &'static str)> {
        use crate::station::GENRES;
        if filter.is_empty() {
            return GENRES.iter().map(|&(t, l)| (t, l)).collect();
        }
        let f = filter.to_lowercase();
        GENRES.iter()
            .filter(|(_, label)| label.to_lowercase().contains(&f))
            .map(|&(t, l)| (t, l))
            .collect()
    }

    fn filter_countries(filter: &str) -> Vec<(&'static str, &'static str)> {
        use crate::station::COUNTRIES;
        if filter.is_empty() {
            return COUNTRIES.iter().map(|&(t, l)| (t, l)).collect();
        }
        let f = filter.to_lowercase();
        COUNTRIES.iter()
            .filter(|(_, label)| label.to_lowercase().contains(&f))
            .map(|&(t, l)| (t, l))
            .collect()
    }

    fn perform_country_search(&mut self, country: &str) {
        let c = country.to_string();
        self.spawn_search(move || async move { search_stations_by_country(&c, 30).await });
    }

    async fn on_key_modal_country(&mut self, key: KeyCode) {
        if !self.search_results.is_empty() {
            match key {
                KeyCode::Esc => {
                    self.search_results.clear();
                    self.genre_query.clear();
                    self.modal_selected = 0;
                }
                KeyCode::Enter => {
                    let idx = self.modal_selected.min(self.search_results.len() - 1);
                    self.show_search_modal = false;
                    self.play_dynamic_station(idx).await;
                }
                KeyCode::Char('R') => {
                    if let Some(idx) = self.play_random_result() {
                        self.show_search_modal = false;
                        self.play_dynamic_station(idx).await;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.modal_selected > 0 { self.modal_selected -= 1; }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.modal_selected + 1 < self.search_results.len() {
                        self.modal_selected += 1;
                    }
                }
                _ => {}
            }
            return;
        }

        let filtered = Self::filter_countries(&self.country_filter);
        match key {
            KeyCode::Esc => {
                if !self.country_filter.is_empty() {
                    self.country_filter.clear();
                    self.country_selected = 0;
                } else {
                    self.show_search_modal = false;
                    self.modal_mode = SearchMode::Name;
                    self.country_selected = 0;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.country_selected > 0 { self.country_selected -= 1; }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.country_selected + 1 < filtered.len() {
                    self.country_selected += 1;
                }
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
        const SETTINGS_COUNT: usize = 2;
        match key {
            KeyCode::Esc => {
                self.show_search_modal = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_selected > 0 { self.settings_selected -= 1; }
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
                    let state = self.player.state();
                    self.config.volume = state.volume;
                    self.config.last_selected = self.selected.min(self.stations.len().saturating_sub(1));
                    self.config.save();
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
        if let Some(t) = self.search_task.take() { t.abort(); }
        self.search_result_rx = None;
        self.search_loading = true;

        let existing_urls: Vec<String> = self.stations.iter().map(|s| s.url.clone()).collect();
        let (tx, rx) = std::sync::mpsc::channel();
        self.search_result_rx = Some(rx);

        self.search_task = Some(tokio::spawn(async move {
            let results = build().await.unwrap_or_default();
            let refs: Vec<&str> = existing_urls.iter().map(|s| s.as_str()).collect();
            let filtered: Vec<DynamicStation> = results
                .into_iter()
                .filter(|s| !is_duplicate(&s.url, &refs))
                .collect();
            let _ = tx.send(filtered);
        }));
    }

    pub fn poll_search_results(&mut self) {
        if let Some(rx) = self.search_result_rx.take() {
            match rx.try_recv() {
                Ok(results) => {
                    self.search_results = results;
                    self.search_loading = false;
                    let max = self.stations.len() + self.search_results.len();
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
                (Self::filter_genres(&self.genre_filter).len(), &mut self.genre_selected)
            } else if self.search_results.is_empty() && matches!(self.modal_mode, SearchMode::Country) {
                (Self::filter_countries(&self.country_filter).len(), &mut self.country_selected)
            } else {
                (self.search_results.len(), &mut self.modal_selected)
            };
            if len == 0 { return; }
            if delta > 0 {
                *sel = (*sel + delta as usize).min(len - 1);
            } else {
                *sel = sel.saturating_sub((-delta) as usize);
            }
            return;
        }

        match self.focus {
            AppFocus::RecentTracks => {
                let titles = self.player.state().recent_titles;
                let len = titles.len();
                if len == 0 {
                    return;
                }
                if delta > 0 {
                    self.recent_selected = (self.recent_selected + delta as usize).min(len - 1);
                } else {
                    self.recent_selected = self.recent_selected.saturating_sub((-delta) as usize);
                }
            }
            AppFocus::OnDemandList => {
                let len = self.on_demand_shows.len();
                if len == 0 {
                    return;
                }
                if delta > 0 {
                    self.on_demand_selected = (self.on_demand_selected + delta as usize).min(len - 1);
                } else {
                    self.on_demand_selected = self.on_demand_selected.saturating_sub((-delta) as usize);
                }
            }
            AppFocus::Stations | AppFocus::StationSearch => {
                if delta > 0 {
                    self.selected = (self.selected + delta as usize).min(self.total_stations().saturating_sub(1));
                } else {
                    self.selected = self.selected.saturating_sub((-delta) as usize);
                }
            }
        }
    }

    pub async fn on_double_click(&mut self) {
        match self.focus {
            AppFocus::Stations | AppFocus::StationSearch => {
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
                let state    = self.player.state();
                let pos      = state.playback_pos_secs.unwrap_or(0.0);
                let duration = state.playback_duration_secs.unwrap_or(f32::MAX);
                self.player.send(PlayerCommand::Seek((pos + 60.0).min(duration))).await;
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
        match idx {
            0 => {
                self.config.autoplay_last = !self.config.autoplay_last;
                self.config.save();
            }
            1 => {
                self.config.overlay_mode = self.config.overlay_mode.next();
                self.config.save();
                if let Some(ref tx) = self.overlay_mode_tx {
                    let _ = tx.send(self.config.overlay_mode);
                }
            }
            _ => {}
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
                        self.save_notice = Some(format!("Guardado: {title}"));
                    }
                    SaveResult::AlreadySaved => {
                        self.save_notice = Some(format!("Ya guardada: {title}"));
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
