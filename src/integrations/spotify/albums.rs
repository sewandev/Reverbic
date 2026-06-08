use super::{search::parse_track, SpotifyAlbum, SpotifyError, SpotifyTrack};

pub fn parse_album(item: &serde_json::Value) -> Option<SpotifyAlbum> {
    let name = item["name"].as_str()?.to_string();
    let artist = item["artists"][0]["name"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();
    let uri = item["uri"].as_str()?.to_string();
    let total_tracks = item["total_tracks"].as_u64().unwrap_or(0) as u32;

    Some(SpotifyAlbum {
        name,
        artist,
        total_tracks,
        uri,
    })
}

pub async fn get_saved_albums(
    access_token: &str,
    offset: usize,
) -> Result<(Vec<SpotifyAlbum>, bool), SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = format!("https://api.spotify.com/v1/me/albums?limit=50&offset={offset}");

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

    let albums: Vec<SpotifyAlbum> = json["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| parse_album(&item["album"]))
                .collect()
        })
        .unwrap_or_default();

    let has_more = json["next"].is_string();
    Ok((albums, has_more))
}

pub async fn get_album_tracks(
    access_token: &str,
    album_id: &str,
) -> Result<Vec<SpotifyTrack>, SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = format!(
        "https://api.spotify.com/v1/albums/{}/tracks?limit=50",
        album_id
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
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let track = parse_track(item)?;
                    // The album tracks endpoint does not include the album object in the track,
                    // but we can just leave it empty or we could pass the album name.
                    // parse_track will likely fall back to "Unknown Album" if it's not present.
                    Some(track)
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(tracks)
}
