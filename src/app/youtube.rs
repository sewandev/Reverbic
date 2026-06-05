use std::sync::mpsc;

use crate::audio::PlayerCommand;
use crate::integrations::youtube::{install, resolve, search, ResolvedYoutubePlayback};
use crate::station::Station;

use super::{abort_task, App, YoutubeStatus};

impl App {
    pub fn ensure_youtube_ready(&mut self) {
        if install::is_installed() {
            if !matches!(self.youtube.status, YoutubeStatus::Resolving) {
                self.youtube.status = YoutubeStatus::Ready;
            }
            return;
        }

        if matches!(self.youtube.status, YoutubeStatus::Installing) {
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.youtube.install_rx = Some(rx);
        self.youtube.status = YoutubeStatus::Installing;
        self.youtube.install_task = Some(tokio::spawn(async move {
            let result = install::ensure_installed().await;
            let _ = tx.send(result);
        }));
    }

    pub fn perform_youtube_search(&mut self) {
        let query = self.youtube.query.trim().to_string();
        if query.is_empty() {
            self.youtube.results.clear();
            self.youtube.loading = false;
            self.youtube.selected = 0;
            abort_task(&mut self.youtube.search_task);
            self.youtube.search_rx = None;
            return;
        }

        if !install::is_installed() {
            self.ensure_youtube_ready();
            return;
        }

        self.youtube.status = YoutubeStatus::Ready;
        self.youtube.loading = true;
        self.youtube.selected = 0;
        abort_task(&mut self.youtube.search_task);
        self.youtube.search_rx = None;

        let binary = install::managed_binary_path();
        let (tx, rx) = mpsc::channel();
        self.youtube.search_rx = Some(rx);
        self.youtube.search_task = Some(tokio::spawn(async move {
            let result = search::search_videos(&binary, &query, 20).await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_youtube_install(&mut self) {
        if let Some(rx) = self.youtube.install_rx.take() {
            match rx.try_recv() {
                Ok(Ok(_)) => {
                    self.youtube.install_task = None;
                    self.youtube.status = YoutubeStatus::Ready;
                    if !self.youtube.query.trim().is_empty() {
                        self.perform_youtube_search();
                    }
                }
                Ok(Err(err)) => {
                    self.youtube.install_task = None;
                    self.youtube.loading = false;
                    self.youtube.status = YoutubeStatus::Error(err.to_string());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.youtube.install_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.youtube.install_task = None;
                    self.youtube.loading = false;
                    self.youtube.status =
                        YoutubeStatus::Error(crate::i18n::t("modal.youtube.install_failed"));
                }
            }
        }
    }

    pub fn poll_youtube_search(&mut self) {
        if let Some(rx) = self.youtube.search_rx.take() {
            match rx.try_recv() {
                Ok(Ok(results)) => {
                    self.youtube.search_task = None;
                    self.youtube.loading = false;
                    self.youtube.status = YoutubeStatus::Ready;
                    self.youtube.results = results;
                    self.youtube.selected = 0;
                }
                Ok(Err(err)) => {
                    self.youtube.search_task = None;
                    self.youtube.loading = false;
                    self.youtube.status = YoutubeStatus::Error(err.to_string());
                    self.youtube.results.clear();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.youtube.search_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.youtube.search_task = None;
                    self.youtube.loading = false;
                    self.youtube.status =
                        YoutubeStatus::Error(crate::i18n::t("modal.youtube.search_failed"));
                }
            }
        }
    }

    pub fn start_youtube_resolve(&mut self) {
        let Some(video) = self.youtube.results.get(self.youtube.selected).cloned() else {
            return;
        };
        if !install::is_installed() {
            self.ensure_youtube_ready();
            return;
        }

        self.youtube.status = YoutubeStatus::Resolving;
        abort_task(&mut self.youtube.resolve_task);
        self.youtube.resolve_rx = None;

        let binary = install::managed_binary_path();
        let (tx, rx) = mpsc::channel();
        self.youtube.resolve_rx = Some(rx);
        self.youtube.resolve_task = Some(tokio::spawn(async move {
            let result = resolve::resolve_audio_url(&binary, &video.watch_url)
                .await
                .map(|stream_url| ResolvedYoutubePlayback { video, stream_url });
            let _ = tx.send(result);
        }));
    }

    pub async fn poll_youtube_resolve(&mut self) {
        if let Some(rx) = self.youtube.resolve_rx.take() {
            match rx.try_recv() {
                Ok(Ok(resolved)) => {
                    self.youtube.resolve_task = None;
                    self.youtube.status = YoutubeStatus::Ready;
                    self.play_youtube_resolved(resolved).await;
                }
                Ok(Err(err)) => {
                    self.youtube.resolve_task = None;
                    self.youtube.status = YoutubeStatus::Error(err.to_string());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.youtube.resolve_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.youtube.resolve_task = None;
                    self.youtube.status =
                        YoutubeStatus::Error(crate::i18n::t("modal.youtube.resolve_failed"));
                }
            }
        }
    }

    async fn play_youtube_resolved(&mut self, resolved: ResolvedYoutubePlayback) {
        let video = resolved.video.clone();
        let station = Station {
            key: format!("youtube:{}", resolved.video.id),
            name: resolved.video.title.clone(),
            url: resolved.stream_url,
            metadata_api_url: None,
            history_api_url: None,
            schedule_url: None,
            show_countdown: false,
            bitrate_kbps: None,
        };

        self.play_station_transient(station).await;
        let _ = self
            .player
            .send(PlayerCommand::ApiMetadata {
                title: video.title,
                artist: video.channel,
                show: "YouTube".to_string(),
                recent: Vec::new(),
            })
            .await;
        if video.duration_secs > 0 {
            let _ = self
                .player
                .send(PlayerCommand::SetPlaybackDuration(Some(
                    video.duration_secs as f32,
                )))
                .await;
        }
    }
}
