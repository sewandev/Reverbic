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

    parse_saved_albums_body(&body)
}

pub(crate) fn parse_saved_albums_body(
    body: &str,
) -> Result<(Vec<SpotifyAlbum>, bool), SpotifyError> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

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

        let (tracks, next_url) = parse_album_tracks_body(&body)?;
        all_tracks.extend(tracks);

        if let Some(next_url) = next_url {
            url = next_url;
        } else {
            break;
        }
    }

    Ok(all_tracks)
}

pub(crate) fn parse_album_tracks_body(
    body: &str,
) -> Result<(Vec<SpotifyTrack>, Option<String>), SpotifyError> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

    let tracks = json["items"]
        .as_array()
        .map(|items| items.iter().filter_map(parse_track).collect())
        .unwrap_or_default();
    let next_url = json["next"].as_str().map(str::to_string);

    Ok((tracks, next_url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::spotify::test_fixtures;

    #[test]
    fn parse_saved_albums_body_reads_wrapped_albums_and_next() {
        let (albums, has_more) = parse_saved_albums_body(test_fixtures::SAVED_ALBUMS_CURRENT)
            .expect("valid saved albums body");

        assert!(has_more);
        assert_eq!(albums.len(), 2);
        assert_eq!(albums[0].name, "Un Verano Sin Ti");
        assert_eq!(albums[0].artist, "Bad Bunny");
        assert_eq!(albums[0].total_tracks, 23);
        assert_eq!(albums[0].uri, "spotify:album:album-1");
    }

    #[test]
    fn parse_saved_albums_body_defaults_missing_optional_album_fields() {
        let (albums, _) = parse_saved_albums_body(test_fixtures::SAVED_ALBUMS_CURRENT)
            .expect("valid saved albums body");

        assert_eq!(albums.len(), 2);
        assert_eq!(albums[1].name, "Sparse Album");
        assert_eq!(albums[1].artist, "Unknown");
        assert_eq!(albums[1].total_tracks, 0);
        assert_eq!(albums[1].uri, "spotify:album:sparse-album");
    }

    #[test]
    fn parse_album_tracks_body_reads_tracks_and_next_url() {
        let (tracks, next_url) = parse_album_tracks_body(test_fixtures::ALBUM_TRACKS_CURRENT)
            .expect("valid album tracks body");

        assert_eq!(
            next_url.as_deref(),
            Some("https://api.spotify.com/v1/albums/album-1/tracks?offset=50&limit=50")
        );
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].name, "Moscow Mule");
        assert_eq!(tracks[0].artist, "Bad Bunny");
        assert_eq!(tracks[0].album, "");
        assert_eq!(tracks[0].duration_ms, 245939);
        assert_eq!(tracks[0].uri, "spotify:track:album-track-1");
    }
}
