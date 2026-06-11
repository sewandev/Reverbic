use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

use crate::audio::PlayerCommand;
use crate::integrations::youtube::{
    cookies, install, playlists, quickjs, resolve, runtime_installed, search,
    ResolvedYoutubePlayback, YoutubePlaylist, YoutubeVideo,
};
use crate::station::Station;

use super::{abort_task, App, YoutubeStatus, YoutubeSubTab};

const YOUTUBE_SEARCH_DEBOUNCE: Duration = Duration::from_millis(700);
const LIKED_VIDEOS_LIMIT: usize = 50;
const PLAYLISTS_LIMIT: usize = 50;
const PLAYLIST_VIDEOS_LIMIT: usize = 50;

impl App {
    pub fn ensure_youtube_ready(&mut self) {
        if runtime_installed() {
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
            let result = async {
                install::ensure_installed().await?;
                quickjs::ensure_installed().await
            }
            .await;
            let _ = tx.send(result);
        }));
    }

    pub fn perform_youtube_search(&mut self) {
        self.schedule_youtube_search();
    }

    pub fn schedule_youtube_search(&mut self) {
        let query = self.youtube.query.trim().to_string();
        if query.is_empty() {
            self.youtube.results.clear();
            self.youtube.loading = false;
            self.youtube.selected = 0;
            self.youtube.scroll_offset = 0;
            self.youtube.search_pending_until = None;
            abort_task(&mut self.youtube.search_task);
            self.youtube.search_rx = None;
            return;
        }

        self.youtube.loading = true;
        self.youtube.results.clear();
        self.youtube.selected = 0;
        self.youtube.scroll_offset = 0;
        self.youtube.search_pending_until = Some(Instant::now() + YOUTUBE_SEARCH_DEBOUNCE);
        abort_task(&mut self.youtube.search_task);
        self.youtube.search_rx = None;

        if !runtime_installed() {
            self.ensure_youtube_ready();
        }
    }

    pub fn start_youtube_search_now(&mut self) {
        let query = self.youtube.query.trim().to_string();
        self.youtube.search_pending_until = None;
        if query.is_empty() {
            self.youtube.results.clear();
            self.youtube.loading = false;
            self.youtube.selected = 0;
            self.youtube.scroll_offset = 0;
            abort_task(&mut self.youtube.search_task);
            self.youtube.search_rx = None;
            return;
        }

        if !runtime_installed() {
            self.ensure_youtube_ready();
            return;
        }

        self.youtube.status = YoutubeStatus::Ready;
        self.youtube.loading = true;
        self.youtube.selected = 0;
        self.youtube.scroll_offset = 0;
        abort_task(&mut self.youtube.search_task);
        self.youtube.search_rx = None;

        let binary = install::managed_binary_path();
        let quickjs_path = quickjs::managed_binary_path();
        let cookies_path =
            cookies::configured_cookies_path(self.config.youtube.cookies_path.as_deref());
        let (tx, rx) = mpsc::channel();
        self.youtube.search_rx = Some(rx);
        self.youtube.search_task = Some(tokio::spawn(async move {
            let result =
                search::search_videos(&binary, &query, 20, cookies_path.as_deref(), &quickjs_path)
                    .await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_youtube_search_debounce(&mut self) {
        let Some(pending_until) = self.youtube.search_pending_until else {
            return;
        };

        if Instant::now() >= pending_until {
            self.start_youtube_search_now();
        }
    }

    pub fn poll_youtube_install(&mut self) {
        if let Some(rx) = self.youtube.install_rx.take() {
            match rx.try_recv() {
                Ok(Ok(_)) => {
                    self.youtube.install_task = None;
                    self.youtube.status = YoutubeStatus::Ready;
                    if !self.youtube.query.trim().is_empty() {
                        self.schedule_youtube_search();
                    }
                    match self.youtube.sub_tab {
                        YoutubeSubTab::Liked if self.youtube.liked_videos.is_empty() => {
                            self.fetch_youtube_liked();
                        }
                        YoutubeSubTab::Playlists if self.youtube.playlists.is_empty() => {
                            self.fetch_youtube_playlists();
                        }
                        _ => {}
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
                    self.youtube.scroll_offset = 0;
                }
                Ok(Err(err)) => {
                    self.youtube.search_task = None;
                    self.youtube.loading = false;
                    self.youtube.status = YoutubeStatus::Error(err.to_string());
                    self.youtube.results.clear();
                    self.youtube.selected = 0;
                    self.youtube.scroll_offset = 0;
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
        self.start_youtube_resolve_video(video);
    }

    pub(super) fn start_youtube_resolve_video(&mut self, video: YoutubeVideo) {
        if !runtime_installed() {
            self.ensure_youtube_ready();
            return;
        }

        self.youtube.status = YoutubeStatus::Resolving;
        abort_task(&mut self.youtube.resolve_task);
        self.youtube.resolve_rx = None;

        let binary = install::managed_binary_path();
        let quickjs_path = quickjs::managed_binary_path();
        let cookies_path =
            cookies::configured_cookies_path(self.config.youtube.cookies_path.as_deref());
        let (tx, rx) = mpsc::channel();
        self.youtube.resolve_rx = Some(rx);
        self.youtube.resolve_task = Some(tokio::spawn(async move {
            let result = resolve::resolve_audio_url(
                &binary,
                &video.watch_url,
                cookies_path.as_deref(),
                &quickjs_path,
            )
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

    pub fn fetch_youtube_liked(&mut self) {
        abort_task(&mut self.youtube.liked_task);
        self.youtube.liked_rx = None;
        self.youtube.liked_videos.clear();
        self.youtube.liked_selected = 0;
        self.youtube.liked_scroll_offset = 0;

        let Some(cookies_path) =
            cookies::configured_cookies_path(self.config.youtube.cookies_path.as_deref())
        else {
            return;
        };

        if !runtime_installed() {
            self.ensure_youtube_ready();
            return;
        }

        self.youtube.liked_loading = true;
        let binary = install::managed_binary_path();
        let quickjs_path = quickjs::managed_binary_path();
        let (tx, rx) = mpsc::channel();
        self.youtube.liked_rx = Some(rx);
        self.youtube.liked_task = Some(tokio::spawn(async move {
            let result = playlists::fetch_liked_videos(
                &binary,
                Some(&cookies_path),
                &quickjs_path,
                LIKED_VIDEOS_LIMIT,
            )
            .await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_youtube_liked(&mut self) {
        let Some(rx) = self.youtube.liked_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(videos)) => {
                self.youtube.liked_loading = false;
                self.youtube.liked_videos = videos;
                self.youtube.liked_selected = 0;
                self.youtube.liked_scroll_offset = 0;
            }
            Ok(Err(e)) => {
                self.youtube.liked_loading = false;
                tracing::warn!("youtube liked videos fetch: {e}");
                self.save_notice = Some(format!("YouTube: {e}"));
                self.notice_until = Some(Instant::now() + Duration::from_secs(8));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.youtube.liked_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.youtube.liked_loading = false;
            }
        }
    }

    pub fn fetch_youtube_playlists(&mut self) {
        abort_task(&mut self.youtube.playlists_task);
        self.youtube.playlists_rx = None;
        self.youtube.playlists.clear();
        self.youtube.playlists_selected = 0;
        self.youtube.playlists_scroll_offset = 0;

        let Some(cookies_path) =
            cookies::configured_cookies_path(self.config.youtube.cookies_path.as_deref())
        else {
            return;
        };

        if !runtime_installed() {
            self.ensure_youtube_ready();
            return;
        }

        self.youtube.playlists_loading = true;
        let binary = install::managed_binary_path();
        let quickjs_path = quickjs::managed_binary_path();
        let (tx, rx) = mpsc::channel();
        self.youtube.playlists_rx = Some(rx);
        self.youtube.playlists_task = Some(tokio::spawn(async move {
            let result = playlists::fetch_playlists(
                &binary,
                Some(&cookies_path),
                &quickjs_path,
                PLAYLISTS_LIMIT,
            )
            .await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_youtube_playlists(&mut self) {
        let Some(rx) = self.youtube.playlists_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(playlists)) => {
                self.youtube.playlists_loading = false;
                self.youtube.playlists = playlists;
                self.youtube.playlists_selected = 0;
                self.youtube.playlists_scroll_offset = 0;
            }
            Ok(Err(e)) => {
                self.youtube.playlists_loading = false;
                tracing::warn!("youtube playlists fetch: {e}");
                self.save_notice = Some(format!("YouTube: {e}"));
                self.notice_until = Some(Instant::now() + Duration::from_secs(8));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.youtube.playlists_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.youtube.playlists_loading = false;
            }
        }
    }

    pub fn fetch_youtube_playlist_videos(&mut self, playlist: YoutubePlaylist) {
        abort_task(&mut self.youtube.playlist_videos_task);
        self.youtube.playlist_videos_rx = None;
        self.youtube.playlist_videos.clear();
        self.youtube.playlist_videos_selected = 0;
        self.youtube.playlist_videos_scroll_offset = 0;

        let playlist_id = playlist.id.clone();
        self.youtube.open_playlist = Some(playlist);

        if !runtime_installed() {
            self.ensure_youtube_ready();
            return;
        }

        self.youtube.playlist_videos_loading = true;
        let binary = install::managed_binary_path();
        let quickjs_path = quickjs::managed_binary_path();
        let cookies_path =
            cookies::configured_cookies_path(self.config.youtube.cookies_path.as_deref());
        let (tx, rx) = mpsc::channel();
        self.youtube.playlist_videos_rx = Some(rx);
        self.youtube.playlist_videos_task = Some(tokio::spawn(async move {
            let result = playlists::fetch_playlist_videos(
                &binary,
                cookies_path.as_deref(),
                &quickjs_path,
                &playlist_id,
                PLAYLIST_VIDEOS_LIMIT,
            )
            .await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_youtube_playlist_videos(&mut self) {
        let Some(rx) = self.youtube.playlist_videos_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(videos)) => {
                self.youtube.playlist_videos_loading = false;
                self.youtube.playlist_videos = videos;
                self.youtube.playlist_videos_selected = 0;
                self.youtube.playlist_videos_scroll_offset = 0;
            }
            Ok(Err(e)) => {
                self.youtube.playlist_videos_loading = false;
                tracing::warn!("youtube playlist videos fetch: {e}");
                self.save_notice = Some(format!("YouTube: {e}"));
                self.notice_until = Some(Instant::now() + Duration::from_secs(8));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.youtube.playlist_videos_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.youtube.playlist_videos_loading = false;
            }
        }
    }

    pub fn close_youtube_playlist(&mut self) {
        abort_task(&mut self.youtube.playlist_videos_task);
        self.youtube.playlist_videos_rx = None;
        self.youtube.playlist_videos.clear();
        self.youtube.playlist_videos_selected = 0;
        self.youtube.playlist_videos_scroll_offset = 0;
        self.youtube.playlist_videos_loading = false;
        self.youtube.open_playlist = None;
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

        if video.duration_secs > 0 {
            self.play_station_transient_with_duration(station, video.duration_secs as f32)
                .await;
        } else {
            self.play_station_transient(station).await;
        }
        let _ = self
            .player
            .send(PlayerCommand::ApiMetadata {
                title: video.title,
                artist: video.channel,
                show: "YouTube".to_string(),
                recent: Vec::new(),
            })
            .await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::YOUTUBE_SEARCH_DEBOUNCE;

    #[test]
    fn youtube_search_debounce_waits_after_last_keypress() {
        assert_eq!(YOUTUBE_SEARCH_DEBOUNCE, Duration::from_millis(700));
    }
}
