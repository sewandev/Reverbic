use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::i18n::t;
use crate::ui::strings;
use crate::ui::theme::Palette;

pub struct SpotifyProfileWidget<'a> {
    name: Option<&'a str>,
    country: Option<&'a str>,
    followers: Option<u32>,
    is_premium: bool,
    bg: ratatui::style::Color,
    palette: &'a Palette,
}

impl<'a> SpotifyProfileWidget<'a> {
    pub fn new(
        name: Option<&'a str>,
        country: Option<&'a str>,
        followers: Option<u32>,
        is_premium: bool,
        bg: ratatui::style::Color,
        palette: &'a Palette,
    ) -> Self {
        Self {
            name,
            country,
            followers,
            is_premium,
            bg,
            palette,
        }
    }

    fn has_name(&self) -> bool {
        self.name.is_some_and(|n| !n.trim().is_empty())
    }

    fn has_meta(&self) -> bool {
        self.is_premium
            || self.country.is_some_and(|c| !c.trim().is_empty())
            || self.followers.is_some()
    }

    pub fn has_content(&self) -> bool {
        self.has_name() || self.has_meta()
    }

    pub fn content_rows(&self) -> u16 {
        u16::from(self.has_name()) + u16::from(self.has_meta())
    }

    fn separator(&self) -> Span<'static> {
        Span::styled("  ·  ", Style::default().fg(self.palette.dim))
    }
}

impl Widget for SpotifyProfileWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let style_bg = Style::default().bg(self.bg);
        let mut y = area.y;

        if self.has_name() && y < area.bottom() {
            let name = strings::title_case(self.name.unwrap_or_default().trim());
            Paragraph::new(Span::styled(
                name,
                Style::default()
                    .fg(self.palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center)
            .style(style_bg)
            .render(Rect::new(area.x, y, area.width, 1), buf);
            y += 1;
        }

        if self.has_meta() && y < area.bottom() {
            let mut spans: Vec<Span<'static>> = Vec::new();
            if self.is_premium {
                spans.push(Span::styled(
                    t("integrations.spotify.premium"),
                    Style::default()
                        .fg(self.palette.playing)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            if let Some(country) = self.country.map(str::trim).filter(|c| !c.is_empty()) {
                if !spans.is_empty() {
                    spans.push(self.separator());
                }
                spans.push(Span::styled(
                    country.to_uppercase(),
                    Style::default().fg(self.palette.muted),
                ));
            }
            if let Some(followers) = self.followers {
                if !spans.is_empty() {
                    spans.push(self.separator());
                }
                spans.push(Span::styled(
                    strings::group_thousands(followers),
                    Style::default()
                        .fg(self.palette.highlight)
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(
                    format!(" {}", t("screensaver.followers")),
                    Style::default().fg(self.palette.dim),
                ));
            }
            Paragraph::new(Line::from(spans))
                .alignment(Alignment::Center)
                .style(style_bg)
                .render(Rect::new(area.x, y, area.width, 1), buf);
        }
    }
}
