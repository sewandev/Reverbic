
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use crate::i18n::Language;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastStation {
    pub key:          String,
    pub name:         String,
    pub url:          String,
    pub bitrate_kbps: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OverlayMode {
    #[default]
    WhenPlaying,
    Always,
    Hidden,
    Games,
}

impl OverlayMode {
    pub fn display(self) -> String {
        use crate::i18n::t;
        match self {
            Self::WhenPlaying => t("overlay.when_playing"),
            Self::Always      => t("overlay.always"),
            Self::Hidden      => t("overlay.hidden"),
            Self::Games       => t("overlay.games"),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::WhenPlaying => Self::Always,
            Self::Always      => Self::Hidden,
            Self::Hidden      => Self::Games,
            Self::Games       => Self::WhenPlaying,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub volume:         f32,
    pub last_selected:  usize,
    #[serde(default)]
    pub autoplay_last:  bool,
    #[serde(default)]
    pub search_history: Vec<String>,
    #[serde(default)]
    pub overlay_mode:   OverlayMode,
    #[serde(default)]
    pub last_station:   Option<LastStation>,
    #[serde(default)]
    pub crossfade_secs: u8,
    #[serde(default)]
    pub media_keys:     bool,
    #[serde(default)]
    pub tray_icon:      bool,
    #[serde(default)]
    pub notifications:  bool,
    #[serde(default)]
    pub language:       Language,
    #[serde(default = "default_true")]
    pub restore_volume: bool,
    #[serde(default)]
    pub duck_enabled:    bool,
    #[serde(default = "default_duck_volume")]
    pub duck_volume:     u8,
    #[serde(default = "default_overlay_alpha")]
    pub overlay_alpha:   u8,
}

fn default_true() -> bool { true }
fn default_duck_volume() -> u8 { 40 }
fn default_overlay_alpha() -> u8 { 90 }

impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 1.0,
            last_selected: 0,
            autoplay_last: false,
            search_history: Vec::new(),
            overlay_mode: OverlayMode::WhenPlaying,
            last_station: None,
            crossfade_secs: 0,
            media_keys: false,
            tray_icon: false,
            notifications: false,
            language: Language::default(),
            restore_volume: true,
            duck_enabled:    false,
            duck_volume:     40,
            overlay_alpha:   90,
        }
    }
}

impl Config {
    pub fn crossfade_display(&self) -> String {
        use crate::i18n::t;
        match self.crossfade_secs {
            0 => t("crossfade.off"),
            1 => t("crossfade.1s"),
            2 => t("crossfade.2s"),
            _ => t("crossfade.3s"),
        }
    }

    pub fn crossfade_next(&mut self) {
        self.crossfade_secs = match self.crossfade_secs {
            0 => 1,
            1 => 2,
            2 => 3,
            _ => 0,
        };
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
