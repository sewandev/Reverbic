use crate::station::{
    search_stations, search_stations_by_country, search_stations_by_tag, DynamicStation,
};

use super::{abort_task, App};

impl App {
    pub(super) fn perform_search(&mut self) {
        let query = self.search_query.clone();
        if query.trim().is_empty() {
            self.search_results.clear();
            self.search_loading = false;
            self.radio_search_scroll_offset = 0;
            if let Some(t) = self.search_task.take() {
                t.abort();
            }
            return;
        }
        self.spawn_search(move || async move { search_stations(&query, 20).await });
    }

    pub(super) fn perform_genre_search(&mut self, tag: &str) {
        let tag = tag.to_string();
        self.spawn_search(move || async move { search_stations_by_tag(&tag, 20).await });
    }

    pub(super) fn perform_country_search(&mut self, country: &str) {
        let c = country.to_string();
        self.spawn_search(move || async move { search_stations_by_country(&c, 30).await });
    }

    pub(super) fn spawn_search<F, Fut>(&mut self, build: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Option<Vec<DynamicStation>>> + Send + 'static,
    {
        abort_task(&mut self.search_task);
        self.search_result_rx = None;
        self.search_loading = true;

        let existing_urls: std::collections::HashSet<String> =
            self.stations.iter().map(|s| s.url.clone()).collect();
        let (tx, rx) = std::sync::mpsc::channel();
        self.search_result_rx = Some(rx);

        self.search_task = Some(tokio::spawn(async move {
            let filtered: Vec<DynamicStation> = build()
                .await
                .unwrap_or_default()
                .into_iter()
                .filter(|s| !existing_urls.contains(&s.url))
                .collect();
            let _ = tx.send(filtered);
        }));
    }

    pub fn poll_search_results(&mut self) {
        if let Some(rx) = self.search_result_rx.take() {
            match rx.try_recv() {
                Ok(results) => {
                    self.search_results = results;
                    self.search_loading = false;
                    self.radio_search_scroll_offset = 0;
                    self.radio_genre_results_scroll_offset = 0;
                    self.radio_country_results_scroll_offset = 0;
                    let max = self.total_stations();
                    if self.selected >= max && max > 0 {
                        self.selected = max - 1;
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.search_result_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.search_loading = false;
                }
            }
        }
    }
}
