use super::{SpotifyError, SpotifyTrack};

pub async fn search_tracks(
    query:        &str,
    access_token: &str,
    offset:       usize,
) -> Result<(Vec<SpotifyTrack>, bool), SpotifyError> {
    if query.is_empty() {
        return Ok((vec![], false));
    }

    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("No se pudo crear cliente HTTP".to_string()))?;

    let encoded = query.bytes().fold(String::new(), |mut s, b| {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => s.push(b as char),
            _ => s.push_str(&format!("%{b:02X}")),
        }
        s
    });
    let url = format!(
        "https://api.spotify.com/v1/search?type=track&limit=10&offset={offset}&q={encoded}"
    );

    let response = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| SpotifyError::Network(e.to_string()))?;

    let status = response.status();
    let body   = response.text().await.map_err(|e| SpotifyError::Network(e.to_string()))?;

    tracing::debug!("spotify /v1/search — status={status} body={}", &body[..body.len().min(400)]);

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        tracing::warn!("spotify search: rate limit activo, intenta de nuevo en unos minutos");
        return Err(SpotifyError::RateLimit);
    }

    if !status.is_success() {
        return Err(SpotifyError::from_status(status, &body));
    }

    parse_body(&body)
}

fn parse_body(body: &str) -> Result<(Vec<SpotifyTrack>, bool), SpotifyError> {
    let json: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| SpotifyError::Parse(e.to_string()))?;

    let tracks_obj = &json["tracks"];
    let tracks: Vec<SpotifyTrack> = tracks_obj["items"]
        .as_array()
        .map(|items| items.iter().filter_map(parse_track).collect())
        .unwrap_or_default();

    let has_more = tracks_obj["next"].is_string();

    Ok((tracks, has_more))
}

fn parse_track(item: &serde_json::Value) -> Option<SpotifyTrack> {
    let name = item["name"].as_str()?.to_string();
    let artist = item["artists"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|a| a["name"].as_str())
        .unwrap_or("Desconocido")
        .to_string();
    let album = item["album"]["name"].as_str().unwrap_or("").to_string();
    let duration_ms = item["duration_ms"].as_u64().unwrap_or(0) as u32;
    let uri = item["uri"].as_str()?.to_string();

    Some(SpotifyTrack { name, artist, album, duration_ms, uri })
}
