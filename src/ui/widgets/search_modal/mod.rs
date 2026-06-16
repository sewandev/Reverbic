use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Widget},
};

use crate::app::{SearchMode, SpotifyAuthStatus};
use crate::i18n::t;
use crate::station::DynamicStation;
use crate::ui::theme::{self, Palette};

use helpers::{key, sep_s};
pub(crate) use layout::{
    auth_notice_box, filter_list_layout, header_list_layout, modal_content_area, modal_layout,
    modal_rect, modal_tab_at, one_line_list_index_at, radio_favorites_list_area,
    radio_favorites_list_layout, radio_filter_list_area, radio_filtered_results_list_area,
    radio_name_layout, radio_playlist_stations_list_area, radio_playlists_list_area,
    radio_search_results_list_area, radio_subtab_at, settings_items_area, settings_layout,
    settings_visible_rows, spotify_auth_notice_at, spotify_body_area, spotify_layout,
    spotify_no_device_notice_at, spotify_search_layout, spotify_search_list_area,
    spotify_subtab_at, spotify_titled_track_list_area, spotify_titled_track_list_layout,
    two_line_list_index_at, visible_items, visible_rows_excluding_scrollbar,
    youtube_auth_notice_at, youtube_layout, youtube_liked_list_area,
    youtube_playlist_videos_list_area, youtube_playlists_list_area, youtube_public_list_area,
    youtube_search_layout, youtube_search_list_area, youtube_subtab_at, ListItemHeight,
};
pub(in crate::ui) use layout::{MODAL_MIN_HEIGHT, MODAL_MIN_WIDTH};

mod helpers;
mod layout;
mod notice_panel;
mod settings;
mod spotify;
mod tabs;
mod youtube;

