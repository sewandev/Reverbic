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
    PublicPlaylists,
    Bookmarks,
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
    ScreensaverLogo,
    ScreensaverVisualizer,
    ScreensaverRecentTracks,
    ScreensaverProgressBar,
    ScreensaverStationDetails,
    ScreensaverNowPlaying,
    SpotifyStopOnQuit,
    SpotifyStartOnSpotify,
    SpotifyClientId,
    SpotifyPlaybackMode,
    SpotifyCrossfade,
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
            Self::ScreensaverLogo => t("config.setting.screensaver_logo"),
            Self::ScreensaverVisualizer => t("config.setting.screensaver_visualizer"),
            Self::ScreensaverRecentTracks => t("config.setting.screensaver_recent_tracks"),
            Self::ScreensaverProgressBar => t("config.setting.screensaver_progress_bar"),
            Self::ScreensaverStationDetails => t("config.setting.screensaver_station_details"),
            Self::ScreensaverNowPlaying => t("config.setting.screensaver_now_playing"),
            Self::SpotifyStopOnQuit => t("config.setting.spotify_stop_on_quit"),
            Self::SpotifyStartOnSpotify => t("config.setting.spotify_start_on_spotify"),
            Self::SpotifyClientId => t("config.setting.spotify_client_id"),
            Self::SpotifyPlaybackMode => t("config.setting.spotify_playback_mode"),
            Self::SpotifyCrossfade => t("config.setting.spotify_crossfade"),
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
            Self::ScreensaverLogo => "config.tooltip.screensaver_logo",
            Self::ScreensaverVisualizer => "config.tooltip.screensaver_visualizer",
            Self::ScreensaverRecentTracks => "config.tooltip.screensaver_recent_tracks",
            Self::ScreensaverProgressBar => "config.tooltip.screensaver_progress_bar",
            Self::ScreensaverStationDetails => "config.tooltip.screensaver_station_details",
            Self::ScreensaverNowPlaying => "config.tooltip.screensaver_now_playing",
            Self::SpotifyStopOnQuit => "config.tooltip.spotify_stop_on_quit",
            Self::SpotifyStartOnSpotify => "config.tooltip.spotify_start_on_spotify",
            Self::SpotifyClientId => "config.tooltip.spotify_client_id",
            Self::SpotifyPlaybackMode => "config.tooltip.spotify_playback_mode",
            Self::SpotifyCrossfade => "config.tooltip.spotify_crossfade",
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
            | Self::Prebuffer => "config.group.radio",
            Self::SpotifyRadioMode
            | Self::SpotifyStopOnQuit
            | Self::SpotifyStartOnSpotify
            | Self::SpotifyClientId
            | Self::SpotifyPlaybackMode
            | Self::SpotifyCrossfade => "config.group.spotify",
            Self::YoutubeRadioMode
            | Self::YoutubeSponsorblock
            | Self::YoutubeCrossfade
            | Self::YoutubeCookiesPath
            | Self::YoutubeCookiesValidate => "config.group.youtube",
            Self::OverlayMode | Self::OverlayAlpha | Self::OverlayPosition | Self::OverlayStyle => {
                "config.group.overlay"
            }
            Self::Screensaver
            | Self::ScreensaverClock
            | Self::ScreensaverLogo
            | Self::ScreensaverVisualizer
            | Self::ScreensaverRecentTracks
            | Self::ScreensaverProgressBar
            | Self::ScreensaverStationDetails
            | Self::ScreensaverNowPlaying => "config.group.ambient",
            Self::DuckEnabled | Self::DuckVolume => "config.group.game",
            Self::MediaKeys
            | Self::TrayIcon
            | Self::Notifications
            | Self::AutoUpdate
            | Self::DiscordRpc
            | Self::ReplayOnboarding
            | Self::OpenLogs => "config.group.system",
            Self::Language | Self::Theme => "config.group.appearance",
        }
    }

    fn is_windows_only(self) -> bool {
        matches!(
            self,
            Self::OverlayMode
                | Self::OverlayAlpha
                | Self::OverlayPosition
                | Self::OverlayStyle
                | Self::DuckEnabled
                | Self::DuckVolume
                | Self::MediaKeys
                | Self::TrayIcon
                | Self::Notifications
                | Self::DiscordRpc
        )
    }
}

pub fn settings_items(duck_enabled: bool) -> Vec<SettingItem> {
    let mut items = vec![
        SettingItem::Autoplay,
        SettingItem::RestoreVolume,
        SettingItem::Crossfade,
        SettingItem::VolumeStep,
        SettingItem::Prebuffer,
        SettingItem::SpotifyRadioMode,
        SettingItem::SpotifyStopOnQuit,
        SettingItem::SpotifyStartOnSpotify,
        SettingItem::SpotifyClientId,
        SettingItem::SpotifyPlaybackMode,
        SettingItem::SpotifyCrossfade,
        SettingItem::YoutubeRadioMode,
        SettingItem::YoutubeSponsorblock,
        SettingItem::YoutubeCrossfade,
        SettingItem::YoutubeCookiesPath,
        SettingItem::YoutubeCookiesValidate,
        SettingItem::OverlayMode,
        SettingItem::Screensaver,
    ];

    items.push(SettingItem::DuckEnabled);
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
    ]);

    if !cfg!(target_os = "windows") {
        items.retain(|item| !item.is_windows_only());
    }

    items
}

pub fn ambient_items() -> Vec<SettingItem> {
    vec![
        SettingItem::Screensaver,
        SettingItem::ScreensaverClock,
        SettingItem::ScreensaverLogo,
        SettingItem::ScreensaverVisualizer,
        SettingItem::ScreensaverRecentTracks,
        SettingItem::ScreensaverProgressBar,
        SettingItem::ScreensaverStationDetails,
        SettingItem::ScreensaverNowPlaying,
    ]
}

pub fn overlay_items() -> Vec<SettingItem> {
    vec![
        SettingItem::OverlayMode,
        SettingItem::OverlayStyle,
        SettingItem::OverlayAlpha,
        SettingItem::OverlayPosition,
    ]
}

pub enum AppFocus {
    Stations,
    RecentTracks,
    StationSearch,
    OnDemandList,
}
