use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
};

use crate::station::DynamicStation;
use crate::ui::theme;
use super::spinner_frame;

pub struct SearchModalWidget<'a> {
    pub query:    &'a str,
    pub results:  &'a [DynamicStation],
    pub loading:  bool,
    pub selected: usize,
}

impl Widget for SearchModalWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::default()
            .title_top(Line::from(Span::styled(
                " ♪ BUSCAR RADIO ",
                Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
            )).alignment(Alignment::Center))
            .title_bottom(Line::from(Span::styled(
                " [↵] Play  [↑↓] Navegar  [Esc] Cerrar ",
                Style::default().fg(theme::MUTED),
            )).alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ACCENT));

        let inner = block.inner(area);
        block.render(area, buf);

        let [input_area, sep_area, list_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(inner);

        // Línea de input
        let cursor = Span::styled("_", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD));
        let input_line = if self.query.is_empty() {
            Line::from(vec![
                Span::styled("> ", Style::default().fg(theme::MUTED)),
                cursor,
            ])
        } else {
            Line::from(vec![
                Span::styled("> ", Style::default().fg(theme::ACCENT)),
                Span::styled(self.query, Style::default().fg(theme::HIGHLIGHT)),
                cursor,
            ])
        };
        Paragraph::new(input_line).render(input_area, buf);

        // Separador
        let sep: String = "─".repeat(sep_area.width as usize);
        Paragraph::new(Span::styled(sep, Style::default().fg(theme::MUTED))).render(sep_area, buf);

        // Área de resultados
        if self.loading {
            let frame = spinner_frame();
            Paragraph::new(Span::styled(
                format!(" {frame} Buscando…"),
                Style::default().fg(theme::MUTED),
            ))
            .render(list_area, buf);
            return;
        }

        if self.results.is_empty() {
            let msg = if self.query.is_empty() {
                " Escribí el nombre de una radio…"
            } else {
                " Sin resultados"
            };
            Paragraph::new(Span::styled(msg, Style::default().fg(theme::MUTED))).render(list_area, buf);
            return;
        }

        let visible = list_area.height as usize;
        let offset = if self.selected >= visible { self.selected - visible + 1 } else { 0 };
        let name_w = list_area.width.saturating_sub(10) as usize; // reservar para bitrate

        let items: Vec<ListItem> = self
            .results
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible)
            .map(|(i, s)| {
                let active = i == self.selected;
                let prefix = if active { "▶ " } else { "  " };
                let name: String = if s.name.chars().count() > name_w {
                    s.name.chars().take(name_w.saturating_sub(1)).collect::<String>() + "…"
                } else {
                    format!("{:<width$}", s.name, width = name_w)
                };
                let bitrate = s
                    .bitrate_kbps
                    .map(|b| format!("{b:>4}k"))
                    .unwrap_or_else(|| "     ".to_string());

                let (name_style, bitrate_style) = if active {
                    (
                        Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD),
                        Style::default().fg(theme::ACCENT),
                    )
                } else {
                    (
                        Style::default().fg(theme::HIGHLIGHT),
                        Style::default().fg(theme::MUTED),
                    )
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, name_style),
                    Span::styled(name, name_style),
                    Span::styled(bitrate, bitrate_style),
                ]))
            })
            .collect();

        List::new(items).render(list_area, buf);
    }
}
