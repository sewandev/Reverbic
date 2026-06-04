use crate::i18n::t;

#[derive(Clone, Copy)]
pub enum SearchMode {
    Name,
    Genre,
    Country,
    Settings,
    Spotify,
    Youtube,
}

pub enum SpotifyAuthStatus {
    Idle,
    Connecting,
    LoggedIn,
    Error(String),
}

#[derive(PartialEq)]
pub enum SpotifyPlayerStatus {
    Idle,
    Loading,
    Playing,
    Paused,
    Error(String),
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SpotifySubTab {
    #[default]
    Search,
    Devices,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum RadioSubTab {
    #[default]
    Search,
    Favorites,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SettingItem {
    Autoplay,
    RestoreVolume,
    Crossfade,
    VolumeStep,
    Prebuffer,
    OverlayMode,
    OverlayAlpha,
    OverlayPosition,
    Screensaver,
    DuckEnabled,
    DuckVolume,
    MediaKeys,
    TrayIcon,
    Notifications,
    Language,
    ScreensaverClock,
    SpotifyStopOnQuit,
    SpotifyStartOnSpotify,
    SpotifyClientId,
    AutoUpdate,
    DiscordRpc,
}

impl SettingItem {
    pub fn label(self) -> String {
        match self {
            Self::Autoplay => t("config.setting.autoplay"),
            Self::RestoreVolume => t("config.setting.restore_volume"),
            Self::Crossfade => t("config.setting.crossfade"),
            Self::VolumeStep => t("config.setting.volume_step"),
            Self::Prebuffer => t("config.setting.prebuffer"),
            Self::OverlayMode => t("config.setting.overlay"),
            Self::OverlayAlpha => t("config.setting.overlay_alpha"),
            Self::OverlayPosition => t("config.setting.overlay_position"),
            Self::Screensaver => t("config.setting.screensaver"),
            Self::DuckEnabled => t("config.setting.duck"),
            Self::DuckVolume => t("config.setting.duck_volume"),
            Self::MediaKeys => t("config.setting.media_keys"),
            Self::TrayIcon => t("config.setting.tray"),
            Self::Notifications => t("config.setting.notifications"),
            Self::Language => t("config.setting.language"),
            Self::ScreensaverClock => t("config.setting.screensaver_clock"),
            Self::SpotifyStopOnQuit => t("config.setting.spotify_stop_on_quit"),
            Self::SpotifyStartOnSpotify => t("config.setting.spotify_start_on_spotify"),
            Self::SpotifyClientId => t("config.setting.spotify_client_id"),
            Self::AutoUpdate => t("config.setting.auto_update"),
            Self::DiscordRpc => t("config.setting.discord_rpc"),
        }
    }

    pub fn tooltip_key(self) -> &'static str {
        match self {
            Self::Autoplay => "config.tooltip.autoplay",
            Self::RestoreVolume => "config.tooltip.restore_volume",
            Self::Crossfade => "config.tooltip.crossfade",
            Self::VolumeStep => "config.tooltip.volume_step",
            Self::Prebuffer => "config.tooltip.prebuffer",
            Self::OverlayMode => "config.tooltip.overlay",
            Self::OverlayAlpha => "config.tooltip.overlay_alpha",
            Self::OverlayPosition => "config.tooltip.overlay_position",
            Self::Screensaver => "config.tooltip.screensaver",
            Self::DuckEnabled => "config.tooltip.duck",
            Self::DuckVolume => "config.tooltip.duck_volume",
            Self::MediaKeys => "config.tooltip.media_keys",
            Self::TrayIcon => "config.tooltip.tray",
            Self::Notifications => "config.tooltip.notifications",
            Self::Language => "config.tooltip.language",
            Self::ScreensaverClock => "config.tooltip.screensaver_clock",
            Self::SpotifyStopOnQuit => "config.tooltip.spotify_stop_on_quit",
            Self::SpotifyStartOnSpotify => "config.tooltip.spotify_start_on_spotify",
            Self::SpotifyClientId => "config.tooltip.spotify_client_id",
            Self::AutoUpdate => "config.tooltip.auto_update",
            Self::DiscordRpc => "config.tooltip.discord_rpc",
        }
    }

    pub fn group_key(self) -> &'static str {
        match self {
            Self::Autoplay
            | Self::RestoreVolume
            | Self::Crossfade
            | Self::VolumeStep
            | Self::Prebuffer => "config.group.playback",
            Self::OverlayMode
            | Self::OverlayAlpha
            | Self::OverlayPosition
            | Self::Screensaver
            | Self::ScreensaverClock => "config.group.overlay",
            Self::DuckEnabled | Self::DuckVolume => "config.group.game",
            Self::MediaKeys
            | Self::TrayIcon
            | Self::Notifications
            | Self::AutoUpdate
            | Self::DiscordRpc => "config.group.system",
            Self::Language => "config.group.appearance",
            Self::SpotifyStopOnQuit | Self::SpotifyStartOnSpotify | Self::SpotifyClientId => {
                "config.group.integrations"
            }
        }
    }
}

pub fn settings_items(duck_enabled: bool) -> Vec<SettingItem> {
    let mut items = vec![
        SettingItem::Autoplay,
        SettingItem::RestoreVolume,
        SettingItem::Crossfade,
        SettingItem::VolumeStep,
        SettingItem::Prebuffer,
        SettingItem::OverlayMode,
        SettingItem::OverlayAlpha,
        SettingItem::OverlayPosition,
        SettingItem::Screensaver,
        SettingItem::ScreensaverClock,
        SettingItem::DuckEnabled,
    ];
    if duck_enabled {
        items.push(SettingItem::DuckVolume);
    }
    items.extend([
        SettingItem::MediaKeys,
        SettingItem::TrayIcon,
        SettingItem::Notifications,
        SettingItem::AutoUpdate,
        SettingItem::DiscordRpc,
        SettingItem::Language,
        SettingItem::SpotifyStopOnQuit,
        SettingItem::SpotifyStartOnSpotify,
        SettingItem::SpotifyClientId,
    ]);
    items
}

pub enum AppFocus {
    Stations,
    RecentTracks,
    StationSearch,
    OnDemandList,
}
