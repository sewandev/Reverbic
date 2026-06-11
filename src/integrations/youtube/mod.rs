pub mod cookies;
pub mod error;
pub mod install;
pub mod playlists;
pub mod quickjs;
pub mod resolve;
pub mod search;

pub use error::YoutubeError;

pub fn runtime_installed() -> bool {
    install::is_installed() && quickjs::is_installed()
}

#[derive(Clone, Debug, PartialEq)]
pub struct YoutubeVideo {
    pub id: String,
    pub title: String,
    pub channel: String,
    pub duration_secs: u32,
    pub watch_url: String,
    pub thumbnail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ResolvedYoutubePlayback {
    pub video: YoutubeVideo,
    pub stream_url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct YoutubePlaylist {
    pub id: String,
    pub title: String,
    pub video_count: u32,
}
