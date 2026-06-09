use crossterm::event::KeyCode;

use crate::favorites::{self, FavoriteStation};
use crate::i18n::t;

use super::App;

impl App {
    pub(super) fn toggle_selected_favorite(&mut self) {
        let fav = self.build_favorite_from_selected().or_else(|| {
            let s = self.player.state().station?;
            Some(crate::favorites::FavoriteStation {
                key: s.key.clone(),
                name: s.name.clone(),
                url: s.url.clone(),
                bitrate_kbps: s.bitrate_kbps,
                country: String::new(),
                tags: Vec::new(),
                homepage: String::new(),
            })
        });
        if let Some(fav) = fav {
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
            self.notice_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(3));
        }
    }

    pub(super) fn toggle_modal_favorite(&mut self) {
        if let Some(station) = self.search_results.get(self.modal_selected) {
            let fav = crate::favorites::FavoriteStation {
                key: station.key.clone(),
                name: station.name.clone(),
                url: station.url.clone(),
                bitrate_kbps: station.bitrate_kbps,
                country: station.country.clone(),
                tags: station.tags.clone(),
                homepage: station.homepage.clone(),
            };
            let added = crate::favorites::toggle(&mut self.favorites, fav);
            crate::favorites::save(&self.favorites);
            self.save_notice_is_dup = false;
            self.save_notice = Some(if added {
                t("notice.fav_added")
            } else {
                t("notice.fav_removed")
            });
            self.notice_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(3));
        }
    }

    pub(super) fn start_favorites_enrichment(&mut self) {
        let to_enrich: Vec<(usize, String, String)> = self
            .favorites
            .iter()
            .enumerate()
            .filter(|(_, f)| f.country.is_empty() && f.tags.is_empty() && f.homepage.is_empty())
            .map(|(i, f)| (i, f.key.clone(), f.name.clone()))
            .collect();

        if to_enrich.is_empty() {
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.fav_enrich_rx = Some(rx);

        let mut snapshot = self.favorites.clone();
        let handle = tokio::spawn(async move {
            for (i, key, name) in to_enrich {
                let details = if crate::station::is_uuid(&key) {
                    crate::station::fetch_station_details(&key).await
                } else {
                    crate::station::fetch_station_details_by_name(&name).await
                };
                if let Some(d) = details {
                    if let Some(f) = snapshot.get_mut(i) {
                        f.country = d.country;
                        f.tags = d.tags;
                        f.homepage = d.homepage;
                    }
                }
            }
            let _ = tx.send(snapshot);
        });
        self.fav_enrich_task = Some(handle);
    }

    pub fn poll_favorites_enrichment(&mut self) {
        if let Some(ref rx) = self.fav_enrich_rx {
            if let Ok(updated) = rx.try_recv() {
                self.apply_favorites_enrichment_result(updated);
                favorites::save(&self.favorites);
                self.fav_enrich_rx = None;
                self.fav_enrich_task = None;
            }
        }
    }

    pub(super) fn apply_favorites_enrichment_result(&mut self, updated: Vec<FavoriteStation>) {
        self.favorites = updated;
    }

    pub(super) fn remove_radio_fav_selected(&mut self) {
        if self.radio_fav_selected < self.favorites.len() {
            self.favorites.remove(self.radio_fav_selected);
            favorites::save(&self.favorites);
            let max = self.favorites.len().saturating_sub(1);
            self.radio_fav_selected = self.radio_fav_selected.min(max);
            self.save_notice_is_dup = false;
            self.save_notice = Some(t("notice.fav_removed"));
            self.notice_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(3));
        }
    }

    pub(super) fn on_key_rename(&mut self, key: KeyCode) {
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
                        favorites::save(&self.favorites);
                    }
                }
                self.renaming_favorite = None;
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
}
