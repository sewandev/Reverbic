use crate::audio::PlayerStatus;
use crate::i18n::t;

use super::App;

pub(super) enum ControlSource {
    Spotify,
    Youtube,
    Radio,
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum RadioSource {
    Favorites,
    Search,
}

impl App {
    pub(super) fn active_control_source(&self) -> ControlSource {
        if self.active_source_is_spotify() {
            return ControlSource::Spotify;
        }
        let state = self.player.state();
        if matches!(state.status, PlayerStatus::Idle | PlayerStatus::Error(_)) {
            return ControlSource::None;
        }
        match state.station.as_ref() {
            Some(station) if station.key.starts_with("youtube:") => ControlSource::Youtube,
            Some(_) => ControlSource::Radio,
            None => ControlSource::None,
        }
    }

    pub(super) async fn play_next(&mut self) {
        match self.active_control_source() {
            ControlSource::Youtube => self.youtube_jump(1),
            ControlSource::Radio => self.radio_jump(1).await,
            ControlSource::Spotify => {}
            ControlSource::None => self.notify_info(t("notice.control.nothing_playing")),
        }
    }

    pub(super) async fn play_previous(&mut self) {
        match self.active_control_source() {
            ControlSource::Youtube => self.youtube_jump(-1),
            ControlSource::Radio => self.radio_jump(-1).await,
            ControlSource::Spotify => {}
            ControlSource::None => self.notify_info(t("notice.control.nothing_playing")),
        }
    }

    async fn radio_jump(&mut self, delta: i32) {
        if self.active_playlist.is_some() {
            self.playlist_jump(delta).await;
            return;
        }
        let Some((source, stored)) = self.radio_context else {
            self.notify_info(t("notice.control.no_list"));
            return;
        };
        let len = match source {
            RadioSource::Favorites => self.favorites.len(),
            RadioSource::Search => self.search_results.len(),
        };
        if len == 0 {
            self.notify_info(t("notice.control.no_list"));
            return;
        }
        let playing_url = self.player.state().station.as_ref().map(|s| s.url.clone());
        let pos = playing_url
            .and_then(|url| match source {
                RadioSource::Favorites => self.favorites.iter().position(|s| s.url == url),
                RadioSource::Search => self.search_results.iter().position(|s| s.url == url),
            })
            .unwrap_or_else(|| stored.min(len - 1));
        let target = (pos as i32 + delta).clamp(0, len as i32 - 1) as usize;
        if target == pos {
            let key = if delta > 0 {
                "notice.control.end_of_list"
            } else {
                "notice.control.start_of_list"
            };
            self.notify_info(t(key));
            return;
        }
        match source {
            RadioSource::Favorites => self.play_favorite_station(target).await,
            RadioSource::Search => self.play_dynamic_station(target).await,
        }
    }
}
