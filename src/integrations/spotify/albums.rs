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

    let mut url = format!(
        "https://api.spotify.com/v1/albums/{}/tracks?limit=50",
        album_id
    );

    let mut all_tracks = Vec::new();

    loop {
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

        all_tracks.extend(tracks);

        if let Some(next_url) = json["next"].as_str() {
            url = next_url.to_string();
        } else {
            break;
        }
    }

    Ok(all_tracks)
}
