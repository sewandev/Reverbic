use std::time::Instant;

use crossterm::event::KeyCode;

use crate::audio::{PlayerCommand, PlayerStatus};
use crate::i18n::t;
use crate::library;
use crate::preview::{deezer_preview, parse_seek_input};
use crate::station::{filter_items, Station, COUNTRIES, GENRES};

use super::{abort_task, cycle_next, cycle_prev, scroll_by, App};
use super::modal::{AppFocus, IntegrationView, SearchMode};
use super::modal::settings_items;

impl App {
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
                SearchMode::Integrations => {}
            }
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
        if key == KeyCode::Enter {
            if let Some(&(tag, label)) = filtered.get(self.genre_selected) {
                self.genre_query = label.to_string();
                self.modal_selected = 0;
                self.perform_genre_search(tag);
            }
            return;
        }
        let len = filtered.len();
        if super::handle_filter_list_key(key, &mut self.genre_filter, &mut self.genre_selected, len) {
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
        if super::handle_filter_list_key(key, &mut self.country_filter, &mut self.country_selected, len) {
            self.modal_mode = SearchMode::Name;
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
        use crate::i18n;
        let Some(&item) = settings_items(self.config.duck_enabled).get(idx) else { return };
        match item {
            super::modal::SettingItem::Autoplay        => self.config.autoplay_last  = !self.config.autoplay_last,
            super::modal::SettingItem::RestoreVolume   => self.config.restore_volume = !self.config.restore_volume,
            super::modal::SettingItem::Crossfade       => self.config.crossfade_next(),
            super::modal::SettingItem::OverlayMode     => self.config.overlay_mode     = self.config.overlay_mode.next(),
            super::modal::SettingItem::OverlayAlpha    => {
                self.config.overlay_alpha = match self.config.overlay_alpha {
                    v if v < 30 => 30,
                    v if v < 50 => 50,
                    v if v < 70 => 70,
                    v if v < 90 => 90,
                    _           => 20,
                };
            }
            super::modal::SettingItem::OverlayPosition => self.config.overlay_position = self.config.overlay_position.next(),
            super::modal::SettingItem::Screensaver     => self.config.screensaver_next(),
            super::modal::SettingItem::DuckEnabled     => self.config.duck_enabled = !self.config.duck_enabled,
            super::modal::SettingItem::DuckVolume      => {
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
            super::modal::SettingItem::MediaKeys       => self.config.media_keys    = !self.config.media_keys,
            super::modal::SettingItem::TrayIcon        => self.config.tray_icon     = !self.config.tray_icon,
            super::modal::SettingItem::Notifications   => self.config.notifications = !self.config.notifications,
            super::modal::SettingItem::Language => {
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
