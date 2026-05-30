use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph, Widget},
};

use crate::station::DynamicStation;
use crate::ui::theme;

const BG: Color = Color::Rgb(15, 15, 15);

const EXAMPLES: &[&str] = &[
    "The Jazz Radio",
    "BBC World Service",
    "Classic FM",
    "Radio Nacional",
    "Indie Rock",
    "Tomorrowland Radio",
    "Salsa y Bachata",
    "Lofi Hip Hop",
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
        // Fondo oscuro propio
        Clear.render(area, buf);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_bg(BG);
            }
        }

        // Padding horizontal: igual que opencode (paddingLeft/Right = 2)
        let h_pad: u16 = 2;
        let content_w = area.width.saturating_sub(h_pad * 2);
        let content_x = area.x + h_pad;

        // Layout vertical: centering + logo + gap + input + cap + results + hint
        let [top_pad, logo_row, _gap1, input_row, cap_row, results_area, hint_row] =
            Layout::vertical([
                Constraint::Fill(1),      // centering superior
                Constraint::Length(1),    // logo ♫ REVERBIC
                Constraint::Length(2),    // gap
                Constraint::Length(1),    // input (┃ …)
                Constraint::Length(1),    // ╹  (pie del borde)
                Constraint::Fill(2),      // resultados
                Constraint::Length(1),    // hint
            ])
            .areas(area);
        let _ = top_pad; // usada implícitamente por el layout

        // ── Logo ─────────────────────────────────────────────────────
        let logo_area = Rect::new(content_x, logo_row.y, content_w, 1);
        Paragraph::new(Span::styled(
            "♫  REVERBIC",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .render(logo_area, buf);

        // ── Input ────────────────────────────────────────────────────
        // Borde ┃ en col content_x, contenido desde content_x + 2
        let input_area = Rect::new(content_x, input_row.y, content_w, 1);
        buf[(content_x, input_row.y)]
            .set_symbol("┃")
            .set_fg(theme::ACCENT)
            .set_bg(BG);

        let text_x   = content_x + 2;
        let text_w   = content_w.saturating_sub(2);
        let text_area = Rect::new(text_x, input_row.y, text_w, 1);

        if self.query.is_empty() {
            let example = placeholder_example();
            let placeholder = format!("Buscar radio… \"{example}\"");
            Paragraph::new(Span::styled(
                placeholder,
                Style::default().fg(theme::MUTED),
            ))
            .render(text_area, buf);
        } else {
            let cursor = Span::styled(
                "_",
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            );
            // Truncar query si es más larga que el área
            let max_q = text_w.saturating_sub(2) as usize;
            let visible_q: String = if self.query.chars().count() > max_q {
                self.query.chars().rev().take(max_q).collect::<String>().chars().rev().collect()
            } else {
                self.query.to_owned()
            };
            Paragraph::new(Line::from(vec![
                Span::styled(visible_q, Style::default().fg(theme::HIGHLIGHT)),
                cursor,
            ]))
            .render(text_area, buf);
        }
        let _ = input_area; // ya dibujado manualmente

        // ── Pie del borde (╹) ────────────────────────────────────────
        buf[(content_x, cap_row.y)]
            .set_symbol("╹")
            .set_fg(theme::ACCENT)
            .set_bg(BG);

        // ── Resultados ───────────────────────────────────────────────
        let list_x = content_x + 2;
        let list_w = content_w.saturating_sub(2);
        let list_area = Rect::new(list_x, results_area.y, list_w, results_area.height);

        if self.loading {
            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            const SPIN: &[&str] = &["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
            let frame = SPIN[(t / 80) as usize % SPIN.len()];
            Paragraph::new(Span::styled(
                format!("{frame}  Buscando…"),
                Style::default().fg(theme::MUTED),
            ))
            .render(list_area, buf);
        } else if self.results.is_empty() {
            let msg = if self.query.is_empty() {
                "Escribí para buscar radios de todo el mundo"
            } else {
                "Sin resultados"
            };
            Paragraph::new(Span::styled(msg, Style::default().fg(theme::MUTED)))
                .render(list_area, buf);
        } else {
            let visible = list_area.height as usize;
            let offset  = if self.selected >= visible { self.selected - visible + 1 } else { 0 };
            let bk_w    = list_w.saturating_sub(6) as usize; // reservar para bitrate

            let items: Vec<ListItem> = self
                .results
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible)
                .map(|(i, s)| {
                    let active  = i == self.selected;
                    let prefix  = if active { "▶  " } else { "   " };
                    let name: String = if s.name.chars().count() > bk_w {
                        s.name.chars().take(bk_w.saturating_sub(1)).collect::<String>() + "…"
                    } else {
                        format!("{:<width$}", s.name, width = bk_w)
                    };
                    let bitrate = s
                        .bitrate_kbps
                        .map(|b| format!("{b:>3}k"))
                        .unwrap_or_else(|| "    ".to_string());

                    let (name_style, br_style) = if active {
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
                        Span::styled(name,   name_style),
                        Span::styled(bitrate, br_style),
                    ]))
                })
                .collect();

            List::new(items).render(list_area, buf);
        }

        // ── Hint ─────────────────────────────────────────────────────
        let hint_area = Rect::new(content_x, hint_row.y, content_w, 1);
        Paragraph::new(Line::from(vec![
            key("[↵]"), sep(" Play  "),
            key("[↑↓]"), sep(" Navegar  "),
            key("[Esc]"), sep(" Cerrar"),
        ]))
        .alignment(Alignment::Center)
        .render(hint_area, buf);
    }
}

fn key(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD))
}

fn sep(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::MUTED))
}
