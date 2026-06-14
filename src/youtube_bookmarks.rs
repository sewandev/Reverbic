use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::integrations::youtube::YoutubeVideo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YoutubeBookmark {
    pub id: String,
    pub title: String,
    pub channel: String,
    #[serde(default)]
    pub duration_secs: u32,
    pub watch_url: String,
}

impl YoutubeBookmark {
    fn to_video(&self) -> YoutubeVideo {
        YoutubeVideo {
            id: self.id.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            duration_secs: self.duration_secs,
            watch_url: self.watch_url.clone(),
            thumbnail: None,
            is_live: false,
        }
    }

    fn from_video(video: &YoutubeVideo) -> Self {
        Self {
            id: video.id.clone(),
            title: video.title.clone(),
            channel: video.channel.clone(),
            duration_secs: video.duration_secs,
            watch_url: video.watch_url.clone(),
        }
    }
}

pub fn load() -> Vec<YoutubeVideo> {
    match std::fs::read_to_string(path()) {
        Ok(data) => match serde_json::from_str::<Vec<YoutubeBookmark>>(&data) {
            Ok(bookmarks) => bookmarks.iter().map(YoutubeBookmark::to_video).collect(),
            Err(e) => {
                tracing::warn!("YouTube bookmarks file is corrupt, using an empty list: {e}");
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    }
}

pub fn save(bookmarks: &[YoutubeVideo]) {
    let entries: Vec<YoutubeBookmark> = bookmarks.iter().map(YoutubeBookmark::from_video).collect();
    let _ = crate::config::save_json_atomic(&path(), &entries);
}

pub fn toggle(bookmarks: &mut Vec<YoutubeVideo>, video: YoutubeVideo) -> bool {
    if let Some(i) = bookmarks.iter().position(|b| b.id == video.id) {
        bookmarks.remove(i);
        false
    } else {
        bookmarks.push(video);
        true
    }
}

fn path() -> PathBuf {
    crate::paths::youtube_bookmarks_file()
}
