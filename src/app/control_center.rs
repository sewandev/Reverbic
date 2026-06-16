use crate::audio::PlayerStatus;

use super::App;

pub(super) enum ControlSource {
    Spotify,
    Youtube,
    Radio,
    None,
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
            ControlSource::Spotify | ControlSource::Radio | ControlSource::None => {}
        }
    }

    pub(super) async fn play_previous(&mut self) {
        match self.active_control_source() {
            ControlSource::Youtube => self.youtube_jump(-1),
            ControlSource::Spotify | ControlSource::Radio | ControlSource::None => {}
        }
    }
}
