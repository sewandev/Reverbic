use serde::Deserialize;

use crate::integrations::spotify::SpotifyError;

fn spotify_client(timeout_secs: u64) -> Result<reqwest::Client, SpotifyError> {
    crate::http::http_client_timeout(timeout_secs)
        .ok_or_else(|| SpotifyError::Network("No se pudo crear cliente HTTP".to_string()))
}

#[derive(Debug, Clone)]
pub struct SpotifyPlaybackState {
    pub is_playing:  bool,
    pub progress_ms: u32,
    pub duration_ms: u32,
    pub track_name:  String,
    pub artist:      String,
    pub album:       String,
    pub device_name: String,
    pub volume_pct:  u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpotifyDevice {
    pub id:          Option<String>,
    pub name:        String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub is_active:   bool,
}

pub async fn list_devices(token: &str) -> Result<Vec<SpotifyDevice>, SpotifyError> {
    let client = spotify_client(10)?;
    let resp = client
        .get("https://api.spotify.com/v1/me/player/devices")
        .bearer_auth(token)
        .send().await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    let status = resp.status();
    tracing::info!("spotify devices: HTTP {status}");

    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("spotify devices error body: {body}");
        return Err(SpotifyError::from_status(status, &body));
    }

    let body = resp.text().await.map_err(|e| SpotifyError::Network(e.to_string()))?;
    tracing::info!("spotify devices body: {body}");

    #[derive(Deserialize)]
    struct Wrapper { devices: Vec<SpotifyDevice> }
    let parsed: Wrapper = serde_json::from_str(&body)
        .map_err(|e| SpotifyError::Parse(e.to_string()))?;

    tracing::info!("spotify devices: {} dispositivos encontrados", parsed.devices.len());
    for d in &parsed.devices {
        tracing::info!("  dispositivo: name={:?} id={:?} type={} active={}", d.name, d.id, d.device_type, d.is_active);
    }

    Ok(parsed.devices)
}

pub async fn play_on_device(token: &str, device_id: &str, uri: &str) -> Result<(), SpotifyError> {
    let client = spotify_client(10)?;
    tracing::info!("spotify play_on_device: device_id={device_id} uri={uri}");
    let resp = client
        .put(format!("https://api.spotify.com/v1/me/player/play?device_id={device_id}"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "uris": [uri] }))
        .send().await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;
    let status = resp.status();
    tracing::info!("spotify play_on_device: HTTP {status}");
    if status.is_success() || status.as_u16() == 204 {
        Ok(())
    } else {
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("spotify play_on_device error: {body}");
        Err(SpotifyError::from_status(status, &body))
    }
}

pub async fn pause_device(token: &str, device_id: &str) -> Result<(), SpotifyError> {
    let client = spotify_client(10)?;
    let resp = client
        .put(format!("https://api.spotify.com/v1/me/player/pause?device_id={device_id}"))
        .bearer_auth(token)
        .header("Content-Length", "0")
        .send().await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;
    let status = resp.status();
    if status.is_success() || status.as_u16() == 204 {
        Ok(())
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(SpotifyError::from_status(status, &body))
    }
}

pub async fn resume_device(token: &str, device_id: &str) -> Result<(), SpotifyError> {
    let client = spotify_client(10)?;
    let resp = client
        .put(format!("https://api.spotify.com/v1/me/player/play?device_id={device_id}"))
        .bearer_auth(token)
        .header("Content-Length", "0")
        .send().await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;
    let status = resp.status();
    if status.is_success() || status.as_u16() == 204 {
        Ok(())
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(SpotifyError::from_status(status, &body))
    }
}

pub async fn set_volume(token: &str, device_id: &str, volume_pct: u8) -> Result<(), SpotifyError> {
    let client = spotify_client(8)?;
    let pct = volume_pct.min(100);
    let url = if device_id.is_empty() {
        format!("https://api.spotify.com/v1/me/player/volume?volume_percent={pct}")
    } else {
        format!("https://api.spotify.com/v1/me/player/volume?volume_percent={pct}&device_id={device_id}")
    };
    tracing::info!("spotify set_volume: device_id={device_id} pct={pct}");
    let resp = client
        .put(url)
        .bearer_auth(token)
        .header("Content-Length", "0")
        .send().await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;
    let status = resp.status();
    tracing::info!("spotify set_volume: HTTP {status}");
    if status.is_success() || status.as_u16() == 204 {
        Ok(())
    } else {
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("spotify set_volume error: {body}");
        Err(SpotifyError::from_status(status, &body))
    }
}

pub async fn transfer_playback(token: &str, device_id: &str) -> Result<(), SpotifyError> {
    let client = spotify_client(10)?;
    tracing::info!("spotify transfer: enviando a device_id={device_id}");
    let resp = client
        .put("https://api.spotify.com/v1/me/player")
        .bearer_auth(token)
        .json(&serde_json::json!({ "device_ids": [device_id], "play": false }))
        .send().await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;
    let status = resp.status();
    tracing::info!("spotify transfer: HTTP {status}");
    if status.is_success() || status.as_u16() == 204 {
        Ok(())
    } else {
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("spotify transfer error: {body}");
        Err(SpotifyError::from_status(status, &body))
    }
}

pub async fn seek_playback(token: &str, device_id: &str, position_ms: u32) -> Result<(), SpotifyError> {
    let client = spotify_client(8)?;
    let url = if device_id.is_empty() {
        format!("https://api.spotify.com/v1/me/player/seek?position_ms={position_ms}")
    } else {
        format!("https://api.spotify.com/v1/me/player/seek?position_ms={position_ms}&device_id={device_id}")
    };
    let resp = client
        .put(url)
        .bearer_auth(token)
        .header("Content-Length", "0")
        .send().await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;
    let status = resp.status();
    tracing::info!("spotify seek: position_ms={position_ms} HTTP {status}");
    if status.is_success() || status.as_u16() == 204 {
        Ok(())
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(SpotifyError::from_status(status, &body))
    }
}

pub async fn get_playback(token: &str) -> Result<Option<SpotifyPlaybackState>, SpotifyError> {
    let client = spotify_client(8)?;
    let resp = client
        .get("https://api.spotify.com/v1/me/player")
        .bearer_auth(token)
        .send().await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    if resp.status().as_u16() == 204 {
        return Ok(None);
    }
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(SpotifyError::from_status(status, &body));
    }

    let json: serde_json::Value = resp.json().await
        .map_err(|e| SpotifyError::Parse(e.to_string()))?;

    let is_playing  = json["is_playing"].as_bool().unwrap_or(false);
    let progress_ms = json["progress_ms"].as_u64().unwrap_or(0) as u32;
    let item        = &json["item"];
    let duration_ms = item["duration_ms"].as_u64().unwrap_or(0) as u32;
    let track_name  = item["name"].as_str().unwrap_or("").to_string();
    let artist      = item["artists"][0]["name"].as_str().unwrap_or("").to_string();
    let album       = item["album"]["name"].as_str().unwrap_or("").to_string();
    let device_name = json["device"]["name"].as_str().unwrap_or("").to_string();
    let volume_pct  = json["device"]["volume_percent"].as_u64().unwrap_or(0) as u8;

    Ok(Some(SpotifyPlaybackState {
        is_playing, progress_ms, duration_ms,
        track_name, artist, album, device_name, volume_pct,
    }))
}
