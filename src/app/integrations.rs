use super::modal::{SearchMode, SpotifyAuthStatus, SpotifyPlayerStatus};
use super::{abort_task, App};

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
        self.config.spotify.display_name = None;
        self.config.save();
        self.spotify.is_premium = false;
        self.spotify.status = SpotifyAuthStatus::Idle;
        self.spotify.player_status = SpotifyPlayerStatus::Idle;
        self.spotify.now_playing = None;
        self.spotify.access_token = None;
        self.spotify.player_tx = None;
        self.spotify.player_rx = None;
        self.spotify.search_results.clear();
        self.spotify.search_query.clear();
        self.spotify.active_device_id = None;
        self.spotify.devices.clear();
        self.stop_playback_polling();
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
                    {
                        let (evt_tx, evt_rx) = std::sync::mpsc::sync_channel(32);
                        let handle =
                            crate::integrations::spotify::player::spawn_player(audio_token, evt_tx);
                        self.spotify.player_tx = Some(handle);
                        self.spotify.player_rx = Some(evt_rx);
                    }
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

    pub fn poll_spotify_player_events(&mut self) {
        use crate::integrations::spotify::SpotifyPlayerEvent;
        let Some(rx) = &self.spotify.player_rx else {
            return;
        };
        loop {
            match rx.try_recv() {
                Ok(SpotifyPlayerEvent::Playing) => {
                    self.spotify.player_status = SpotifyPlayerStatus::Playing
                }
                Ok(SpotifyPlayerEvent::Paused) => {
                    self.spotify.player_status = SpotifyPlayerStatus::Paused
                }
                Ok(SpotifyPlayerEvent::Stopped) => {
                    self.spotify.player_status = SpotifyPlayerStatus::Idle;
                    self.spotify.now_playing = None;
                }
                Ok(SpotifyPlayerEvent::EndOfTrack) => {
                    self.spotify.player_status = SpotifyPlayerStatus::Idle;
                    self.spotify.now_playing = None;
                }
                Ok(SpotifyPlayerEvent::Error(e)) => {
                    self.spotify.player_status = SpotifyPlayerStatus::Error(e)
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.spotify.player_rx = None;
                    break;
                }
            }
        }
    }

    pub(super) fn perform_spotify_search(&mut self) {
        use crate::integrations::spotify::{search::search_tracks, SpotifyError};
        if let Some(until) = self.spotify.rate_limited_until {
            if std::time::Instant::now() < until {
                return;
            }
            self.spotify.rate_limited_until = None;
            self.spotify.search_rate_limited = false;
        }
        self.spotify.search_rate_limited = false;
        self.spotify.search_loading_more = false;
        abort_task(&mut self.spotify.search_task);
        let query = self.spotify.search_query.clone();
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
            let _ = tx.send((results, has_more, rate_limit_secs));
        });
        self.spotify.search_task = Some(handle);
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

                    let active_id = self.spotify.active_device_id.as_deref();
                    let selected = self
                        .spotify
                        .devices
                        .iter()
                        .position(|d| {
                            active_id
                                .map(|id| d.id.as_deref() == Some(id))
                                .unwrap_or(false)
                        })
                        .or_else(|| self.spotify.devices.iter().position(|d| d.is_active))
                        .unwrap_or(0);

                    self.spotify.devices_selected = selected;

                    if self.spotify.active_device_id.is_none() {
                        if let Some(dev) = self.spotify.devices.get(selected) {
                            self.spotify.active_device_id = dev.id.clone();
                        }
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!("spotify devices: {e}");
                    self.spotify.devices_loading = false;
                    if !matches!(e, SpotifyError::RateLimit(_)) {
                        self.save_notice = Some(format!("Dispositivos Spotify: {e}"));
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
        let Some(device_id) = self.spotify.active_device_id.clone() else {
            return;
        };
        let Some(token) = self.spotify.access_token.clone() else {
            return;
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
                self.spotify.active_device_id = Some(device_id.clone());
                for d in self.spotify.devices.iter_mut() {
                    d.is_active = d.id.as_deref() == Some(&device_id);
                }
                self.start_playback_polling();
                tracing::info!("spotify: dispositivo activo → {device_id}");
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
                    tracing::info!("spotify: access_token renovado");
                }
                Ok(Err(e)) => {
                    tracing::warn!("spotify token refresh fallido: {e}");
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
                    self.spotify.player_status = SpotifyPlayerStatus::Playing;
                }
                Ok(Err(SpotifyError::Unauthorized)) => {
                    self.spotify.player_status = SpotifyPlayerStatus::Error(crate::i18n::t(
                        "integrations.spotify.error.generic",
                    ));
                    tracing::warn!("spotify play: token expirado, renovando");
                    self.spotify.token_refreshed_at =
                        Some(std::time::Instant::now() - std::time::Duration::from_secs(60 * 60));
                }
                Ok(Err(e)) => {
                    tracing::warn!("spotify play_on_device: {e}");
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
                Ok((results, has_more, rate_limit_secs)) => {
                    let rate_limited = rate_limit_secs.is_some();
                    self.spotify.search_rate_limited = rate_limited;
                    if let Some(secs) = rate_limit_secs {
                        self.spotify.rate_limited_until =
                            Some(std::time::Instant::now() + std::time::Duration::from_secs(secs));
                    }
                    self.spotify.search_has_more = !rate_limited && has_more;
                    self.spotify.search_results = results;
                    self.spotify.search_loading = false;
                    self.spotify.search_selected = 0;
                    self.spotify.search_offset = 0;
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
            let _ = tx.send(result);
        });
        self.spotify.search_more_task = Some(handle);
    }

    pub fn poll_spotify_search_more(&mut self) {
        if let Some(rx) = self.spotify.search_more_rx.take() {
            match rx.try_recv() {
                Ok((more, has_more, _)) => {
                    self.spotify.search_offset += 10;
                    self.spotify.search_has_more = has_more;
                    self.spotify.search_loading_more = false;
                    self.spotify.search_results.extend(more);
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
    }

    pub fn poll_remote_playback(&mut self) {
        if let Some(rx) = &self.spotify.playback_rx {
            loop {
                match rx.try_recv() {
                    Ok(new_state) => {
                        self.spotify.playback = match new_state {
                            None => None,
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
}