pub struct SearchModalWidget<'a> {
    pub palette: &'a Palette,
    pub query: &'a str,
    pub results: &'a [DynamicStation],
    pub loading: bool,
    pub selected: usize,
    pub mode: &'a SearchMode,
    pub genre_selected: usize,
    pub genre_filter_scroll_offset: usize,
    pub genre_filter: &'a str,
    pub genre_query: &'a str,
    pub country_selected: usize,
    pub country_filter_scroll_offset: usize,
    pub country_filter: &'a str,
    pub settings_selected: usize,
    pub settings_scroll_offset: usize,
    pub autoplay_last: bool,
    pub overlay_mode: String,
    pub overlay_style: String,
    pub overlay_position: String,
    pub crossfade: String,
    pub youtube_crossfade: String,
    pub spotify_crossfade: String,
    pub youtube_radio_mode: bool,
    pub youtube_sponsorblock: bool,
    pub media_keys: bool,
    pub tray_icon: bool,
    pub notifications: bool,
    pub theme: crate::ui::theme::ThemeId,
    pub restore_volume: bool,
    pub duck_enabled: bool,
    pub duck_volume: u8,
    pub overlay_alpha: u8,
    pub screensaver_secs: u16,
    pub screensaver_clock: bool,
    pub screensaver_logo: bool,
    pub screensaver_visualizer: bool,
    pub screensaver_recent_tracks: bool,
    pub screensaver_progress_bar: bool,
    pub screensaver_station_details: bool,
    pub screensaver_now_playing: bool,
    pub volume_step: u8,
    pub prebuffer_secs: u8,
    pub spotify_status: &'a SpotifyAuthStatus,
    pub spotify_query: &'a str,
    pub spotify_results: &'a [crate::integrations::spotify::SpotifyTrack],
    pub spotify_loading: bool,
    pub spotify_selected: usize,
    pub spotify_scroll_offset: usize,
    pub spotify_is_premium: Option<bool>,
    pub spotify_devices: &'a [crate::integrations::spotify::devices::SpotifyDevice],
    pub spotify_active_device_id: Option<&'a str>,
    pub spotify_remote_blocked: bool,
    pub spotify_devices_loading: bool,
    pub spotify_stop_on_quit: bool,
    pub spotify_start_on_spotify: bool,
    pub spotify_playback_mode: String,
    pub spotify_playback_mode_kind: crate::config::SpotifyPlaybackMode,
    pub spotify_radio_mode: bool,
    pub spotify_search_rate_limited: bool,
    pub spotify_rate_limited_secs: u64,
    pub spotify_sub_tab: crate::app::SpotifySubTab,
    pub spotify_loading_more: bool,
    pub spotify_client_id: &'a str,
    pub spotify_liked_tracks: &'a [crate::integrations::spotify::SpotifyTrack],
    pub spotify_liked_selected: usize,
    pub spotify_liked_scroll_offset: usize,
    pub spotify_liked_loading: bool,
    pub spotify_playlists: &'a [crate::integrations::spotify::playlists::SpotifyPlaylist],
    pub spotify_playlists_selected: usize,
    pub spotify_playlists_scroll_offset: usize,
    pub spotify_playlists_loading: bool,
    pub spotify_open_playlist: Option<&'a crate::integrations::spotify::playlists::SpotifyPlaylist>,
    pub spotify_playlist_tracks: &'a [crate::integrations::spotify::SpotifyTrack],
    pub spotify_playlist_tracks_selected: usize,
    pub spotify_playlist_tracks_scroll_offset: usize,
    pub spotify_playlist_tracks_loading: bool,
    pub spotify_top_tracks: &'a [crate::integrations::spotify::SpotifyTrack],
    pub spotify_top_tracks_selected: usize,
    pub spotify_top_tracks_scroll_offset: usize,
    pub spotify_top_tracks_loading: bool,
    pub spotify_recent_tracks: &'a [crate::integrations::spotify::SpotifyTrack],
    pub spotify_recent_tracks_selected: usize,
    pub spotify_recent_tracks_scroll_offset: usize,
    pub spotify_recent_tracks_loading: bool,
    pub spotify_albums: &'a [crate::integrations::spotify::SpotifyAlbum],
    pub spotify_albums_selected: usize,
    pub spotify_albums_scroll_offset: usize,
    pub spotify_albums_loading: bool,
    pub spotify_open_album: Option<&'a crate::integrations::spotify::SpotifyAlbum>,
    pub spotify_album_tracks: &'a [crate::integrations::spotify::SpotifyTrack],
    pub spotify_album_tracks_selected: usize,
    pub spotify_album_tracks_scroll_offset: usize,
    pub spotify_album_tracks_loading: bool,
    pub youtube_status: &'a crate::app::YoutubeStatus,
    pub youtube_query: &'a str,
    pub youtube_results: &'a [crate::integrations::youtube::YoutubeVideo],
    pub youtube_loading: bool,
    pub youtube_selected: usize,
    pub youtube_scroll_offset: usize,
    pub youtube_cookies_configured: bool,
    pub youtube_session_health: Option<bool>,
    pub youtube_validating: bool,
    pub youtube_sub_tab: crate::app::YoutubeSubTab,
    pub youtube_public_query: &'a str,
    pub youtube_public_results: &'a [crate::integrations::youtube::YoutubePlaylist],
    pub youtube_public_selected: usize,
    pub youtube_public_scroll_offset: usize,
    pub youtube_public_loading: bool,
    pub youtube_bookmarks: &'a [crate::integrations::youtube::YoutubeVideo],
    pub youtube_bookmarks_selected: usize,
    pub youtube_bookmarks_scroll_offset: usize,
    pub youtube_liked_videos: &'a [crate::integrations::youtube::YoutubeVideo],
    pub youtube_liked_selected: usize,
    pub youtube_liked_scroll_offset: usize,
    pub youtube_liked_loading: bool,
    pub youtube_playlists: &'a [crate::integrations::youtube::YoutubePlaylist],
    pub youtube_playlists_selected: usize,
    pub youtube_playlists_scroll_offset: usize,
    pub youtube_playlists_loading: bool,
    pub youtube_open_playlist: Option<&'a crate::integrations::youtube::YoutubePlaylist>,
    pub youtube_playlist_videos: &'a [crate::integrations::youtube::YoutubeVideo],
    pub youtube_playlist_videos_selected: usize,
    pub youtube_playlist_videos_scroll_offset: usize,
    pub youtube_playlist_videos_loading: bool,
    pub radio_sub_tab: crate::app::RadioSubTab,
    pub favorites: &'a [crate::favorites::FavoriteStation],
    pub radio_fav_selected: usize,
    pub radio_fav_scroll_offset: usize,
    pub playlists: &'a [crate::playlists::RadioPlaylist],
    pub radio_playlist_selected: usize,
    pub radio_playlist_scroll_offset: usize,
    pub radio_open_playlist: Option<usize>,
    pub radio_playlist_station_selected: usize,
    pub radio_playlist_station_scroll_offset: usize,
    pub playing_playlist_station_index: Option<usize>,
    pub radio_search_scroll_offset: usize,
    pub radio_genre_results_scroll_offset: usize,
    pub radio_country_results_scroll_offset: usize,
    pub playing_favorite_index: Option<usize>,
    pub auto_update: bool,
    pub discord_rpc: bool,
    pub save_notice: Option<String>,
    pub save_notice_severity: crate::app::NoticeSeverity,
    pub tab_dots: crate::app::TabDots,
    pub border_tick: u32,
}

