use crate::audio::{PlayerCommand, PlayerStatus};
use crate::config::LastStation;
use crate::library;
use crate::station::Station;

use super::App;

impl App {
    pub(super) async fn adjust_volume(&mut self, delta: f32) {
        let new_vol = (self.player.state().volume + delta).clamp(0.0, 1.0);
        self.player.send(PlayerCommand::SetVolume(new_vol)).await;
        self.config.volume = new_vol;
        self.config.save();
    }

    pub(super) async fn play_station(&mut self, station: Station) {
        self.play_station_with_persistence(station, true, None)
            .await;
    }

    pub(super) async fn play_station_transient(&mut self, station: Station) {
        self.play_station_with_persistence(station, false, None)
            .await;
    }

    pub(super) async fn play_station_transient_with_duration(
        &mut self,
        station: Station,
        duration_secs: f32,
    ) {
        self.play_station_with_persistence(station, false, Some(duration_secs))
            .await;
    }

    async fn play_station_with_persistence(
        &mut self,
        station: Station,
        persist_last: bool,
        playback_duration_secs: Option<f32>,
    ) {
        self.pause_spotify_for_radio().await;
        self.stop_playback_polling();
        if persist_last {
            self.config.last_station = Some(LastStation::from_station(&station));
            self.save_config();
        }
        self.stop_metadata_polling();

        let is_youtube = station.key.starts_with("youtube:");
        let fade = if is_youtube {
            self.config.youtube_crossfade_secs
        } else {
            self.config.crossfade_secs
        };
        let is_active = matches!(
            self.player.state().status,
            PlayerStatus::Playing | PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_)
        );

        if fade > 0 && is_active && (is_youtube || playback_duration_secs.is_none()) {
            self.player
                .send(PlayerCommand::CrossfadeTo {
                    station: station.clone(),
                    secs: fade,
                    duration_secs: playback_duration_secs.filter(|d| *d > 0.0),
                })
                .await;
        } else if let Some(duration_secs) = playback_duration_secs.filter(|d| *d > 0.0) {
            self.player
                .send(PlayerCommand::PlayWithDuration {
                    station: station.clone(),
                    duration_secs,
                })
                .await;
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

    pub(super) async fn play_favorite_station(&mut self, index: usize) {
        if index >= self.favorites.len() {
            return;
        }
        let station = self.favorites[index].to_station();
        self.active_playlist = None;
        self.radio_context = Some((super::control_center::RadioSource::Favorites, index));
        self.play_station(station).await;
    }

    pub(super) async fn play_dynamic_station(&mut self, index: usize) {
        if index >= self.search_results.len() {
            return;
        }
        let station = self.search_results[index].to_station();
        tracing::info!("Resolved dynamic station '{}'", station.name);

        self.active_playlist = None;
        self.radio_context = Some((super::control_center::RadioSource::Search, index));
        self.play_station(station).await;
    }

    pub fn poll_dead_url(&mut self) {
        let state = self.player.state();
        if state.is_dead_url {
            if let Some(station) = &state.station {
                self.dead_urls.insert(station.url.clone());
                if let Some(video_id) = station.key.strip_prefix("youtube:") {
                    crate::integrations::youtube::resolve::invalidate_cached_url(&format!(
                        "https://www.youtube.com/watch?v={video_id}"
                    ));
                }
            }
        }
    }

    pub(super) fn play_random_result(&self) -> Option<usize> {
        if self.search_results.is_empty() {
            return None;
        }
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        Some((ms as usize) % self.search_results.len())
    }
}
