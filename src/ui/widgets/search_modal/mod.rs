use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Widget},
};

use crate::app::{SearchMode, SpotifyAuthStatus};
use crate::i18n::t;
use crate::station::DynamicStation;
use crate::ui::theme;

use helpers::{key, sep_s};

mod helpers;
mod settings;
mod spotify;
mod tabs;

pub(super) const BG:         Color = theme::PANEL_BG;
pub(super) const OVERLAY_BG: Color = theme::OVERLAY_COLOR;

pub struct SearchModalWidget<'a> {
    pub query:             &'a str,
    pub results:           &'a [DynamicStation],
    pub loading:           bool,
    pub selected:          usize,
    pub mode:              &'a SearchMode,
    pub genre_selected:    usize,
    pub genre_filter:      &'a str,
    pub genre_query:       &'a str,
    pub country_selected:  usize,
    pub country_filter:    &'a str,
    pub settings_selected:  usize,
    pub autoplay_last:      bool,
    pub overlay_mode:       String,
    pub overlay_position:   String,
    pub crossfade:          String,
    pub media_keys:         bool,
    pub tray_icon:          bool,
    pub notifications:      bool,
    pub restore_volume:     bool,
    pub duck_enabled:              bool,
    pub duck_volume:               u8,
    pub overlay_alpha:             u8,
    pub screensaver_secs:          u16,
    pub screensaver_clock:         bool,
    pub volume_step:               u8,
    pub prebuffer_secs:            u8,
    pub spotify_status:            &'a SpotifyAuthStatus,
    pub spotify_query:             &'a str,
    pub spotify_results:           &'a [crate::integrations::spotify::SpotifyTrack],
    pub spotify_loading:           bool,
    pub spotify_selected:          usize,
    pub spotify_is_premium:        bool,
    pub spotify_devices:          &'a [crate::integrations::spotify::devices::SpotifyDevice],
    pub spotify_devices_selected: usize,
    pub spotify_devices_loading:  bool,
    pub spotify_stop_on_quit:        bool,
    pub spotify_start_on_spotify:    bool,
    pub spotify_search_rate_limited: bool,
    pub spotify_rate_limited_secs:   u64,
    pub spotify_sub_tab:             crate::app::SpotifySubTab,
    pub spotify_loading_more:        bool,
    pub spotify_client_id:           &'a str,
    pub radio_sub_tab:               crate::app::RadioSubTab,
    pub favorites:                   &'a [crate::favorites::FavoriteStation],
    pub radio_fav_selected:          usize,
    pub playing_favorite_index:      Option<usize>,
    pub auto_update:                 bool,
    pub save_notice:                 Option<String>,
    pub border_tick:                 u32,
}

impl Widget for SearchModalWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_bg(OVERLAY_BG);
            }
        }

        let w = (area.width * 78 / 100).clamp(52, 120);
        let h = (area.height * 75 / 100).clamp(14, 30);
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y + area.height.saturating_sub(h) / 2;
        let panel = Rect::new(x, y, w, h);

        Clear.render(panel, buf);

        let bottom_hint = self.bottom_hint();
        let block = Block::default()
            .title_bottom(
                Line::from(bottom_hint).alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::border_color(self.border_tick)))
            .style(Style::default().bg(BG));

        let inner = block.inner(panel);
        block.render(panel, buf);

        let h_pad: u16 = 2;
        let content_x = inner.x + h_pad;
        let content_w = inner.width.saturating_sub(h_pad * 2);

        let [tabs_row, body_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(inner);

        self.render_tabs(tabs_row, content_x, content_w, buf);

        match self.mode {
            SearchMode::Name | SearchMode::Genre | SearchMode::Country =>
                self.render_radio_body(body_area, content_x, content_w, buf),
            SearchMode::Settings =>
                self.render_settings_body(body_area, content_x, content_w, buf),
            SearchMode::Spotify =>
                self.render_spotify_body(body_area, content_x, content_w, buf),
            SearchMode::Youtube =>
                self.render_coming_soon(body_area, buf),
        }
    }
}

