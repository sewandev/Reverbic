pub mod devices;
pub mod error;
pub mod library;
pub mod oauth;
pub mod player;
pub mod playlists;
pub mod radio;
pub mod search;
pub mod albums;

pub use error::SpotifyError;

pub use devices::SpotifyPlaybackState;

pub enum AuthResult {
    Success {
        username: String,
        search_token: String,
        refresh_token: String,
        audio_token: String,
        is_premium: bool,
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
}

pub enum SpotifyPlayerEvent {
    Playing,
    Paused,
    Stopped,
    EndOfTrack,
    TrackChanged(SpotifyTrack),
    Error(String),
}
