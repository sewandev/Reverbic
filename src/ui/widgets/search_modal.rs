use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Widget},
};

use crate::station::DynamicStation;
use crate::ui::theme;

const BG: Color = Color::Rgb(13, 13, 13);
const OVERLAY_BG: Color = Color::Rgb(5, 5, 5);

const EXAMPLES: &[&str] = &[
    "The Jazz Radio",
    "BBC World Service",
    "Classic FM",
    "Radio Nacional",
    "Indie Rock",
    "Tomorrowland Radio",
    "Salsa y Bachata",
    "Lofi Hip Hop",
    "NPR News",
    "Deep House",
];

fn placeholder_example() -> &'static str {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    EXAMPLES[(secs / 3) as usize % EXAMPLES.len()]
}

pub struct SearchModalWidget<'a> {
    pub query:    &'a str,
    pub results:  &'a [DynamicStation],
    pub loading:  bool,
    pub selected: usize,
}

impl Widget for SearchModalWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Oscurecer fondo
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_bg(OVERLAY_BG);
            }
        }

        // Panel rectangular: mas ancho que alto
        let w = area.width.min(66).max(44);
        let h = area.height.min(14).max(10);
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y + area.height.saturating_sub(h) / 2;
        let panel = Rect::new(x, y, w, h);

        Clear.render(panel, buf);

        let block = Block::default()
            .title_top(
                Line::from(vec![
                    Span::styled(
                        " BUSCAR RADIO ",
                        Style::default()
                            .fg(theme::HIGHLIGHT)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
                .alignment(Alignment::Center),
            )
            .title_bottom(
                Line::from(vec![
                    Span::raw(" "),
                    key("[↵]"), sep(" Play  "),
                    key("[↑↓]"), sep(" Navegar  "),
                    key("[Esc]"), sep(" Cerrar "),
                ])
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::ACCENT))
            .style(Style::default().bg(BG));

        let inner = block.inner(panel);
        block.render(panel, buf);

        let h_pad: u16    = 2;
        let content_x     = inner.x + h_pad;
        let content_w     = inner.width.saturating_sub(h_pad * 2);

        // Layout interno compacto: input + cap + resultados
        let [input_row, cap_row, list_area] = Layout::vertical([
            Constraint::Length(1),  // ┃ input
            Constraint::Length(1),  // ╹
            Constraint::Fill(1),    // resultados
        ])
        .areas(inner);

        // ── Input ─────────────────────────────────────────────────────
        buf[(content_x, input_row.y)]
            .set_symbol("┃")
            .set_fg(theme::ACCENT)
            .set_bg(BG);

        let text_x    = content_x + 2;
        let text_w    = content_w.saturating_sub(2);
        let text_area = Rect::new(text_x, input_row.y, text_w, 1);

        if self.query.is_empty() {
            Paragraph::new(Span::styled(
                format!("Buscar radio… \"{}\"", placeholder_example()),
                Style::default().fg(theme::MUTED),
            ))
            .render(text_area, buf);
        } else {
            let max_q   = text_w.saturating_sub(2) as usize;
            let visible: String = if self.query.chars().count() > max_q {
                self.query.chars().rev().take(max_q).collect::<String>().chars().rev().collect()
            } else {
                self.query.to_owned()
            };
            Paragraph::new(Line::from(vec![
                Span::styled(visible, Style::default().fg(theme::HIGHLIGHT)),
                Span::styled("_", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            ]))
            .render(text_area, buf);
        }

        // ── Cap ──────────────────────────────────────────────────────
        buf[(content_x, cap_row.y)]
            .set_symbol("╹")
            .set_fg(theme::ACCENT)
            .set_bg(BG);

        // ── Resultados + scrollbar ────────────────────────────────────
        let list_x      = content_x + 2;
        let visible_n   = list_area.height as usize;
        // reservar 1 col para scrollbar cuando haya resultados
        let needs_scroll = self.results.len() > visible_n;
        let name_col_w  = content_w.saturating_sub(if needs_scroll { 9 } else { 8 }) as usize;
        let items_x     = list_x;
        let items_w     = content_w.saturating_sub(if needs_scroll { 3 } else { 2 });
        let items_area  = Rect::new(items_x, list_area.y, items_w, list_area.height);

        if self.loading {
            let ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            const SPIN: &[&str] = &["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
            let frame = SPIN[(ms / 80) as usize % SPIN.len()];
            Paragraph::new(Span::styled(
                format!("{frame}  Buscando…"),
                Style::default().fg(theme::MUTED),
            ))
            .render(items_area, buf);
            return;
        }

        if self.results.is_empty() {
            let msg = if self.query.is_empty() {
                "Escribi para buscar radios de todo el mundo"
            } else {
                "Sin resultados"
            };
            Paragraph::new(Span::styled(msg, Style::default().fg(theme::MUTED)))
                .render(items_area, buf);
            return;
        }

        let offset = if self.selected >= visible_n { self.selected - visible_n + 1 } else { 0 };

        let items: Vec<ListItem> = self
            .results
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, s)| {
                let active  = i == self.selected;
                let prefix  = if active { "▶  " } else { "   " };
                let name: String = if s.name.chars().count() > name_col_w {
                    s.name.chars().take(name_col_w.saturating_sub(1)).collect::<String>() + "…"
                } else {
                    format!("{:<width$}", s.name, width = name_col_w)
                };
                let bitrate = s
                    .bitrate_kbps
                    .map(|b| format!("{b:>4}k"))
                    .unwrap_or_else(|| "    ".to_string());

                let (name_st, br_st) = if active {
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
                    Span::styled(prefix,  name_st),
                    Span::styled(name,    name_st),
                    Span::styled(bitrate, br_st),
                ]))
            })
            .collect();

        List::new(items).render(items_area, buf);

        // ── Scrollbar vertical ────────────────────────────────────────
        if needs_scroll {
            let sb_x    = inner.x + inner.width.saturating_sub(2); // última col antes del borde
            let total   = self.results.len();
            let track_h = visible_n;

            // posición del thumb
            let thumb_pos = if total <= 1 {
                0
            } else {
                (self.selected * (track_h.saturating_sub(1))) / (total - 1)
            };

            for row in 0..track_h {
                let sy = list_area.y + row as u16;
                let (sym, fg) = if row == 0 && offset > 0 {
                    ("▲", theme::DIM)
                } else if row == track_h - 1 && offset + visible_n < total {
                    ("▼", theme::DIM)
                } else if row == thumb_pos {
                    ("┃", theme::ACCENT)
                } else {
                    ("│", theme::MUTED)
                };
                buf[(sb_x, sy)].set_symbol(sym).set_fg(fg).set_bg(BG);
            }
        }
    }
}

fn key(s: &'static str) -> Span<'static> {
    Span::styled(
        s,
        Style::default()
            .fg(theme::HIGHLIGHT)
            .add_modifier(Modifier::BOLD),
    )
}

fn sep(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::MUTED))
}
