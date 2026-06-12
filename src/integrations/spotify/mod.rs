pub mod albums;
pub mod devices;
pub mod error;
pub mod library;
pub mod oauth;
pub mod player;
pub mod playlists;
pub mod radio;
pub mod search;

pub use error::SpotifyError;

pub use devices::SpotifyPlaybackState;

pub enum AuthResult {
    Success {
        username: Option<String>,
        search_token: String,
        refresh_token: String,
        audio_token: String,
        native_error: Option<String>,
        is_premium: Option<bool>,
        country: Option<String>,
        followers: Option<u32>,
    },
    Failure(String),
}

#[derive(Clone)]
pub struct SpotifyTrack {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u32,
    pub uri: String,
}

#[derive(Clone)]
pub struct SpotifyAlbum {
    pub name: String,
    pub artist: String,
    pub total_tracks: u32,
    pub uri: String,
}

pub enum SpotifyPlayerCmd {
    Play { uris: Vec<String> },
    Pause,
    Resume,
    SetCrossfade { secs: u8 },
    CrossfadeTo { uri: String },
}

pub enum SpotifyPlayerEvent {
    Playing,
    Paused,
    Stopped,
    EndOfTrack,
    TrackNearEnd,
    TrackChanged(SpotifyTrack),
    Error(String),
}

#[cfg(test)]
pub(crate) mod test_fixtures {
    pub(crate) const SEARCH_TRACKS_CURRENT: &str =
        include_str!("fixtures/search_tracks_current.json");
    pub(crate) const SEARCH_TRACKS_MISSING_OPTIONAL_FIELDS: &str =
        include_str!("fixtures/search_tracks_missing_optional_fields.json");
    pub(crate) const SAVED_TRACKS_CURRENT: &str =
        include_str!("fixtures/saved_tracks_current.json");
    pub(crate) const TOP_TRACKS_CURRENT: &str = include_str!("fixtures/top_tracks_current.json");
    pub(crate) const RECENTLY_PLAYED_CURRENT: &str =
        include_str!("fixtures/recently_played_current.json");
    pub(crate) const SAVED_ALBUMS_CURRENT: &str =
        include_str!("fixtures/saved_albums_current.json");
    pub(crate) const ALBUM_TRACKS_CURRENT: &str =
        include_str!("fixtures/album_tracks_current.json");
    pub(crate) const USER_PLAYLISTS_CURRENT_ITEMS_TOTAL: &str =
        include_str!("fixtures/user_playlists_current_items_total.json");
    pub(crate) const USER_PLAYLISTS_LEGACY_TRACKS_TOTAL: &str =
        include_str!("fixtures/user_playlists_legacy_tracks_total.json");
    pub(crate) const PLAYLIST_ITEMS_CURRENT: &str =
        include_str!("fixtures/playlist_items_current.json");
    pub(crate) const PLAYLIST_ITEMS_LEGACY_DIRECT_TRACKS: &str =
        include_str!("fixtures/playlist_items_legacy_direct_tracks.json");
    pub(crate) const PLAYBACK_STATE_TRACK_CURRENT: &str =
        include_str!("fixtures/playback_state_track_current.json");
    pub(crate) const PLAYBACK_STATE_EPISODE_OR_MISSING_ITEM: &str =
        include_str!("fixtures/playback_state_episode_or_missing_item.json");
    pub(crate) const PLAYBACK_STATE_EPISODE_CURRENT: &str =
        include_str!("fixtures/playback_state_episode_current.json");
    pub(crate) const PLAYBACK_STATE_MISSING_DEVICE: &str =
        include_str!("fixtures/playback_state_missing_device.json");
    pub(crate) const DEVICES_CURRENT: &str = include_str!("fixtures/devices_current.json");
    pub(crate) const PROFILE_CURRENT_MINIMAL: &str =
        include_str!("fixtures/profile_current_minimal.json");
    pub(crate) const PROFILE_LEGACY_FULL: &str = include_str!("fixtures/profile_legacy_full.json");

    pub(crate) const ALL: &[(&str, &str)] = &[
        ("search_tracks_current", SEARCH_TRACKS_CURRENT),
        (
            "search_tracks_missing_optional_fields",
            SEARCH_TRACKS_MISSING_OPTIONAL_FIELDS,
        ),
        ("saved_tracks_current", SAVED_TRACKS_CURRENT),
        ("top_tracks_current", TOP_TRACKS_CURRENT),
        ("recently_played_current", RECENTLY_PLAYED_CURRENT),
        ("saved_albums_current", SAVED_ALBUMS_CURRENT),
        ("album_tracks_current", ALBUM_TRACKS_CURRENT),
        (
            "user_playlists_current_items_total",
            USER_PLAYLISTS_CURRENT_ITEMS_TOTAL,
        ),
        (
            "user_playlists_legacy_tracks_total",
            USER_PLAYLISTS_LEGACY_TRACKS_TOTAL,
        ),
        ("playlist_items_current", PLAYLIST_ITEMS_CURRENT),
        (
            "playlist_items_legacy_direct_tracks",
            PLAYLIST_ITEMS_LEGACY_DIRECT_TRACKS,
        ),
        ("playback_state_track_current", PLAYBACK_STATE_TRACK_CURRENT),
        (
            "playback_state_episode_or_missing_item",
            PLAYBACK_STATE_EPISODE_OR_MISSING_ITEM,
        ),
        (
            "playback_state_episode_current",
            PLAYBACK_STATE_EPISODE_CURRENT,
        ),
        (
            "playback_state_missing_device",
            PLAYBACK_STATE_MISSING_DEVICE,
        ),
        ("devices_current", DEVICES_CURRENT),
        ("profile_current_minimal", PROFILE_CURRENT_MINIMAL),
        ("profile_legacy_full", PROFILE_LEGACY_FULL),
    ];

    #[test]
    fn all_spotify_fixtures_are_valid_json() {
        for (name, body) in ALL {
            serde_json::from_str::<serde_json::Value>(body)
                .unwrap_or_else(|error| panic!("{name} should be valid JSON: {error}"));
        }
    }
}
