use super::modal::{SearchMode, SpotifyAuthStatus, SpotifyPlayerStatus};
use super::spotify_state::{SpotifyPlaybackBackend, SpotifySearchPage};
use super::{abort_task, App, SpotifyControlTarget};
use crate::config::{Config, SpotifyPlaybackMode};

#[derive(Debug, PartialEq)]
enum SpotifyPlaybackTarget {
    Remote { token: String, device_id: String },
    Native,
    Unavailable(String),
}

fn resolve_spotify_playback_target(
    mode: SpotifyPlaybackMode,
    access_token: Option<&str>,
    active_device_id: Option<&str>,
    has_native_player: bool,
    active_backend: Option<SpotifyPlaybackBackend>,
) -> SpotifyPlaybackTarget {
    match mode {
        SpotifyPlaybackMode::Auto => match active_backend {
            Some(SpotifyPlaybackBackend::Native) if has_native_player => {
                SpotifyPlaybackTarget::Native
            }
            _ => resolve_auto_spotify_target(access_token, active_device_id, has_native_player),
        },
        SpotifyPlaybackMode::Remote => {
            if let Some(device_id) = active_device_id {
                resolve_remote_spotify_target(access_token, device_id)
            } else {
                SpotifyPlaybackTarget::Unavailable(crate::i18n::t(
                    "integrations.spotify.error.no_remote_device",
                ))
            }
        }
        SpotifyPlaybackMode::Native => {
            if has_native_player {
                SpotifyPlaybackTarget::Native
            } else {
                SpotifyPlaybackTarget::Unavailable(crate::i18n::t(
                    "integrations.spotify.error.native_unavailable",
                ))
            }
        }
    }
}

fn resolve_auto_spotify_target(
    access_token: Option<&str>,
    active_device_id: Option<&str>,
    has_native_player: bool,
) -> SpotifyPlaybackTarget {
    if let Some(device_id) = active_device_id {
        resolve_remote_spotify_target(access_token, device_id)
    } else if has_native_player {
        SpotifyPlaybackTarget::Native
    } else {
        SpotifyPlaybackTarget::Unavailable(crate::i18n::t(
            "integrations.spotify.error.playback_unavailable",
        ))
    }
}

fn resolve_remote_spotify_target(
    access_token: Option<&str>,
    device_id: &str,
) -> SpotifyPlaybackTarget {
    match access_token.filter(|token| !token.is_empty()) {
        Some(token) => SpotifyPlaybackTarget::Remote {
            token: token.to_string(),
            device_id: device_id.to_string(),
        },
        None => SpotifyPlaybackTarget::Unavailable(crate::i18n::t(
            "integrations.spotify.error.no_access_token",
        )),
    }
}

fn resolve_active_spotify_device(
    devices: &[crate::integrations::spotify::devices::SpotifyDevice],
    current_active_id: Option<&str>,
) -> (usize, Option<String>) {
    let selected = current_active_id
        .and_then(|id| devices.iter().position(|d| d.id.as_deref() == Some(id)))
        .or_else(|| devices.iter().position(|d| d.is_active && d.id.is_some()))
        .unwrap_or(0);

    let active_device_id = devices.get(selected).and_then(|device| device.id.clone());
    (selected, active_device_id)
}

fn spotify_native_error_message(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    let key = if raw == "native_missing_credentials" {
        "integrations.spotify.error.native_missing_credentials"
    } else if raw.starts_with("native_auth_failed") {
        "integrations.spotify.error.native_auth_failed"
    } else if raw == "native_audio_backend_missing" || lower.contains("audio backend") {
        "integrations.spotify.error.native_audio_backend_missing"
    } else if lower.contains("premium") {
        "integrations.spotify.error.native_premium_required"
    } else if raw.starts_with("native_session_connect") {
        "integrations.spotify.error.native_session_connect"
    } else if raw.starts_with("native_mixer") {
        "integrations.spotify.error.native_mixer"
    } else if raw.starts_with("native_track_unavailable")
        || lower.contains("track unavailable")
        || lower.contains("pista no disponible")
    {
        "integrations.spotify.error.native_track_unavailable"
    } else if raw.starts_with("native_uri_parse") {
        "integrations.spotify.error.native_uri_invalid"
    } else {
        "integrations.spotify.error.native_generic"
    };
    crate::i18n::t(key)
}

fn spotify_native_error_is_fatal(raw: &str) -> bool {
    !(raw.starts_with("native_track_unavailable") || raw.starts_with("native_uri_parse"))
}

fn friendly_spotify_error(raw: &str, client_id: &str) -> String {
    let is_invalid_client = raw.contains("invalid_client")
        || raw.contains("invalid client")
        || raw.contains("access_token");
    if client_id.is_empty() || is_invalid_client {
        crate::i18n::t("integrations.spotify.error.invalid_client")
    } else {
        crate::i18n::t("integrations.spotify.error.generic")
    }
}

impl App {
    pub fn init_integrations(&mut self) {
        let has_session = self.config.spotify.display_name.is_some()
            && self.config.spotify.refresh_token.is_some();
        if !has_session {
            return;
        }
        self.spotify.is_premium = self.config.spotify.is_premium.unwrap_or(false);

        let refresh_token = self
            .config
            .spotify
            .refresh_token
            .clone()
            .expect("checked above");
        let client_id = self.config.spotify.client_id.clone();
        let is_premium_cached = self.config.spotify.is_premium.unwrap_or(false);
        let country_cached = self.config.spotify.country.clone();
        let followers_cached = self.config.spotify.followers;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.auth_rx = Some(rx);
        self.spotify.status = SpotifyAuthStatus::Connecting;
        let handle = tokio::spawn(async move {
            let result = crate::integrations::spotify::oauth::refresh_search_token(
                &client_id,
                &refresh_token,
            )
            .await;
            let auth = match result {
                Ok((access, new_refresh)) => {
                    let username =
                        crate::integrations::spotify::oauth::fetch_username_from_token(&access)
                            .await
                            .unwrap_or_default();
                    crate::integrations::spotify::AuthResult::Success {
                        username,
                        search_token: access,
                        refresh_token: new_refresh,
                        audio_token: String::new(),
                        native_error: None,
                        is_premium: is_premium_cached,
                        country: country_cached,
                        followers: followers_cached,
                    }
                }
                Err(e) => crate::integrations::spotify::AuthResult::Failure(e),
            };
            let _ = tx.send(auth);
        });
        self.spotify.auth_task = Some(handle);
    }

