use super::{search::parse_track, SpotifyError, SpotifyTrack};

#[derive(Clone)]
pub struct SpotifyPlaylist {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub tracks_total: u32,
}

pub async fn get_user_playlists(
    access_token: &str,
    offset: usize,
) -> Result<(Vec<SpotifyPlaylist>, bool), SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = format!("https://api.spotify.com/v1/me/playlists?limit=20&offset={offset}");

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

    let playlists: Vec<SpotifyPlaylist> = json["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let id = item["id"].as_str()?.to_string();
                    let name = item["name"].as_str()?.to_string();
                    let owner = item["owner"]["display_name"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    let tracks_total = item["tracks"]["total"].as_u64().unwrap_or(0) as u32;
                    Some(SpotifyPlaylist {
                        id,
                        name,
                        owner,
                        tracks_total,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let has_more = json["next"].is_string();

    Ok((playlists, has_more))
}

pub async fn get_playlist_tracks(
    playlist_id: &str,
    access_token: &str,
    offset: usize,
) -> Result<(Vec<SpotifyTrack>, bool), SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = format!(
        "https://api.spotify.com/v1/playlists/{playlist_id}/tracks?limit=50&offset={offset}"
    );

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

    tracing::debug!(
        "spotify /v1/playlists/{playlist_id}/tracks — status={status} body={}",
        &body[..body.len().min(600)]
    );

    if !status.is_success() {
        return Err(SpotifyError::from_status(status, &body));
    }

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

    let raw_items = json["items"].as_array().map(|v| v.len()).unwrap_or(0);
    let tracks: Vec<SpotifyTrack> = json["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let track = &item["track"];
                    if track.is_null() {
                        None
                    } else {
                        parse_track(track)
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    tracing::debug!(
        "spotify playlist tracks — raw_items={raw_items} parsed={}",
        tracks.len()
    );

    let has_more = json["next"].is_string();

    Ok((tracks, has_more))
}
