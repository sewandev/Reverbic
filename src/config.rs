use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use crate::i18n::Language;
pub use crate::ui::theme::ThemeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastStation {
    pub key: String,
    pub name: String,
    pub url: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameIntegrationsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub dota2: bool,
}

impl OverlayMode {
    pub fn display(self) -> String {
        use crate::i18n::t;
        match self {
            Self::WhenPlaying => t("overlay.when_playing"),
            Self::Always => t("overlay.always"),
            Self::Hidden => t("overlay.hidden"),
            Self::Games => t("overlay.games"),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::WhenPlaying => Self::Always,
            Self::Always => Self::Hidden,
            Self::Hidden => Self::Games,
            Self::Games => Self::WhenPlaying,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OverlayStyle {
    #[default]
    Full,
    Compact,
}

impl OverlayStyle {
    pub fn display(self) -> String {
        use crate::i18n::t;
        match self {
            Self::Full => t("overlay.style.full"),
            Self::Compact => t("overlay.style.compact"),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Full => Self::Compact,
            Self::Compact => Self::Full,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OverlayPosition {
    #[default]
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

impl OverlayPosition {
    pub fn display(self) -> String {
        use crate::i18n::t;
        match self {
            Self::TopLeft => t("overlay.top_left"),
            Self::TopRight => t("overlay.top_right"),
            Self::BottomRight => t("overlay.bottom_right"),
            Self::BottomLeft => t("overlay.bottom_left"),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::TopLeft => Self::TopRight,
            Self::TopRight => Self::BottomRight,
            Self::BottomRight => Self::BottomLeft,
            Self::BottomLeft => Self::TopLeft,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpotifyConfig {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub search_token: Option<String>,
    #[serde(skip_serializing, default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub is_premium: Option<bool>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub followers: Option<u32>,
    #[serde(default = "default_true")]
    pub stop_on_quit: bool,
    #[serde(default)]
    pub start_on_spotify: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub volume: f32,
    pub last_selected: usize,
    #[serde(default)]
    pub autoplay_last: bool,
    #[serde(default)]
    pub search_history: Vec<String>,
    #[serde(default)]
    pub overlay_mode: OverlayMode,
    #[serde(default)]
    pub last_station: Option<LastStation>,
    #[serde(default)]
    pub crossfade_secs: u8,
    #[serde(default)]
    pub media_keys: bool,
    #[serde(default)]
    pub tray_icon: bool,
    #[serde(default)]
    pub notifications: bool,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub theme: ThemeId,
    #[serde(default = "default_true")]
    pub restore_volume: bool,
    #[serde(default)]
    pub duck_enabled: bool,
    #[serde(default = "default_duck_volume")]
    pub duck_volume: u8,
    #[serde(default = "default_overlay_alpha")]
    pub overlay_alpha: u8,
    #[serde(default)]
    pub overlay_position: OverlayPosition,
    #[serde(default)]
    pub overlay_style: OverlayStyle,
    #[serde(default = "default_screensaver_secs")]
    pub screensaver_secs: u16,
    #[serde(default)]
    pub game_integrations: GameIntegrationsConfig,
    #[serde(default)]
    pub spotify: SpotifyConfig,
    #[serde(default = "default_volume_step")]
    pub volume_step: u8,
    #[serde(default = "default_prebuffer_secs")]
    pub prebuffer_secs: u8,
    #[serde(default = "default_true")]
    pub screensaver_clock: bool,
    #[serde(default = "default_true")]
    pub auto_update: bool,
    #[serde(default)]
    pub discord_rpc: bool,
}

fn default_true() -> bool {
    true
}
fn default_duck_volume() -> u8 {
    40
}
fn default_overlay_alpha() -> u8 {
    90
}
fn default_screensaver_secs() -> u16 {
    10
}
fn default_volume_step() -> u8 {
    5
}
fn default_prebuffer_secs() -> u8 {
    30
}

impl Config {
    pub fn screensaver_next(&mut self) {
        self.screensaver_secs = match self.screensaver_secs {
            0 => 10,
            10 => 20,
            20 => 30,
            30 => 60,
            60 => 120,
            120 => 300,
            _ => 0,
        };
    }
}

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
            theme: ThemeId::default(),
            restore_volume: true,
            duck_enabled: false,
            duck_volume: 40,
            overlay_alpha: 90,
            overlay_position: OverlayPosition::TopLeft,
            overlay_style: OverlayStyle::Full,
            screensaver_secs: 10,
            game_integrations: GameIntegrationsConfig::default(),
            spotify: SpotifyConfig::default(),
            volume_step: 5,
            prebuffer_secs: 30,
            screensaver_clock: true,
            auto_update: true,
            discord_rpc: false,
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
    pub fn volume_step_next(&mut self) {
        self.volume_step = match self.volume_step {
            1 => 2,
            2 => 5,
            5 => 10,
            _ => 1,
        };
    }

    pub fn prebuffer_next(&mut self) {
        self.prebuffer_secs = match self.prebuffer_secs {
            10 => 30,
            30 => 60,
            _ => 10,
        };
    }
}

impl Config {
    pub fn add_to_history(&mut self, query: String) {
        if query.trim().is_empty() {
            return;
        }
        self.search_history.retain(|h| h != &query);
        self.search_history.insert(0, query);
        self.search_history.truncate(10);
    }
}

#[cfg(target_os = "windows")]
fn detect_system_language() -> Language {
    use windows::Win32::Globalization::GetUserDefaultLocaleName;
    let mut buf = [0u16; 85];
    let len = unsafe { GetUserDefaultLocaleName(&mut buf) };
    if len > 1 {
        let locale = String::from_utf16_lossy(&buf[..(len as usize - 1)]);
        if locale.to_ascii_lowercase().starts_with("es") {
            return Language::Es;
        }
    }
    Language::En
}

#[cfg(not(target_os = "windows"))]
fn detect_system_language() -> Language {
    Language::En
}

impl Config {
    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            return Self {
                language: detect_system_language(),
                ..Default::default()
            };
        }
        let Ok(data) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let mut config = match serde_json::from_str::<Self>(&data) {
            Ok(c) => {
                tracing::info!("Config loaded from {:?}", path);
                c
            }
            Err(e) => {
                tracing::warn!("Invalid config ({e}), using defaults");
                Self::default()
            }
        };
        if let Err(error) = load_spotify_refresh_token(&mut config, spotify_token_entry()) {
            tracing::warn!(
                "Failed to load Spotify refresh token from keyring: {}",
                error.message()
            );
        }
        config
    }

    pub fn save(&self) {
        if let Err(error) =
            save_spotify_refresh_token(self.spotify.refresh_token.as_deref(), spotify_token_entry())
        {
            tracing::warn!(
                "Failed to update Spotify refresh token in keyring: {}",
                error.message()
            );
        }
        let path = Self::path();
        match save_json_atomic(&path, self) {
            Ok(()) => tracing::info!("Config saved to {:?}", path),
            Err(e) => tracing::error!("Failed to save config: {e}"),
        }
    }

    fn path() -> PathBuf {
        reverbic_dir().join("config.json")
    }
}

#[derive(Debug)]
enum SpotifyTokenPersistenceError {
    KeyringUnavailable(String),
    LoadFailed(String),
    SaveFailed(String),
    DeleteFailed(String),
}

impl SpotifyTokenPersistenceError {
    fn message(&self) -> &str {
        match self {
            Self::KeyringUnavailable(message)
            | Self::LoadFailed(message)
            | Self::SaveFailed(message)
            | Self::DeleteFailed(message) => message,
        }
    }
}

trait SpotifyTokenStore {
    fn get_password(&self) -> Result<String, String>;
    fn set_password(&self, token: &str) -> Result<(), String>;
    fn delete_credential(&self) -> Result<(), String>;
}

impl SpotifyTokenStore for keyring::Entry {
    fn get_password(&self) -> Result<String, String> {
        keyring::Entry::get_password(self).map_err(|e| e.to_string())
    }

    fn set_password(&self, token: &str) -> Result<(), String> {
        keyring::Entry::set_password(self, token).map_err(|e| e.to_string())
    }

    fn delete_credential(&self) -> Result<(), String> {
        keyring::Entry::delete_credential(self).map_err(|e| e.to_string())
    }
}

fn spotify_token_entry() -> Result<keyring::Entry, SpotifyTokenPersistenceError> {
    keyring::Entry::new("reverbic", "spotify_refresh_token")
        .map_err(|e| SpotifyTokenPersistenceError::KeyringUnavailable(e.to_string()))
}

fn load_spotify_refresh_token<S: SpotifyTokenStore>(
    config: &mut Config,
    entry: Result<S, SpotifyTokenPersistenceError>,
) -> Result<(), SpotifyTokenPersistenceError> {
    match entry {
        Ok(entry) => match entry.get_password() {
            Ok(token) => {
                config.spotify.refresh_token = Some(token);
                Ok(())
            }
            Err(error) => Err(SpotifyTokenPersistenceError::LoadFailed(error)),
        },
        Err(error) => Err(error),
    }
}

fn save_spotify_refresh_token<S: SpotifyTokenStore>(
    refresh_token: Option<&str>,
    entry: Result<S, SpotifyTokenPersistenceError>,
) -> Result<(), SpotifyTokenPersistenceError> {
    match entry {
        Ok(entry) => match refresh_token {
            Some(token) => entry
                .set_password(token)
                .map_err(SpotifyTokenPersistenceError::SaveFailed),
            None => entry
                .delete_credential()
                .map_err(SpotifyTokenPersistenceError::DeleteFailed),
        },
        Err(error) => Err(error),
    }
}

pub fn save_json_atomic<T: Serialize + ?Sized>(
    path: &std::path::Path,
    data: &T,
) -> std::io::Result<()> {
    let dir = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "no parent directory")
    })?;
    std::fs::create_dir_all(dir)?;
    let tmp = path.with_extension("tmp");
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&tmp, &json)?;
    replace_file(&tmp, path)
}

fn replace_file(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if !dst.exists() {
        return std::fs::rename(src, dst);
    }

    #[cfg(target_os = "windows")]
    {
        let backup = dst.with_extension("old");
        if backup.exists() {
            let _ = std::fs::remove_file(&backup);
        }

        std::fs::rename(dst, &backup)?;
        match std::fs::rename(src, dst) {
            Ok(()) => {
                let _ = std::fs::remove_file(&backup);
                Ok(())
            }
            Err(err) => {
                let _ = std::fs::rename(&backup, dst);
                Err(err)
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::fs::rename(src, dst)
    }
}

pub(crate) fn reverbic_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".reverbic")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct MockSpotifyTokenStore {
        get_password: Result<String, String>,
        set_password: Result<(), String>,
        delete_credential: Result<(), String>,
    }

    impl MockSpotifyTokenStore {
        fn ok() -> Self {
            Self {
                get_password: Ok("refresh-token".to_string()),
                set_password: Ok(()),
                delete_credential: Ok(()),
            }
        }

        fn with_get_error(message: &str) -> Self {
            Self {
                get_password: Err(message.to_string()),
                ..Self::ok()
            }
        }

        fn with_set_error(message: &str) -> Self {
            Self {
                set_password: Err(message.to_string()),
                ..Self::ok()
            }
        }

        fn with_delete_error(message: &str) -> Self {
            Self {
                delete_credential: Err(message.to_string()),
                ..Self::ok()
            }
        }
    }

    impl SpotifyTokenStore for MockSpotifyTokenStore {
        fn get_password(&self) -> Result<String, String> {
            self.get_password.clone()
        }

        fn set_password(&self, _token: &str) -> Result<(), String> {
            self.set_password.clone()
        }

        fn delete_credential(&self) -> Result<(), String> {
            self.delete_credential.clone()
        }
    }

    #[test]
    fn loads_refresh_token_from_keyring() {
        let mut config = Config::default();

        let result = load_spotify_refresh_token(&mut config, Ok(MockSpotifyTokenStore::ok()));

        assert!(result.is_ok());
        assert_eq!(
            config.spotify.refresh_token.as_deref(),
            Some("refresh-token")
        );
    }

    #[test]
    fn reports_keyring_entry_creation_failure_when_loading_refresh_token() {
        let mut config = Config::default();

        let result = load_spotify_refresh_token::<MockSpotifyTokenStore>(
            &mut config,
            Err(SpotifyTokenPersistenceError::KeyringUnavailable(
                "no keyring backend".to_string(),
            )),
        );

        assert!(matches!(
            result,
            Err(SpotifyTokenPersistenceError::KeyringUnavailable(_))
        ));
        assert!(config.spotify.refresh_token.is_none());
    }

    #[test]
    fn reports_keyring_read_failure_when_loading_refresh_token() {
        let mut config = Config::default();

        let result = load_spotify_refresh_token(
            &mut config,
            Ok(MockSpotifyTokenStore::with_get_error("read failed")),
        );

        assert!(matches!(
            result,
            Err(SpotifyTokenPersistenceError::LoadFailed(_))
        ));
        assert!(config.spotify.refresh_token.is_none());
    }

    #[test]
    fn reports_keyring_write_failure_when_saving_refresh_token() {
        let result = save_spotify_refresh_token(
            Some("refresh-token"),
            Ok(MockSpotifyTokenStore::with_set_error("write failed")),
        );

        assert!(matches!(
            result,
            Err(SpotifyTokenPersistenceError::SaveFailed(_))
        ));
    }

    #[test]
    fn reports_keyring_delete_failure_when_clearing_refresh_token() {
        let result = save_spotify_refresh_token(
            None,
            Ok(MockSpotifyTokenStore::with_delete_error("delete failed")),
        );

        assert!(matches!(
            result,
            Err(SpotifyTokenPersistenceError::DeleteFailed(_))
        ));
    }

    #[test]
    fn theme_defaults_for_old_configs_and_serializes_for_persistence() {
        let old_config = json!({
            "volume": 0.75,
            "last_selected": 3
        });

        let config: Config =
            serde_json::from_value(old_config).expect("old config without theme should load");

        assert_eq!(config.theme, ThemeId::Reverbic);

        let saved = serde_json::to_value(&config).expect("config should serialize");

        assert_eq!(saved["theme"], json!("reverbic"));
    }

    #[test]
    fn save_json_atomic_overwrites_existing_file() {
        let dir = temp_test_dir();
        std::fs::create_dir_all(&dir).expect("test temp dir should be created");
        let path = dir.join("config.json");

        save_json_atomic(&path, &json!({ "volume": 1 }))
            .expect("first save should create the config file");
        save_json_atomic(&path, &json!({ "volume": 2 }))
            .expect("second save should replace the config file");

        let saved = std::fs::read_to_string(&path).expect("saved config should be readable");
        assert!(saved.contains("\"volume\": 2"));
        assert!(!path.with_extension("tmp").exists());
        assert!(!path.with_extension("old").exists());

        let _ = std::fs::remove_dir_all(dir);
    }

    fn temp_test_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "reverbic-save-json-{}-{}",
            std::process::id(),
            unique
        ))
    }
}