    pub(super) fn start_oauth_flow(&mut self) {
        if self.config.spotify.client_id.is_empty() {
            self.spotify.status =
                SpotifyAuthStatus::Error(crate::i18n::t("config.spotify.no_client_id"));
            return;
        }
        let client_id = self.config.spotify.client_id.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.auth_rx = Some(rx);
        self.spotify.status = SpotifyAuthStatus::Connecting;
        let handle = tokio::spawn(async move {
            let result = crate::integrations::spotify::oauth::start_flow(&client_id).await;
            let _ = tx.send(result);
        });
        self.spotify.auth_task = Some(handle);
    }

    pub(super) fn spotify_logout(&mut self) {
        self.spotify_logout_with_save(|config| config.save());
    }

    pub(super) fn spotify_logout_with_save(&mut self, save_config: impl FnOnce(&Config)) {
        self.config.spotify.display_name = None;
        self.config.spotify.search_token = None;
        self.config.spotify.refresh_token = None;
        self.config.spotify.is_premium = None;
        self.config.spotify.country = None;
        self.config.spotify.followers = None;
        save_config(&self.config);
        self.spotify.cleanup();
        self.spotify = Default::default();
    }

    pub fn poll_spotify_auth(&mut self) {
        use crate::integrations::spotify::AuthResult;
        if let Some(rx) = self.spotify.auth_rx.take() {
            match rx.try_recv() {
                Ok(AuthResult::Success {
                    username,
                    search_token,
                    refresh_token,
                    audio_token,
                    native_error,
                    is_premium,
                    country,
                    followers,
                }) => {
                    if !username.is_empty() {
                        self.config.spotify.display_name = Some(username);
                    }
                    self.config.spotify.search_token = Some(search_token.clone());
                    self.config.spotify.refresh_token = Some(refresh_token);
                    self.config.spotify.is_premium = Some(is_premium);
                    self.spotify.is_premium = is_premium;
                    self.config.spotify.country = country;
                    self.config.spotify.followers = followers;
                    self.config.save();
                    self.spotify.status = SpotifyAuthStatus::LoggedIn;
                    if self.config.spotify.start_on_spotify {
                        self.modal_mode = SearchMode::Spotify;
                    }
                    self.spotify.access_token = Some(search_token);
                    self.spotify.token_refreshed_at = Some(std::time::Instant::now());
                    self.fetch_spotify_devices();
                    self.start_playback_polling();
                    self.configure_spotify_native_player(audio_token, native_error);
                }
                Ok(AuthResult::Failure(msg)) => {
                    if self.config.spotify.display_name.is_some() {
                        self.config.spotify.refresh_token = None;
                        self.config.spotify.search_token = None;
                        self.config.save();
                    }
                    self.spotify.status = SpotifyAuthStatus::Error(friendly_spotify_error(
                        &msg,
                        &self.config.spotify.client_id,
                    ));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.spotify.auth_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.spotify.status = SpotifyAuthStatus::Error(crate::i18n::t(
                        "integrations.spotify.error.generic",
                    ));
                }
            }
        }
    }

    fn configure_spotify_native_player(
        &mut self,
        audio_token: String,
        native_error: Option<String>,
    ) {
        self.spotify.player_tx = None;
        self.spotify.player_rx = None;
        if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Native) {
            self.spotify.active_backend = None;
        }

        let has_audio_token = !audio_token.trim().is_empty();
        let has_cached_credentials = crate::integrations::spotify::player::has_cached_credentials();

        if has_audio_token || has_cached_credentials {
            let (evt_tx, evt_rx) = std::sync::mpsc::sync_channel(32);
            let handle = crate::integrations::spotify::player::spawn_player(audio_token, evt_tx);
            self.spotify.player_tx = Some(handle);
            self.spotify.player_rx = Some(evt_rx);
            self.spotify.native_available = true;
            self.spotify.native_error = None;
            return;
        }

