use std::collections::VecDeque;

use rand::seq::SliceRandom;

use super::{search::parse_track, SpotifyError, SpotifyTrack};

pub async fn fetch_radio_pool(
    artist_id: &str,
    seed_uri: &str,
    recently_played: VecDeque<String>,
    access_token: &str,
) -> Result<Vec<SpotifyTrack>, SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url =
        format!("https://api.spotify.com/v1/artists/{artist_id}/top-tracks?market=from_token");

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

    let mut tracks: Vec<SpotifyTrack> = json["tracks"]
        .as_array()
        .map(|items| items.iter().filter_map(parse_track).collect())
        .unwrap_or_default();

    tracks.retain(|t| t.uri != seed_uri && !recently_played.contains(&t.uri));

    let mut rng = rand::thread_rng();
    tracks.shuffle(&mut rng);

    Ok(tracks)
}
