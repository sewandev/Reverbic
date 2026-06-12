use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::reverbic_dir;
use crate::favorites::FavoriteStation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RadioPlaylist {
    pub name: String,
    #[serde(default)]
    pub stations: Vec<FavoriteStation>,
}

pub fn load() -> Vec<RadioPlaylist> {
    match std::fs::read_to_string(path()) {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(playlists) => playlists,
            Err(e) => {
                tracing::warn!("Playlists file is corrupt, using an empty list: {e}");
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    }
}

pub fn save(playlists: &[RadioPlaylist]) {
    let _ = crate::config::save_json_atomic(&path(), playlists);
}

fn path() -> PathBuf {
    reverbic_dir().join("playlists.json")
}
