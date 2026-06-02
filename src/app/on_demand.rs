use crate::station::on_demand;

use super::{abort_task, App};

impl App {
    pub(super) fn start_on_demand_fetch(&mut self) {
        abort_task(&mut self.on_demand_task);
        self.on_demand_rx = None;
        self.on_demand_loading = true;
        self.on_demand_shows.clear();

        let playlist_id = crate::station::on_demand::PROGRAMS
            .get(self.selected_program)
            .map(|p| p.playlist_id)
            .unwrap_or(crate::station::on_demand::PROGRAMS[0].playlist_id);

        let (tx, rx) = std::sync::mpsc::channel();
        self.on_demand_rx = Some(rx);

        let handle = tokio::spawn(async move {
            match tokio::time::timeout(
                std::time::Duration::from_secs(8),
                on_demand::fetch_shows_for_playlist(playlist_id),
            ).await {
                Ok(result) => { let _ = tx.send(result.unwrap_or_default()); }
                Err(_) => {
                    tracing::warn!("on_demand fetch timeout para playlist {playlist_id}");
                    let _ = tx.send(Vec::new());
                }
            }
        });
        self.on_demand_task = Some(handle);
    }

    pub fn poll_on_demand_results(&mut self) {
        if let Some(rx) = self.on_demand_rx.take() {
            match rx.try_recv() {
                Ok(shows) => {
                    self.on_demand_shows = shows;
                    self.on_demand_loading = false;
                    self.on_demand_selected = 0;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.on_demand_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.on_demand_loading = false;
                }
            }
        }
    }
}
