use std::collections::VecDeque;

use rand::seq::SliceRandom;

use super::{search::parse_track, SpotifyError, SpotifyTrack};

pub async fn fetch_radio_pool(
    artist_name: &str,
    seed_uri: &str,
    recently_played: VecDeque<String>,
    access_token: &str,
) -> Result<Vec<SpotifyTrack>, SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let encoded: String = artist_name
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![b as char]
            }
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect();

    let url = format!("https://api.spotify.com/v1/search?q=artist%3A{encoded}&type=track&limit=20");

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

    let mut tracks: Vec<SpotifyTrack> = json["tracks"]["items"]
        .as_array()
        .map(|items| items.iter().filter_map(parse_track).collect())
        .unwrap_or_default();

    tracks.retain(|t| t.uri != seed_uri && !recently_played.contains(&t.uri));

    let mut rng = rand::thread_rng();
    tracks.shuffle(&mut rng);

    Ok(tracks)
}
