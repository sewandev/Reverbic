use crossterm::event::KeyCode;

use crate::favorites::FavoriteStation;
use crate::i18n::t;
use crate::playlists::{self as playlist_store, RadioPlaylist};

use super::modal::{RadioSubTab, SearchMode};
use super::App;

pub struct PlaylistPicker {
    pub station: Option<FavoriteStation>,
    pub selected: usize,
    pub creating: bool,
    pub input: String,
}

#[derive(Clone)]
pub struct ActivePlaylist {
    pub name: String,
    pub pos: usize,
}

impl App {
    pub(super) fn open_playlist_picker_from_context(&mut self) {
        let station = match self.modal_mode {
            SearchMode::Name if matches!(self.radio_sub_tab, RadioSubTab::Favorites) => {
                self.favorites.get(self.radio_fav_selected).cloned()
            }
            SearchMode::Name if matches!(self.radio_sub_tab, RadioSubTab::Playlists) => None,
            SearchMode::Name | SearchMode::Genre | SearchMode::Country => self
                .search_results
                .get(self.modal_selected)
                .map(|s| FavoriteStation {
                    key: s.key.clone(),
                    name: s.name.clone(),
                    url: s.url.clone(),
                    bitrate_kbps: s.bitrate_kbps,
                    country: s.country.clone(),
                    tags: s.tags.clone(),
                    homepage: s.homepage.clone(),
                }),
            _ => None,
        };
        if let Some(station) = station {
            let creating = self.playlists.is_empty();
            self.playlist_picker = Some(PlaylistPicker {
                station: Some(station),
                selected: 0,
                creating,
                input: String::new(),
            });
        }
    }

    pub(super) fn open_new_playlist_input(&mut self) {
        self.playlist_picker = Some(PlaylistPicker {
            station: None,
            selected: 0,
            creating: true,
            input: String::new(),
        });
    }

    pub(super) fn on_key_playlist_picker(&mut self, key: KeyCode) {
        let Some(mut picker) = self.playlist_picker.take() else {
            return;
        };
        if picker.creating {
            match key {
                KeyCode::Esc => {
                    if picker.station.is_some() && !self.playlists.is_empty() {
                        picker.creating = false;
                        picker.input.clear();
                        self.playlist_picker = Some(picker);
                    }
                }
                KeyCode::Enter => {
                    let name = picker.input.trim().to_string();
                    if name.is_empty() {
                        self.playlist_picker = Some(picker);
                    } else {
                        self.create_playlist_and_add(name, picker.station);
                    }
                }
                KeyCode::Backspace => {
                    picker.input.pop();
                    self.playlist_picker = Some(picker);
                }
                KeyCode::Char(c) if !c.is_control() => {
                    picker.input.push(c);
                    self.playlist_picker = Some(picker);
                }
                _ => self.playlist_picker = Some(picker),
            }
            return;
        }
        let len = self.playlists.len();
        match key {
            KeyCode::Esc => {}
            KeyCode::Up | KeyCode::Char('k') => {
                picker.selected = super::cycle_prev(picker.selected, len + 1);
                self.playlist_picker = Some(picker);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                picker.selected = super::cycle_next(picker.selected, len + 1);
                self.playlist_picker = Some(picker);
            }
            KeyCode::Enter => {
                if picker.selected < len {
                    if let Some(station) = picker.station {
                        self.add_station_to_playlist(picker.selected, station);
                    }
                } else {
                    picker.creating = true;
                    self.playlist_picker = Some(picker);
                }
            }
            _ => self.playlist_picker = Some(picker),
        }
    }

    fn create_playlist_and_add(&mut self, name: String, station: Option<FavoriteStation>) {
        if let Some(idx) = self
            .playlists
            .iter()
            .position(|p| p.name.eq_ignore_ascii_case(&name))
        {
            match station {
                Some(station) => self.add_station_to_playlist(idx, station),
                None => self.playlist_notice(t("notice.playlist.name_taken")),
            }
            return;
        }
        let stations: Vec<FavoriteStation> = station.into_iter().collect();
        let had_station = !stations.is_empty();
        self.playlists.push(RadioPlaylist {
            name: name.clone(),
            stations,
        });
        playlist_store::save(&self.playlists);
        let notice = if had_station {
            t("notice.playlist.added").replace("{}", &name)
        } else {
            t("notice.playlist.created").replace("{}", &name)
        };
        self.playlist_notice(notice);
    }

    fn add_station_to_playlist(&mut self, idx: usize, station: FavoriteStation) {
        let Some(playlist) = self.playlists.get_mut(idx) else {
            return;
        };
        let name = playlist.name.clone();
        if playlist.stations.iter().any(|s| s.url == station.url) {
            self.playlist_notice(t("notice.playlist.duplicate").replace("{}", &name));
            return;
        }
        playlist.stations.push(station);
        playlist_store::save(&self.playlists);
        self.playlist_notice(t("notice.playlist.added").replace("{}", &name));
    }