impl<'a> From<&'a crate::app::App> for SearchModalWidget<'a> {
    fn from(app: &'a crate::app::App) -> Self {
        let sp = &app.spotify;
        Self {
            query:             &app.search_query,
            results:           &app.search_results,
            loading:           app.search_loading,
            selected:          app.modal_selected,
            mode:              &app.modal_mode,
            genre_selected:    app.genre_selected,
            genre_filter:      &app.genre_filter,
            genre_query:       &app.genre_query,
            country_selected:  app.country_selected,
            country_filter:    &app.country_filter,
            settings_selected:  app.settings_selected,
            autoplay_last:      app.config.autoplay_last,
            overlay_mode:       app.config.overlay_mode.display(),
            overlay_position:   app.config.overlay_position.display(),
            crossfade:          app.config.crossfade_display(),
            media_keys:         app.config.media_keys,
            tray_icon:          app.config.tray_icon,
            notifications:      app.config.notifications,
            restore_volume:     app.config.restore_volume,
            duck_enabled:              app.config.duck_enabled,
            duck_volume:               app.config.duck_volume,
            overlay_alpha:             app.config.overlay_alpha,
            screensaver_secs:          app.config.screensaver_secs,
            screensaver_clock:         app.config.screensaver_clock,
            volume_step:               app.config.volume_step,
            prebuffer_secs:            app.config.prebuffer_secs,
            spotify_status:            &sp.status,
            spotify_query:             &sp.search_query,
            spotify_results:           &sp.search_results,
            spotify_loading:           sp.search_loading,
            spotify_selected:          sp.search_selected,
            spotify_is_premium:        sp.is_premium,
            spotify_devices:          &sp.devices,
            spotify_devices_selected: sp.devices_selected,
            spotify_devices_loading:  sp.devices_loading,
            spotify_stop_on_quit:      app.config.spotify.stop_on_quit,
            spotify_start_on_spotify:  app.config.spotify.start_on_spotify,
            spotify_search_rate_limited: sp.search_rate_limited,
            spotify_rate_limited_secs:   sp.rate_limited_until
                .map(|u| u.saturating_duration_since(std::time::Instant::now()).as_secs())
                .unwrap_or(0),
            spotify_sub_tab:      sp.sub_tab,
            spotify_loading_more: sp.search_loading_more,
            spotify_client_id:    &app.config.spotify.client_id,
            radio_sub_tab:          app.radio_sub_tab,
            favorites:              &app.favorites,
            radio_fav_selected:     app.radio_fav_selected,
            playing_favorite_index: {
                let state = app.player.state();
                state.station.as_ref().and_then(|p| app.favorites.iter().position(|f| f.url == p.url))
            },
            auto_update:  app.config.auto_update,
            save_notice:  app.save_notice.clone(),
            border_tick:  app.border_tick,
        }
    }
}

