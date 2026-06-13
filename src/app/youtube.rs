use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

use crate::audio::PlayerCommand;
use crate::integrations::youtube::{
    cookies, deno, install, playlists, resolve, runtime_installed, search, sponsorblock,
    ResolvedYoutubePlayback, YoutubePlaylist, YoutubeVideo,
};
use crate::station::Station;

use super::youtube_state::YoutubePlaybackContext;
use super::{abort_task, App, YoutubeStatus, YoutubeSubTab};

const YOUTUBE_SEARCH_DEBOUNCE: Duration = Duration::from_millis(700);
const YOUTUBE_PRERESOLVE_DEBOUNCE: Duration = Duration::from_millis(600);
const LIKED_VIDEOS_LIMIT: usize = 50;
const PLAYLISTS_LIMIT: usize = 50;
const PLAYLIST_VIDEOS_LIMIT: usize = 50;
const MIX_FETCH_LIMIT: usize = 25;
const MIX_EXTEND_THRESHOLD: usize = 3;

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
                deno::ensure_installed().await
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
        let deno_path = deno::managed_binary_path();
        let cookies_path =
            cookies::configured_cookies_path(self.config.youtube.cookies_path.as_deref());
        let (tx, rx) = mpsc::channel();
        self.youtube.search_rx = Some(rx);
        self.youtube.search_task = Some(tokio::spawn(async move {
            let result =
                search::search_videos(&binary, &query, 20, cookies_path.as_deref(), &deno_path)
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

    pub(super) fn play_youtube_from_context(
        &mut self,
        ctx: YoutubePlaybackContext,
        index: usize,
    ) -> bool {
        let list = self.youtube_context_list(&ctx);
        let Some(video) = list.get(index).cloned() else {
            return false;
        };
        if video.is_live {
            self.show_notice(
                crate::app::NoticeSeverity::Warning,
                crate::i18n::t("modal.youtube.live_not_supported"),
                8,
            );
            return false;
        }
        let next_video = list.get(index + 1).cloned();
        let is_mix = matches!(ctx, YoutubePlaybackContext::Mix);
        self.youtube.playback_context = Some((ctx, index));
        self.start_youtube_resolve_video(video, next_video);
        if is_mix {
            self.maybe_extend_youtube_mix(index);
        }
        true
    }

    fn youtube_context_list(&self, ctx: &YoutubePlaybackContext) -> &[YoutubeVideo] {
        match ctx {
            YoutubePlaybackContext::SearchResults => &self.youtube.results,
            YoutubePlaybackContext::Bookmarks => &self.youtube.bookmarks,
            YoutubePlaybackContext::LikedVideos => &self.youtube.liked_videos,
            YoutubePlaybackContext::PlaylistVideos => &self.youtube.playlist_videos,
            YoutubePlaybackContext::Mix => &self.youtube.mix_videos,
        }
    }

    pub fn start_youtube_mix(&mut self) {
        let Some(seed) = self.highlighted_youtube_video() else {
            return;
        };
        if !runtime_installed() {
            self.ensure_youtube_ready();
            return;
        }
        tracing::info!(video_id = %seed.id, title = %seed.title, "youtube: starting mix");
        self.show_notice(
            crate::app::NoticeSeverity::Info,
            format!(
                "{}: {}",
                crate::i18n::t("modal.youtube.mix_starting"),
                seed.title
            ),
            5,
        );
        self.youtube.mix_is_extension = false;
        self.youtube.mix_resume_on_extend = false;
        self.youtube.mix_seed_to_skip = None;
        self.spawn_mix_fetch(seed.id);
    }

    fn maybe_extend_youtube_mix(&mut self, index: usize) {
        if self.youtube.mix_loading {
            return;
        }
        let len = self.youtube.mix_videos.len();
        if len == 0 || index + MIX_EXTEND_THRESHOLD < len {
            return;
        }
        let Some(last) = self.youtube.mix_videos.last() else {
            return;
        };
        tracing::info!(seed = %last.id, "youtube: extending mix queue");
        self.youtube.mix_is_extension = true;
        self.spawn_mix_fetch(last.id.clone());
    }

    fn spawn_mix_fetch(&mut self, seed_id: String) {
        abort_task(&mut self.youtube.mix_task);
        self.youtube.mix_rx = None;
        self.youtube.mix_loading = true;
        let binary = install::managed_binary_path();
        let deno_path = deno::managed_binary_path();
        let (tx, rx) = mpsc::channel();
        self.youtube.mix_rx = Some(rx);
        self.youtube.mix_task = Some(tokio::spawn(async move {
            let result =
                playlists::fetch_mix_videos(&binary, &seed_id, &deno_path, MIX_FETCH_LIMIT).await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_youtube_mix(&mut self) {
        let Some(rx) = self.youtube.mix_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(result) => {
                self.youtube.mix_task = None;
                self.youtube.mix_loading = false;
                match result {
                    Ok(videos) => self.on_mix_videos(videos),
                    Err(e) => {
                        tracing::warn!("youtube: mix fetch failed: {e}");
                        if !self.youtube.mix_is_extension {
                            self.show_notice(crate::app::NoticeSeverity::Error, e.to_string(), 6);
                        }
                    }
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.youtube.mix_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.youtube.mix_task = None;
                self.youtube.mix_loading = false;
            }
        }
    }

    fn on_mix_videos(&mut self, videos: Vec<YoutubeVideo>) {
        if self.youtube.mix_is_extension {
            let existing: std::collections::HashSet<&str> = self
                .youtube
                .mix_videos
                .iter()
                .map(|v| v.id.as_str())
                .collect();
            let new_videos: Vec<YoutubeVideo> = videos
                .into_iter()
                .filter(|v| !existing.contains(v.id.as_str()))
                .collect();
            let added = new_videos.len();
            self.youtube.mix_videos.extend(new_videos);
            tracing::info!(
                added,
                total = self.youtube.mix_videos.len(),
                "youtube: mix queue extended"
            );
            if added > 0 && self.youtube.mix_resume_on_extend {
                self.youtube.mix_resume_on_extend = false;
                self.advance_youtube_playback();
            }
            return;
        }

        if videos.is_empty() {
            self.show_notice(
                crate::app::NoticeSeverity::Error,
                crate::i18n::t("modal.youtube.mix_failed"),
                6,
            );
            return;
        }
        let start_index = match self.youtube.mix_seed_to_skip.take() {
            Some(seed_id) if videos.first().is_some_and(|v| v.id == seed_id) => 1,
            _ => 0,
        };
        tracing::info!(
            count = videos.len(),
            start_index,
            "youtube: mix queue loaded, playing"
        );
        self.youtube.mix_videos = videos;
        self.play_youtube_from_context(YoutubePlaybackContext::Mix, start_index);
    }

    pub(super) fn start_youtube_resolve_video(
        &mut self,
        video: YoutubeVideo,
        next_video: Option<YoutubeVideo>,
    ) {
        if !runtime_installed() {
            self.ensure_youtube_ready();
            return;
        }

        tracing::info!(
            video_id = %video.id,
            title = %video.title,
            "youtube: resolving stream"
        );
        self.youtube.status = YoutubeStatus::Resolving;
        abort_task(&mut self.youtube.resolve_task);
        self.youtube.resolve_rx = None;

        let binary = install::managed_binary_path();
        let deno_path = deno::managed_binary_path();
        let cookies_path_val = self.config.youtube.cookies_path.clone();
        let (tx, rx) = mpsc::channel();
        self.youtube.resolve_rx = Some(rx);
        self.youtube.resolve_task = Some(tokio::spawn(async move {
            let cookies_path = cookies::configured_cookies_path(cookies_path_val.as_deref());
            let result = resolve::resolve_audio_url(
                &binary,
                &video.watch_url,
                cookies_path.as_deref(),
                &deno_path,
            )
            .await
            .map(|(stream_url, headers, chapters)| ResolvedYoutubePlayback {
                video,
                stream_url,
                headers,
                chapters,
            });

            let _ = tx.send(result);

            if let Some(next) = next_video {
                if let Err(e) = resolve::resolve_audio_url(
                    &binary,
                    &next.watch_url,
                    cookies_path.as_deref(),
                    &deno_path,
                )
                .await
                {
                    tracing::warn!(video_id = %next.id, "youtube: pre-resolve of next video failed: {e}");
                }
            }
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
                    tracing::error!("youtube: resolve failed, surfacing to user: {err}");
                    self.youtube.status = YoutubeStatus::Error(err.to_string());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.youtube.resolve_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.youtube.resolve_task = None;
                    tracing::error!("youtube: resolve task ended without sending a result");
                    self.youtube.status =
                        YoutubeStatus::Error(crate::i18n::t("modal.youtube.resolve_failed"));
                }
            }
        }
    }

    pub fn poll_youtube_preresolve(&mut self) {
        let Some(video) = self.highlighted_youtube_video() else {
            self.youtube.preresolve_last_id = None;
            self.youtube.preresolve_deadline = None;
            self.youtube.preresolve_video = None;
            return;
        };

        if self.youtube.preresolve_last_id.as_deref() != Some(video.id.as_str()) {
            self.youtube.preresolve_last_id = Some(video.id.clone());
            self.youtube.preresolve_deadline = Some(Instant::now() + YOUTUBE_PRERESOLVE_DEBOUNCE);
            self.youtube.preresolve_video = Some(video);
            return;
        }

        let Some(deadline) = self.youtube.preresolve_deadline else {
            return;
        };
        if Instant::now() < deadline {
            return;
        }
        if matches!(self.youtube.status, YoutubeStatus::Resolving) {
            self.youtube.preresolve_deadline = Some(Instant::now() + YOUTUBE_PRERESOLVE_DEBOUNCE);
            return;
        }
        self.youtube.preresolve_deadline = None;
        let Some(video) = self.youtube.preresolve_video.take() else {
            return;
        };

        let cookies_path_val = self.config.youtube.cookies_path.clone();
        let configured_cookies = cookies::configured_cookies_path(cookies_path_val.as_deref());
        if !runtime_installed()
            || resolve::is_cached(&video.watch_url, configured_cookies.as_deref())
        {
            return;
        }

        tracing::debug!(video_id = %video.id, "youtube: pre-resolving highlighted video");
        abort_task(&mut self.youtube.preresolve_task);
        let binary = install::managed_binary_path();
        let deno_path = deno::managed_binary_path();
        self.youtube.preresolve_task = Some(tokio::spawn(async move {
            if let Err(e) = resolve::resolve_audio_url(
                &binary,
                &video.watch_url,
                configured_cookies.as_deref(),
                &deno_path,
            )
            .await
            {
                tracing::debug!(video_id = %video.id, "youtube: pre-resolve failed: {e}");
            }
        }));
    }

    fn highlighted_youtube_video(&self) -> Option<YoutubeVideo> {
        if !self.show_search_modal || !matches!(self.modal_mode, super::modal::SearchMode::Youtube)
        {
            return None;
        }
        match self.youtube.sub_tab {
            YoutubeSubTab::Search => self.youtube.results.get(self.youtube.selected).cloned(),
            YoutubeSubTab::Bookmarks => self
                .youtube
                .bookmarks
                .get(self.youtube.bookmarks_selected)
                .cloned(),
            YoutubeSubTab::Liked => self
                .youtube
                .liked_videos
                .get(self.youtube.liked_selected)
                .cloned(),
            YoutubeSubTab::Playlists => {
                if self.youtube.open_playlist.is_some() {
                    self.youtube
                        .playlist_videos
                        .get(self.youtube.playlist_videos_selected)
                        .cloned()
                } else {
                    None
                }
            }
        }
    }

    pub fn poll_youtube_playback(&mut self) {
        let state = self.player.state();
        let station_key = state.station.as_ref().map(|s| s.key.clone());
        let is_youtube = station_key
            .as_deref()
            .is_some_and(|key| key.starts_with("youtube:"));

        if matches!(state.status, crate::audio::PlayerStatus::Idle) {
            let should_advance =
                self.youtube.was_playing && is_youtube && self.youtube.crossfade_from.is_none();
            self.youtube.was_playing = false;
            self.youtube.crossfade_from = None;
            if should_advance {
                self.advance_youtube_playback();
            }
            return;
        }

        if !is_youtube {
            self.youtube.was_playing = false;
            self.youtube.crossfade_from = None;
            return;
        }
        self.youtube.was_playing = true;

        let Some(current_key) = station_key else {
            return;
        };
        if self
            .youtube
            .crossfade_from
            .as_deref()
            .is_some_and(|from| from != current_key)
        {
            self.youtube.crossfade_from = None;
        }

        if !matches!(state.status, crate::audio::PlayerStatus::Playing) {
            return;
        }
        let crossfade_secs = self.config.youtube_crossfade_secs as f32;
        if crossfade_secs <= 0.0 || self.youtube.crossfade_from.is_some() {
            return;
        }
        let (Some(pos), Some(dur)) = (state.playback_pos_secs, state.playback_duration_secs) else {
            return;
        };
        if dur > crossfade_secs * 2.0 && dur - pos <= crossfade_secs {
            tracing::info!(
                station = %current_key,
                pos,
                dur,
                crossfade_secs,
                "youtube: crossfade window reached, advancing early"
            );
            self.youtube.crossfade_from = Some(current_key);
            self.advance_youtube_playback();
        }
    }

    fn advance_youtube_playback(&mut self) {
        let Some((ctx, index)) = self.youtube.playback_context.clone() else {
            tracing::debug!("youtube: track ended without playback context, nothing to advance");
            return;
        };
        tracing::info!(
            ?ctx,
            next_index = index + 1,
            "youtube: auto-advancing playlist"
        );
        if !self.play_youtube_from_context(ctx.clone(), index + 1) {
            if matches!(ctx, YoutubePlaybackContext::Mix) {
                tracing::info!("youtube: mix queue exhausted, waiting for extension");
                self.youtube.mix_resume_on_extend = true;
                self.maybe_extend_youtube_mix(index);
            } else if self.config.youtube_radio_mode {
                let seed = self.youtube_context_list(&ctx).get(index).cloned();
                self.youtube.playback_context = None;
                if let Some(seed) = seed {
                    tracing::info!(
                        seed = %seed.id,
                        "youtube: list ended, radio mode continuing with a mix"
                    );
                    self.show_notice(
                        crate::app::NoticeSeverity::Info,
                        format!(
                            "{}: {}",
                            crate::i18n::t("modal.youtube.mix_starting"),
                            seed.title
                        ),
                        5,
                    );
                    self.youtube.mix_is_extension = false;
                    self.youtube.mix_resume_on_extend = false;
                    self.youtube.mix_seed_to_skip = Some(seed.id.clone());
                    self.spawn_mix_fetch(seed.id);
                }
            } else {
                tracing::info!("youtube: reached end of list, stopping auto-advance");
                self.youtube.playback_context = None;
            }
        }
    }

    fn youtube_library_cookies_path(&mut self) -> Option<PathBuf> {
        let path = self.config.youtube.cookies_path.clone()?;
        match cookies::validate_cookies_path(&path) {
            Ok(valid) => {
                self.youtube.cookies_invalid = false;
                Some(valid)
            }
            Err(err) => {
                if !self.youtube.cookies_invalid {
                    self.notify_error(format!("YouTube: {err}"));
                }
                self.youtube.cookies_invalid = true;
                None
            }
        }
    }

    pub fn fetch_youtube_liked(&mut self) {
        abort_task(&mut self.youtube.liked_task);
        self.youtube.liked_rx = None;
        self.youtube.liked_videos.clear();
        self.youtube.liked_selected = 0;
        self.youtube.liked_scroll_offset = 0;

        let Some(cookies_path) = self.youtube_library_cookies_path() else {
            return;
        };

        if !runtime_installed() {
            self.ensure_youtube_ready();
            return;
        }

        self.youtube.liked_loading = true;
        let binary = install::managed_binary_path();
        let deno_path = deno::managed_binary_path();
        let (tx, rx) = mpsc::channel();
        self.youtube.liked_rx = Some(rx);
        self.youtube.liked_task = Some(tokio::spawn(async move {
            let result = playlists::fetch_liked_videos(
                &binary,
                Some(&cookies_path),
                &deno_path,
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
                self.notify_error(format!("YouTube: {e}"));
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

        let Some(cookies_path) = self.youtube_library_cookies_path() else {
            return;
        };

        if !runtime_installed() {
            self.ensure_youtube_ready();
            return;
        }

        self.youtube.playlists_loading = true;
        let binary = install::managed_binary_path();
        let deno_path = deno::managed_binary_path();
        let (tx, rx) = mpsc::channel();
        self.youtube.playlists_rx = Some(rx);
        self.youtube.playlists_task = Some(tokio::spawn(async move {
            let result = playlists::fetch_playlists(
                &binary,
                Some(&cookies_path),
                &deno_path,
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
                self.notify_error(format!("YouTube: {e}"));
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
        let deno_path = deno::managed_binary_path();
        let cookies_path =
            cookies::configured_cookies_path(self.config.youtube.cookies_path.as_deref());
        let (tx, rx) = mpsc::channel();
        self.youtube.playlist_videos_rx = Some(rx);
        self.youtube.playlist_videos_task = Some(tokio::spawn(async move {
            let result = playlists::fetch_playlist_videos(
                &binary,
                cookies_path.as_deref(),
                &deno_path,
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
                self.notify_error(format!("YouTube: {e}"));
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
        self.start_sponsorblock_fetch(&video.id);
        self.youtube.playing_chapters = resolved.chapters;
        self.youtube.chapter_station_key = Some(format!("youtube:{}", video.id));
        self.youtube.chapter_video_title = video.title.clone();
        self.youtube.chapter_video_channel = video.channel.clone();
        self.youtube.current_chapter = None;
        if !self.youtube.playing_chapters.is_empty() {
            tracing::info!(
                count = self.youtube.playing_chapters.len(),
                "youtube: video has chapters"
            );
        }
        let station = Station {
            key: format!("youtube:{}", resolved.video.id),
            name: resolved.video.title.clone(),
            url: resolved.stream_url,
            metadata_api_url: None,
            history_api_url: None,
            schedule_url: None,
            show_countdown: false,
            bitrate_kbps: None,
            custom_headers: Some(resolved.headers),
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
    pub(super) fn toggle_youtube_bookmark(&mut self) {
        use super::modal::YoutubeSubTab;
        let video = match self.youtube.sub_tab {
            YoutubeSubTab::Search => self.youtube.results.get(self.youtube.selected).cloned(),
            YoutubeSubTab::Bookmarks => self
                .youtube
                .bookmarks
                .get(self.youtube.bookmarks_selected)
                .cloned(),
            YoutubeSubTab::Liked => self
                .youtube
                .liked_videos
                .get(self.youtube.liked_selected)
                .cloned(),
            YoutubeSubTab::Playlists if self.youtube.open_playlist.is_some() => self
                .youtube
                .playlist_videos
                .get(self.youtube.playlist_videos_selected)
                .cloned(),
            YoutubeSubTab::Playlists => None,
        };
        let Some(video) = video else {
            return;
        };
        let added = crate::youtube_bookmarks::toggle(&mut self.youtube.bookmarks, video);
        crate::youtube_bookmarks::save(&self.youtube.bookmarks);
        let max = self.youtube.bookmarks.len().saturating_sub(1);
        self.youtube.bookmarks_selected = self.youtube.bookmarks_selected.min(max);
        self.notify_info(crate::i18n::t(if added {
            "notice.youtube_bookmark_added"
        } else {
            "notice.youtube_bookmark_removed"
        }));
    }

    pub fn validate_youtube_cookies(&mut self) {
        let Some(cookies_path) =
            cookies::configured_cookies_path(self.config.youtube.cookies_path.as_deref())
        else {
            self.show_notice(
                crate::app::NoticeSeverity::Warning,
                crate::i18n::t("modal.youtube.auth_required"),
                4,
            );
            return;
        };

        if !runtime_installed() {
            self.show_notice(
                crate::app::NoticeSeverity::Info,
                crate::i18n::t("modal.youtube.installing"),
                4,
            );
            self.ensure_youtube_ready();
            return;
        }

        self.show_notice(
            crate::app::NoticeSeverity::Info,
            crate::i18n::t("modal.youtube.validating"),
            8,
        );

        self.spawn_youtube_session_check(cookies_path, false);
    }

    pub fn start_youtube_session_health_check(&mut self) {
        if self.youtube.validate_task.is_some() {
            return;
        }
        let Some(cookies_path) =
            cookies::configured_cookies_path(self.config.youtube.cookies_path.as_deref())
        else {
            return;
        };
        if !runtime_installed() {
            return;
        }
        self.spawn_youtube_session_check(cookies_path, true);
    }

    fn spawn_youtube_session_check(&mut self, cookies_path: std::path::PathBuf, silent: bool) {
        abort_task(&mut self.youtube.validate_task);
        self.youtube.validate_silent = silent;
        let binary = install::managed_binary_path();
        let deno_path = deno::managed_binary_path();
        let (tx, rx) = mpsc::channel();
        self.youtube.validate_rx = Some(rx);
        self.youtube.validate_task = Some(tokio::spawn(async move {
            let result =
                playlists::fetch_liked_videos(&binary, Some(&cookies_path), &deno_path, 1).await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_youtube_validate(&mut self) {
        let Some(rx) = self.youtube.validate_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(_)) => {
                self.youtube.validate_task = None;
                self.youtube.session_health = Some(true);
                if !self.youtube.validate_silent {
                    self.show_notice(
                        crate::app::NoticeSeverity::Info,
                        crate::i18n::t("modal.youtube.validate_ok"),
                        5,
                    );
                }
            }
            Ok(Err(e)) => {
                self.youtube.validate_task = None;
                self.youtube.session_health = Some(false);
                tracing::warn!("youtube session check failed: {e}");
                if !self.youtube.validate_silent {
                    self.show_notice(
                        crate::app::NoticeSeverity::Error,
                        format!("{}: {e}", crate::i18n::t("modal.youtube.validate_failed")),
                        8,
                    );
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.youtube.validate_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.youtube.validate_task = None;
            }
        }
    }

    fn show_notice(&mut self, severity: crate::app::NoticeSeverity, message: String, secs: u64) {
        self.notify(severity, message, secs);
    }

    pub fn poll_youtube_chapters(&mut self) {
        if self.youtube.playing_chapters.is_empty() {
            return;
        }
        let state = self.player.state();
        let same_station = state.station.as_ref().map(|s| s.key.as_str())
            == self.youtube.chapter_station_key.as_deref();
        if !same_station {
            self.youtube.playing_chapters.clear();
            self.youtube.chapter_station_key = None;
            self.youtube.current_chapter = None;
            return;
        }
        if !matches!(state.status, crate::audio::PlayerStatus::Playing) {
            return;
        }
        let Some(pos) = state.playback_pos_secs else {
            return;
        };

        let idx = self
            .youtube
            .playing_chapters
            .iter()
            .rposition(|chapter| pos >= chapter.start_secs);
        if idx == self.youtube.current_chapter {
            return;
        }
        self.youtube.current_chapter = idx;
        let Some(idx) = idx else {
            return;
        };
        let chapter_title = self.youtube.playing_chapters[idx].title.clone();
        tracing::info!(chapter = %chapter_title, "youtube: entered chapter");
        let _ = self
            .player
            .clone_sender()
            .try_send(PlayerCommand::ApiMetadata {
                title: self.youtube.chapter_video_title.clone(),
                artist: self.youtube.chapter_video_channel.clone(),
                show: format!("YouTube · {chapter_title}"),
                recent: Vec::new(),
            });
    }

    pub async fn youtube_chapter_jump(&mut self, direction: i32) -> bool {
        if self.youtube.playing_chapters.is_empty() {
            return false;
        }
        let state = self.player.state();
        let same_station = state.station.as_ref().map(|s| s.key.as_str())
            == self.youtube.chapter_station_key.as_deref();
        if !same_station {
            return false;
        }
        let Some(pos) = state.playback_pos_secs else {
            return false;
        };

        let chapters = &self.youtube.playing_chapters;
        let current = chapters.iter().rposition(|c| pos >= c.start_secs);
        let target = if direction > 0 {
            match current {
                Some(idx) => idx + 1,
                None => 0,
            }
        } else {
            match current {
                Some(idx) if pos - chapters[idx].start_secs > 3.0 => idx,
                Some(idx) => idx.saturating_sub(1),
                None => return false,
            }
        };
        let Some(chapter) = chapters.get(target) else {
            return false;
        };
        tracing::info!(chapter = %chapter.title, start = chapter.start_secs, "youtube: chapter jump");
        self.player
            .send(PlayerCommand::Seek(chapter.start_secs))
            .await;
        true
    }

    fn start_sponsorblock_fetch(&mut self, video_id: &str) {
        self.youtube.sb_segments.clear();
        self.youtube.sb_station_key = None;
        self.youtube.sb_cooldown_until = None;
        abort_task(&mut self.youtube.sb_task);
        self.youtube.sb_rx = None;

        if !self.config.youtube_sponsorblock {
            return;
        }

        self.youtube.sb_station_key = Some(format!("youtube:{video_id}"));
        let video_id = video_id.to_string();
        let (tx, rx) = mpsc::channel();
        self.youtube.sb_rx = Some(rx);
        self.youtube.sb_task = Some(tokio::spawn(async move {
            let result = sponsorblock::fetch_music_offtopic_segments(&video_id).await;
            let _ = tx.send(result);
        }));
    }

    pub fn poll_youtube_sponsorblock(&mut self) {
        if let Some(rx) = self.youtube.sb_rx.take() {
            match rx.try_recv() {
                Ok(Ok(segments)) => {
                    self.youtube.sb_task = None;
                    if !segments.is_empty() {
                        tracing::info!(
                            count = segments.len(),
                            "sponsorblock: non-music segments loaded"
                        );
                    }
                    self.youtube.sb_segments = segments;
                }
                Ok(Err(e)) => {
                    self.youtube.sb_task = None;
                    tracing::debug!("sponsorblock: fetch failed: {e}");
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.youtube.sb_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.youtube.sb_task = None;
                }
            }
        }

        if self.youtube.sb_segments.is_empty() {
            return;
        }
        if self
            .youtube
            .sb_cooldown_until
            .is_some_and(|until| Instant::now() < until)
        {
            return;
        }

        let state = self.player.state();
        if !matches!(state.status, crate::audio::PlayerStatus::Playing) {
            return;
        }
        let same_station = state.station.as_ref().map(|s| s.key.as_str())
            == self.youtube.sb_station_key.as_deref();
        if !same_station {
            return;
        }
        let Some(pos) = state.playback_pos_secs else {
            return;
        };

        let Some(&(start, end)) = self
            .youtube
            .sb_segments
            .iter()
            .find(|(start, end)| pos >= *start && pos < *end - 1.0)
        else {
            return;
        };

        tracing::info!(pos, start, end, "sponsorblock: skipping non-music segment");
        self.youtube.sb_cooldown_until = Some(Instant::now() + Duration::from_secs(2));
        let _ = self
            .player
            .clone_sender()
            .try_send(PlayerCommand::Seek(end));
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
