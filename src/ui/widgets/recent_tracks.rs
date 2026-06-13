use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::i18n::t;
use crate::ui::strings;
use crate::ui::theme::Palette;

pub struct RecentTracksWidget<'a> {
    tracks: &'a [String],
    current_badge: &'a str,
    bg: ratatui::style::Color,
    palette: &'a Palette,
}

impl<'a> RecentTracksWidget<'a> {
    pub fn new(
        tracks: &'a [String],
        current_badge: &'a str,
        bg: ratatui::style::Color,
        palette: &'a Palette,
    ) -> Self {
        Self {
            tracks,
            current_badge,
            bg,
            palette,
        }
    }
}

impl<'a> Widget for RecentTracksWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.tracks.is_empty() || area.height == 0 || area.width == 0 {
            return;
        }

        let cw = area.width;
        let mut y = area.y;

        let header = Paragraph::new(Span::styled(
            t("screensaver.recent_tracks"),
            Style::default().fg(self.palette.muted),
        ))
        .style(Style::default().bg(self.bg));
        header.render(Rect::new(area.x, y, cw, 1), buf);
        y += 1;

        if y >= area.bottom() {
            return;
        }

        let badge = self.current_badge;
        let badge_w = badge.chars().count() as u16 + 6;
        let max_cur = cw.saturating_sub(3 + badge_w) as usize;

        if let Some(current) = self.tracks.first() {
            let display = strings::truncate(current, max_cur);
            let badge_color = self.palette.accent;
            let p = Paragraph::new(Line::from(vec![
                Span::styled("▶  ", Style::default().fg(self.palette.accent)),
                Span::styled(
                    display,
                    Style::default()
                        .fg(self.palette.highlight)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ", Style::default().bg(self.bg)),
                Span::styled("\u{e0b6}", Style::default().fg(badge_color).bg(self.bg)),
                Span::styled(
                    format!(" {badge} "),
                    Style::default()
                        .fg(self.bg)
                        .bg(badge_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("\u{e0b4}", Style::default().fg(badge_color).bg(self.bg)),
            ]))
            .style(Style::default().bg(self.bg));

            p.render(Rect::new(area.x, y, cw, 1), buf);
            y += 1;
        }

        let max_prev = cw.saturating_sub(4) as usize;
        for track in self.tracks.iter().skip(1).take(4) {
            if y >= area.bottom() {
                break;
            }
            let display = strings::truncate(track, max_prev);
            let p = Paragraph::new(Line::from(vec![
                Span::styled("↳  ", Style::default().fg(self.palette.dim)),
                Span::styled(display, Style::default().fg(self.palette.highlight)),
            ]))
            .style(Style::default().bg(self.bg));

            p.render(Rect::new(area.x, y, cw, 1), buf);
            y += 1;
        }
    }
}
