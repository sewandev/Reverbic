use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::reverbic_dir;
use crate::station::{enrich, find_enrichment, Station};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FavoriteStation {
    pub key: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub bitrate_kbps: Option<u16>,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub homepage: String,
}

impl FavoriteStation {
    pub fn to_station(&self) -> Station {
        let mut s = Station {
            key:              self.key.clone(),
            name:             self.name.clone(),
            url:              self.url.clone(),
            metadata_api_url: None,
            history_api_url:  None,
            schedule_url:     None,
            show_countdown:   false,
            bitrate_kbps:     self.bitrate_kbps,
        };
        if let Some(enrichment) = find_enrichment(&self.name) {
            enrich(&mut s, enrichment);
        }
        s
    }
}

pub fn load() -> Vec<FavoriteStation> {
    match std::fs::read_to_string(path()) {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(favs) => favs,
            Err(e) => {
                tracing::warn!("Archivo de favoritos corrupto, usando lista vacía: {e}");
                Vec::new()
            }
        },
        Err(_)   => Vec::new(),
    }
}

pub fn save(favorites: &[FavoriteStation]) {
    let _ = crate::config::save_json_atomic(&path(), favorites);
}
pub fn toggle(favorites: &mut Vec<FavoriteStation>, fav: FavoriteStation) -> bool {
    if let Some(i) = favorites.iter().position(|f| f.url == fav.url) {
        favorites.remove(i);
        false
    } else {
        favorites.push(fav);
        true
    }
}


pub fn move_up(favorites: &mut [FavoriteStation], i: usize) {
    if i > 0 && i < favorites.len() {
        favorites.swap(i, i - 1);
    }
}

pub fn move_down(favorites: &mut [FavoriteStation], i: usize) {
    if i + 1 < favorites.len() {
        favorites.swap(i, i + 1);
    }
}

fn path() -> PathBuf {
    reverbic_dir().join("favorites.json")
}
