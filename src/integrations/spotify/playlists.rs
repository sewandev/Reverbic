use super::{search::parse_track, SpotifyError, SpotifyTrack};

const USER_PLAYLISTS_PAGE_SIZE: usize = 20;
const PLAYLIST_TRACKS_PAGE_SIZE: usize = 50;

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

    let url = user_playlists_url(offset);

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

    parse_user_playlists_body(&body)
}

pub async fn get_playlist_tracks(
    playlist_id: &str,
    access_token: &str,
    offset: usize,
) -> Result<(Vec<SpotifyTrack>, bool), SpotifyError> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| SpotifyError::Network("Failed to create HTTP client".to_string()))?;

    let url = playlist_items_url(playlist_id, offset);

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
        "spotify /v1/playlists/{playlist_id}/items — status={status} body={}",
        &body[..body.len().min(400)]
    );

    if !status.is_success() {
        return Err(SpotifyError::from_status(status, &body));
    }

    parse_playlist_tracks_body(&body)
}

pub(crate) fn user_playlists_url(offset: usize) -> String {
    format!(
        "https://api.spotify.com/v1/me/playlists?limit={USER_PLAYLISTS_PAGE_SIZE}&offset={offset}"
    )
}

pub(crate) fn playlist_items_url(playlist_id: &str, offset: usize) -> String {
    format!(
        "https://api.spotify.com/v1/playlists/{playlist_id}/items?limit={PLAYLIST_TRACKS_PAGE_SIZE}&offset={offset}"
    )
}

pub(crate) fn parse_user_playlists_body(
    body: &str,
) -> Result<(Vec<SpotifyPlaylist>, bool), SpotifyError> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

    let playlists: Vec<SpotifyPlaylist> = json["items"]
        .as_array()
        .map(|items| items.iter().filter_map(parse_playlist).collect())
        .unwrap_or_default();

    let has_more = json["next"].is_string();

    Ok((playlists, has_more))
}

fn parse_playlist(item: &serde_json::Value) -> Option<SpotifyPlaylist> {
    let id = item["id"].as_str()?.to_string();
    let name = item["name"].as_str()?.to_string();
    let owner = item["owner"]["display_name"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let tracks_total = playlist_tracks_total(item);

    Some(SpotifyPlaylist {
        id,
        name,
        owner,
        tracks_total,
    })
}

fn playlist_tracks_total(item: &serde_json::Value) -> u32 {
    item["items"]["total"]
        .as_u64()
        .or_else(|| item["tracks"]["total"].as_u64())
        .or_else(|| item["total_tracks"].as_u64())
        .unwrap_or(0) as u32
}

pub(crate) fn parse_playlist_tracks_body(
    body: &str,
) -> Result<(Vec<SpotifyTrack>, bool), SpotifyError> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| SpotifyError::Parse(e.to_string()))?;

    let raw_items = json["items"].as_array().map(|v| v.len()).unwrap_or(0);
    let tracks: Vec<SpotifyTrack> = json["items"]
        .as_array()
        .map(|items| items.iter().filter_map(parse_playlist_item_track).collect())
        .unwrap_or_default();

    tracing::debug!(
        "spotify playlist tracks — raw_items={raw_items} parsed={}",
        tracks.len()
    );

    let has_more = json["next"].is_string();

    Ok((tracks, has_more))
}

