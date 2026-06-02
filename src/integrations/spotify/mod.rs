pub mod devices;
pub mod error;
pub mod oauth;
pub mod player;
pub mod search;

pub use error::SpotifyError;

pub use devices::SpotifyPlaybackState;

pub enum AuthResult {
    Success {
        username:      String,
        search_token:  String,
        refresh_token: String,
        audio_token:   String,
        is_premium:    bool,
        country:       Option<String>,
        followers:     Option<u32>,
    },
    Failure(String),
}

#[derive(Clone)]
pub struct SpotifyTrack {
    pub name:        String,
    pub artist:      String,
    pub album:       String,
    pub duration_ms: u32,
    pub uri:         String,
}

pub enum SpotifyPlayerCmd {
    Play(SpotifyTrack),
    Pause,
    Resume,
}

pub enum SpotifyPlayerEvent {
    Playing,
    Paused,
    Stopped,
    EndOfTrack,
    Error(String),
}
