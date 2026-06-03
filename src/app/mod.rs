mod modal;
mod player_ctrl;
mod search;
mod metadata;
mod on_demand;
mod favorites;
mod integrations;
mod input;
mod spotify_state;

pub use modal::{
    AppFocus, SearchMode, SettingItem, SpotifyAuthStatus, SpotifyPlayerStatus,
    settings_items,
};
pub use spotify_state::SpotifyState;

use std::time::Instant;

use crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::audio::{AudioPlayer, PlayerState};
use crate::config::Config;
use crate::favorites::{self as fav_store, FavoriteStation};
use crate::station::{DynamicStation, Station, StationDetails};
use crate::station::on_demand::OnDemandShow;

pub(super) fn cycle_prev(sel: usize, len: usize) -> usize {
    if len == 0 { 0 } else { sel.checked_sub(1).unwrap_or(len - 1) }
}

pub(super) fn cycle_next(sel: usize, len: usize) -> usize {
    if len == 0 { 0 } else { (sel + 1) % len }
}

pub(super) fn handle_filter_list_key(key: KeyCode, filter: &mut String, selected: &mut usize, len: usize) -> bool {
    match key {
        KeyCode::Esc => {
            if !filter.is_empty() { filter.clear(); *selected = 0; }
            else { return true; }
        }
        KeyCode::Up         => *selected = cycle_prev(*selected, len),
        KeyCode::Down       => *selected = cycle_next(*selected, len),
        KeyCode::Backspace  => { filter.pop(); *selected = 0; }
        KeyCode::Char(c) if !c.is_control() => { filter.push(c); *selected = 0; }
        _ => {}
    }
    false
}

pub(super) fn abort_task(task: &mut Option<tokio::task::JoinHandle<()>>) {
    if let Some(h) = task.take() { h.abort(); }
}

pub(super) fn scroll_by(sel: usize, delta: i32, len: usize) -> usize {
    if delta > 0 {
        (sel + delta as usize).min(len.saturating_sub(1))
    } else {
        sel.saturating_sub((-delta) as usize)
    }
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
    pub show_help:              bool,
    pub spotify:               SpotifyState,
    pub radio_enriched_track:  Option<crate::metadata::EnrichedTrack>,
    pub(super) radio_enriched_for:       Option<String>,
    pub(super) radio_enrichment_task:    Option<tokio::task::JoinHandle<()>>,
    pub(super) radio_enrichment_rx:      Option<std::sync::mpsc::Receiver<Option<crate::metadata::EnrichedTrack>>>,
    metadata_task:             Option<tokio::task::JoinHandle<()>>,
    search_task:             Option<tokio::task::JoinHandle<()>>,
    search_result_rx:        Option<std::sync::mpsc::Receiver<Vec<DynamicStation>>>,
    on_demand_task:          Option<tokio::task::JoinHandle<()>>,
    on_demand_rx:            Option<std::sync::mpsc::Receiver<Vec<OnDemandShow>>>,
    station_details_rx:      Option<std::sync::mpsc::Receiver<StationDetails>>,
    last_details_uuid:       Option<String>,
    pub notice_until:        Option<std::time::Instant>,
}

impl App {
    pub async fn new() -> Self {
        let config = Config::load();
        let player = AudioPlayer::spawn();
        let initial_vol = if config.restore_volume { config.volume } else { 1.0 };
        player.send(crate::audio::PlayerCommand::SetVolume(initial_vol)).await;
        player.send(crate::audio::PlayerCommand::SetPrebuffer(config.prebuffer_secs as f32)).await;

        let favorites = fav_store::load();
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
            show_help:                false,
            spotify:                  SpotifyState::default(),
            radio_enriched_track:     None,
            radio_enriched_for:       None,
            radio_enrichment_task:    None,
            radio_enrichment_rx:      None,
            metadata_task:            None,
            search_task:        None,
            search_result_rx:   None,
            on_demand_task:     None,
            on_demand_rx:       None,
            station_details_rx: None,
            last_details_uuid:  None,
            notice_until:       None,
        }
    }

    pub fn screensaver_active(&self) -> bool {
        let secs = self.config.screensaver_secs;
        secs > 0
            && self.show_search_modal
            && self.last_activity.elapsed().as_secs() >= secs as u64
    }

    pub(super) fn total_stations(&self) -> usize {
        self.favorites.len() + self.stations.len() + self.search_results.len()
    }

    pub(super) fn is_favorite_selected(&self) -> bool {
        self.selected < self.favorites.len()
    }

    pub(super) fn favorite_index(&self) -> Option<usize> {
        if self.is_favorite_selected() { Some(self.selected) } else { None }
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

    pub(super) fn save_config(&mut self) {
        self.config.volume = self.player.state().volume;
        self.config.save();
    }

    pub fn player_state(&self) -> PlayerState {
        self.player.state()
    }

    pub fn abort_all_tasks(&mut self) {
        abort_task(&mut self.metadata_task);
        abort_task(&mut self.search_task);
        abort_task(&mut self.on_demand_task);
        self.spotify.cleanup();
    }
}
