use crate::audio::{PlayerCommand, PlayerStatus};
use crate::config::LastStation;
use crate::library;
use crate::station::{enrich, find_enrichment, Station};

use super::App;

impl App {
    pub(super) async fn adjust_volume(&mut self, delta: f32) {
        let new_vol = (self.player.state().volume + delta).clamp(0.0, 1.0);
        self.player.send(PlayerCommand::SetVolume(new_vol)).await;
        self.config.volume = new_vol;
        self.config.save();
    }

    pub(super) async fn play_station(&mut self, station: Station) {
        if let Some(handle) = &self.spotify.player_tx {
            handle.pause();
        }
        self.stop_playback_polling();
        self.config.last_station = Some(LastStation {
            key:          station.key.clone(),
            name:         station.name.clone(),
            url:          station.url.clone(),
            bitrate_kbps: station.bitrate_kbps,
        });
        self.save_config();
        self.stop_metadata_polling();

        let fade = self.config.crossfade_secs;
        let is_active = matches!(
            self.player.state().status,
            PlayerStatus::Playing | PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_)
        );

        if fade > 0 && is_active {
            self.player.send(PlayerCommand::CrossfadeTo { station: station.clone(), secs: fade }).await;
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
        if index >= self.favorites.len() { return; }
        let station = self.favorites[index].to_station();
        self.play_station(station).await;
    }

    pub(super) async fn play_dynamic_station(&mut self, index: usize) {
        if index >= self.search_results.len() { return; }
        let ds = &self.search_results[index];

        let mut station = Station {
            key:              ds.key.clone(),
            name:             ds.name.clone(),
            url:              ds.url.clone(),
            metadata_api_url: None,
            history_api_url:  None,
            schedule_url:     None,
            show_countdown:   false,
            bitrate_kbps:     ds.bitrate_kbps,
        };

        if let Some(enrichment) = find_enrichment(&station.name) {
            enrich(&mut station, enrichment);
            tracing::info!("Enriquecimiento activado para '{}'", station.name);
        }

        self.play_station(station).await;
    }

    pub fn poll_dead_url(&mut self) {
        let state = self.player.state();
        if state.is_dead_url {
            if let Some(station) = &state.station {
                self.dead_urls.insert(station.url.clone());
            }
        }
    }

    pub(super) fn play_random_result(&self) -> Option<usize> {
        if self.search_results.is_empty() { return None; }
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        Some((ms as usize) % self.search_results.len())
    }
}
