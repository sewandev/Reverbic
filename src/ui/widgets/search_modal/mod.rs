use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Widget},
};

use crate::app::{IntegrationView, SearchMode, SpotifyAuthStatus};
use crate::i18n::t;
use crate::station::DynamicStation;
use crate::ui::theme;

use helpers::{key, sep, sep_s};

mod helpers;
mod integrations;
mod settings;
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
    pub integration_view:          IntegrationView,
    pub integration_selected:      usize,
    pub spotify_status:            &'a SpotifyAuthStatus,
    pub spotify_saved:             Option<&'a str>,
}

impl Widget for SearchModalWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_bg(OVERLAY_BG);
            }
        }

        let w = area.width.clamp(44, 72);
        let h = area.height.clamp(10, 14);
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
            .border_style(Style::default().fg(theme::ACCENT))
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
            SearchMode::Name         => self.render_name_body(body_area, content_x, content_w, buf),
            SearchMode::Genre        => self.render_genre_body(body_area, content_x, content_w, buf),
            SearchMode::Country      => self.render_country_body(body_area, content_x, content_w, buf),
            SearchMode::Settings     => self.render_settings_body(body_area, content_x, content_w, buf),
            SearchMode::Integrations => self.render_integrations_body(body_area, content_x, content_w, buf),
        }
    }
}

impl SearchModalWidget<'_> {
    fn bottom_hint(&self) -> Vec<Span<'static>> {
        let showing = !self.results.is_empty();
        if showing {
            return vec![
                Span::raw(" "),
                key("[↵]"),    sep_s(format!(" {}  ", t("hint.play"))),
                key("[R]"),    sep_s(format!(" {}  ", t("hint.random"))),
                key("[↑↓]"),  sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Esc]"),  sep_s(format!(" {} ",  t("hint.back"))),
            ];
        }
        match self.mode {
            SearchMode::Name => vec![
                Span::raw(" "),
                key("[↵]"),    sep(" Play  "),
                key("[↑↓]"),  sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Tab]"),  sep_s(format!(" {}  ", t("hint.next_tab"))),
                key("[Esc]"),  sep_s(format!(" {} ",  t("hint.close"))),
            ],
            SearchMode::Genre | SearchMode::Country => vec![
                Span::raw(" "),
                key("[↵]"),    sep_s(format!(" {}  ", t("hint.search"))),
                key("[↑↓]"),  sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Tab]"),  sep_s(format!(" {}  ", t("hint.next_tab"))),
                key("[Esc]"),  sep_s(format!(" {} ",  t("hint.close"))),
            ],
            SearchMode::Settings => vec![
                Span::raw(" "),
                key("[Space]"), sep_s(format!(" {}  ", t("hint.change"))),
                key("[↑↓]"),  sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Tab]"),  sep_s(format!(" {}  ", t("hint.next_tab"))),
                key("[Esc]"),  sep_s(format!(" {} ",  t("hint.close"))),
            ],
            SearchMode::Integrations => match self.integration_view {
                IntegrationView::ServiceList => vec![
                    Span::raw(" "),
                    key("[↵]"),   sep_s(format!(" {}  ", t("hint.play"))),
                    key("[↑↓]"),  sep_s(format!(" {}  ", t("hint.nav"))),
                    key("[Tab]"), sep_s(format!(" {}  ", t("hint.next_tab"))),
                    key("[Esc]"), sep_s(format!(" {} ",  t("hint.close"))),
                ],
                IntegrationView::SpotifyDetail => {
                    if matches!(self.spotify_status, SpotifyAuthStatus::LoggedIn) {
                        vec![
                            Span::raw(" "),
                            key("[D]"),   sep_s(format!(" {}  ", t("integrations.spotify.hint_disconnect"))),
                            key("[Esc]"), sep_s(format!(" {} ",  t("hint.back"))),
                        ]
                    } else {
                        let mut h = vec![
                            Span::raw(" "),
                            key("[↵]"),   sep_s(format!(" {}  ", t("hint.play"))),
                        ];
                        if self.spotify_saved.is_some() {
                            h.push(key("[D]"));
                            h.push(sep_s(format!(" {}  ", t("integrations.spotify.hint_disconnect"))));
                        }
                        h.push(key("[Esc]"));
                        h.push(sep_s(format!(" {} ", t("hint.back"))));
                        h
                    }
                }
                IntegrationView::SpotifyWebBrowser => {
                    match self.spotify_status {
                        SpotifyAuthStatus::Connecting => vec![
                            Span::raw(" "),
                            key("[Esc]"), sep_s(format!(" {} ", t("hint.back"))),
                        ],
                        SpotifyAuthStatus::Error(_) => vec![
                            Span::raw(" "),
                            key("[Esc]"), sep_s(format!(" {} ", t("hint.back"))),
                        ],
                        _ => vec![
                            Span::raw(" "),
                            key("[↵]"),   sep_s(format!(" {}  ", t("integrations.spotify.web.open_short"))),
                            key("[Esc]"), sep_s(format!(" {} ",  t("hint.back"))),
                        ],
                    }
                }
            },
        }
    }
}
