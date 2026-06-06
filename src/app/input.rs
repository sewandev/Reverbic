use std::time::Instant;

use crossterm::event::KeyCode;

use crate::audio::{PlayerCommand, PlayerStatus};
use crate::i18n::t;
use crate::library;
use crate::preview::{deezer_preview, parse_seek_input};
use crate::station::{filter_items, Station, COUNTRIES, GENRES};

use super::modal::settings_items;
use super::modal::{AppFocus, RadioSubTab, SearchMode, SpotifyAuthStatus, SpotifyPlayerStatus};
use super::{abort_task, cycle_next, cycle_prev, scroll_by, App};

impl App {
    pub async fn on_key_event(&mut self, event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyModifiers;

        if event.modifiers.contains(KeyModifiers::SHIFT)
            && !self.show_search_modal
            && matches!(self.focus, AppFocus::Stations)
        {
            if let Some(idx) = self.favorite_index() {
                match event.code {
                    KeyCode::Up => {
                        crate::favorites::move_up(&mut self.favorites, idx);
                        crate::favorites::save(&self.favorites);
                        if idx > 0 {
                            self.selected -= 1;
                        }
                        return;
                    }
                    KeyCode::Down => {
                        crate::favorites::move_down(&mut self.favorites, idx);
                        crate::favorites::save(&self.favorites);
                        if idx + 1 < self.favorites.len() {
                            self.selected += 1;
                        }
                        return;
                    }
                    _ => {}
                }
            }
        }

        if event.modifiers.contains(KeyModifiers::SHIFT)
            && self.show_search_modal
            && matches!(self.modal_mode, SearchMode::Name)
            && matches!(self.radio_sub_tab, RadioSubTab::Favorites)
        {
            let idx = self.radio_fav_selected;
            match event.code {
                KeyCode::Up => {
                    crate::favorites::move_up(&mut self.favorites, idx);
                    crate::favorites::save(&self.favorites);
                    if idx > 0 {
                        self.radio_fav_selected -= 1;
                    }
                    return;
                }
                KeyCode::Down => {
                    crate::favorites::move_down(&mut self.favorites, idx);
                    crate::favorites::save(&self.favorites);
                    if idx + 1 < self.favorites.len() {
                        self.radio_fav_selected += 1;
                    }
                    return;
                }
                _ => {}
            }
        }

        if self.renaming_favorite.is_some() {
            self.on_key_rename(event.code);
            return;
        }

        if self.editing_client_id {
            self.on_key_client_id_input(event.code);
            return;
        }

        if event.modifiers.contains(KeyModifiers::ALT) {
            self.handle_alt_key(event.code).await;
            return;
        }

        self.on_key(event.code).await;
    }