impl Widget for SearchModalWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_bg(self.palette.overlay_color);
            }
        }

        let Some(layout) = modal_layout(area) else {
            return;
        };
        let panel = layout.panel;

        Clear.render(panel, buf);

        let bottom_hint = self.bottom_hint();
        let block = Block::default()
            .title_bottom(Line::from(bottom_hint).alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(
                Style::default().fg(theme::border_color_for(self.palette, self.border_tick)),
            )
            .style(Style::default().bg(self.palette.panel_bg));

        block.render(panel, buf);

        let Some(content) = modal_content_area(area) else {
            return;
        };
        let content_x = content.x;
        let content_w = content.width;

        self.render_tabs(layout.tabs, content_x, content_w, buf);

        match self.mode {
            SearchMode::Name | SearchMode::Genre | SearchMode::Country => {
                self.render_radio_body(layout.body, content_x, content_w, buf)
            }
            SearchMode::Settings => {
                self.render_settings_body(layout.body, content_x, content_w, buf)
            }
            SearchMode::Spotify => self.render_spotify_body(layout.body, content_x, content_w, buf),
            SearchMode::Youtube => self.render_youtube_body(layout.body, content_x, content_w, buf),
        }
    }
}

impl<'a> SearchModalWidget<'a> {
    pub(in crate::ui) fn from_app(app: &'a crate::app::App, palette: &'a Palette) -> Self {
        let sp = &app.spotify;
        let yt = &app.youtube;
        Self {
            palette,
            query: &app.search_query,
            results: &app.search_results,
            loading: app.search_loading,
            selected: app.modal_selected,
            mode: &app.modal_mode,
            genre_selected: app.genre_selected,
            genre_filter_scroll_offset: app.genre_filter_scroll_offset,
            genre_filter: &app.genre_filter,
            genre_query: &app.genre_query,
            country_selected: app.country_selected,
            country_filter_scroll_offset: app.country_filter_scroll_offset,
            country_filter: &app.country_filter,
            settings_selected: app.settings_selected,
            settings_scroll_offset: app.settings_scroll_offset,
            autoplay_last: app.config.autoplay_last,
            overlay_mode: app.config.overlay_mode.display(),
            overlay_style: app.config.overlay_style.display(),
            overlay_position: app.config.overlay_position.display(),
            crossfade: app.config.crossfade_display(),
            youtube_crossfade: app.config.youtube_crossfade_display(),
            spotify_crossfade: app.config.spotify_crossfade_display(),
            youtube_radio_mode: app.config.youtube_radio_mode,
            youtube_sponsorblock: app.config.youtube_sponsorblock,
            media_keys: app.config.media_keys,
            tray_icon: app.config.tray_icon,
            notifications: app.config.notifications,
            theme: app.config.theme,
            restore_volume: app.config.restore_volume,
            duck_enabled: app.config.duck_enabled,
            duck_volume: app.config.duck_volume,
            overlay_alpha: app.config.overlay_alpha,
            screensaver_secs: app.config.screensaver_secs,
            screensaver_clock: app.config.screensaver_clock,
            screensaver_logo: app.config.screensaver_logo,
            screensaver_visualizer: app.config.screensaver_visualizer,
            screensaver_recent_tracks: app.config.screensaver_recent_tracks,
            screensaver_progress_bar: app.config.screensaver_progress_bar,
            screensaver_station_details: app.config.screensaver_station_details,
            screensaver_now_playing: app.config.screensaver_now_playing,
            volume_step: app.config.volume_step,
            prebuffer_secs: app.config.prebuffer_secs,
            spotify_status: &sp.status,
            spotify_query: &sp.search_query,
            spotify_results: &sp.search_results,
            spotify_loading: sp.search_loading,
            spotify_selected: sp.search_selected,
            spotify_scroll_offset: sp.search_scroll_offset,
            spotify_is_premium: sp.is_premium,
            spotify_devices: &sp.devices,
            spotify_active_device_id: sp.active_device_id.as_deref(),
            spotify_remote_blocked: app.config.spotify.playback_mode
                == crate::config::SpotifyPlaybackMode::Remote
                && sp.active_device_id.is_none(),
            spotify_devices_loading: sp.devices_loading,
            spotify_stop_on_quit: app.config.spotify.stop_on_quit,
            spotify_start_on_spotify: app.config.spotify.start_on_spotify,
            spotify_playback_mode: app.config.spotify.playback_mode.display(),
            spotify_playback_mode_kind: app.config.spotify.playback_mode,
            spotify_radio_mode: app.config.spotify.radio_enabled,
            spotify_search_rate_limited: sp.search_rate_limited,
            spotify_rate_limited_secs: sp
                .rate_limited_until
                .map(|u| {
                    u.saturating_duration_since(std::time::Instant::now())
                        .as_secs()
                })
                .unwrap_or(0),
            spotify_sub_tab: sp.sub_tab,
            spotify_loading_more: sp.search_loading_more,
            spotify_client_id: &app.config.spotify.client_id,
            spotify_liked_tracks: &sp.liked_tracks,
            spotify_liked_selected: sp.liked_selected,
            spotify_liked_scroll_offset: sp.liked_scroll_offset,
            spotify_liked_loading: sp.liked_loading,
            spotify_playlists: &sp.playlists,
            spotify_playlists_selected: sp.playlists_selected,
            spotify_playlists_scroll_offset: sp.playlists_scroll_offset,
            spotify_playlists_loading: sp.playlists_loading,
            spotify_open_playlist: sp.open_playlist.as_ref(),
            spotify_playlist_tracks: &sp.playlist_tracks,
            spotify_playlist_tracks_selected: sp.playlist_tracks_selected,
            spotify_playlist_tracks_scroll_offset: sp.playlist_tracks_scroll_offset,
            spotify_playlist_tracks_loading: sp.playlist_tracks_loading,
            spotify_top_tracks: &sp.top_tracks,
            spotify_top_tracks_selected: sp.top_tracks_selected,
            spotify_top_tracks_scroll_offset: sp.top_tracks_scroll_offset,
            spotify_top_tracks_loading: sp.top_tracks_loading,
            spotify_recent_tracks: &sp.recent_tracks,
            spotify_recent_tracks_selected: sp.recent_tracks_selected,
            spotify_recent_tracks_scroll_offset: sp.recent_tracks_scroll_offset,
            spotify_recent_tracks_loading: sp.recent_tracks_loading,
            spotify_albums: &sp.albums,
            spotify_albums_selected: sp.albums_selected,
            spotify_albums_scroll_offset: sp.albums_scroll_offset,
            spotify_albums_loading: sp.albums_loading,
            spotify_open_album: sp.open_album.as_ref(),
            spotify_album_tracks: &sp.album_tracks,
            spotify_album_tracks_selected: sp.album_tracks_selected,
            spotify_album_tracks_scroll_offset: sp.album_tracks_scroll_offset,
            spotify_album_tracks_loading: sp.album_tracks_loading,
            youtube_status: &yt.status,
            youtube_query: &yt.query,
            youtube_results: &yt.results,
            youtube_loading: yt.loading,
            youtube_selected: yt.selected,
            youtube_scroll_offset: yt.scroll_offset,
            youtube_cookies_configured: app.config.youtube.cookies_path.is_some()
                && !yt.cookies_invalid,
            youtube_session_health: yt.session_health,
            youtube_validating: yt.validating(),
            youtube_sub_tab: yt.sub_tab,
            youtube_public_query: &yt.public_query,
            youtube_public_results: &yt.public_results,
            youtube_public_selected: yt.public_selected,
            youtube_public_scroll_offset: yt.public_scroll_offset,
            youtube_public_loading: yt.public_loading,
            youtube_bookmarks: &yt.bookmarks,
            youtube_bookmarks_selected: yt.bookmarks_selected,
            youtube_bookmarks_scroll_offset: yt.bookmarks_scroll_offset,
            youtube_liked_videos: &yt.liked_videos,
            youtube_liked_selected: yt.liked_selected,
            youtube_liked_scroll_offset: yt.liked_scroll_offset,
            youtube_liked_loading: yt.liked_loading,
            youtube_playlists: &yt.playlists,
            youtube_playlists_selected: yt.playlists_selected,
            youtube_playlists_scroll_offset: yt.playlists_scroll_offset,
            youtube_playlists_loading: yt.playlists_loading,
            youtube_open_playlist: yt.open_playlist.as_ref(),
            youtube_playlist_videos: &yt.playlist_videos,
            youtube_playlist_videos_selected: yt.playlist_videos_selected,
            youtube_playlist_videos_scroll_offset: yt.playlist_videos_scroll_offset,
            youtube_playlist_videos_loading: yt.playlist_videos_loading,
            radio_sub_tab: app.radio_sub_tab,
            favorites: &app.favorites,
            radio_fav_selected: app.radio_fav_selected,
            radio_fav_scroll_offset: app.radio_fav_scroll_offset,
            playlists: &app.playlists,
            radio_playlist_selected: app.radio_playlist_selected,
            radio_playlist_scroll_offset: app.radio_playlist_scroll_offset,
            radio_open_playlist: app.radio_open_playlist,
            radio_playlist_station_selected: app.radio_playlist_station_selected,
            radio_playlist_station_scroll_offset: app.radio_playlist_station_scroll_offset,
            playing_playlist_station_index: {
                let state = app.player.state();
                app.radio_open_playlist
                    .and_then(|idx| app.playlists.get(idx))
                    .zip(state.station.as_ref())
                    .and_then(|(playlist, playing)| {
                        playlist.stations.iter().position(|s| s.url == playing.url)
                    })
            },
            radio_search_scroll_offset: app.radio_search_scroll_offset,
            radio_genre_results_scroll_offset: app.radio_genre_results_scroll_offset,
            radio_country_results_scroll_offset: app.radio_country_results_scroll_offset,
            playing_favorite_index: {
                let state = app.player.state();
                state
                    .station
                    .as_ref()
                    .and_then(|p| app.favorites.iter().position(|f| f.url == p.url))
            },
            auto_update: app.config.auto_update,
            discord_rpc: app.config.discord_rpc,
            save_notice: app.save_notice.clone(),
            save_notice_severity: app.save_notice_severity,
            tab_dots: app.tab_dots(),
            border_tick: app.border_tick,
        }
    }
}

