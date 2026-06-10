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

    parse_saved_tracks_body(&body)
}

pub(crate) fn parse_saved_tracks_body(
    body: &str,
) -> Result<(Vec<SpotifyTrack>, bool), SpotifyError> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

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

    parse_top_tracks_body(&body)
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

    parse_recently_played_body(&body)
}

pub(crate) fn parse_top_tracks_body(body: &str) -> Result<Vec<SpotifyTrack>, SpotifyError> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

    Ok(json["items"]
        .as_array()
        .map(|items| items.iter().filter_map(parse_track).collect())
        .unwrap_or_default())
}

pub(crate) fn parse_recently_played_body(body: &str) -> Result<Vec<SpotifyTrack>, SpotifyError> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

    Ok(json["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| parse_track(&item["track"]))
                .collect()
        })
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::spotify::test_fixtures;

    #[test]
    fn parse_saved_tracks_body_reads_wrapped_tracks_and_next() {
        let (tracks, has_more) = parse_saved_tracks_body(test_fixtures::SAVED_TRACKS_CURRENT)
            .expect("valid saved tracks body");

        assert!(has_more);
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].name, "Sweet Disposition");
        assert_eq!(tracks[0].artist, "The Temper Trap");
        assert_eq!(tracks[0].album, "Conditions");
        assert_eq!(tracks[0].duration_ms, 232000);
        assert_eq!(tracks[0].uri, "spotify:track:saved-track-1");
    }

    #[test]
    fn parse_top_tracks_body_reads_direct_track_items() {
        let tracks = parse_top_tracks_body(test_fixtures::TOP_TRACKS_CURRENT)
            .expect("valid top tracks body");

        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].name, "Electric Feel");
        assert_eq!(tracks[0].artist, "MGMT");
        assert_eq!(tracks[0].album, "Oracular Spectacular");
        assert_eq!(tracks[0].duration_ms, 229000);
        assert_eq!(tracks[0].uri, "spotify:track:top-track-1");
    }

    #[test]
    fn parse_recently_played_body_reads_wrapped_tracks() {
        let tracks = parse_recently_played_body(test_fixtures::RECENTLY_PLAYED_CURRENT)
            .expect("valid recently played body");

        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].name, "Lisztomania");
        assert_eq!(tracks[0].artist, "Phoenix");
        assert_eq!(tracks[0].album, "Wolfgang Amadeus Phoenix");
        assert_eq!(tracks[0].duration_ms, 241000);
        assert_eq!(tracks[0].uri, "spotify:track:recent-track-1");
    }
}