        let raw_error = native_error.unwrap_or_else(|| "native_missing_credentials".to_string());
        let message = spotify_native_error_message(&raw_error);
        tracing::warn!("spotify native player unavailable: {raw_error}");
        self.spotify.native_available = false;
        self.spotify.native_error = Some(message.clone());
        if self.config.spotify.playback_mode == SpotifyPlaybackMode::Native {
            self.spotify.player_status = SpotifyPlayerStatus::Error(message.clone());
            self.save_notice = Some(format!("Spotify: {message}"));
            self.save_notice_is_dup = false;
            self.notice_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(8));
        }
    }

    pub fn poll_spotify_player_events(&mut self) {
        use crate::integrations::spotify::SpotifyPlayerEvent;

        let mut events: Vec<Result<SpotifyPlayerEvent, std::sync::mpsc::TryRecvError>> = vec![];
        let mut disconnected = false;
        {
            let Some(rx) = self.spotify.player_rx.as_ref() else {
                return;
            };
            loop {
                match rx.try_recv() {
                    Ok(evt) => events.push(Ok(evt)),
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(e) => {
                        disconnected = true;
                        let _ = e;
                        break;
                    }
                }
            }
        }

        if disconnected {
            self.spotify.player_rx = None;
        }

        for evt in events {
            let Ok(evt) = evt else { continue };
            match evt {
                SpotifyPlayerEvent::Playing => {
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        continue;
                    }
                    self.spotify.native_available = true;
                    self.spotify.native_error = None;
                    self.spotify.active_backend = Some(SpotifyPlaybackBackend::Native);
                    self.spotify.player_status = SpotifyPlayerStatus::Playing;
                }
                SpotifyPlayerEvent::Paused => {
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        continue;
                    }
                    self.spotify.active_backend = Some(SpotifyPlaybackBackend::Native);
                    self.spotify.player_status = SpotifyPlayerStatus::Paused
                }
                SpotifyPlayerEvent::TrackChanged(track) => {
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        continue;
                    }
                    let uri = track.uri.clone();
                    self.spotify.native_available = true;
                    self.spotify.native_error = None;
                    self.spotify.now_playing = Some(track);
                    self.spotify.active_backend = Some(SpotifyPlaybackBackend::Native);
                    self.spotify.player_status = SpotifyPlayerStatus::Playing;
                    self.spotify.recently_played.push_back(uri);
                    if self.spotify.recently_played.len() > 30 {
                        self.spotify.recently_played.pop_front();
                    }
                }
                SpotifyPlayerEvent::Stopped => {
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        continue;
                    }
                    tracing::debug!(
                        "spotify event: Stopped (queue={})",
                        self.spotify.playback_queue.len()
                    );
                    self.spotify.player_status = SpotifyPlayerStatus::Idle;
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Native) {
                        self.spotify.active_backend = None;
                    }
                    if self.spotify.playback_queue.is_empty() && self.spotify.radio_rx.is_none() {
                        self.spotify.now_playing = None;
                    }
                }
                SpotifyPlayerEvent::EndOfTrack => {
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        continue;
                    }
                    tracing::debug!(
                        "spotify event: EndOfTrack (queue={})",
                        self.spotify.playback_queue.len()
                    );
                    self.advance_playback_queue();
                }
                SpotifyPlayerEvent::Error(e) => {
                    tracing::warn!("spotify native playback error: {e}");
                    let message = spotify_native_error_message(&e);
                    if spotify_native_error_is_fatal(&e) {
                        self.spotify.native_available = false;
                        self.spotify.native_error = Some(message.clone());
                    }
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        continue;
                    }
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Native) {
                        self.spotify.active_backend = None;
                    }
                    self.spotify.player_status = SpotifyPlayerStatus::Error(message.clone());
                    if self.config.spotify.playback_mode == SpotifyPlaybackMode::Native {
                        self.save_notice = Some(format!("Spotify: {message}"));
                        self.save_notice_is_dup = false;
                        self.notice_until =
                            Some(std::time::Instant::now() + std::time::Duration::from_secs(8));
                    }
                }
            }
        }
    }

    fn fetch_radio_tracks(&mut self, artist_name: String, seed_uri: String) {
        use crate::integrations::spotify::radio::fetch_radio_pool;
        abort_task(&mut self.spotify.radio_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        let recently_played = self.spotify.recently_played.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.radio_rx = Some(rx);
        let handle = tokio::spawn(async move {
            match fetch_radio_pool(&artist_name, &seed_uri, recently_played, &token).await {
                Ok(tracks) => {
                    let _ = tx.send(tracks);
                }
                Err(e) => {
                    tracing::debug!("radio fetch: {e}");
                    let _ = tx.send(vec![]);
                }
            }
        });
        self.spotify.radio_task = Some(handle);
    }

    pub fn poll_spotify_radio(&mut self) {
        let rx = match self.spotify.radio_rx.take() {
            Some(r) => r,
            None => return,
        };
        match rx.try_recv() {
            Ok(mut tracks) => {
                self.spotify.radio_queue.extend(tracks.drain(..));
                self.play_next_radio_track();
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.spotify.radio_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
        }
    }

    fn play_next_radio_track(&mut self) {
        let Some(track) = self.spotify.radio_queue.pop_front() else {
            return;
        };
        let Some(handle) = &self.spotify.player_tx else {
            return;
        };
        handle.play(vec![track.uri.clone()]);
        self.spotify.now_playing = Some(track);
        self.spotify.player_status = SpotifyPlayerStatus::Loading;
    }

    pub(super) fn perform_spotify_search(&mut self) {
        use crate::integrations::spotify::{search::search_tracks, SpotifyError};
        self.reset_spotify_search_paging();
        abort_task(&mut self.spotify.search_task);
        self.spotify.search_rx = None;
        self.spotify.search_generation = self.spotify.search_generation.wrapping_add(1).max(1);
        if let Some(until) = self.spotify.rate_limited_until {
            if std::time::Instant::now() < until {
                self.spotify.search_loading = false;
                return;
            }
            self.spotify.rate_limited_until = None;
            self.spotify.search_rate_limited = false;
        }
        self.spotify.search_rate_limited = false;
        let query = self.spotify.search_query.clone();
        let generation = self.spotify.search_generation;
        let Some(token) = self.spotify.access_token.clone() else {
            self.spotify.search_loading = false;
            return;
        };
        if query.is_empty() {
            self.spotify.search_results.clear();
            self.spotify.search_loading = false;
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.search_rx = Some(rx);
        self.spotify.search_loading = true;
        let handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            let (results, has_more, rate_limit_secs) = match search_tracks(&query, &token, 0).await
            {
                Ok((r, more)) => (r, more, None),
                Err(SpotifyError::RateLimit(secs)) => (vec![], false, Some(secs)),
                Err(e) => {
                    tracing::warn!("spotify search: {e}");
                    (vec![], false, None)
                }
            };
            let _ = tx.send(SpotifySearchPage {
                generation,
                query,
                offset: 0,
                results,
                has_more,
                rate_limit_secs,
            });
        });
        self.spotify.search_task = Some(handle);
    }

    pub(super) fn reset_spotify_search_paging(&mut self) {
        abort_task(&mut self.spotify.search_more_task);
        self.spotify.search_more_rx = None;
        self.spotify.search_loading_more = false;
        self.spotify.search_has_more = false;
        self.spotify.search_offset = 0;
    }

    pub(super) fn fetch_spotify_devices(&mut self) {
        use crate::integrations::spotify::devices::list_devices;
        abort_task(&mut self.spotify.devices_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.devices_rx = Some(rx);
        self.spotify.devices_loading = true;
        let handle = tokio::spawn(async move {
            let result: Result<_, crate::integrations::spotify::SpotifyError> =
                list_devices(&token).await;
            let _ = tx.send(result);
        });
        self.spotify.devices_task = Some(handle);
    }

    pub fn poll_spotify_devices(&mut self) {
        use crate::integrations::spotify::SpotifyError;
        if let Some(rx) = self.spotify.devices_rx.take() {
            match rx.try_recv() {
                Ok(Ok(devices)) => {
                    self.spotify.devices = devices
                        .into_iter()
                        .filter(|d| d.name.to_lowercase() != "reverbic")
                        .collect();
                    self.spotify.devices_loading = false;

                    let (selected, active_device_id) = resolve_active_spotify_device(
                        &self.spotify.devices,
                        self.spotify.active_device_id.as_deref(),
                    );
                    self.spotify.devices_selected = selected;
                    self.spotify.active_device_id = active_device_id;
                }
                Ok(Err(e)) => {
                    tracing::warn!("spotify devices: {e}");
                    self.spotify.devices_loading = false;
                    if !matches!(e, SpotifyError::RateLimit(_)) {
                        self.save_notice = Some(format!("Spotify devices: {e}"));
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.spotify.devices_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.spotify.devices_loading = false;
                }
            }
        }
    }

    pub async fn adjust_spotify_volume(&mut self, delta: i8) {
        use crate::integrations::spotify::devices::set_volume;
        let (token, device_id) = match self.spotify_control_target() {
            SpotifyControlTarget::Remote { token, device_id } => (token, device_id),
            SpotifyControlTarget::Native | SpotifyControlTarget::None => return,
        };
        let current = self
            .spotify
            .playback
            .as_ref()
            .map(|p| p.volume_pct)
            .unwrap_or(50);
        let new_vol = (current as i16 + delta as i16).clamp(0, 100) as u8;
        if let Some(ref mut p) = self.spotify.playback {
            p.volume_pct = new_vol;
        }
        self.spotify.volume_pending_until =
            Some(std::time::Instant::now() + std::time::Duration::from_secs(4));
        tokio::spawn(async move {
            if let Err(e) = set_volume(&token, &device_id, new_vol).await {
                tracing::warn!("spotify set_volume error: {e}");
            }
        });
    }

    pub async fn transfer_to_spotify_device(&mut self, device_id: String) {
        use crate::integrations::spotify::devices::transfer_playback;
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        match transfer_playback(&token, &device_id).await {
            Ok(()) => {
                if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Native) {
                    if let Some(handle) = &self.spotify.player_tx {
                        handle.pause();
                    }
                }
                self.spotify.active_device_id = Some(device_id.clone());
                self.spotify.active_backend = Some(SpotifyPlaybackBackend::Remote);
                for d in self.spotify.devices.iter_mut() {
                    d.is_active = d.id.as_deref() == Some(&device_id);
                }
                self.start_playback_polling();
                tracing::info!("spotify: active device -> {device_id}");
            }
            Err(e) => {
                tracing::warn!("transfer playback: {e}");
                self.save_notice = Some(format!("Spotify: {e}"));
            }
        }
    }

    pub fn poll_token_refresh(&mut self) {
        if self.spotify.token_refresh_rx.is_none() {
            if let Some(refreshed_at) = self.spotify.token_refreshed_at {
                if refreshed_at.elapsed() >= std::time::Duration::from_secs(55 * 60) {
                    let Some(refresh_token) = self.config.spotify.refresh_token.clone() else {
                        return;
                    };
                    let client_id = self.config.spotify.client_id.clone();
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.spotify.token_refresh_rx = Some(rx);
                    self.spotify.token_refreshed_at = None;
                    let handle = tokio::spawn(async move {
                        let result = crate::integrations::spotify::oauth::refresh_search_token(
                            &client_id,
                            &refresh_token,
                        )
                        .await;
                        let _ = tx.send(result);
                    });
                    self.spotify.token_refresh_task = Some(handle);
                }
            }
        }
        if let Some(rx) = self.spotify.token_refresh_rx.take() {
            match rx.try_recv() {
                Ok(Ok((new_access, new_refresh))) => {
                    self.spotify.access_token = Some(new_access.clone());
                    self.config.spotify.search_token = Some(new_access);
                    self.config.spotify.refresh_token = Some(new_refresh);
                    self.config.save();
                    self.spotify.token_refreshed_at = Some(std::time::Instant::now());
                    self.start_playback_polling();
                    tracing::info!("spotify: access_token refreshed");
                }
                Ok(Err(e)) => {
                    tracing::warn!("spotify token refresh failed: {e}");
                    self.spotify.token_refreshed_at =
                        Some(std::time::Instant::now() - std::time::Duration::from_secs(50 * 60));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.spotify.token_refresh_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.spotify.token_refreshed_at =
                        Some(std::time::Instant::now() - std::time::Duration::from_secs(50 * 60));
                }
            }
        }
    }

    pub fn poll_spotify_play_result(&mut self) {
        use crate::integrations::spotify::SpotifyError;
        if let Some(rx) = self.spotify.play_result_rx.take() {
            match rx.try_recv() {
                Ok(Ok(())) => {
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        self.spotify.player_status = SpotifyPlayerStatus::Playing;
                    }
                }
                Ok(Err(SpotifyError::Unauthorized)) => {
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        self.spotify.active_backend = None;
                    }
                    self.spotify.player_status = SpotifyPlayerStatus::Error(crate::i18n::t(
                        "integrations.spotify.error.generic",
                    ));
                    tracing::warn!("spotify play: token expirado, renovando");
                    self.spotify.token_refreshed_at =
                        Some(std::time::Instant::now() - std::time::Duration::from_secs(60 * 60));
                }
                Ok(Err(e)) => {
                    tracing::warn!("spotify play_on_device: {e}");
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        self.spotify.active_backend = None;
                    }
                    self.spotify.player_status = SpotifyPlayerStatus::Error(crate::i18n::t(
                        "integrations.spotify.error.generic",
                    ));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.spotify.play_result_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
            }
        }
    }

    pub fn poll_spotify_search(&mut self) {
        if let Some(rx) = self.spotify.search_rx.take() {
            match rx.try_recv() {
                Ok(page) => {
                    if page.generation != self.spotify.search_generation
                        || page.query != self.spotify.search_query
                        || page.offset != 0
                    {
                        self.spotify.search_loading = false;
                        return;
                    }
                    let rate_limited = page.rate_limit_secs.is_some();
                    self.spotify.search_rate_limited = rate_limited;
                    if let Some(secs) = page.rate_limit_secs {
                        self.spotify.rate_limited_until =
                            Some(std::time::Instant::now() + std::time::Duration::from_secs(secs));
                    }
                    self.spotify.search_has_more = !rate_limited && page.has_more;
                    self.spotify.search_results = page.results;
                    self.spotify.search_loading = false;
                    self.spotify.search_selected = 0;
                    self.spotify.search_offset = page.offset;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.spotify.search_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.spotify.search_loading = false;
                }
            }
        }
    }

    pub fn load_more_spotify_results(&mut self) {
        use crate::integrations::spotify::search::search_tracks;
        abort_task(&mut self.spotify.search_more_task);
        let query = self.spotify.search_query.clone();
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        if query.is_empty() || !self.spotify.search_has_more {
            return;
        }
        let offset = self.spotify.search_offset + 10;
        let generation = self.spotify.search_generation;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.search_more_rx = Some(rx);
        self.spotify.search_has_more = false;
        self.spotify.search_loading_more = true;
        let handle = tokio::spawn(async move {
            let result = match search_tracks(&query, &token, offset).await {
                Ok((r, more)) => (r, more, None),
                Err(e) => {
                    tracing::warn!("spotify load_more failed: {e}");
                    (vec![], false, None)
                }
            };
            let (results, has_more, rate_limit_secs) = result;
            let _ = tx.send(SpotifySearchPage {
                generation,
                query,
                offset,
                results,
                has_more,
                rate_limit_secs,
            });
        });
        self.spotify.search_more_task = Some(handle);
    }

    pub fn poll_spotify_search_more(&mut self) {
        if let Some(rx) = self.spotify.search_more_rx.take() {
            match rx.try_recv() {
                Ok(page) => {
                    let expected_offset = self.spotify.search_offset + 10;
                    if page.generation != self.spotify.search_generation
                        || page.query != self.spotify.search_query
                        || page.offset != expected_offset
                    {
                        self.spotify.search_loading_more = false;
                        return;
                    }
                    self.spotify.search_offset = page.offset;
                    self.spotify.search_has_more = page.has_more;
                    self.spotify.search_loading_more = false;
                    self.spotify.search_results.extend(page.results);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.spotify.search_more_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.spotify.search_loading_more = false;
                }
            }
        }
    }

    pub fn start_playback_polling(&mut self) {
        use crate::integrations::spotify::devices::get_playback;
        abort_task(&mut self.spotify.playback_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.playback_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let mut delay_secs = 2u64;
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                match get_playback(&token).await {
                    Ok(state) => {
                        delay_secs = 2;
                        if tx.send(state).is_err() {
                            break;
                        }
                    }
                    Err(crate::integrations::spotify::SpotifyError::RateLimit(_)) => {
                        delay_secs = (delay_secs * 2).min(300);
                        tracing::debug!("playback poll: rate limited, backoff {}s", delay_secs);
                    }
                    Err(e) => {
                        tracing::debug!("playback poll: {e}");
                    }
                }
            }
        });
        self.spotify.playback_task = Some(handle);
    }

    pub fn stop_playback_polling(&mut self) {
        abort_task(&mut self.spotify.playback_task);
        self.spotify.playback_rx = None;
        self.spotify.playback = None;
        if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
            self.spotify.active_backend = None;
        }
    }

    pub async fn play_spotify_track_with_queue(
        &mut self,
        track: crate::integrations::spotify::SpotifyTrack,
        queue: Vec<crate::integrations::spotify::SpotifyTrack>,
    ) {
        use crate::audio::PlayerCommand;
        self.player.send(PlayerCommand::Stop).await;
        self.spotify.playback_queue = queue.into_iter().collect();
        self.spotify.radio_queue.clear();
        abort_task(&mut self.spotify.radio_task);
        tracing::debug!(
            "play_spotify_track_with_queue: track='{}' queue_len={}",
            track.name,
            self.spotify.playback_queue.len()
        );
        let target = resolve_spotify_playback_target(
            self.config.spotify.playback_mode,
            self.spotify.access_token.as_deref(),
            self.spotify.active_device_id.as_deref(),
            self.spotify.native_available && self.spotify.player_tx.is_some(),
            self.spotify.active_backend,
        );
        match target {
            SpotifyPlaybackTarget::Remote { token, device_id } => {
                if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Native) {
                    if let Some(handle) = &self.spotify.player_tx {
                        handle.pause();
                    }
                }
                let mut uris: Vec<String> = std::iter::once(track.uri.clone())
                    .chain(self.spotify.playback_queue.iter().map(|t| t.uri.clone()))
                    .collect();
                uris.truncate(100);
                self.spotify.now_playing = Some(track);
                self.spotify.active_backend = Some(SpotifyPlaybackBackend::Remote);
                self.spotify.player_status = SpotifyPlayerStatus::Loading;
                let (tx, rx) = std::sync::mpsc::channel();
                self.spotify.play_result_rx = Some(rx);
                tokio::spawn(async move {
                    let result = crate::integrations::spotify::devices::play_tracks_on_device(
                        &token, &device_id, uris,
                    )
                    .await;
                    let _ = tx.send(result);
                });
                self.start_playback_polling();
            }
            SpotifyPlaybackTarget::Native => {
                let remote_to_pause =
                    if self.spotify.active_backend == Some(SpotifyPlaybackBackend::Remote) {
                        self.spotify
                            .access_token
                            .clone()
                            .zip(self.spotify.active_device_id.clone())
                    } else {
                        None
                    };
                if let Some((token, device_id)) = remote_to_pause {
                    if let Err(e) =
                        crate::integrations::spotify::devices::pause_device(&token, &device_id)
                            .await
                    {
                        tracing::warn!("spotify pause before native playback error: {e}");
                    }
                }
                self.spotify.play_result_rx = None;
                let Some(handle) = &self.spotify.player_tx else {
                    let message = crate::i18n::t("integrations.spotify.error.native_unavailable");
                    self.spotify.player_status = SpotifyPlayerStatus::Error(message.clone());
                    self.save_notice = Some(format!("Spotify: {message}"));
                    self.save_notice_is_dup = false;
                    return;
                };
                handle.play(vec![track.uri.clone()]);
                self.spotify.now_playing = Some(track);
                self.spotify.active_backend = Some(SpotifyPlaybackBackend::Native);
                self.spotify.player_status = SpotifyPlayerStatus::Loading;
            }
            SpotifyPlaybackTarget::Unavailable(mut message) => {
                if self.config.spotify.playback_mode == SpotifyPlaybackMode::Native {
                    if let Some(native_error) = self.spotify.native_error.clone() {
                        message = native_error;
                    }
                }
                tracing::warn!("spotify playback unavailable: {message}");
                self.spotify.player_status = SpotifyPlayerStatus::Error(message.clone());
                self.save_notice = Some(format!("Spotify: {message}"));
                self.save_notice_is_dup = false;
            }
        }
    }

    fn advance_playback_queue(&mut self) {
        tracing::debug!(
            "advance_playback_queue: queue_len={}",
            self.spotify.playback_queue.len()
        );
        if let Some(next) = self.spotify.playback_queue.pop_front() {
            tracing::debug!("advance_playback_queue: playing next '{}'", next.name);
            if let Some(handle) = &self.spotify.player_tx {
                handle.play(vec![next.uri.clone()]);
                self.spotify.now_playing = Some(next);
                self.spotify.player_status = SpotifyPlayerStatus::Loading;
            } else {
                tracing::debug!("advance_playback_queue: no player handle");
            }
        } else if self.config.spotify.radio_enabled {
            if !self.spotify.radio_queue.is_empty() {
                self.play_next_radio_track();
            } else {
                let artist_name = self
                    .spotify
                    .now_playing
                    .as_ref()
                    .map(|t| t.artist.clone())
                    .filter(|a| !a.is_empty());
                let seed_uri = self
                    .spotify
                    .now_playing
                    .as_ref()
                    .map(|t| t.uri.clone())
                    .unwrap_or_default();
                if let Some(name) = artist_name {
                    self.fetch_radio_tracks(name, seed_uri);
                }
            }
        }
    }

    pub fn fetch_liked_tracks(&mut self) {
        use crate::integrations::spotify::library::get_saved_tracks;
        if let Some(until) = self.spotify.liked_rate_limited_until {
            if std::time::Instant::now() < until {
                return;
            }
            self.spotify.liked_rate_limited_until = None;
        }
        abort_task(&mut self.spotify.liked_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        self.spotify.liked_tracks.clear();
        self.spotify.liked_selected = 0;
        self.spotify.liked_scroll_offset = 0;
        self.spotify.liked_loading = true;
        self.spotify.liked_offset = 0;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.liked_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_saved_tracks(&token, 0).await);
        });
        self.spotify.liked_task = Some(handle);
    }

    pub fn poll_liked_tracks(&mut self) {
        use crate::integrations::spotify::SpotifyError;

        let Some(rx) = self.spotify.liked_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok((tracks, has_more))) => {
                self.spotify.liked_loading = false;
                self.spotify.liked_tracks.extend(tracks);
                self.spotify.liked_has_more = has_more;
                self.spotify.liked_offset += 50;
            }
            Ok(Err(e)) => {
                self.spotify.liked_loading = false;
                tracing::warn!("liked tracks fetch: {e}");
                if let SpotifyError::RateLimit(secs) = e {
                    self.spotify.liked_rate_limited_until =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(secs));
                } else {
                    self.save_notice = Some(format!("Spotify liked: {e}"));
                    self.notice_until =
                        Some(std::time::Instant::now() + std::time::Duration::from_secs(8));
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.spotify.liked_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.spotify.liked_loading = false;
            }
        }
    }

    pub fn load_more_spotify_liked(&mut self) {
        use crate::integrations::spotify::library::get_saved_tracks;
        if self.spotify.liked_loading || !self.spotify.liked_has_more {
            return;
        }
        if let Some(until) = self.spotify.liked_rate_limited_until {
            if std::time::Instant::now() < until {
                return;
            }
            self.spotify.liked_rate_limited_until = None;
        }
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        self.spotify.liked_loading = true;
        let offset = self.spotify.liked_offset;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.liked_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_saved_tracks(&token, offset).await);
        });
        self.spotify.liked_task = Some(handle);
    }

    pub fn fetch_playlists(&mut self) {
        use crate::integrations::spotify::playlists::get_user_playlists;
        abort_task(&mut self.spotify.playlists_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        self.spotify.playlists.clear();
        self.spotify.playlists_selected = 0;
        self.spotify.playlists_scroll_offset = 0;
        self.spotify.playlists_loading = true;
        self.spotify.playlists_offset = 0;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.playlists_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_user_playlists(&token, 0).await);
        });
        self.spotify.playlists_task = Some(handle);
    }

    pub fn poll_playlists(&mut self) {
        let Some(rx) = self.spotify.playlists_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok((playlists, has_more))) => {
                self.spotify.playlists_loading = false;
                self.spotify.playlists.extend(playlists);
                self.spotify.playlists_has_more = has_more;
                self.spotify.playlists_offset += 20;
            }
            Ok(Err(e)) => {
                self.spotify.playlists_loading = false;
                tracing::warn!("playlists fetch: {e}");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.spotify.playlists_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.spotify.playlists_loading = false;
            }
        }
    }

    pub fn load_more_spotify_playlists(&mut self) {
        use crate::integrations::spotify::playlists::get_user_playlists;
        if self.spotify.playlists_loading || !self.spotify.playlists_has_more {
            return;
        }
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        self.spotify.playlists_loading = true;
        let offset = self.spotify.playlists_offset;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.playlists_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_user_playlists(&token, offset).await);
        });
        self.spotify.playlists_task = Some(handle);
    }

    pub fn fetch_playlist_tracks(&mut self, playlist_id: String) {
        use crate::integrations::spotify::playlists::get_playlist_tracks;
        abort_task(&mut self.spotify.playlist_tracks_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        self.spotify.playlist_tracks_loading = true;
        self.spotify.playlist_tracks_offset = 0;
        self.spotify.playlist_tracks = Vec::new();
        self.spotify.playlist_tracks_selected = 0;
        self.spotify.playlist_tracks_scroll_offset = 0;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.playlist_tracks_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_playlist_tracks(&playlist_id, &token, 0).await);
        });
        self.spotify.playlist_tracks_task = Some(handle);
    }

    pub fn poll_playlist_tracks(&mut self) {
        let Some(rx) = self.spotify.playlist_tracks_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok((tracks, has_more))) => {
                let loaded_count = tracks.len() as u32;
                self.spotify.playlist_tracks_loading = false;
                self.spotify.playlist_tracks.extend(tracks);
                self.spotify.playlist_tracks_has_more = has_more;
                self.spotify.playlist_tracks_offset += 50;
                if let Some(pl) = self.spotify.open_playlist.as_mut() {
                    if pl.tracks_total == 0 {
                        pl.tracks_total = loaded_count;
                    }
                }
                if let Some(open_id) = self.spotify.open_playlist.as_ref().map(|p| p.id.clone()) {
                    if let Some(pl) = self.spotify.playlists.iter_mut().find(|p| p.id == open_id) {
                        if pl.tracks_total == 0 {
                            pl.tracks_total = loaded_count;
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                self.spotify.playlist_tracks_loading = false;
                tracing::warn!("playlist tracks fetch: {e}");
                self.save_notice = Some(format!("Spotify playlist: {e}"));
                self.notice_until =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(8));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.spotify.playlist_tracks_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.spotify.playlist_tracks_loading = false;
            }
        }
    }

    pub fn load_more_spotify_playlist_tracks(&mut self) {
        use crate::integrations::spotify::playlists::get_playlist_tracks;
        if self.spotify.playlist_tracks_loading || !self.spotify.playlist_tracks_has_more {
            return;
        }
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        let Some(playlist_id) = self.spotify.open_playlist.as_ref().map(|pl| pl.id.clone()) else {
            return;
        };
        self.spotify.playlist_tracks_loading = true;
        let offset = self.spotify.playlist_tracks_offset;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.playlist_tracks_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_playlist_tracks(&playlist_id, &token, offset).await);
        });
        self.spotify.playlist_tracks_task = Some(handle);
    }

    pub fn poll_remote_playback(&mut self) {
        if let Some(rx) = &self.spotify.playback_rx {
            loop {
                match rx.try_recv() {
                    Ok(new_state) => {
                        self.spotify.playback = match new_state {
                            None => {
                                if self.spotify.active_backend
                                    == Some(SpotifyPlaybackBackend::Remote)
                                {
                                    self.spotify.active_backend = None;
                                }
                                None
                            }
                            Some(mut state) => {
                                if let Some(until) = self.spotify.volume_pending_until {
                                    if std::time::Instant::now() < until {
                                        if let Some(ref current) = self.spotify.playback {
                                            state.volume_pct = current.volume_pct;
                                        }
                                    } else {
                                        self.spotify.volume_pending_until = None;
                                    }
                                }
                                Some(state)
                            }
                        };
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.spotify.playback_rx = None;
                        break;
                    }
                }
            }
        }
    }

    pub fn fetch_top_tracks(&mut self) {
        use crate::integrations::spotify::library::get_top_tracks;
        abort_task(&mut self.spotify.top_tracks_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        self.spotify.top_tracks_loading = true;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.top_tracks_rx = Some(rx);
        let range = self.spotify.top_tracks_range.clone();
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_top_tracks(&token, &range).await);
        });
        self.spotify.top_tracks_task = Some(handle);
    }

    pub fn poll_top_tracks(&mut self) {
        let Some(rx) = self.spotify.top_tracks_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(tracks)) => {
                self.spotify.top_tracks_loading = false;
                self.spotify.top_tracks = tracks;
            }
            Ok(Err(e)) => {
                self.spotify.top_tracks_loading = false;
                tracing::warn!("top tracks fetch: {e}");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.spotify.top_tracks_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.spotify.top_tracks_loading = false;
            }
        }
    }

    pub fn fetch_recent_tracks(&mut self) {
        use crate::integrations::spotify::library::get_recently_played;
        abort_task(&mut self.spotify.recent_tracks_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        self.spotify.recent_tracks_loading = true;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.recent_tracks_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_recently_played(&token).await);
        });
        self.spotify.recent_tracks_task = Some(handle);
    }

    pub fn poll_recent_tracks(&mut self) {
        let Some(rx) = self.spotify.recent_tracks_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(tracks)) => {
                self.spotify.recent_tracks_loading = false;
                self.spotify.recent_tracks = tracks;
            }
            Ok(Err(e)) => {
                self.spotify.recent_tracks_loading = false;
                tracing::warn!("recent tracks fetch: {e}");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.spotify.recent_tracks_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.spotify.recent_tracks_loading = false;
            }
        }
    }

    pub fn fetch_albums(&mut self) {
        use crate::integrations::spotify::albums::get_saved_albums;
        abort_task(&mut self.spotify.albums_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        self.spotify.albums.clear();
        self.spotify.albums_selected = 0;
        self.spotify.albums_loading = true;
        self.spotify.albums_offset = 0;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.albums_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_saved_albums(&token, 0).await);
        });
        self.spotify.albums_task = Some(handle);
    }

    pub fn poll_albums(&mut self) {
        let Some(rx) = self.spotify.albums_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok((albums, has_more))) => {
                self.spotify.albums_loading = false;
                self.spotify.albums.extend(albums);
                self.spotify.albums_has_more = has_more;
                self.spotify.albums_offset += 50;
            }
            Ok(Err(e)) => {
                self.spotify.albums_loading = false;
                tracing::warn!("albums fetch: {e}");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.spotify.albums_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.spotify.albums_loading = false;
            }
        }
    }

    pub fn load_more_spotify_albums(&mut self) {
        use crate::integrations::spotify::albums::get_saved_albums;
        if self.spotify.albums_loading || !self.spotify.albums_has_more {
            return;
        }
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        self.spotify.albums_loading = true;
        let offset = self.spotify.albums_offset;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.albums_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let _ = tx.send(get_saved_albums(&token, offset).await);
        });
        self.spotify.albums_task = Some(handle);
    }

    pub fn fetch_album_tracks(&mut self) {
        use crate::integrations::spotify::albums::get_album_tracks;
        abort_task(&mut self.spotify.album_tracks_task);
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        let Some(album) = self.spotify.open_album.clone() else {
            return;
        };
        self.spotify.album_tracks_loading = true;
        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.album_tracks_rx = Some(rx);
        let handle = tokio::spawn(async move {
            let id = album.uri.split(':').next_back().unwrap_or("").to_string();
            let _ = tx.send(get_album_tracks(&token, &id).await);
        });
        self.spotify.album_tracks_task = Some(handle);
    }

    pub fn poll_album_tracks(&mut self) {
        let Some(rx) = self.spotify.album_tracks_rx.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(tracks)) => {
                self.spotify.album_tracks_loading = false;
                self.spotify.album_tracks = tracks;
            }
            Ok(Err(e)) => {
                self.spotify.album_tracks_loading = false;
                tracing::warn!("album tracks fetch: {e}");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.spotify.album_tracks_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.spotify.album_tracks_loading = false;
            }
        }
    }

    pub fn like_spotify_track(&mut self, track: &crate::integrations::spotify::SpotifyTrack) {
        use crate::integrations::spotify::library::save_track;
        let Some(token) = self.spotify.access_token.clone() else {
            return;
        };
        let uri = track.uri.clone();
        let name = track.name.clone();
        let id = uri.split(':').next_back().unwrap_or("").to_string();
        if id.is_empty() {
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.spotify.save_track_rx = Some(rx);
        self.save_notice = Some(format!("Guardando {}...", name));
        self.save_notice_is_dup = false;
        self.notice_until = Some(std::time::Instant::now() + std::time::Duration::from_secs(4));

        tokio::spawn(async move {
            match save_track(&token, &id).await {
                Ok(_) => {
                    let _ = tx.send(Ok(name));
                }
                Err(e) => {
                    tracing::warn!("Failed to like spotify track: {e}");
                    let _ = tx.send(Err(format!("{e}")));
                }
            }
        });
    }

    pub fn poll_save_track(&mut self) {
        let rx = match self.spotify.save_track_rx.take() {
            Some(r) => r,
            None => return,
        };
        match rx.try_recv() {
            Ok(Ok(name)) => {
                self.save_notice = Some(format!("Guardado en Tus Me Gusta: {name}"));
                self.notice_until =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(4));
            }
            Ok(Err(e)) => {
                self.save_notice = Some(format!("Error al guardar: {e}"));
                self.notice_until =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(4));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.spotify.save_track_rx = Some(rx);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::spotify::devices::SpotifyDevice;

    fn spotify_device(id: Option<&str>, is_active: bool) -> SpotifyDevice {
        SpotifyDevice {
            id: id.map(str::to_string),
            name: id.unwrap_or("device").to_string(),
            device_type: "computer".to_string(),
            is_active,
        }
    }

    #[test]
    fn auto_mode_prefers_remote_device_when_available() {
        let target = resolve_spotify_playback_target(
            SpotifyPlaybackMode::Auto,
            Some("token"),
            Some("device"),
            true,
            None,
        );

        assert_eq!(
            target,
            SpotifyPlaybackTarget::Remote {
                token: "token".to_string(),
                device_id: "device".to_string(),
            }
        );
    }

    #[test]
    fn auto_mode_uses_native_when_no_remote_device_exists() {
        let target = resolve_spotify_playback_target(
            SpotifyPlaybackMode::Auto,
            Some("token"),
            None,
            true,
            None,
        );

        assert_eq!(target, SpotifyPlaybackTarget::Native);
    }

    #[test]
    fn auto_mode_keeps_native_when_it_is_the_active_backend() {
        let target = resolve_spotify_playback_target(
            SpotifyPlaybackMode::Auto,
            Some("token"),
            Some("device"),
            true,
            Some(SpotifyPlaybackBackend::Native),
        );

        assert_eq!(target, SpotifyPlaybackTarget::Native);
    }

    #[test]
    fn native_mode_does_not_use_remote_device() {
        let target = resolve_spotify_playback_target(
            SpotifyPlaybackMode::Native,
            Some("token"),
            Some("device"),
            true,
            Some(SpotifyPlaybackBackend::Remote),
        );

        assert_eq!(target, SpotifyPlaybackTarget::Native);
    }

    #[test]
    fn native_mode_reports_unavailable_without_local_player() {
        let target = resolve_spotify_playback_target(
            SpotifyPlaybackMode::Native,
            Some("token"),
            Some("device"),
            false,
            Some(SpotifyPlaybackBackend::Remote),
        );

        assert_eq!(
            target,
            SpotifyPlaybackTarget::Unavailable(
                "integrations.spotify.error.native_unavailable".to_string()
            )
        );
    }

    #[test]
    fn remote_mode_does_not_fall_back_to_native_without_device() {
        let target = resolve_spotify_playback_target(
            SpotifyPlaybackMode::Remote,
            Some("token"),
            None,
            true,
            None,
        );

        assert_eq!(
            target,
            SpotifyPlaybackTarget::Unavailable(
                "integrations.spotify.error.no_remote_device".to_string()
            )
        );
    }

    #[test]
    fn remote_mode_reports_unavailable_without_access_token() {
        let target = resolve_spotify_playback_target(
            SpotifyPlaybackMode::Remote,
            None,
            Some("device"),
            true,
            None,
        );

        assert_eq!(
            target,
            SpotifyPlaybackTarget::Unavailable(
                "integrations.spotify.error.no_access_token".to_string()
            )
        );
    }

    #[test]
    fn auto_mode_reports_unavailable_without_any_target() {
        let target = resolve_spotify_playback_target(
            SpotifyPlaybackMode::Auto,
            Some("token"),
            None,
            false,
            None,
        );

        assert_eq!(
            target,
            SpotifyPlaybackTarget::Unavailable(
                "integrations.spotify.error.playback_unavailable".to_string()
            )
        );
    }

    #[test]
    fn stale_remote_device_id_is_replaced_with_current_active_device() {
        let devices = vec![
            spotify_device(Some("available"), false),
            spotify_device(Some("active"), true),
        ];

        let (selected, active_device_id) = resolve_active_spotify_device(&devices, Some("stale"));

        assert_eq!(selected, 1);
        assert_eq!(active_device_id.as_deref(), Some("active"));
    }

    #[test]
    fn stale_remote_device_id_is_cleared_when_no_remote_devices_exist() {
        let devices = vec![];

        let (selected, active_device_id) = resolve_active_spotify_device(&devices, Some("stale"));

        assert_eq!(selected, 0);
        assert_eq!(active_device_id, None);
    }

    #[test]
    fn current_remote_device_id_is_preserved_when_still_present() {
        let devices = vec![
            spotify_device(Some("current"), false),
            spotify_device(Some("active"), true),
        ];

        let (selected, active_device_id) = resolve_active_spotify_device(&devices, Some("current"));

        assert_eq!(selected, 0);
        assert_eq!(active_device_id.as_deref(), Some("current"));
    }

    #[test]
    fn native_error_message_maps_missing_credentials() {
        assert_eq!(
            spotify_native_error_message("native_missing_credentials"),
            "integrations.spotify.error.native_missing_credentials"
        );
    }

    #[test]
    fn native_error_message_maps_auth_failures() {
        assert_eq!(
            spotify_native_error_message("native_auth_failed: Puerto 8898 ocupado"),
            "integrations.spotify.error.native_auth_failed"
        );
    }

    #[test]
    fn native_error_message_maps_premium_failures() {
        assert_eq!(
            spotify_native_error_message("native_session_connect: Premium account required"),
            "integrations.spotify.error.native_premium_required"
        );
    }

    #[test]
    fn native_error_message_maps_audio_backend_failures() {
        assert_eq!(
            spotify_native_error_message("native_audio_backend_missing"),
            "integrations.spotify.error.native_audio_backend_missing"
        );
    }

    #[test]
    fn native_error_message_maps_track_unavailable() {
        assert_eq!(
            spotify_native_error_message("native_track_unavailable: spotify:track:123"),
            "integrations.spotify.error.native_track_unavailable"
        );
    }

    #[test]
    fn native_track_errors_do_not_disable_native_player() {
        assert!(!spotify_native_error_is_fatal(
            "native_track_unavailable: spotify:track:123"
        ));
        assert!(!spotify_native_error_is_fatal("native_uri_parse: bad uri"));
    }

    #[test]
    fn native_session_errors_disable_native_player() {
        assert!(spotify_native_error_is_fatal(
            "native_session_connect: Premium account required"
        ));
        assert!(spotify_native_error_is_fatal(
            "native_audio_backend_missing"
        ));
    }
}
