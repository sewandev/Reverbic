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
    Liked,
    Playlists,
    TopTracks,
    Recent,
    Albums,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum RadioSubTab {
    #[default]
    Search,
    Favorites,
    Playlists,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum YoutubeSubTab {
    #[default]
    Search,
    Liked,
    Playlists,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SettingItem {
    Autoplay,
    RestoreVolume,
    Crossfade,
    YoutubeCrossfade,
    VolumeStep,
    Prebuffer,
    OverlayMode,
    OverlayAlpha,
    OverlayPosition,
    OverlayStyle,
    Screensaver,
    DuckEnabled,
    DuckVolume,
    MediaKeys,
    TrayIcon,
    Notifications,
    Language,
    Theme,
    ScreensaverClock,
    SpotifyStopOnQuit,
    SpotifyStartOnSpotify,
    SpotifyClientId,
    SpotifyPlaybackMode,
    SpotifyRadioMode,
    YoutubeRadioMode,
    YoutubeSponsorblock,
    YoutubeCookiesPath,
    YoutubeCookiesValidate,
    AutoUpdate,
    DiscordRpc,
    ReplayOnboarding,
    OpenLogs,
}

impl SettingItem {
    pub fn label(self) -> String {
        match self {
            Self::Autoplay => t("config.setting.autoplay"),
            Self::RestoreVolume => t("config.setting.restore_volume"),
            Self::Crossfade => t("config.setting.crossfade"),
            Self::YoutubeCrossfade => t("config.setting.youtube_crossfade"),
            Self::VolumeStep => t("config.setting.volume_step"),
            Self::Prebuffer => t("config.setting.prebuffer"),
            Self::OverlayMode => t("config.setting.overlay"),
            Self::OverlayAlpha => t("config.setting.overlay_alpha"),
            Self::OverlayPosition => t("config.setting.overlay_position"),
            Self::OverlayStyle => t("config.setting.overlay_style"),
            Self::Screensaver => t("config.setting.screensaver"),
            Self::DuckEnabled => t("config.setting.duck"),
            Self::DuckVolume => t("config.setting.duck_volume"),
            Self::MediaKeys => t("config.setting.media_keys"),
            Self::TrayIcon => t("config.setting.tray"),
            Self::Notifications => t("config.setting.notifications"),
            Self::Language => t("config.setting.language"),
            Self::Theme => t("config.setting.theme"),
            Self::ScreensaverClock => t("config.setting.screensaver_clock"),
            Self::SpotifyStopOnQuit => t("config.setting.spotify_stop_on_quit"),
            Self::SpotifyStartOnSpotify => t("config.setting.spotify_start_on_spotify"),
            Self::SpotifyClientId => t("config.setting.spotify_client_id"),
            Self::SpotifyPlaybackMode => t("config.setting.spotify_playback_mode"),
            Self::AutoUpdate => t("config.setting.auto_update"),
            Self::DiscordRpc => t("config.setting.discord_rpc"),
            Self::ReplayOnboarding => t("config.setting.replay_onboarding"),
            Self::OpenLogs => t("config.setting.open_logs"),
            Self::SpotifyRadioMode => t("config.setting.spotify_radio_mode"),
            Self::YoutubeRadioMode => t("config.setting.youtube_radio_mode"),
            Self::YoutubeSponsorblock => t("config.setting.youtube_sponsorblock"),
            Self::YoutubeCookiesPath => t("config.setting.youtube_cookies_path"),
            Self::YoutubeCookiesValidate => t("config.setting.youtube_cookies_validate"),
        }
    }

    pub fn tooltip_key(self) -> &'static str {
        match self {
            Self::Autoplay => "config.tooltip.autoplay",
            Self::RestoreVolume => "config.tooltip.restore_volume",
            Self::Crossfade => "config.tooltip.crossfade",
            Self::YoutubeCrossfade => "config.tooltip.youtube_crossfade",
            Self::VolumeStep => "config.tooltip.volume_step",
            Self::Prebuffer => "config.tooltip.prebuffer",
            Self::OverlayMode => "config.tooltip.overlay",
            Self::OverlayAlpha => "config.tooltip.overlay_alpha",
            Self::OverlayPosition => "config.tooltip.overlay_position",
            Self::OverlayStyle => "config.tooltip.overlay_style",
            Self::Screensaver => "config.tooltip.screensaver",
            Self::DuckEnabled => "config.tooltip.duck",
            Self::DuckVolume => "config.tooltip.duck_volume",
            Self::MediaKeys => "config.tooltip.media_keys",
            Self::TrayIcon => "config.tooltip.tray",
            Self::Notifications => "config.tooltip.notifications",
            Self::Language => "config.tooltip.language",
            Self::Theme => "config.tooltip.theme",
            Self::ScreensaverClock => "config.tooltip.screensaver_clock",
            Self::SpotifyStopOnQuit => "config.tooltip.spotify_stop_on_quit",
            Self::SpotifyStartOnSpotify => "config.tooltip.spotify_start_on_spotify",
            Self::SpotifyClientId => "config.tooltip.spotify_client_id",
            Self::SpotifyPlaybackMode => "config.tooltip.spotify_playback_mode",
            Self::AutoUpdate => "config.tooltip.auto_update",
            Self::DiscordRpc => "config.tooltip.discord_rpc",
            Self::ReplayOnboarding => "config.tooltip.replay_onboarding",
            Self::OpenLogs => "config.tooltip.open_logs",
            Self::SpotifyRadioMode => "config.tooltip.spotify_radio_mode",
            Self::YoutubeRadioMode => "config.tooltip.youtube_radio_mode",
            Self::YoutubeSponsorblock => "config.tooltip.youtube_sponsorblock",
            Self::YoutubeCookiesPath => "config.tooltip.youtube_cookies_path",
            Self::YoutubeCookiesValidate => "config.tooltip.youtube_cookies_validate",
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
            | Self::OverlayStyle
            | Self::Screensaver
            | Self::ScreensaverClock => "config.group.overlay",
            Self::DuckEnabled | Self::DuckVolume => "config.group.game",
            Self::MediaKeys
            | Self::TrayIcon
            | Self::Notifications
            | Self::AutoUpdate
            | Self::DiscordRpc
            | Self::ReplayOnboarding
            | Self::OpenLogs => "config.group.system",
            Self::Language | Self::Theme => "config.group.appearance",
            Self::SpotifyStopOnQuit
            | Self::SpotifyStartOnSpotify
            | Self::SpotifyClientId
            | Self::SpotifyPlaybackMode
            | Self::SpotifyRadioMode
            | Self::YoutubeCrossfade
            | Self::YoutubeRadioMode
            | Self::YoutubeSponsorblock
            | Self::YoutubeCookiesPath
            | Self::YoutubeCookiesValidate => "config.group.integrations",
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
        SettingItem::OverlayStyle,
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
        SettingItem::ReplayOnboarding,
        SettingItem::OpenLogs,
        SettingItem::Language,
        SettingItem::Theme,
        SettingItem::SpotifyStopOnQuit,
        SettingItem::SpotifyStartOnSpotify,
        SettingItem::SpotifyClientId,
        SettingItem::SpotifyPlaybackMode,
        SettingItem::SpotifyRadioMode,
        SettingItem::YoutubeCrossfade,
        SettingItem::YoutubeRadioMode,
        SettingItem::YoutubeSponsorblock,
        SettingItem::YoutubeCookiesPath,
        SettingItem::YoutubeCookiesValidate,
    ]);
    items
}

pub enum AppFocus {
    Stations,
    RecentTracks,
    StationSearch,
    OnDemandList,
}
