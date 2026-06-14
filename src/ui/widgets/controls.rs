use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::ui::theme::Palette;

pub struct ControlsWidget<'a> {
    shortcuts: Vec<(String, String)>,
    vol_pct: u32,
    vol_color: ratatui::style::Color,
    bg: ratatui::style::Color,
    palette: &'a Palette,
}

impl<'a> ControlsWidget<'a> {
    pub const HEIGHT: u16 = 2;

    pub fn new(
        shortcuts: Vec<(String, String)>,
        vol_pct: u32,
        vol_color: ratatui::style::Color,
        bg: ratatui::style::Color,
        palette: &'a Palette,
    ) -> Self {
        Self {
            shortcuts,
            vol_pct,
            vol_color,
            bg,
            palette,
        }
    }
}

impl Widget for ControlsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let style_bg = Style::default().bg(self.bg);

        let mut spans: Vec<Span<'static>> = Vec::new();
        for (i, (key, action)) in self.shortcuts.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ·  ", Style::default().fg(self.palette.dim)));
            }
            spans.push(Span::styled(
                key.clone(),
                Style::default()
                    .fg(self.palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(" {action}"),
                Style::default().fg(self.palette.dim),
            ));
        }
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .style(style_bg)
            .render(Rect::new(area.x, area.y, area.width, 1), buf);

        if area.height < 2 {
            return;
        }
        let vol_w = area.width.saturating_sub(12) as usize;
        let progress_bar = crate::ui::widgets::progress::ProgressBarWidget::new(
            self.vol_pct as f32 / 100.0,
            self.vol_color,
            self.palette.dim,
            self.bg,
        );
        let mut vol_spans = vec![Span::styled(
            "VOL  ",
            Style::default().fg(self.palette.muted),
        )];
        vol_spans.extend(progress_bar.into_spans(vol_w));
        vol_spans.push(Span::styled(
            format!("  {:3}%", self.vol_pct),
            Style::default().fg(self.palette.muted),
        ));
        Paragraph::new(Line::from(vol_spans))
            .style(style_bg)
            .render(Rect::new(area.x, area.y + 1, area.width, 1), buf);
    }
}
