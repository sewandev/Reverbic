use crate::metadata::track_enrichment::enrich;
use crate::schedule::poll_metadata_loop;
use crate::station::{fetch_station_details, fetch_station_details_by_name, is_uuid};

use super::{abort_task, App};

impl App {
    pub(super) fn start_metadata_polling(
        &mut self,
        url: &'static str,
        history_url: Option<&'static str>,
        schedule_url: Option<&'static str>,
    ) {
        self.stop_metadata_polling();
        let cmd_tx = self.player.clone_sender();
        self.metadata_task = Some(tokio::spawn(poll_metadata_loop(
            url.to_string(),
            history_url.map(str::to_string),
            schedule_url.map(str::to_string),
            cmd_tx,
        )));
    }

    pub(super) fn stop_metadata_polling(&mut self) {
        abort_task(&mut self.metadata_task);
    }

    pub fn trigger_track_enrichment(&mut self, icy_title: String) {
        if self.radio_enriched_for.as_deref() == Some(&icy_title) {
            return;
        }
        self.radio_enriched_for = Some(icy_title.clone());
        self.radio_enriched_track = None;
        abort_task(&mut self.radio_enrichment_task);
        let (tx, rx) = std::sync::mpsc::channel();
        self.radio_enrichment_rx = Some(rx);
        self.radio_enrichment_task = Some(tokio::spawn(async move {
            let result = enrich(&icy_title).await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_track_enrichment(&mut self) {
        if let Some(rx) = self.radio_enrichment_rx.take() {
            match rx.try_recv() {
                Ok(result) => {
                    self.radio_enriched_track = result;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.radio_enrichment_rx = Some(rx);
                }
                Err(_) => {}
            }
        }
    }

    pub fn poll_station_details(&mut self) {
        if let Some(rx) = self.station_details_rx.take() {
            match rx.try_recv() {
                Ok(details) => {
                    self.station_details = Some(details);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.station_details_rx = Some(rx);
                }
                Err(_) => {}
            }
        }

        let current_uuid = self.player.state().station.as_ref().map(|s| s.key.clone());
        if current_uuid == self.last_details_uuid {
            return;
        }

        self.last_details_uuid = current_uuid.clone();
        self.station_details = None;

        if let Some(key) = current_uuid {
            if key.is_empty() || key.starts_with("ondemand_") {
                return;
            }

            let station_name = self
                .player
                .state()
                .station
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_default();

            let (tx, rx) = std::sync::mpsc::channel();
            self.station_details_rx = Some(rx);
            tokio::spawn(async move {
                let fetch_fut = async {
                    if is_uuid(&key) {
                        fetch_station_details(&key).await
                    } else {
                        fetch_station_details_by_name(&station_name).await
                    }
                };
                match tokio::time::timeout(std::time::Duration::from_secs(10), fetch_fut).await {
                    Ok(Some(d)) => {
                        let _ = tx.send(d);
                    }
                    Ok(None) => {}
                    Err(_) => {
                        tracing::warn!("station_details fetch timed out for {key}");
                    }
                }
            });
        }
    }
}
