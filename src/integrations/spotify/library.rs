use super::{search::parse_track, SpotifyError, SpotifyTrack};

pub async fn get_saved_tracks(
    access_token: &str,
    offset: usize,
) -> Result<(Vec<SpotifyTrack>, bool), SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = format!("https://api.spotify.com/v1/me/tracks?limit=50&offset={offset}");

    let response = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    let status = response.status();
    let retry_after = response
        .headers()
        .get("Retry-After")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60);
    let body = response
        .text()
        .await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(SpotifyError::RateLimit(retry_after));
    }

    if !status.is_success() {
        return Err(SpotifyError::from_status(status, &body));
    }

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

    let tracks: Vec<SpotifyTrack> = json["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| parse_track(&item["track"]))
                .collect()
        })
        .unwrap_or_default();

    let has_more = json["next"].is_string();

    Ok((tracks, has_more))
}

pub async fn save_track(access_token: &str, track_id: &str) -> Result<(), SpotifyError> {
    tracing::info!(
        "spotify save_track: attempting to save track_id='{}'",
        track_id
    );
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = format!(
        "https://api.spotify.com/v1/me/library?uris=spotify:track:{}",
        track_id
    );

    let response = client
        .put(&url)
        .bearer_auth(access_token)
        .header("Content-Length", "0")
        .send()
        .await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    let status = response.status();
    tracing::info!("spotify save_track: HTTP {}", status);
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        tracing::warn!("spotify save_track error response: {}", body);
        return Err(SpotifyError::from_status(status, &body));
    }
    tracing::info!(
        "spotify save_track: successfully saved track_id='{}'",
        track_id
    );
    Ok(())
}

pub async fn get_top_tracks(
    access_token: &str,
    time_range: &str,
) -> Result<Vec<SpotifyTrack>, SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = format!(
        "https://api.spotify.com/v1/me/top/tracks?limit=50&time_range={}",
        time_range
    );

    let response = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(SpotifyError::from_status(status, &body));
    }

    let body = response
        .text()
        .await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

    let tracks: Vec<SpotifyTrack> = json["items"]
        .as_array()
        .map(|items| items.iter().filter_map(parse_track).collect())
        .unwrap_or_default();

    Ok(tracks)
}

pub async fn get_recently_played(access_token: &str) -> Result<Vec<SpotifyTrack>, SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = "https://api.spotify.com/v1/me/player/recently-played?limit=50";

    let response = client
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(SpotifyError::from_status(status, &body));
    }

    let body = response
        .text()
        .await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

    let tracks: Vec<SpotifyTrack> = json["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| parse_track(&item["track"]))
                .collect()
        })
        .unwrap_or_default();

    Ok(tracks)
}