    pub(super) fn remove_playlist_entry_selected(&mut self) {
        if let Some(pl_idx) = self.radio_open_playlist {
            let Some(playlist) = self.playlists.get_mut(pl_idx) else {
                return;
            };
            if self.radio_playlist_station_selected >= playlist.stations.len() {
                return;
            }
            playlist
                .stations
                .remove(self.radio_playlist_station_selected);
            let max = playlist.stations.len().saturating_sub(1);
            self.radio_playlist_station_selected = self.radio_playlist_station_selected.min(max);
            playlist_store::save(&self.playlists);
            self.playlist_notice(t("notice.playlist.removed"));
            self.keep_radio_playlist_stations_visible();
        } else if self.radio_playlist_selected < self.playlists.len() {
            let removed = self.playlists.remove(self.radio_playlist_selected);
            let max = self.playlists.len().saturating_sub(1);
            self.radio_playlist_selected = self.radio_playlist_selected.min(max);
            if self
                .active_playlist
                .as_ref()
                .is_some_and(|a| a.name == removed.name)
            {
                self.active_playlist = None;
            }
            playlist_store::save(&self.playlists);
            self.playlist_notice(t("notice.playlist.deleted"));
            self.keep_radio_playlists_visible();
        }
    }

    pub(super) fn on_key_rename_playlist(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.renaming_playlist = None;
                self.rename_input.clear();
            }
            KeyCode::Enter => {
                if let Some(idx) = self.renaming_playlist {
                    let new_name = self.rename_input.trim().to_string();
                    if !new_name.is_empty() {
                        if let Some(playlist) = self.playlists.get_mut(idx) {
                            let old_name = playlist.name.clone();
                            playlist.name = new_name.clone();
                            if let Some(active) = self.active_playlist.as_mut() {
                                if active.name == old_name {
                                    active.name = new_name;
                                }
                            }
                            playlist_store::save(&self.playlists);
                        }
                    }
                }
                self.renaming_playlist = None;
                self.rename_input.clear();
            }
            KeyCode::Backspace => {
                self.rename_input.pop();
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.rename_input.push(c);
            }
            _ => {}
        }
    }

    pub(super) fn move_playlist_station(&mut self, pl_idx: usize, idx: usize, delta: i32) {
        let Some(playlist) = self.playlists.get_mut(pl_idx) else {
            return;
        };
        let len = playlist.stations.len();
        let other = if delta > 0 {
            if idx + 1 >= len {
                return;
            }
            idx + 1
        } else {
            if idx == 0 || idx >= len {
                return;
            }
            idx - 1
        };
        playlist.stations.swap(idx, other);
        let name = playlist.name.clone();
        if let Some(active) = self.active_playlist.as_mut() {
            if active.name == name {
                if active.pos == idx {
                    active.pos = other;
                } else if active.pos == other {
                    active.pos = idx;
                }
            }
        }
        playlist_store::save(&self.playlists);
        self.radio_playlist_station_selected = other;
        self.keep_radio_playlist_stations_visible();
    }

    pub(super) async fn play_playlist_station(&mut self, pl_idx: usize, st_idx: usize) {
        let Some(playlist) = self.playlists.get(pl_idx) else {
            return;
        };
        let Some(entry) = playlist.stations.get(st_idx) else {
            return;
        };
        let station = entry.to_station();
        let name = playlist.name.clone();
        self.play_station(station).await;
        self.active_playlist = Some(ActivePlaylist { name, pos: st_idx });
    }

    pub(super) async fn playlist_jump(&mut self, delta: i32) {
        let Some(active) = self.active_playlist.clone() else {
            self.playlist_notice(t("notice.playlist.no_active"));
            return;
        };
        let Some(pl_idx) = self.playlists.iter().position(|p| p.name == active.name) else {
            self.active_playlist = None;
            self.playlist_notice(t("notice.playlist.no_active"));
            return;
        };
        let len = self.playlists[pl_idx].stations.len();
        if len == 0 {
            return;
        }
        let playing_url = self.player.state().station.as_ref().map(|s| s.url.clone());
        let pos = playing_url
            .and_then(|url| {
                self.playlists[pl_idx]
                    .stations
                    .iter()
                    .position(|s| s.url == url)
            })
            .unwrap_or_else(|| active.pos.min(len - 1));
        let next = if delta > 0 {
            (pos + 1) % len
        } else {
            (pos + len - 1) % len
        };
        self.play_playlist_station(pl_idx, next).await;
    }

    fn playlist_notice(&mut self, text: String) {
        self.save_notice_is_dup = false;
        self.save_notice = Some(text);
        self.notice_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(3));
    }
}
