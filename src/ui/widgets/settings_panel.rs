use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::ui::theme;

pub enum SettingsValue {
    Toggle(bool),
    Choice(&'static str),
}

pub struct SettingsItem {
    pub label: &'static str,
    pub value: SettingsValue,
}

pub struct SettingsPanelWidget<'a> {
    pub items:    &'a [SettingsItem],
    pub selected: usize,
}

impl<'a> Widget for SettingsPanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let w = 58u16.min(area.width.saturating_sub(4));
        let h = (self.items.len() as u16 + 4).max(6).min(area.height.saturating_sub(2));
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y + area.height.saturating_sub(h) / 2;
        let modal = Rect::new(x, y, w, h);

        Clear.render(modal, buf);

        let block = Block::default()
            .title(" CONFIGURACION  [↑↓] Nav  [Space] Cambiar  [o] Cerrar ")
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme::ACCENT));

        let inner = block.inner(modal);
        block.render(modal, buf);

        let lines: Vec<Line> = std::iter::once(Line::from(""))
            .chain(self.items.iter().enumerate().map(|(i, item)| {
                let is_sel = i == self.selected;
                let base   = if is_sel {
                    Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(theme::MUTED)
                };
                let prefix = if is_sel { "▶ " } else { "  " };

                let value_span = match &item.value {
                    SettingsValue::Toggle(on) => {
                        let (txt, fg) = if *on { (" ON ", theme::PLAYING) } else { ("OFF", theme::MUTED) };
                        vec![
                            Span::styled("  [", Style::new().fg(theme::MUTED)),
                            Span::styled(txt, Style::new().fg(fg).add_modifier(Modifier::BOLD)),
                            Span::styled("]", Style::new().fg(theme::MUTED)),
                        ]
                    }
                    SettingsValue::Choice(val) => vec![
                        Span::styled("  [", Style::new().fg(theme::MUTED)),
                        Span::styled(*val, Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
                        Span::styled("]", Style::new().fg(theme::MUTED)),
                    ],
                };

                let mut spans = vec![
                    Span::styled(prefix, base),
                    Span::styled(item.label, base),
                ];
                spans.extend(value_span);
                Line::from(spans)
            }))
            .collect();

        Paragraph::new(lines).render(inner, buf);
    }
}
