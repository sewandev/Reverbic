use crossterm::event::KeyCode;

use crate::favorites;
use crate::i18n::t;

use super::App;

impl App {
    pub(super) fn toggle_selected_favorite(&mut self) {
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

    pub(super) fn toggle_modal_favorite(&mut self) {
        if let Some(station) = self.search_results.get(self.modal_selected) {
            let fav = crate::favorites::FavoriteStation {
                key:          station.key.clone(),
                name:         station.name.clone(),
                url:          station.url.clone(),
                bitrate_kbps: station.bitrate_kbps,
            };
            let added = crate::favorites::toggle(&mut self.favorites, fav);
            crate::favorites::save(&self.favorites);
            self.save_notice_is_dup = false;
            self.save_notice = Some(if added {
                t("notice.fav_added")
            } else {
                t("notice.fav_removed")
            });
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
            KeyCode::Backspace => { self.rename_input.pop(); }
            KeyCode::Char(c) if !c.is_control() => { self.rename_input.push(c); }
            _ => {}
        }
    }
}