impl SearchModalWidget<'_> {
    fn bottom_hint(&self) -> Vec<Span<'static>> {
        if let Some(ref notice) = self.save_notice {
            let color = match self.save_notice_severity {
                crate::app::NoticeSeverity::Error => self.palette.danger,
                crate::app::NoticeSeverity::Warning => self.palette.warning,
                crate::app::NoticeSeverity::Info => match self.mode {
                    crate::app::SearchMode::Spotify => self.palette.spotify,
                    crate::app::SearchMode::Youtube => self.palette.youtube,
                    _ => self.palette.playing,
                },
            };
            return vec![
                Span::raw("  "),
                Span::styled(
                    notice.clone(),
                    ratatui::style::Style::default()
                        .fg(color)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::raw("  "),
            ];
        }
        let key = |s| key(self.palette, s);
        let sep_s = |s| sep_s(self.palette, s);
        let showing = !self.results.is_empty();
        let spotify_can_cycle_device = matches!(self.mode, SearchMode::Spotify)
            && self.spotify_playback_mode_kind != crate::config::SpotifyPlaybackMode::Native;
        if showing {
            let mut spans = vec![
                Span::raw(" "),
                key("[↵]"),
                sep_s(format!(" {}  ", t("hint.play"))),
            ];
            if matches!(self.mode, SearchMode::Genre | SearchMode::Country) {
                spans.push(key("[Space]"));
                spans.push(sep_s(format!(" {}  ", t("hint.pause"))));
            }
            if matches!(self.mode, crate::app::SearchMode::Spotify) {
                spans.push(key("[Alt+L]"));
                spans.push(sep_s(format!(" {}  ", t("hint.like"))));
                if spotify_can_cycle_device {
                    spans.push(key("[Ctrl+D]"));
                    spans.push(sep_s(format!(" {}  ", t("help.shortcut.switch_device"))));
                }
            } else {
                spans.push(key("[Alt+F]"));
                spans.push(sep_s(format!(" {}  ", t("hint.fav"))));
                if matches!(
                    self.mode,
                    SearchMode::Name | SearchMode::Genre | SearchMode::Country
                ) {
                    spans.push(key("[Alt+P]"));
                    spans.push(sep_s(format!(" {}  ", t("hint.playlist"))));
                }
            }
            spans.extend(vec![
                key("[↑↓]"),
                sep_s(format!(" {}  ", t("hint.nav"))),
                key("[?]"),
                sep_s(format!(" {} ", t("hint.help"))),
            ]);
            return spans;
        }
        match self.mode {
            SearchMode::Name => {
                use crate::app::RadioSubTab;
                if matches!(self.radio_sub_tab, RadioSubTab::Favorites) {
                    if self.favorites.is_empty() {
                        return vec![
                            Span::raw(" "),
                            key("[←→]"),
                            sep_s(format!(" {}  ", t("hint.tabs"))),
                            key("[?]"),
                            sep_s(format!(" {} ", t("hint.help"))),
                        ];
                    }
                    return vec![
                        Span::raw(" "),
                        key("[↵]"),
                        sep_s(format!(" {}  ", t("hint.play"))),
                        key("[↑↓]"),
                        sep_s(format!(" {}  ", t("hint.nav"))),
                        key("[Alt+F]"),
                        sep_s(format!(" {}  ", t("hint.fav"))),
                        key("[Alt+P]"),
                        sep_s(format!(" {}  ", t("hint.playlist"))),
                        key("[←→]"),
                        sep_s(format!(" {}  ", t("hint.tabs"))),
                        key("[?]"),
                        sep_s(format!(" {} ", t("hint.help"))),
                    ];
                }
                if matches!(self.radio_sub_tab, RadioSubTab::Playlists) {
                    if self.radio_open_playlist.is_some() {
                        return vec![
                            Span::raw(" "),
                            key("[↵]"),
                            sep_s(format!(" {}  ", t("hint.play"))),
                            key("[↑↓]"),
                            sep_s(format!(" {}  ", t("hint.nav"))),
                            key("[Alt+F]"),
                            sep_s(format!(" {}  ", t("hint.fav"))),
                            key("[Esc]"),
                            sep_s(format!(" {}  ", t("hint.back"))),
                            key("[?]"),
                            sep_s(format!(" {} ", t("hint.help"))),
                        ];
                    }
                    if self.playlists.is_empty() {
                        return vec![
                            Span::raw(" "),
                            key("[N]"),
                            sep_s(format!(" {}  ", t("hint.new"))),
                            key("[←→]"),
                            sep_s(format!(" {}  ", t("hint.tabs"))),
                            key("[?]"),
                            sep_s(format!(" {} ", t("hint.help"))),
                        ];
                    }
                    return vec![
                        Span::raw(" "),
                        key("[↵]"),
                        sep_s(format!(" {}  ", t("hint.open"))),
                        key("[N]"),
                        sep_s(format!(" {}  ", t("hint.new"))),
                        key("[R]"),
                        sep_s(format!(" {}  ", t("hint.rename"))),
                        key("[Alt+F]"),
                        sep_s(format!(" {}  ", t("hint.fav"))),
                        key("[?]"),
                        sep_s(format!(" {} ", t("hint.help"))),
                    ];
                }
                if self.query.is_empty() {
                    vec![
                        Span::raw(" "),
                        key("[Alt+G]"),
                        sep_s(format!(" {}  ", t("hint.genre"))),
                        key("[Alt+C]"),
                        sep_s(format!(" {}  ", t("hint.country"))),
                        key("[Alt+O]"),
                        sep_s(format!(" {}  ", t("hint.config"))),
                        key("[Tab]"),
                        sep_s(format!(" {}  ", t("hint.next_tab"))),
                        key("[?]"),
                        sep_s(format!(" {} ", t("hint.help"))),
                    ]
                } else {
                    vec![
                        Span::raw(" "),
                        key("[↵]"),
                        sep_s(format!(" {}  ", t("hint.play"))),
                        key("[↑↓]"),
                        sep_s(format!(" {}  ", t("hint.nav"))),
                        key("[Esc]"),
                        sep_s(format!(" {}  ", t("hint.delete"))),
                        key("[?]"),
                        sep_s(format!(" {} ", t("hint.help"))),
                    ]
                }
            }
            SearchMode::Genre | SearchMode::Country => vec![
                Span::raw(" "),
                key("[↵]"),
                sep_s(format!(" {}  ", t("hint.search"))),
                key("[↑↓]"),
                sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Esc]"),
                sep_s(format!(" {}  ", t("hint.back"))),
                key("[?]"),
                sep_s(format!(" {} ", t("hint.help"))),
            ],
            SearchMode::Settings => vec![
                Span::raw(" "),
                key("[Space]"),
                sep_s(format!(" {}  ", t("hint.change"))),
                key("[↑↓]"),
                sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Esc]"),
                sep_s(format!(" {}  ", t("hint.close"))),
                key("[?]"),
                sep_s(format!(" {} ", t("hint.help"))),
            ],
            SearchMode::Youtube => vec![
                Span::raw(" "),
                key("[↵]"),
                sep_s(format!(" {}  ", t("hint.play"))),
                key("[Ctrl+R]"),
                sep_s(format!(" {}  ", t("hint.mix"))),
                key("[Alt+F]"),
                sep_s(format!(" {}  ", t("hint.bookmark"))),
                key("[↑↓]"),
                sep_s(format!(" {}  ", t("hint.nav"))),
                key("[←→]"),
                sep_s(format!(" {}  ", t("hint.tabs"))),
                key("[?]"),
                sep_s(format!(" {} ", t("hint.help"))),
            ],
            SearchMode::Spotify => {
                use crate::app::SpotifyAuthStatus;
                match self.spotify_status {
                    SpotifyAuthStatus::Connecting => vec![
                        Span::raw(" "),
                        key("[Esc]"),
                        sep_s(format!(" {} ", t("hint.back"))),
                    ],
                    SpotifyAuthStatus::LoggedIn => {
                        if !self.spotify_results.is_empty() {
                            let mut hints = vec![
                                Span::raw(" "),
                                key("[↵]"),
                                sep_s(format!(" {}  ", t("hint.play"))),
                                key("[Space]"),
                                sep_s(format!(" {}  ", t("hint.pause"))),
                                key("[Alt+L]"),
                                sep_s(format!(" {}  ", t("hint.like"))),
                                key("[↑↓]"),
                                sep_s(format!(" {}  ", t("hint.nav"))),
                                key("[←→]"),
                                sep_s(format!(" {}  ", t("hint.tabs"))),
                            ];
                            if spotify_can_cycle_device {
                                hints.extend([
                                    key("[Ctrl+D]"),
                                    sep_s(format!(" {}  ", t("help.shortcut.switch_device"))),
                                ]);
                            }
                            hints.extend([key("[?]"), sep_s(format!(" {} ", t("hint.help")))]);
                            hints
                        } else {
                            let mut hints = vec![
                                Span::raw(" "),
                                key("[Space]"),
                                sep_s(format!(" {}  ", t("hint.pause"))),
                                key("[Alt+L]"),
                                sep_s(format!(" {}  ", t("hint.like"))),
                                key("[←→]"),
                                sep_s(format!(" {}  ", t("hint.tabs"))),
                                key("[Alt+D]"),
                                sep_s(format!(" {}  ", t("hint.disconnect"))),
                            ];
                            if spotify_can_cycle_device {
                                hints.extend([
                                    key("[Ctrl+D]"),
                                    sep_s(format!(" {}  ", t("help.shortcut.switch_device"))),
                                ]);
                            }
                            hints.extend([key("[?]"), sep_s(format!(" {} ", t("hint.help")))]);
                            hints
                        }
                    }
                    _ => vec![
                        Span::raw(" "),
                        key("[↵]"),
                        sep_s(format!(" {}  ", t("modal.spotify.connect_action"))),
                        key("[Tab]"),
                        sep_s(format!(" {}  ", t("hint.next_tab"))),
                        key("[?]"),
                        sep_s(format!(" {} ", t("hint.help"))),
                    ],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{modal_rect, MODAL_MIN_HEIGHT, MODAL_MIN_WIDTH};
    use ratatui::layout::Rect;

    #[test]
    fn modal_rect_does_not_exceed_small_terminal_area() {
        let area = Rect::new(0, 0, 40, MODAL_MIN_HEIGHT - 4);
        let modal = modal_rect(area);

        assert!(modal.right() <= area.right());
        assert!(modal.bottom() <= area.bottom());
        assert_eq!(modal.width, 40);
        assert_eq!(modal.height, MODAL_MIN_HEIGHT - 4);
    }

    #[test]
    fn modal_rect_uses_minimum_size_when_area_allows_it() {
        let area = Rect::new(0, 0, MODAL_MIN_WIDTH, MODAL_MIN_HEIGHT);
        let modal = modal_rect(area);

        assert_eq!(modal, area);
    }
}
