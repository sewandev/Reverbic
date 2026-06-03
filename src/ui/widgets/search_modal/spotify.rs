use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget, Wrap},
};

use crate::app::SpotifyAuthStatus;
use crate::i18n::t;
use crate::ui::strings;
use crate::ui::theme;

use super::helpers::{render_filter_input, spin_frame};
use super::{BG, SearchModalWidget};

fn fmt_duration(ms: u32) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}

impl<'a> SearchModalWidget<'a> {
    pub(super) fn render_spotify_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let lx = content_x + 2;
        let lw = content_w.saturating_sub(2);
        match self.spotify_status {
            SpotifyAuthStatus::LoggedIn   => self.render_spotify_logged_in(area, content_x, content_w, buf),
            SpotifyAuthStatus::Connecting => self.render_spotify_connecting(area, lx, lw, buf),
            SpotifyAuthStatus::Error(msg) => self.render_spotify_error(area, lx, lw, buf, msg.as_str()),
            SpotifyAuthStatus::Idle       => self.render_spotify_connect(area, lx, lw, buf),
        }
    }

    fn render_spotify_logged_in(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        use crate::app::SpotifySubTab;

        let [_gap, subtab_row, body] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);

        {
            let (search_st, devices_st) = match self.spotify_sub_tab {
                SpotifySubTab::Search  => (
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                    Style::default().fg(theme::DIM),
                ),
                SpotifySubTab::Devices => (
                    Style::default().fg(theme::DIM),
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                ),
            };
            Paragraph::new(Line::from(vec![
                Span::styled(t("modal.spotify.subtab.search"),  search_st),
                Span::styled("  ", Style::default()),
                Span::styled(t("modal.spotify.subtab.devices"), devices_st),
            ]))
            .render(Rect::new(text_x, subtab_row.y, text_w, 1), buf);
        }

        match self.spotify_sub_tab {
            SpotifySubTab::Search => {
                let [input_row, cap_row, list_area] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .areas(body);

                buf[(content_x, input_row.y)]
                    .set_symbol("┃").set_fg(theme::ACCENT).set_bg(BG);
                let text_area = Rect::new(text_x, input_row.y, text_w, 1);
                render_filter_input(self.spotify_query, &t("modal.spotify.placeholder"), text_area, buf);

                if self.spotify_is_premium {
                    let badge = format!("★ {}", t("integrations.spotify.premium"));
                    let badge_len = badge.chars().count() as u16;
                    if badge_len + 4 < text_w {
                        let bx = text_x + text_w.saturating_sub(badge_len);
                        Paragraph::new(Span::styled(
                            badge,
                            Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                        ))
                        .render(Rect::new(bx, input_row.y, badge_len, 1), buf);
                    }
                }

                buf[(content_x, cap_row.y)]
                    .set_symbol("╹").set_fg(theme::ACCENT).set_bg(BG);

                self.render_spotify_list(list_area, text_x, text_w, buf);
            }
            SpotifySubTab::Devices => {
                self.render_spotify_devices(body, text_x, text_w, buf);
            }
        }
    }

    fn render_spotify_connect(&self, area: Rect, lx: u16, lw: u16, buf: &mut Buffer) {
        let mut y = area.y + 1;
        if y >= area.bottom() { return; }
        Paragraph::new(Span::styled(
            "SPOTIFY",
            Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD),
        ))
        .alignment(ratatui::layout::Alignment::Center)
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 2;
        if y >= area.bottom() { return; }
        Paragraph::new(Span::styled(
            t("modal.spotify.remote_feature"),
            Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD),
        ))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 1;

        if y >= area.bottom() { return; }
        Paragraph::new(Span::styled(t("modal.spotify.remote_subtitle"), Style::default().fg(theme::MUTED)))
            .wrap(Wrap { trim: true })
            .render(Rect::new(lx, y, lw, 2.min(area.bottom().saturating_sub(y))), buf);
        y += 3;
        if y >= area.bottom() { return; }
        Paragraph::new(Span::styled(
            "─".repeat(lw as usize),
            Style::default().fg(theme::DIM),
        ))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 1;
        if y >= area.bottom() { return; }
        Paragraph::new(Line::from(vec![
            Span::styled("[↵]  ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(t("modal.spotify.connect_action"), Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD)),
        ]))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 2;
        if y >= area.bottom() { return; }
        Paragraph::new(Span::styled(t("modal.spotify.experimental"), Style::default().fg(theme::CAUTION)))
            .wrap(Wrap { trim: true })
            .render(Rect::new(lx, y, lw, area.bottom().saturating_sub(y)), buf);
    }

    fn render_spotify_connecting(&self, area: Rect, lx: u16, lw: u16, buf: &mut Buffer) {
        let mut y = area.y + 2;
        if y >= area.bottom() { return; }
        Paragraph::new(Line::from(vec![
            Span::styled(spin_frame(), Style::default().fg(theme::ACCENT)),
            Span::styled(format!("  {}", t("integrations.spotify.web.waiting")), Style::default().fg(theme::MUTED)),
        ]))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 1;
        if y < area.bottom() {
            Paragraph::new(Span::styled(t("integrations.spotify.web.waiting2"), Style::default().fg(theme::DIM)))
                .render(Rect::new(lx, y, lw, 1), buf);
        }
    }

    fn render_spotify_error(&self, area: Rect, lx: u16, lw: u16, buf: &mut Buffer, msg: &str) {
        let mut y = area.y + 2;
        if y >= area.bottom() { return; }

        Paragraph::new(Span::styled(
            strings::truncate(msg, lw as usize),
            Style::default().fg(theme::WARNING),
        ))
        .render(Rect::new(lx, y, lw, 1), buf);

        y += 1;
        if y < area.bottom() {
            Paragraph::new(Span::styled(
                t("integrations.spotify.error.hint"),
                Style::default().fg(theme::DIM),
            ))
            .render(Rect::new(lx, y, lw, 1), buf);
        }

        y += 2;
        if y < area.bottom() {
            Paragraph::new(Line::from(vec![
                Span::styled("[↵]  ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled(t("modal.spotify.connect_action"), Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD)),
            ]))
            .render(Rect::new(lx, y, lw, 1), buf);
        }
    }

    fn render_spotify_list(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        if self.spotify_loading {
            Paragraph::new(Span::styled(
                format!("{}  {}", spin_frame(), t("modal.loading.spotify")),
                Style::default().fg(theme::MUTED),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        if self.spotify_search_rate_limited {
            let mut y = area.y;
            let countdown = if self.spotify_rate_limited_secs > 0 {
                format!("{}  {}:{:02}",
                    t("modal.spotify.rate_limit_countdown"),
                    self.spotify_rate_limited_secs / 60,
                    self.spotify_rate_limited_secs % 60)
            } else {
                t("modal.spotify.rate_limit_countdown")
            };
            Paragraph::new(Span::styled(
                countdown,
                Style::default().fg(theme::WARNING).add_modifier(Modifier::BOLD),
            ))
            .render(Rect::new(list_x, y, list_w, 1), buf);
            y += 1;
            if y < area.bottom() {
                use ratatui::widgets::Wrap;
                Paragraph::new(Span::styled(
                    t("modal.spotify.rate_limited"),
                    Style::default().fg(theme::WARNING),
                ))
                .wrap(Wrap { trim: true })
                .render(Rect::new(list_x, y, list_w, area.bottom().saturating_sub(y)), buf);
            }
            return;
        }

        if self.spotify_results.is_empty() {
            if !self.spotify_query.is_empty() {
                Paragraph::new(Span::styled(
                    t("modal.empty.no_results"),
                    Style::default().fg(theme::MUTED),
                ))
                .render(Rect::new(list_x, area.y, list_w, 1), buf);
            }
            return;
        }

        const ITEM_HEIGHT: usize = 2;
        let visible_n = (area.height as usize) / ITEM_HEIGHT;
        let offset    = super::super::scroll_offset(self.spotify_selected, visible_n);

        let items: Vec<ListItem> = self
            .spotify_results
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, track)| {
                let active = i == self.spotify_selected;
                if active {
                    let name_max = list_w.saturating_sub(4) as usize;
                    let dur      = fmt_duration(track.duration_ms);
                    let meta_max = list_w.saturating_sub(3 + dur.len() as u16) as usize;
                    let name    = strings::truncate(&track.name, name_max);
                    let meta_raw = format!("{} · {}", track.artist, track.album);
                    let meta    = strings::truncate(&meta_raw, meta_max);
                    let name_st = Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD);
                    let meta_st = Style::default().fg(theme::ACCENT);
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled("▶  ", name_st),
                            Span::styled(name, name_st),
                            Span::styled(format!("  {}", dur), meta_st),
                        ]),
                        Line::from(vec![
                            Span::styled("   ", meta_st),
                            Span::styled(meta, meta_st),
                        ]),
                    ])
                } else {
                    let name_max   = list_w.saturating_sub(3) as usize;
                    let name       = strings::truncate(&track.name, name_max);
                    let artist_max = list_w.saturating_sub(3) as usize;
                    let artist     = strings::truncate(&track.artist, artist_max);
                    let name_st   = Style::default().fg(theme::HIGHLIGHT);
                    let artist_st = Style::default().fg(theme::MUTED);
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled("   ", name_st),
                            Span::styled(name, name_st),
                        ]),
                        Line::from(vec![
                            Span::styled("   ", artist_st),
                            Span::styled(artist, artist_st),
                        ]),
                    ])
                }
            })
            .collect();

        let list_area = Rect::new(list_x, area.y, list_w, area.height);
        List::new(items).render(list_area, buf);

        let needs_scroll = self.spotify_results.len() > visible_n;
        if needs_scroll {
            self.render_scrollbar(list_area, self.spotify_results.len(), self.spotify_selected, buf);
        }

        if self.spotify_loading_more {
            let indicator_y = list_area.bottom().saturating_sub(1);
            if indicator_y > list_area.y {
                Paragraph::new(Span::styled(
                    format!("{}  {}", super::helpers::spin_frame(), t("modal.spotify.load_more")),
                    Style::default().fg(theme::DIM),
                ))
                .render(Rect::new(list_area.x, indicator_y, list_area.width, 1), buf);
            }
        }
    }

    pub(super) fn render_spotify_devices(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        let mut y = area.y;
        if y >= area.bottom() { return; }

        Paragraph::new(Line::from(vec![
            Span::styled(t("modal.spotify.devices_header"), Style::default().fg(theme::MUTED)),
        ]))
        .render(Rect::new(list_x, y, list_w, 1), buf);
        y += 1;

        if self.spotify_devices_loading {
            if y < area.bottom() {
                Paragraph::new(Span::styled(
                    format!("{}  {}", super::helpers::spin_frame(), t("modal.spotify.devices_loading")),
                    Style::default().fg(theme::MUTED),
                ))
                .render(Rect::new(list_x, y, list_w, 1), buf);
            }
            return;
        }

        if self.spotify_devices.is_empty() {
            if y < area.bottom() {
                Paragraph::new(Span::styled(t("modal.spotify.no_devices"), Style::default().fg(theme::DIM)))
                    .render(Rect::new(list_x, y, list_w, 1), buf);
            }
            return;
        }

        let visible_n = area.bottom().saturating_sub(y) as usize;
        let items: Vec<ratatui::widgets::ListItem> = self.spotify_devices
            .iter()
            .enumerate()
            .take(visible_n)
            .map(|(i, dev)| {
                let selected = i == self.spotify_devices_selected;
                let playing  = dev.is_active;

                let type_label: String = match dev.device_type.to_lowercase().as_str() {
                    "computer"   => "PC".to_owned(),
                    "smartphone" => t("spotify.device.smartphone"),
                    "speaker"    => t("spotify.device.speaker"),
                    "tv"         => "TV".to_owned(),
                    "tablet"     => "Tablet".to_owned(),
                    _            => t("spotify.device.other"),
                };

                let suffix = if playing {
                    format!("  ·  {}  [{}]", type_label, t("modal.spotify.device_active"))
                } else {
                    format!("  ·  {}", type_label)
                };
                let name_max = list_w.saturating_sub(3 + suffix.chars().count() as u16) as usize;
                let name     = strings::truncate(&dev.name, name_max);

                let (prefix, name_st, meta_st) = if selected {
                    (
                        "▶  ",
                        Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD),
                        Style::default().fg(theme::ACCENT),
                    )
                } else if playing {
                    (
                        "▶  ",
                        Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD),
                        Style::default().fg(theme::ACCENT),
                    )
                } else {
                    (
                        "   ",
                        Style::default().fg(theme::HIGHLIGHT),
                        Style::default().fg(theme::MUTED),
                    )
                };

                ratatui::widgets::ListItem::new(Line::from(vec![
                    Span::styled(prefix, name_st),
                    Span::styled(name,   name_st),
                    Span::styled(suffix, meta_st),
                ]))
            })
            .collect();

        ratatui::widgets::List::new(items).render(
            Rect::new(list_x, y, list_w, area.bottom().saturating_sub(y)),
            buf,
        );
    }

}