impl SearchModalWidget<'_> {
    fn render_coming_soon(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::Paragraph;
        let cy = area.y + area.height / 2;
        Paragraph::new(t("modal.coming_soon"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme::MUTED))
            .render(Rect::new(area.x, cy, area.width, 1), buf);
    }

    fn bottom_hint(&self) -> Vec<Span<'static>> {
        if let Some(ref notice) = self.save_notice {
            return vec![
                Span::raw("  "),
                Span::styled(
                    notice.clone(),
                    ratatui::style::Style::default()
                        .fg(theme::PLAYING)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::raw("  "),
            ];
        }
        let showing = !self.results.is_empty();
        if showing {
            return vec![
                Span::raw(" "),
                key("[↵]"),   sep_s(format!(" {}  ", t("hint.play"))),
                key("[Alt+F]"), sep_s(format!(" {}  ", t("hint.fav"))),
                key("[↑↓]"), sep_s(format!(" {}  ", t("hint.nav"))),
                key("[?]"),   sep_s(format!(" {} ", t("hint.help"))),
            ];
        }
        match self.mode {
            SearchMode::Name => {
                use crate::app::RadioSubTab;
                if matches!(self.radio_sub_tab, RadioSubTab::Favorites) {
                    if self.favorites.is_empty() {
                        return vec![
                            Span::raw(" "),
                            key("[←→]"), sep_s(format!(" {}  ", t("hint.tabs"))),
                            key("[?]"),   sep_s(format!(" {} ", t("hint.help"))),
                        ];
                    }
                    return vec![
                        Span::raw(" "),
                        key("[↵]"),     sep_s(format!(" {}  ", t("hint.play"))),
                        key("[↑↓]"),   sep_s(format!(" {}  ", t("hint.nav"))),
                        key("[Alt+F]"), sep_s(format!(" {}  ", t("hint.fav"))),
                        key("[←→]"),   sep_s(format!(" {}  ", t("hint.tabs"))),
                        key("[?]"),     sep_s(format!(" {} ", t("hint.help"))),
                    ];
                }
                if self.query.is_empty() {
                    vec![
                        Span::raw(" "),
                        key("[Alt+G]"), sep_s(format!(" {}  ", t("hint.genre"))),
                        key("[Alt+C]"), sep_s(format!(" {}  ", t("hint.country"))),
                        key("[Alt+O]"), sep_s(format!(" {}  ", t("hint.config"))),
                        key("[Tab]"),   sep_s(format!(" {}  ", t("hint.next_tab"))),
                        key("[?]"),     sep_s(format!(" {} ", t("hint.help"))),
                    ]
                } else {
                    vec![
                        Span::raw(" "),
                        key("[↵]"),   sep_s(format!(" {}  ", t("hint.play"))),
                        key("[↑↓]"), sep_s(format!(" {}  ", t("hint.nav"))),
                        key("[Esc]"), sep_s(format!(" {}  ", t("hint.delete"))),
                        key("[?]"),   sep_s(format!(" {} ", t("hint.help"))),
                    ]
                }
            }
            SearchMode::Genre | SearchMode::Country => vec![
                Span::raw(" "),
                key("[↵]"),   sep_s(format!(" {}  ", t("hint.search"))),
                key("[↑↓]"), sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Esc]"), sep_s(format!(" {}  ", t("hint.back"))),
                key("[?]"),   sep_s(format!(" {} ", t("hint.help"))),
            ],
            SearchMode::Settings => vec![
                Span::raw(" "),
                key("[Space]"), sep_s(format!(" {}  ", t("hint.change"))),
                key("[↑↓]"),   sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Esc]"),   sep_s(format!(" {}  ", t("hint.close"))),
                key("[?]"),     sep_s(format!(" {} ", t("hint.help"))),
            ],
            SearchMode::Youtube => vec![
                Span::raw(" "),
                key("[Tab]"), sep_s(format!(" {}  ", t("hint.next_tab"))),
                key("[Esc]"), sep_s(format!(" {} ", t("hint.close"))),
            ],
            SearchMode::Spotify => {
                use crate::app::{SpotifyAuthStatus, SpotifySubTab};
                match self.spotify_status {
                    SpotifyAuthStatus::Connecting => vec![
                        Span::raw(" "),
                        key("[Esc]"), sep_s(format!(" {} ", t("hint.back"))),
                    ],
                    SpotifyAuthStatus::LoggedIn => {
                        if matches!(self.spotify_sub_tab, SpotifySubTab::Devices) {
                            vec![
                                Span::raw(" "),
                                key("[↵]"),     sep_s(format!(" {}  ", t("hint.transfer"))),
                                key("[↑↓]"),   sep_s(format!(" {}  ", t("hint.nav"))),
                                key("[←→]"),   sep_s(format!(" {}  ", t("hint.search"))),
                                key("[Alt+R]"), sep_s(format!(" {}  ", t("hint.reload"))),
                                key("[?]"),     sep_s(format!(" {} ", t("hint.help"))),
                            ]
                        } else if !self.spotify_results.is_empty() {
                            vec![
                                Span::raw(" "),
                                key("[↵]"),     sep_s(format!(" {}  ", t("hint.play"))),
                                key("[Space]"), sep_s(format!(" {}  ", t("hint.pause"))),
                                key("[↑↓]"),   sep_s(format!(" {}  ", t("hint.nav"))),
                                key("[←→]"),   sep_s(format!(" {}  ", t("hint.tabs"))),
                                key("[?]"),     sep_s(format!(" {} ", t("hint.help"))),
                            ]
                        } else {
                            vec![
                                Span::raw(" "),
                                key("[←→]"),   sep_s(format!(" {}  ", t("hint.tabs"))),
                                key("[Tab]"),   sep_s(format!(" {}  ", t("hint.radio"))),
                                key("[Alt+D]"), sep_s(format!(" {}  ", t("hint.disconnect"))),
                                key("[?]"),     sep_s(format!(" {} ", t("hint.help"))),
                            ]
                        }
                    }
                    _ => vec![
                        Span::raw(" "),
                        key("[↵]"),  sep_s(format!(" {}  ", t("modal.spotify.connect_action"))),
                        key("[Tab]"), sep_s(format!(" {}  ", t("hint.next_tab"))),
                        key("[?]"),  sep_s(format!(" {} ", t("hint.help"))),
                    ],
                }
            },
        }
    }
}