fn parse_playlist_item_track(item: &serde_json::Value) -> Option<SpotifyTrack> {
    let track = item
        .get("item")
        .filter(|nested| !nested.is_null())
        .unwrap_or(item);

    if let Some(item_type) = track["type"].as_str() {
        if item_type != "track" {
            return None;
        }
    }

    parse_track(track)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::spotify::test_fixtures;

    #[test]
    fn user_playlists_url_uses_expected_page_size() {
        let url = user_playlists_url(40);

        assert_eq!(
            url,
            "https://api.spotify.com/v1/me/playlists?limit=20&offset=40"
        );
    }

    #[test]
    fn playlist_items_url_uses_items_endpoint() {
        let url = playlist_items_url("playlist-id", 50);

        assert_eq!(
            url,
            "https://api.spotify.com/v1/playlists/playlist-id/items?limit=50&offset=50"
        );
    }

    #[test]
    fn parse_user_playlists_body_reads_legacy_tracks_total() {
        let (playlists, has_more) =
            parse_user_playlists_body(test_fixtures::USER_PLAYLISTS_LEGACY_TRACKS_TOTAL)
                .expect("valid playlists body");

        assert!(!has_more);
        assert_eq!(playlists.len(), 3);
        assert_eq!(playlists[0].id, "playlist-legacy-tracks");
        assert_eq!(playlists[0].name, "Legacy Favorites");
        assert_eq!(playlists[0].owner, "Sewan");
        assert_eq!(playlists[0].tracks_total, 42);
    }

    #[test]
    fn parse_user_playlists_body_prefers_current_items_total() {
        let (playlists, has_more) =
            parse_user_playlists_body(test_fixtures::USER_PLAYLISTS_CURRENT_ITEMS_TOTAL)
                .expect("valid playlists body");

        assert!(has_more);
        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].id, "playlist-current");
        assert_eq!(playlists[0].name, "Current Favorites");
        assert_eq!(playlists[0].owner, "Reverbic");
        assert_eq!(playlists[0].tracks_total, 17);
    }

    #[test]
    fn parse_user_playlists_body_reads_total_tracks_fallback() {
        let (playlists, has_more) =
            parse_user_playlists_body(test_fixtures::USER_PLAYLISTS_LEGACY_TRACKS_TOTAL)
                .expect("valid playlists body");

        assert!(!has_more);
        assert_eq!(playlists.len(), 3);
        assert_eq!(playlists[1].id, "playlist-legacy-total-tracks");
        assert_eq!(playlists[1].tracks_total, 7);
    }

    #[test]
    fn parse_user_playlists_body_defaults_missing_track_totals_to_zero() {
        let (playlists, _) =
            parse_user_playlists_body(test_fixtures::USER_PLAYLISTS_LEGACY_TRACKS_TOTAL)
                .expect("valid playlists body");

        assert_eq!(playlists.len(), 3);
        assert_eq!(playlists[2].id, "playlist-missing-total");
        assert_eq!(playlists[2].tracks_total, 0);
    }

    #[test]
    fn parse_playlist_tracks_body_reads_wrapped_items() {
        let (tracks, has_more) = parse_playlist_tracks_body(test_fixtures::PLAYLIST_ITEMS_CURRENT)
            .expect("valid playlist items body");

        assert!(!has_more);
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].name, "New Gold");
        assert_eq!(tracks[0].artist, "Gorillaz");
        assert_eq!(tracks[0].uri, "spotify:track:playlist-track-1");
    }

    #[test]
    fn parse_playlist_tracks_body_accepts_direct_track_items() {
        let (tracks, has_more) =
            parse_playlist_tracks_body(test_fixtures::PLAYLIST_ITEMS_LEGACY_DIRECT_TRACKS)
                .expect("valid playlist items body");

        assert!(has_more);
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].name, "Midnight City");
        assert_eq!(tracks[0].artist, "M83");
        assert_eq!(tracks[0].uri, "spotify:track:legacy-track-1");
    }

    #[test]
    fn parse_playlist_tracks_body_accepts_track_without_type() {
        let (tracks, _) = parse_playlist_tracks_body(test_fixtures::PLAYLIST_ITEMS_CURRENT)
            .expect("valid playlist items body");

        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[1].name, "Untyped Track");
        assert_eq!(tracks[1].uri, "spotify:track:untyped");
    }

    #[test]
    fn parse_playlist_tracks_body_skips_direct_episode_items() {
        let (tracks, _) =
            parse_playlist_tracks_body(test_fixtures::PLAYLIST_ITEMS_LEGACY_DIRECT_TRACKS)
                .expect("valid playlist items body");

        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].uri, "spotify:track:legacy-track-1");
    }
}