    async fn handle_alt_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('o') | KeyCode::Char('O') => {
                self.show_search_modal = true;
                self.modal_mode = SearchMode::Settings;
                self.settings_selected = 0;
            }
            KeyCode::Char('g') | KeyCode::Char('G') if self.show_search_modal => {
                self.modal_mode = SearchMode::Genre;
                self.genre_filter.clear();
                self.genre_selected = 0;
            }
            KeyCode::Char('c') | KeyCode::Char('C') if self.show_search_modal => {
                self.modal_mode = SearchMode::Country;
                self.country_filter.clear();
                self.country_selected = 0;
            }
            KeyCode::Char('d') | KeyCode::Char('D')
                if self.show_search_modal
                    && matches!(self.modal_mode, SearchMode::Spotify)
                    && matches!(self.spotify.status, SpotifyAuthStatus::LoggedIn) =>
            {
                self.spotify_logout();
                self.modal_mode = SearchMode::Name;
            }
            KeyCode::Char('r') | KeyCode::Char('R')
                if self.show_search_modal
                    && matches!(self.modal_mode, SearchMode::Spotify)
                    && matches!(self.spotify.status, SpotifyAuthStatus::LoggedIn) =>
            {
                self.fetch_spotify_devices();
            }
            KeyCode::Char('f') | KeyCode::Char('F')
                if self.show_search_modal
                    && matches!(self.modal_mode, SearchMode::Name)
                    && matches!(self.radio_sub_tab, RadioSubTab::Favorites) =>
            {
                self.remove_radio_fav_selected();
            }
            KeyCode::Char('f') | KeyCode::Char('F')
                if self.show_search_modal && !self.search_results.is_empty() =>
            {
                self.toggle_modal_favorite();
            }
            KeyCode::Char('f') | KeyCode::Char('F') if !self.show_search_modal => {
                self.toggle_selected_favorite();
            }
            KeyCode::Char('r') | KeyCode::Char('R')
                if self.show_search_modal
                    && !matches!(self.modal_mode, SearchMode::Spotify)
                    && !self.search_results.is_empty() =>
            {
                self.last_activity = std::time::Instant::now();
                if let Some(idx) = self.play_random_result() {
                    self.play_dynamic_station(idx).await;
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.stop_metadata_polling();
                self.player.send(PlayerCommand::Stop).await;
                self.saved_tracks = Vec::new();
            }
            _ => {}
        }
    }

    pub fn on_paste(&mut self, text: String) {
        if !self.show_search_modal {
            return;
        }
        let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
        if filtered.is_empty() {
            return;
        }
        match self.modal_mode {
            SearchMode::Name => {
                self.search_query.push_str(&filtered);
                self.modal_selected = 0;
                self.perform_search();
            }
            SearchMode::Genre => {
                self.genre_filter.push_str(&filtered);
                self.genre_selected = 0;
            }
            SearchMode::Country => {
                self.country_filter.push_str(&filtered);
                self.country_selected = 0;
            }
            SearchMode::Youtube => {
                self.youtube.query.push_str(&filtered);
                self.youtube.selected = 0;
                self.perform_youtube_search();
            }
            _ => {}
        }
    }

    pub async fn on_key(&mut self, key: KeyCode) {
        if self.screensaver_active() {
            match key {
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    if self.active_source_is_spotify() {
                        self.adjust_spotify_volume(5).await;
                    } else {
                        self.adjust_volume(self.config.volume_step as f32 / 100.0)
                            .await;
                    }
                    return;
                }
                KeyCode::Char('-') => {
                    if self.active_source_is_spotify() {
                        self.adjust_spotify_volume(-5).await;
                    } else {
                        self.adjust_volume(-(self.config.volume_step as f32 / 100.0))
                            .await;
                    }
                    return;
                }
                KeyCode::Char(' ') => {
                    if self.active_source_is_spotify() {
                        use super::modal::SpotifyPlayerStatus;
                        if let Some(device_id) = self.spotify.active_device_id.clone() {
                            let token = self.spotify.access_token.clone().unwrap_or_default();
                            match self.spotify.player_status {
                                SpotifyPlayerStatus::Playing => {
                                    self.spotify.player_status = SpotifyPlayerStatus::Paused;
                                    std::thread::spawn(move || {
                                        if let Ok(rt) =
                                            tokio::runtime::Builder::new_current_thread()
                                                .enable_all()
                                                .build()
                                        {
                                            let _ = rt.block_on(
                                                crate::integrations::spotify::devices::pause_device(
                                                    &token, &device_id,
                                                ),
                                            );
                                        }
                                    });
                                }
                                SpotifyPlayerStatus::Paused => {
                                    self.spotify.player_status = SpotifyPlayerStatus::Playing;
                                    std::thread::spawn(move || {
                                        if let Ok(rt) =
                                            tokio::runtime::Builder::new_current_thread()
                                                .enable_all()
                                                .build()
                                        {
                                            let _ = rt.block_on(crate::integrations::spotify::devices::resume_device(&token, &device_id));
                                        }
                                    });
                                }
                                _ => {}
                            }
                        } else if let Some(handle) = &self.spotify.player_tx {
                            use super::modal::SpotifyPlayerStatus;
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
                    } else {
                        match self.player.state().status {
                            PlayerStatus::Playing => {
                                self.player.send(PlayerCommand::Pause).await;
                            }
                            PlayerStatus::Paused => {
                                self.player.send(PlayerCommand::Resume).await;
                            }
                            _ => {}
                        }
                    }
                    return;
                }
                _ => {}
            }
            self.last_activity = Instant::now();
            if key == KeyCode::Char('o') || key == KeyCode::Char('O') {
                if let Some(ref d) = self.station_details {
                    if !d.homepage.is_empty() {
                        crate::shell::open_url(&d.homepage);
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

        self.save_notice = None;
        match key {
            KeyCode::Char(' ') => {
                match self.player.state().status {
                    PlayerStatus::Playing => {
                        self.player.send(PlayerCommand::Pause).await;
                    }
                    PlayerStatus::Paused => {
                        self.player.send(PlayerCommand::Resume).await;
                    }
                    _ => {}
                }
                return;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.adjust_volume(self.config.volume_step as f32 / 100.0)
                    .await;
                return;
            }
            KeyCode::Char('-') => {
                self.adjust_volume(-(self.config.volume_step as f32 / 100.0))
                    .await;
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
            KeyCode::Tab => {
                let has_recent = !self.player.state().recent_titles.is_empty();
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
            AppFocus::Stations => self.on_key_stations(key).await,
            AppFocus::RecentTracks => self.on_key_recent(key).await,
            AppFocus::StationSearch => self.on_key_station_search(key).await,
            AppFocus::OnDemandList => self.on_key_on_demand(key).await,
        }
    }

    async fn on_key_search_modal(&mut self, key: KeyCode) {
        if self.show_help {
            self.show_help = false;
            return;
        }
        match key {
            KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if matches!(self.modal_mode, SearchMode::Spotify) && self.active_source_is_spotify()
                {
                    self.adjust_spotify_volume(5).await;
                } else {
                    self.adjust_volume(self.config.volume_step as f32 / 100.0)
                        .await;
                }
                return;
            }
            KeyCode::Char('-') => {
                if matches!(self.modal_mode, SearchMode::Spotify) && self.active_source_is_spotify()
                {
                    self.adjust_spotify_volume(-5).await;
                } else {
                    self.adjust_volume(-(self.config.volume_step as f32 / 100.0))
                        .await;
                }
                return;
            }
            KeyCode::Char(' ')
                if matches!(self.modal_mode, SearchMode::Name)
                    && matches!(self.radio_sub_tab, RadioSubTab::Favorites) =>
            {
                match self.player.state().status {
                    PlayerStatus::Playing => {
                        self.player.send(PlayerCommand::Pause).await;
                    }
                    PlayerStatus::Paused => {
                        self.player.send(PlayerCommand::Resume).await;
                    }
                    _ => {}
                }
                return;
            }
            _ => {}
        }

        if key == KeyCode::Tab {
            self.modal_mode = match &self.modal_mode {
                SearchMode::Name | SearchMode::Genre | SearchMode::Country => SearchMode::Spotify,
                SearchMode::Spotify => SearchMode::Youtube,
                SearchMode::Youtube => SearchMode::Name,
                other => *other,
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
            self.spotify.search_query.clear();
            self.spotify.search_results.clear();
            self.spotify.search_selected = 0;
            abort_task(&mut self.spotify.search_task);
            self.spotify.search_loading = false;
            self.youtube.query.clear();
            self.youtube.results.clear();
            self.youtube.selected = 0;
            self.youtube.loading = false;
            self.youtube.search_pending_until = None;
            abort_task(&mut self.youtube.search_task);
            self.youtube.search_rx = None;
            if matches!(self.modal_mode, SearchMode::Spotify)
                && matches!(self.spotify.status, SpotifyAuthStatus::LoggedIn)
                && self.spotify.devices.is_empty()
                && !self.spotify.devices_loading
            {
                self.fetch_spotify_devices();
            } else if matches!(self.modal_mode, SearchMode::Youtube) {
                self.ensure_youtube_ready();
            }
            return;
        }

        match self.modal_mode {
            SearchMode::Name => self.on_key_modal_name(key).await,
            SearchMode::Genre => self.on_key_modal_genre(key).await,
            SearchMode::Country => self.on_key_modal_country(key).await,
            SearchMode::Settings => self.on_key_modal_settings(key),
            SearchMode::Spotify => self.on_key_modal_spotify(key).await,
            SearchMode::Youtube => self.on_key_modal_youtube(key).await,
        }
    }

    async fn on_key_modal_name(&mut self, key: KeyCode) {
        if matches!(key, KeyCode::Left | KeyCode::Right) {
            self.radio_sub_tab = match self.radio_sub_tab {
                RadioSubTab::Search => RadioSubTab::Favorites,
                RadioSubTab::Favorites => RadioSubTab::Search,
            };
            self.radio_fav_selected = 0;
            return;
        }
        if matches!(self.radio_sub_tab, RadioSubTab::Favorites) {
            self.on_key_radio_favorites(key).await;
            return;
        }
        match key {
            KeyCode::Esc => {
                self.show_help = false;
                if self.search_query.is_empty() && self.search_results.is_empty() {
                    self.should_quit = true;
                } else {
                    self.search_query.clear();
                    self.search_results.clear();
                    self.modal_selected = 0;
                }
            }
            KeyCode::Char('R') if !self.search_results.is_empty() => {
                if let Some(idx) = self.play_random_result() {
                    self.play_dynamic_station(idx).await;
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

    async fn on_key_radio_favorites(&mut self, key: KeyCode) {
        let len = self.favorites.len();
        match key {
            KeyCode::Esc => {
                self.radio_sub_tab = RadioSubTab::Search;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.radio_fav_selected = super::cycle_prev(self.radio_fav_selected, len);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.radio_fav_selected = super::cycle_next(self.radio_fav_selected, len);
            }
            KeyCode::Enter => {
                self.play_favorite_station(self.radio_fav_selected).await;
            }
            KeyCode::Char('R') if self.radio_fav_selected < len => {
                self.renaming_favorite = Some(self.radio_fav_selected);
                self.rename_input = self.favorites[self.radio_fav_selected].name.clone();
            }
            _ => {}
        }
    }

    async fn on_key_modal_results(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.show_help = false;
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
        if key == KeyCode::Enter {
            if let Some(&(tag, label)) = filtered.get(self.genre_selected) {
                self.genre_query = label.to_string();
                self.modal_selected = 0;
                self.perform_genre_search(tag);
            }
            return;
        }
        let len = filtered.len();
        if super::handle_filter_list_key(key, &mut self.genre_filter, &mut self.genre_selected, len)
        {
            self.modal_mode = SearchMode::Name;
        }
    }

    async fn on_key_modal_country(&mut self, key: KeyCode) {
        if !self.search_results.is_empty() {
            self.on_key_modal_results(key).await;
            return;
        }
        let filtered = filter_items(COUNTRIES, &self.country_filter);
        if key == KeyCode::Enter {
            if let Some(&(tag, label)) = filtered.get(self.country_selected) {
                self.genre_query = label.to_string();
                self.modal_selected = 0;
                self.perform_country_search(tag);
            }
            return;
        }
        let len = filtered.len();
        if super::handle_filter_list_key(
            key,
            &mut self.country_filter,
            &mut self.country_selected,
            len,
        ) {
            self.modal_mode = SearchMode::Name;
        }
    }

    fn on_key_modal_settings(&mut self, key: KeyCode) {
        let count = settings_items(self.config.duck_enabled).len();
        match key {
            KeyCode::Esc => {
                self.show_help = false;
                self.modal_mode = SearchMode::Name;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.settings_selected = cycle_prev(self.settings_selected, count);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings_selected = cycle_next(self.settings_selected, count);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let items = settings_items(self.config.duck_enabled);
                if let Some(&super::modal::SettingItem::SpotifyClientId) =
                    items.get(self.settings_selected)
                {
                    self.client_id_input = self.config.spotify.client_id.clone();
                    self.editing_client_id = true;
                } else {
                    self.apply_settings_toggle(self.settings_selected);
                }
            }
            _ => {}
        }
    }

    fn on_key_client_id_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.client_id_input.clear();
                self.editing_client_id = false;
            }
            KeyCode::Enter => {
                self.config.spotify.client_id = self.client_id_input.trim().to_string();
                self.save_config();
                self.client_id_input.clear();
                self.editing_client_id = false;
            }
            KeyCode::Backspace => {
                self.client_id_input.pop();
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.client_id_input.push(c);
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
                } else if matches!(
                    self.player.state().status,
                    PlayerStatus::Error(_) | PlayerStatus::Reconnecting(_)
                ) {
                    self.stop_metadata_polling();
                    self.player.send(PlayerCommand::Stop).await;
                } else {
                    self.config.last_selected =
                        self.selected.min(self.stations.len().saturating_sub(1));
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
                self.selected = self.selected.min(self.stations.len().saturating_sub(1));
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
                    self.selected = self.selected.min(self.stations.len().saturating_sub(1));
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
            KeyCode::Down | KeyCode::Char('j') if self.selected + 1 < self.total_stations() => {
                self.selected += 1;
            }
            _ => {}
        }
    }

    pub async fn on_click(&mut self, col: u16, row: u16) {
        self.last_activity = Instant::now();
        if self.screensaver_active() {
            if let Some(ref playback) = self.spotify.playback.clone() {
                if playback.duration_ms > 0 {
                    let has_name = self.config.spotify.display_name.is_some();
                    let has_plan =
                        self.spotify.is_premium || self.config.spotify.followers.is_some();
                    let profile_rows = u16::from(has_name || self.config.spotify.country.is_some())
                        + u16::from(has_plan);
                    if let Some(prog) = crate::ui::renderer::spotify_screensaver_progress_rect(
                        self.terminal_area,
                        profile_rows,
                        self.config.screensaver_clock,
                    ) {
                        if row == prog.y && col >= prog.x && col < prog.x + prog.width {
                            let time_cur = {
                                let s = playback.progress_ms / 1000;
                                format!("{}:{:02}", s / 60, s % 60)
                            };
                            let time_tot = {
                                let s = playback.duration_ms / 1000;
                                format!("{}:{:02}", s / 60, s % 60)
                            };
                            let vol_str = format!("vol {}%", playback.volume_pct);
                            let prefix_len = (time_cur.len() + 1) as u16;
                            let suffix_len = (format!(" {} {}", time_tot, vol_str).len()) as u16;
                            let bar_start = prog.x + prefix_len;
                            let bar_w = prog.width.saturating_sub(prefix_len + suffix_len);
                            if bar_w > 0 && col >= bar_start {
                                let fill_col = col.saturating_sub(bar_start);
                                let ratio = (fill_col as f32 / bar_w as f32).clamp(0.0, 1.0);
                                let pos_ms = (ratio * playback.duration_ms as f32) as u32;
                                if let Some(ref mut p) = self.spotify.playback {
                                    p.progress_ms = pos_ms;
                                }
                                if let Some(token) = self.spotify.access_token.clone() {
                                    let device_id =
                                        self.spotify.active_device_id.clone().unwrap_or_default();
                                    tokio::spawn(async move {
                                        if let Err(e) =
                                            crate::integrations::spotify::devices::seek_playback(
                                                &token, &device_id, pos_ms,
                                            )
                                            .await
                                        {
                                            tracing::warn!("spotify seek error: {e}");
                                        }
                                    });
                                }
                            }
                            return;
                        }
                    }
                }
            }
            return;
        }

        if self.show_search_modal {
            return;
        }

        let h = self.terminal_area.height;
        if h == 0 {
            return;
        }
        let content_start: u16 = if h >= 11 { 4 } else { 2 };
        let footer_rows: u16 = if h >= 11 { 5 } else { 2 };
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
        self.last_activity = Instant::now();
        if self.show_search_modal {
            let (len, sel) =
                if self.search_results.is_empty() && matches!(self.modal_mode, SearchMode::Genre) {
                    (
                        filter_items(GENRES, &self.genre_filter).len(),
                        &mut self.genre_selected,
                    )
                } else if self.search_results.is_empty()
                    && matches!(self.modal_mode, SearchMode::Country)
                {
                    (
                        filter_items(COUNTRIES, &self.country_filter).len(),
                        &mut self.country_selected,
                    )
                } else if matches!(self.modal_mode, SearchMode::Settings) {
                    (
                        settings_items(self.config.duck_enabled).len(),
                        &mut self.settings_selected,
                    )
                } else {
                    (self.search_results.len(), &mut self.modal_selected)
                };
            if len == 0 {
                return;
            }
            *sel = scroll_by(*sel, delta, len);
            return;
        }

        match self.focus {
            AppFocus::RecentTracks => {
                let len = self.player.state().recent_titles.len();
                if len == 0 {
                    return;
                }
                self.recent_selected = scroll_by(self.recent_selected, delta, len);
            }
            AppFocus::OnDemandList => {
                let len = self.on_demand_shows.len();
                if len == 0 {
                    return;
                }
                self.on_demand_selected = scroll_by(self.on_demand_selected, delta, len);
            }
            AppFocus::Stations | AppFocus::StationSearch => {
                self.selected = scroll_by(self.selected, delta, self.total_stations());
            }
        }
    }

    pub async fn on_double_click(&mut self) {
        self.last_activity = Instant::now();
        if self.show_search_modal {
            return;
        }
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
                        key: format!("ondemand_{}", show.id),
                        name: show.title.clone(),
                        url: show.audio_url.clone(),
                        metadata_api_url: None,
                        history_api_url: None,
                        schedule_url: None,
                        show_countdown: false,
                        bitrate_kbps: None,
                    };
                    self.play_station(station).await;
                }
            }
            KeyCode::Backspace => {
                self.seek_input.pop();
            }
            KeyCode::Char('[') => {
                let pos = self.player.state().playback_pos_secs.unwrap_or(0.0);
                self.player
                    .send(PlayerCommand::Seek((pos - 60.0).max(0.0)))
                    .await;
            }
            KeyCode::Char(']') => {
                let state = self.player.state();
                let pos = state.playback_pos_secs.unwrap_or(0.0);
                let target = pos + 60.0;
                let target = state
                    .playback_duration_secs
                    .map(|d| target.min(d))
                    .unwrap_or(target);
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

    fn apply_settings_toggle(&mut self, idx: usize) {
        use crate::i18n;
        let Some(&item) = settings_items(self.config.duck_enabled).get(idx) else {
            return;
        };
        match item {
            super::modal::SettingItem::Autoplay => {
                self.config.autoplay_last = !self.config.autoplay_last
            }
            super::modal::SettingItem::RestoreVolume => {
                self.config.restore_volume = !self.config.restore_volume
            }
            super::modal::SettingItem::Crossfade => self.config.crossfade_next(),
            super::modal::SettingItem::VolumeStep => self.config.volume_step_next(),
            super::modal::SettingItem::Prebuffer => {
                self.config.prebuffer_next();
                let secs = self.config.prebuffer_secs as f32;
                let cmd_tx = self.player.clone_sender();
                tokio::spawn(async move {
                    let _ = cmd_tx
                        .send(crate::audio::PlayerCommand::SetPrebuffer(secs))
                        .await;
                });
            }
            super::modal::SettingItem::OverlayMode => {
                self.config.overlay_mode = self.config.overlay_mode.next()
            }
            super::modal::SettingItem::OverlayAlpha => {
                self.config.overlay_alpha = match self.config.overlay_alpha {
                    v if v < 30 => 30,
                    v if v < 50 => 50,
                    v if v < 70 => 70,
                    v if v < 90 => 90,
                    _ => 20,
                };
            }
            super::modal::SettingItem::OverlayPosition => {
                self.config.overlay_position = self.config.overlay_position.next()
            }
            super::modal::SettingItem::OverlayStyle => {
                self.config.overlay_style = self.config.overlay_style.next()
            }
            super::modal::SettingItem::Screensaver => self.config.screensaver_next(),
            super::modal::SettingItem::ScreensaverClock => {
                self.config.screensaver_clock = !self.config.screensaver_clock
            }
            super::modal::SettingItem::DuckEnabled => {
                self.config.duck_enabled = !self.config.duck_enabled
            }
            super::modal::SettingItem::DuckVolume => {
                self.config.duck_volume = match self.config.duck_volume {
                    v if v < 20 => 20,
                    v if v < 30 => 30,
                    v if v < 40 => 40,
                    v if v < 50 => 50,
                    v if v < 60 => 60,
                    v if v < 70 => 70,
                    v if v < 80 => 80,
                    _ => 10,
                };
            }
            super::modal::SettingItem::MediaKeys => {
                self.config.media_keys = !self.config.media_keys
            }
            super::modal::SettingItem::TrayIcon => self.config.tray_icon = !self.config.tray_icon,
            super::modal::SettingItem::Notifications => {
                self.config.notifications = !self.config.notifications
            }
            super::modal::SettingItem::Language => {
                self.config.language = self.config.language.next();
                i18n::set_language(self.config.language);
            }
            super::modal::SettingItem::SpotifyStopOnQuit => {
                self.config.spotify.stop_on_quit = !self.config.spotify.stop_on_quit;
            }
            super::modal::SettingItem::SpotifyStartOnSpotify => {
                self.config.spotify.start_on_spotify = !self.config.spotify.start_on_spotify;
            }
            super::modal::SettingItem::SpotifyClientId => {}
            super::modal::SettingItem::AutoUpdate => {
                self.config.auto_update = !self.config.auto_update
            }
            super::modal::SettingItem::DiscordRpc => {
                self.config.discord_rpc = !self.config.discord_rpc
            }
        }
        self.save_config();
        if let Some(ref tx) = self.windows_tx {
            let _ = tx.send(self.config.clone());
        }
    }

    async fn on_key_modal_spotify(&mut self, key: KeyCode) {
        if !matches!(self.spotify.status, SpotifyAuthStatus::LoggedIn) {
            self.on_key_spotify_auth(key);
            return;
        }

        match key {
            KeyCode::Esc => {
                self.show_help = false;
                if !self.spotify.search_query.is_empty() {
                    self.spotify.search_query.clear();
                    self.spotify.search_results.clear();
                    self.spotify.search_selected = 0;
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Left | KeyCode::Right => {
                use super::modal::SpotifySubTab;
                self.spotify.sub_tab = match self.spotify.sub_tab {
                    SpotifySubTab::Search => {
                        if self.spotify.devices.is_empty() && !self.spotify.devices_loading {
                            self.fetch_spotify_devices();
                        }
                        SpotifySubTab::Devices
                    }
                    SpotifySubTab::Devices => SpotifySubTab::Search,
                };
            }
            _ => {
                use super::modal::SpotifySubTab;
                match self.spotify.sub_tab {
                    SpotifySubTab::Search => self.on_key_spotify_search(key).await,
                    SpotifySubTab::Devices => self.on_key_spotify_devices(key).await,
                }
            }
        }
    }

    fn on_key_spotify_auth(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter if !matches!(self.spotify.status, SpotifyAuthStatus::Connecting) => {
                self.start_oauth_flow();
            }
            KeyCode::Esc => {
                self.show_help = false;
                if matches!(self.spotify.status, SpotifyAuthStatus::Connecting) {
                    abort_task(&mut self.spotify.auth_task);
                    self.spotify.auth_rx = None;
                    self.spotify.status = SpotifyAuthStatus::Idle;
                } else {
                    self.should_quit = true;
                }
            }
            _ => {}
        }
    }

    async fn on_key_spotify_devices(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                if self.spotify.devices_selected > 0 {
                    self.spotify.devices_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.spotify.devices_selected + 1 < self.spotify.devices.len() {
                    self.spotify.devices_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(device) = self.spotify.devices.get(self.spotify.devices_selected) {
                    if let Some(id) = device.id.clone() {
                        self.transfer_to_spotify_device(id).await;
                    }
                }
            }
            KeyCode::Char(' ') => {
                if let Some(device_id) = self.spotify.active_device_id.clone() {
                    let token = self.spotify.access_token.clone().unwrap_or_default();
                    match self.spotify.player_status {
                        SpotifyPlayerStatus::Playing => {
                            self.spotify.player_status = SpotifyPlayerStatus::Paused;
                            tokio::spawn(async move {
                                if let Err(e) = crate::integrations::spotify::devices::pause_device(
                                    &token, &device_id,
                                )
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
                                    crate::integrations::spotify::devices::resume_device(
                                        &token, &device_id,
                                    )
                                    .await
                                {
                                    tracing::warn!("spotify resume error: {e}");
                                }
                            });
                        }
                        _ => {}
                    }
                } else if let Some(handle) = &self.spotify.player_tx {
                    match self.spotify.player_status {
                        SpotifyPlayerStatus::Playing => handle.pause(),
                        SpotifyPlayerStatus::Paused => handle.resume(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    async fn on_key_spotify_search(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                if !self.spotify.search_results.is_empty() {
                    self.spotify.search_selected = cycle_prev(
                        self.spotify.search_selected,
                        self.spotify.search_results.len(),
                    );
                }
            }
            KeyCode::Down => {
                if !self.spotify.search_results.is_empty() {
                    let last = self.spotify.search_results.len() - 1;
                    if self.spotify.search_selected >= last && self.spotify.search_has_more {
                        self.load_more_spotify_results();
                    } else {
                        self.spotify.search_selected = cycle_next(
                            self.spotify.search_selected,
                            self.spotify.search_results.len(),
                        );
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(track) = self
                    .spotify
                    .search_results
                    .get(self.spotify.search_selected)
                    .cloned()
                {
                    self.player.send(PlayerCommand::Stop).await;
                    self.save_notice = Some(t("notice.spotify_radio_stopped"));
                    self.save_notice_is_dup = false;
                    self.notice_until =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
                    if let Some(device_id) = self.spotify.active_device_id.clone() {
                        let token = self.spotify.access_token.clone().unwrap_or_default();
                        let uri = track.uri.clone();
                        self.spotify.now_playing = Some(track);
                        self.spotify.player_status = SpotifyPlayerStatus::Loading;
                        let (tx, rx) = std::sync::mpsc::channel();
                        self.spotify.play_result_rx = Some(rx);
                        tokio::spawn(async move {
                            let result = crate::integrations::spotify::devices::play_on_device(
                                &token, &device_id, &uri,
                            )
                            .await;
                            let _ = tx.send(result);
                        });
                        self.start_playback_polling();
                    } else if let Some(handle) = &self.spotify.player_tx {
                        handle.play(track.clone());
                        self.spotify.now_playing = Some(track);
                        self.spotify.player_status = SpotifyPlayerStatus::Loading;
                    } else {
                        self.spotify.player_status = SpotifyPlayerStatus::Error(
                            "Player no inicializado. Reconecta Spotify.".to_string(),
                        );
                    }
                }
            }
            KeyCode::Backspace => {
                self.spotify.search_query.pop();
                self.spotify.search_selected = 0;
                self.perform_spotify_search();
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.spotify.search_query.push(c);
                self.spotify.search_selected = 0;
                self.perform_spotify_search();
            }
            _ => {}
        }
    }

    async fn on_key_modal_youtube(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.show_help = false;
                if self.youtube.query.is_empty() && self.youtube.results.is_empty() {
                    self.should_quit = true;
                } else {
                    self.youtube.query.clear();
                    self.youtube.results.clear();
                    self.youtube.selected = 0;
                    self.youtube.loading = false;
                    self.youtube.search_pending_until = None;
                    abort_task(&mut self.youtube.search_task);
                    self.youtube.search_rx = None;
                    if crate::integrations::youtube::install::is_installed() {
                        self.youtube.status = super::YoutubeStatus::Ready;
                    } else {
                        self.youtube.status = super::YoutubeStatus::Idle;
                    }
                }
            }
            KeyCode::Up => {
                if !self.youtube.results.is_empty() {
                    self.youtube.selected =
                        cycle_prev(self.youtube.selected, self.youtube.results.len());
                }
            }
            KeyCode::Down => {
                if !self.youtube.results.is_empty() {
                    self.youtube.selected =
                        cycle_next(self.youtube.selected, self.youtube.results.len());
                }
            }
            KeyCode::Enter => {
                if !self.youtube.results.is_empty() {
                    self.start_youtube_resolve();
                } else if !self.youtube.query.trim().is_empty() {
                    self.start_youtube_search_now();
                } else {
                    self.ensure_youtube_ready();
                }
            }
            KeyCode::Backspace => {
                self.youtube.query.pop();
                self.youtube.selected = 0;
                self.perform_youtube_search();
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.youtube.query.push(c);
                self.youtube.selected = 0;
                self.perform_youtube_search();
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
                let key_str = state
                    .station
                    .as_ref()
                    .map(|s| s.key.as_str())
                    .unwrap_or("unknown");
                match library::save_track(&title, key_str) {
                    library::SaveResult::Saved => {
                        self.saved_tracks = library::load_saved_tracks(key_str);
                        self.save_notice_is_dup = false;
                        self.save_notice = Some(format!("{} {title}", t("notice.saved")));
                    }
                    library::SaveResult::AlreadySaved => {
                        self.save_notice_is_dup = true;
                        self.save_notice = Some(format!("{} {title}", t("notice.already_saved")));
                    }
                }
            }
            KeyCode::Char('p') => {
                let state = self.player.state();
                if state.preview_title.is_some() || state.preview_searching {
                    self.player.send(PlayerCommand::StopPreview).await;
                    self.player
                        .send(PlayerCommand::SetPreviewSearching(false))
                        .await;
                } else if !titles.is_empty() {
                    let raw = titles[self.recent_selected].clone();
                    let cmd_tx = self.player.clone_sender();
                    let _ = cmd_tx.send(PlayerCommand::SetPreviewSearching(true)).await;
                    let _ = cmd_tx
                        .send(PlayerCommand::SetPreviewLoadingTrack(Some(raw.clone())))
                        .await;
                    tokio::spawn(async move {
                        match deezer_preview(&raw).await {
                            Some((url, title)) => {
                                let _ = cmd_tx
                                    .send(PlayerCommand::SetPreviewLoadingTrack(None))
                                    .await;
                                if cmd_tx
                                    .send(PlayerCommand::PlayPreview {
                                        url,
                                        title,
                                        raw_track: raw,
                                    })
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                tokio::time::sleep(std::time::Duration::from_secs(35)).await;
                                let _ = cmd_tx.send(PlayerCommand::StopPreview).await;
                            }
                            None => {
                                tracing::warn!("Deezer: no result for '{raw}'");
                                let _ = cmd_tx
                                    .send(PlayerCommand::SetPreviewLoadingTrack(None))
                                    .await;
                                let _ =
                                    cmd_tx.send(PlayerCommand::SetPreviewSearching(false)).await;
                                let _ = cmd_tx
                                    .send(PlayerCommand::MarkPreviewUnavailable(raw))
                                    .await;
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
