pub mod error;
pub mod install;
pub mod resolve;
pub mod search;

pub use error::YoutubeError;

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
