use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::i18n::t;
use crate::station::StationDetails;
use crate::ui::strings;
use crate::ui::theme::Palette;

pub struct StationDetailsWidget<'a> {
    details: &'a StationDetails,
    bg: ratatui::style::Color,
    palette: &'a Palette,
}

impl<'a> StationDetailsWidget<'a> {
    pub fn new(
        details: &'a StationDetails,
        bg: ratatui::style::Color,
        palette: &'a Palette,
    ) -> Self {
        Self {
            details,
            bg,
            palette,
        }
    }

    pub fn has_meta(d: &StationDetails) -> bool {
        !d.country.is_empty()
            || !d.state.is_empty()
            || !d.language.is_empty()
            || !d.codec.is_empty()
            || d.bitrate > 0
    }

    pub fn has_popularity(d: &StationDetails) -> bool {
        d.votes > 0 || d.clickcount > 0
    }

    pub fn content_rows(d: &StationDetails) -> u16 {
        u16::from(Self::has_meta(d))
            + u16::from(!d.tags.is_empty())
            + u16::from(Self::has_popularity(d))
            + u16::from(!d.homepage.is_empty())
    }

    pub fn url_rect(&self, area: Rect) -> Option<Rect> {
        let d = self.details;
        if d.homepage.is_empty() || area.width == 0 {
            return None;
        }
        let mut y = area.y;
        y += u16::from(Self::has_meta(d));
        y += u16::from(!d.tags.is_empty());
        y += u16::from(Self::has_popularity(d));
        if y >= area.bottom() {
            return None;
        }
        let url = strings::truncate(d.homepage.trim_end_matches('/'), area.width as usize);
        let url_w = url.chars().count() as u16;
        let x = area.x + area.width.saturating_sub(url_w) / 2;
        Some(Rect::new(x, y, url_w, 1))
    }

    fn separator(&self) -> Span<'static> {
        Span::styled("  ·  ", Style::default().fg(self.palette.dim))
    }
}

impl Widget for StationDetailsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let d = self.details;
        let style_bg = Style::default().bg(self.bg);
        let dim = Style::default().fg(self.palette.dim);
        let muted = Style::default().fg(self.palette.muted);
        let accent = Style::default().fg(self.palette.accent);
        let value = Style::default()
            .fg(self.palette.highlight)
            .add_modifier(Modifier::BOLD);
        let mut y = area.y;

        if Self::has_meta(d) && y < area.bottom() {
            let mut spans: Vec<Span<'static>> = Vec::new();
            if !d.country.is_empty() {
                spans.push(Span::styled(d.country.clone(), value));
            }
            if !d.state.is_empty() {
                if !spans.is_empty() {
                    spans.push(self.separator());
                }
                spans.push(Span::styled(strings::title_case(&d.state), muted));
            }
            if !d.language.is_empty() {
                if !spans.is_empty() {
                    spans.push(self.separator());
                }
                spans.push(Span::styled(strings::title_case(&d.language), muted));
            }
            if !d.codec.is_empty() || d.bitrate > 0 {
                if !spans.is_empty() {
                    spans.push(self.separator());
                }
                if !d.codec.is_empty() {
                    spans.push(Span::styled(
                        d.codec.clone(),
                        accent.add_modifier(Modifier::BOLD),
                    ));
                }
                if d.bitrate > 0 {
                    if !d.codec.is_empty() {
                        spans.push(Span::raw(" "));
                    }
                    spans.push(Span::styled(format!("{} kbps", d.bitrate), accent));
                }
            }
            Paragraph::new(Line::from(spans))
                .alignment(Alignment::Center)
                .style(style_bg)
                .render(Rect::new(area.x, y, area.width, 1), buf);
            y += 1;
        }

        if !d.tags.is_empty() && y < area.bottom() {
            let mut spans: Vec<Span<'static>> = Vec::new();
            for (i, tag) in d.tags.iter().enumerate() {
                if i > 0 {
                    spans.push(self.separator());
                }
                spans.push(Span::styled(tag.clone(), accent));
            }
            Paragraph::new(Line::from(spans))
                .alignment(Alignment::Center)
                .style(style_bg)
                .render(Rect::new(area.x, y, area.width, 1), buf);
            y += 1;
        }

        if Self::has_popularity(d) && y < area.bottom() {
            let mut spans: Vec<Span<'static>> = Vec::new();
            if d.votes > 0 {
                spans.push(Span::styled(group_thousands(d.votes), value));
                spans.push(Span::styled(
                    format!(" {}", t("screensaver.details.votes")),
                    dim,
                ));
            }
            if d.clickcount > 0 {
                if !spans.is_empty() {
                    spans.push(self.separator());
                }
                spans.push(Span::styled(group_thousands(d.clickcount), value));
                spans.push(Span::styled(
                    format!(" {}", t("screensaver.details.plays")),
                    dim,
                ));
            }
            Paragraph::new(Line::from(spans))
                .alignment(Alignment::Center)
                .style(style_bg)
                .render(Rect::new(area.x, y, area.width, 1), buf);
            y += 1;
        }

        if !d.homepage.is_empty() && y < area.bottom() {
            let url = strings::truncate(d.homepage.trim_end_matches('/'), area.width as usize);
            Paragraph::new(Span::styled(url, accent.add_modifier(Modifier::UNDERLINED)))
                .alignment(Alignment::Center)
                .style(style_bg)
                .render(Rect::new(area.x, y, area.width, 1), buf);
        }
    }
}

fn group_thousands(n: u32) -> String {
    let digits = n.to_string();
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let len = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(*b as char);
    }
    out
}
