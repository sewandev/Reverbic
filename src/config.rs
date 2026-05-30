
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub volume:         f32,
    pub last_selected:  usize,
    #[serde(default)]
    pub autoplay_last:  bool,
    #[serde(default)]
    pub search_history: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self { volume: 1.0, last_selected: 0, autoplay_last: false, search_history: Vec::new() }
    }
}

impl Config {
    pub fn add_to_history(&mut self, query: String) {
        if query.trim().is_empty() { return; }
        self.search_history.retain(|h| h != &query);
        self.search_history.insert(0, query);
        self.search_history.truncate(10);
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::path();
        let Ok(data) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        match serde_json::from_str::<Self>(&data) {
            Ok(c) => {
                tracing::info!("Config cargada desde {:?}", path);
                c
            }
            Err(e) => {
                tracing::warn!("Config inválida ({e}), usando defaults");
                Self::default()
            }
        }
    }
    pub fn save(&self) {
        let path = Self::path();
        let Some(dir) = path.parent() else { return };

        if std::fs::create_dir_all(dir).is_err() {
            tracing::error!("No se pudo crear directorio de config: {:?}", dir);
            return;
        }

        let tmp = path.with_extension("tmp");
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if std::fs::write(&tmp, &json).is_ok() {
                    if let Err(e) = std::fs::rename(&tmp, &path) {
                        tracing::error!("No se pudo guardar config: {e}");
                    } else {
                        tracing::info!("Config guardada en {:?}", path);
                    }
                }
            }
            Err(e) => tracing::error!("Error serializando config: {e}"),
        }
    }

    fn path() -> PathBuf {
        reverbic_dir().join("config.json")
    }
}

pub(crate) fn reverbic_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".reverbic")
}
