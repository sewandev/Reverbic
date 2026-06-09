use std::collections::VecDeque;

use rand::seq::SliceRandom;

use super::{
    search::{encode_query_component, parse_search_tracks_body, SPOTIFY_SEARCH_LIMIT},
    SpotifyError, SpotifyTrack,
};

pub async fn fetch_radio_pool(
    artist_name: &str,
    seed_uri: &str,
    recently_played: VecDeque<String>,
    access_token: &str,
) -> Result<Vec<SpotifyTrack>, SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = radio_search_url(artist_name);

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

    let (mut tracks, _) = parse_radio_search_body(&body)?;

    tracks.retain(|t| t.uri != seed_uri && !recently_played.contains(&t.uri));

    let mut rng = rand::thread_rng();
    tracks.shuffle(&mut rng);

    Ok(tracks)
}

pub(crate) fn radio_search_url(artist_name: &str) -> String {
    let encoded = encode_query_component(artist_name);
    format!(
        "https://api.spotify.com/v1/search?q=artist%3A{encoded}&type=track&limit={SPOTIFY_SEARCH_LIMIT}"
    )
}

pub(crate) fn parse_radio_search_body(
    body: &str,
) -> Result<(Vec<SpotifyTrack>, bool), SpotifyError> {
    parse_search_tracks_body(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_search_url_uses_artist_query() {
        let url = radio_search_url("Los Bunkers");

        assert_eq!(
            url,
            "https://api.spotify.com/v1/search?q=artist%3ALos%20Bunkers&type=track&limit=10"
        );
    }

    #[test]
    fn parse_radio_search_body_reads_search_tracks() {
        let body = r#"{
            "tracks": {
                "items": [
                    {
                        "name": "Bailando Solo",
                        "artists": [{ "name": "Los Bunkers" }],
                        "album": { "name": "La Velocidad de la Luz" },
                        "duration_ms": 232000,
                        "uri": "spotify:track:ghi"
                    }
                ],
                "next": null
            }
        }"#;

        let (tracks, has_more) = parse_radio_search_body(body).expect("valid radio search body");

        assert!(!has_more);
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].artist, "Los Bunkers");
        assert_eq!(tracks[0].uri, "spotify:track:ghi");
    }
}
