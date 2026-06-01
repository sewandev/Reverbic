use crate::i18n::t;

pub enum SearchMode {
    Name,
    Genre,
    Country,
    Settings,
    Integrations,
}

pub enum SpotifyAuthStatus {
    Idle,
    Connecting,
    LoggedIn,
    Error(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum IntegrationView {
    ServiceList,
    SpotifyDetail,
    SpotifyWebBrowser,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SettingItem {
    Autoplay,
    RestoreVolume,
    Crossfade,
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
}

impl SettingItem {
    pub fn label(self) -> String {
        match self {
            Self::Autoplay        => t("config.setting.autoplay"),
            Self::RestoreVolume   => t("config.setting.restore_volume"),
            Self::Crossfade       => t("config.setting.crossfade"),
            Self::OverlayMode     => t("config.setting.overlay"),
            Self::OverlayAlpha    => t("config.setting.overlay_alpha"),
            Self::OverlayPosition => t("config.setting.overlay_position"),
            Self::Screensaver     => t("config.setting.screensaver"),
            Self::DuckEnabled     => t("config.setting.duck"),
            Self::DuckVolume      => t("config.setting.duck_volume"),
            Self::MediaKeys       => t("config.setting.media_keys"),
            Self::TrayIcon        => t("config.setting.tray"),
            Self::Notifications   => t("config.setting.notifications"),
            Self::Language        => t("config.setting.language"),
        }
    }

    pub fn tooltip_key(self) -> &'static str {
        match self {
            Self::Autoplay        => "config.tooltip.autoplay",
            Self::RestoreVolume   => "config.tooltip.restore_volume",
            Self::Crossfade       => "config.tooltip.crossfade",
            Self::OverlayMode     => "config.tooltip.overlay",
            Self::OverlayAlpha    => "config.tooltip.overlay_alpha",
            Self::OverlayPosition => "config.tooltip.overlay_position",
            Self::Screensaver     => "config.tooltip.screensaver",
            Self::DuckEnabled     => "config.tooltip.duck",
            Self::DuckVolume      => "config.tooltip.duck_volume",
            Self::MediaKeys       => "config.tooltip.media_keys",
            Self::TrayIcon        => "config.tooltip.tray",
            Self::Notifications   => "config.tooltip.notifications",
            Self::Language        => "config.tooltip.language",
        }
    }

    pub fn group_key(self) -> &'static str {
        match self {
            Self::Autoplay | Self::RestoreVolume | Self::Crossfade
                => "config.group.playback",
            Self::OverlayMode | Self::OverlayAlpha | Self::OverlayPosition | Self::Screensaver
                => "config.group.overlay",
            Self::DuckEnabled | Self::DuckVolume
                => "config.group.game",
            Self::MediaKeys | Self::TrayIcon | Self::Notifications
                => "config.group.system",
            Self::Language
                => "config.group.appearance",
        }
    }
}

pub fn settings_items(duck_enabled: bool) -> Vec<SettingItem> {
    let mut items = vec![
        SettingItem::Autoplay,
        SettingItem::RestoreVolume,
        SettingItem::Crossfade,
        SettingItem::OverlayMode,
        SettingItem::OverlayAlpha,
        SettingItem::OverlayPosition,
        SettingItem::Screensaver,
        SettingItem::DuckEnabled,
    ];
    if duck_enabled {
        items.push(SettingItem::DuckVolume);
    }
    items.extend([
        SettingItem::MediaKeys,
        SettingItem::TrayIcon,
        SettingItem::Notifications,
        SettingItem::Language,
    ]);
    items
}

pub enum AppFocus {
    Stations,
    RecentTracks,
    StationSearch,
    OnDemandList,
}
