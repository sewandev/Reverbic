use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::ui::theme::Palette;

use super::auth_notice_box;
use super::helpers::spin_frame;

pub(super) struct NoticeHint {
    pub key: String,
    pub text: String,
    pub strong: bool,
}

pub(super) struct NoticePanel {
    pub title: String,
    pub emphasis: Option<(String, Color)>,
    pub body: String,
    pub caution: Option<String>,
    pub link: Option<(String, String)>,
    pub spinner_text: Option<String>,
    pub hints: Vec<NoticeHint>,
}

impl NoticePanel {
    pub(super) fn render(&self, area: Rect, buf: &mut Buffer, palette: &Palette) {
        let Some(box_area) = auth_notice_box(area) else {
            return;
        };

        Clear.render(box_area, buf);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(palette.warning))
            .style(Style::default().bg(palette.panel_bg))
            .render(box_area, buf);

        let inner_x = box_area.x + 2;
        let inner_w = box_area.width.saturating_sub(4);
        let bottom = box_area.bottom().saturating_sub(1);
        let mut y = box_area.y + 1;

        Paragraph::new(Span::styled(
            self.title.clone(),
            Style::default()
                .fg(palette.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .render(Rect::new(inner_x, y, inner_w, 1), buf);
        y += 2;
        if y >= bottom {
            return;
        }

        if let Some((text, color)) = &self.emphasis {
            Paragraph::new(Span::styled(
                text.clone(),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ))
            .wrap(Wrap { trim: true })
            .render(Rect::new(inner_x, y, inner_w, 2.min(bottom - y)), buf);
            y += 3;
            if y >= bottom {
                return;
            }
        }

        Paragraph::new(Span::styled(
            self.body.clone(),
            Style::default().fg(palette.highlight),
        ))
        .wrap(Wrap { trim: true })
        .render(Rect::new(inner_x, y, inner_w, 3.min(bottom - y)), buf);
        y += 4;
        if y >= bottom {
            return;
        }

        if let Some(caution) = &self.caution {
            Paragraph::new(Span::styled(
                caution.clone(),
                Style::default().fg(palette.caution),
            ))
            .wrap(Wrap { trim: true })
            .render(Rect::new(inner_x, y, inner_w, 2.min(bottom - y)), buf);
            y += 3;
            if y >= bottom {
                return;
            }
        }

        if let Some((label, url)) = &self.link {
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{label} "), Style::default().fg(palette.dim)),
                Span::styled(
                    url.clone(),
                    Style::default()
                        .fg(palette.accent)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ),
            ]))
            .wrap(Wrap { trim: true })
            .render(Rect::new(inner_x, y, inner_w, 2.min(bottom - y)), buf);
            y += 2;
            if y >= bottom {
                return;
            }
        }

        let mut spans: Vec<Span<'static>> = Vec::new();
        if let Some(text) = &self.spinner_text {
            spans.push(Span::styled(
                spin_frame(),
                Style::default().fg(palette.accent),
            ));
            spans.push(Span::styled(
                format!("  {text}"),
                Style::default().fg(palette.muted),
            ));
        }
        for hint in &self.hints {
            if !spans.is_empty() {
                spans.push(Span::raw("   "));
            }
            spans.push(Span::styled(
                format!("{}  ", hint.key),
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ));
            let text_style = if hint.strong {
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.muted)
            };
            spans.push(Span::styled(hint.text.clone(), text_style));
        }
        if !spans.is_empty() {
            Paragraph::new(Line::from(spans))
                .alignment(Alignment::Center)
                .render(Rect::new(inner_x, y, inner_w, 1), buf);
        }
    }
}
