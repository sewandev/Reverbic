use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Widget},
};

use crate::app::SearchMode;
use crate::station::{DynamicStation, GENRES, COUNTRIES};
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

fn spin_frame() -> &'static str {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    const SPIN: &[&str] = &["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
    SPIN[(ms / 80) as usize % SPIN.len()]
}

pub struct SearchModalWidget<'a> {
    pub query:             &'a str,
    pub results:           &'a [DynamicStation],
    pub loading:           bool,
    pub selected:          usize,
    pub mode:              &'a SearchMode,
    pub genre_selected:    usize,
    pub genre_filter:      &'a str,
    pub genre_query:       &'a str,
    pub country_selected:  usize,
    pub country_filter:    &'a str,
    pub history:           &'a [String],
    pub settings_selected:  usize,
    pub autoplay_last:      bool,
    pub overlay_mode:       &'a str,
    pub crossfade:          &'a str,
    pub media_keys:         bool,
    pub tray_icon:          bool,
    pub notifications:      bool,
    pub trending_results:   &'a [DynamicStation],
    pub trending_loading:   bool,
    pub trending_selected:  usize,
}

impl Widget for SearchModalWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_bg(OVERLAY_BG);
            }
        }

        let w = area.width.min(66).max(44);
        let h = area.height.min(14).max(10);
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y + area.height.saturating_sub(h) / 2;
        let panel = Rect::new(x, y, w, h);

        Clear.render(panel, buf);

        let bottom_hint = self.bottom_hint();
        let block = Block::default()
            .title_top(
                Line::from(Span::styled(
                    " BUSCAR RADIO ",
                    Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Center),
            )
            .title_bottom(
                Line::from(bottom_hint).alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::ACCENT))
            .style(Style::default().bg(BG));

        let inner = block.inner(panel);
        block.render(panel, buf);

        let h_pad: u16 = 2;
        let content_x = inner.x + h_pad;
        let content_w = inner.width.saturating_sub(h_pad * 2);

        let [tabs_row, body_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(inner);

        self.render_tabs(tabs_row, content_x, content_w, buf);

        match self.mode {
            SearchMode::Name     => self.render_name_body(body_area, content_x, content_w, buf),
            SearchMode::Genre    => self.render_genre_body(body_area, content_x, content_w, buf),
            SearchMode::Country  => self.render_country_body(body_area, content_x, content_w, buf),
            SearchMode::Trending => self.render_trending_body(body_area, content_x, content_w, buf),
            SearchMode::Settings => self.render_settings_body(body_area, content_x, content_w, buf),
        }
    }
}

impl SearchModalWidget<'_> {
    fn bottom_hint(&self) -> Vec<Span<'static>> {
        let showing = !self.results.is_empty();
        if showing {
            return vec![
                Span::raw(" "),
                key("[↵]"), sep(" Play  "),
                key("[v]"), sep(" Votar  "),
                key("[R]"), sep(" Random  "),
                key("[↑↓]"), sep(" Nav  "),
                key("[Esc]"), sep(" Volver "),
            ];
        }
        match self.mode {
            SearchMode::Name => vec![
                Span::raw(" "),
                key("[↵]"), sep(" Play  "),
                key("[↑↓]"), sep(" Nav  "),
                key("[Tab]"), sep(" Siguiente  "),
                key("[Esc]"), sep(" Cerrar "),
            ],
            SearchMode::Genre | SearchMode::Country => vec![
                Span::raw(" "),
                key("[↵]"), sep(" Buscar  "),
                key("[↑↓]"), sep(" Nav  "),
                key("[Tab]"), sep(" Siguiente  "),
                key("[Esc]"), sep(" Cerrar "),
            ],
            SearchMode::Trending => vec![
                Span::raw(" "),
                key("[↵]"), sep(" Play  "),
                key("[v]"), sep(" Votar  "),
                key("[R]"), sep(" Random  "),
                key("[r]"), sep(" Recargar  "),
                key("[↑↓]"), sep(" Nav  "),
                key("[Esc]"), sep(" Cerrar "),
            ],
            SearchMode::Settings => vec![
                Span::raw(" "),
                key("[Space]"), sep(" Cambiar  "),
                key("[↑↓]"), sep(" Nav  "),
                key("[Tab]"), sep(" Siguiente  "),
                key("[Esc]"), sep(" Cerrar "),
            ],
        }
    }

    fn render_tabs(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let tab_area = Rect::new(content_x, area.y, content_w, 1);
        let active   = Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD);
        let inactive = Style::default().fg(theme::MUTED);
        let (ns, gs, cs, ts, ss) = match self.mode {
            SearchMode::Name     => (active, inactive, inactive, inactive, inactive),
            SearchMode::Genre    => (inactive, active, inactive, inactive, inactive),
            SearchMode::Country  => (inactive, inactive, active, inactive, inactive),
            SearchMode::Trending => (inactive, inactive, inactive, active, inactive),
            SearchMode::Settings => (inactive, inactive, inactive, inactive, active),
        };
        let line = Line::from(vec![
            Span::styled("[ Nombre ]", ns),
            Span::styled("  ", Style::default()),
            Span::styled("[ Género ]", gs),
            Span::styled("  ", Style::default()),
            Span::styled("[ País ]", cs),
            Span::styled("  ", Style::default()),
            Span::styled("[ Trending ]", ts),
            Span::styled("  ", Style::default()),
            Span::styled("[ Config ]", ss),
        ]);
        Paragraph::new(line).render(tab_area, buf);
    }

    fn render_name_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let [_gap, input_row, cap_row, list_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        buf[(content_x, input_row.y)]
            .set_symbol("┃").set_fg(theme::ACCENT).set_bg(BG);

        let text_x    = content_x + 2;
        let text_w    = content_w.saturating_sub(2);
        let text_area = Rect::new(text_x, input_row.y, text_w, 1);

        if self.query.is_empty() {
            Paragraph::new(Span::styled(
                format!("Buscar radio... \"{}\"", placeholder_example()),
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

        buf[(content_x, cap_row.y)]
            .set_symbol("╹").set_fg(theme::ACCENT).set_bg(BG);

        if self.query.is_empty() && self.results.is_empty() && !self.history.is_empty() {
            self.render_history(list_area, content_x, content_w, buf);
        } else {
            self.render_results(list_area, content_x, content_w, buf);
        }
    }

    fn render_history(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let list_x    = content_x + 2;
        let list_w    = content_w.saturating_sub(2);
        let list_area = Rect::new(list_x, area.y, list_w, area.height);
        let items: Vec<ListItem> = self.history
            .iter()
            .take(list_area.height as usize)
            .map(|q| ListItem::new(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(q.as_str(), Style::default().fg(theme::MUTED)),
            ])))
            .collect();
        List::new(items).render(list_area, buf);
    }

    fn render_genre_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        if !self.results.is_empty() {
            let [header_row, list_area] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(area);

            let header = Rect::new(content_x, header_row.y, content_w, 1);
            Paragraph::new(Line::from(vec![
                Span::styled("< ", Style::default().fg(theme::MUTED)),
                Span::styled(self.genre_query, Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled("  >", Style::default().fg(theme::MUTED)),
            ]))
            .render(header, buf);

            self.render_results(list_area, content_x, content_w, buf);
            return;
        }

        let [_gap, input_row, cap_row, list_body] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        buf[(content_x, input_row.y)]
            .set_symbol("┃").set_fg(theme::ACCENT).set_bg(BG);

        let text_x    = content_x + 2;
        let text_w    = content_w.saturating_sub(2);
        let text_area = Rect::new(text_x, input_row.y, text_w, 1);

        if self.genre_filter.is_empty() {
            Paragraph::new(Span::styled(
                "Filtrar género…",
                Style::default().fg(theme::MUTED),
            ))
            .render(text_area, buf);
        } else {
            Paragraph::new(Line::from(vec![
                Span::styled(self.genre_filter, Style::default().fg(theme::HIGHLIGHT)),
                Span::styled("_", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            ]))
            .render(text_area, buf);
        }

        buf[(content_x, cap_row.y)]
            .set_symbol("╹").set_fg(theme::ACCENT).set_bg(BG);

        if self.loading {
            let area = Rect::new(text_x, list_body.y, text_w, 1);
            Paragraph::new(Span::styled(
                format!("{}  Buscando género…", spin_frame()),
                Style::default().fg(theme::MUTED),
            ))
            .render(area, buf);
            return;
        }

        let filtered = filtered_genres(self.genre_filter);
        let list_x    = content_x + 2;
        let list_w    = content_w.saturating_sub(2);
        let list_area = Rect::new(list_x, list_body.y, list_w, list_body.height);
        let visible_n = list_area.height as usize;
        let offset    = if self.genre_selected >= visible_n {
            self.genre_selected - visible_n + 1
        } else {
            0
        };

        if filtered.is_empty() {
            Paragraph::new(Span::styled("Sin coincidencias", Style::default().fg(theme::MUTED)))
                .render(list_area, buf);
            return;
        }

        let items: Vec<ListItem> = filtered
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, (_, label))| {
                let active = i == self.genre_selected;
                let (prefix, style) = if active {
                    ("▶  ", Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD))
                } else {
                    ("   ", Style::default().fg(theme::HIGHLIGHT))
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(*label, style),
                ]))
            })
            .collect();

        List::new(items).render(list_area, buf);

        if filtered.len() > visible_n {
            self.render_scrollbar(list_area, filtered.len(), self.genre_selected, buf);
        }
    }

    fn render_results(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let list_x  = content_x + 2;
        let visible_n = area.height as usize;
        let needs_scroll = self.results.len() > visible_n;
        let name_w  = content_w.saturating_sub(if needs_scroll { 14 } else { 13 }) as usize;
        let items_w = content_w.saturating_sub(if needs_scroll { 3 } else { 2 });
        let items_area = Rect::new(list_x, area.y, items_w, area.height);

        if self.loading {
            Paragraph::new(Span::styled(
                format!("{}  Buscando…", spin_frame()),
                Style::default().fg(theme::MUTED),
            ))
            .render(items_area, buf);
            return;
        }

        if self.results.is_empty() {
            let msg = if self.query.is_empty() && !matches!(self.mode, SearchMode::Genre) {
                "Escribe el nombre para buscar radios de todo el mundo"
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
                let name: String = if s.name.chars().count() > name_w {
                    s.name.chars().take(name_w.saturating_sub(1)).collect::<String>() + "…"
                } else {
                    format!("{:<width$}", s.name, width = name_w)
                };
                let bitrate = s.bitrate_kbps
                    .map(|b| format!("{b:>4}k"))
                    .unwrap_or_else(|| "    ".to_string());
                let votes = format_votes(s.votes);
                let (name_st, meta_st) = if active {
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
                    Span::styled(bitrate, meta_st),
                    Span::styled(votes,   meta_st),
                ]))
            })
            .collect();

        List::new(items).render(items_area, buf);

        if needs_scroll {
            self.render_scrollbar(items_area, self.results.len(), self.selected, buf);
        }
    }

    fn render_scrollbar(&self, list_area: Rect, total: usize, selected: usize, buf: &mut Buffer) {
        let sb_x    = list_area.x + list_area.width + 1;
        let track_h = list_area.height as usize;
        let offset  = if selected >= track_h { selected - track_h + 1 } else { 0 };
        let thumb   = if total <= 1 { 0 } else {
            (selected * (track_h.saturating_sub(1))) / (total - 1)
        };

        for row in 0..track_h {
            let sy = list_area.y + row as u16;
            let (sym, fg) = if row == 0 && offset > 0 {
                ("▲", theme::DIM)
            } else if row == track_h - 1 && offset + track_h < total {
                ("▼", theme::DIM)
            } else if row == thumb {
                ("┃", theme::ACCENT)
            } else {
                ("│", theme::MUTED)
            };
            buf[(sb_x, sy)].set_symbol(sym).set_fg(fg).set_bg(BG);
        }
    }

    fn render_trending_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let list_x    = content_x + 2;
        let list_w    = content_w.saturating_sub(2);

        if self.trending_loading {
            Paragraph::new(Span::styled(
                format!("{}  Cargando trending…", spin_frame()),
                Style::default().fg(theme::MUTED),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        if self.trending_results.is_empty() {
            Paragraph::new(Span::styled(
                "Sin resultados. Presiona [r] para recargar.",
                Style::default().fg(theme::MUTED),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        let visible_n    = area.height as usize;
        let needs_scroll = self.trending_results.len() > visible_n;
        let name_w       = content_w.saturating_sub(if needs_scroll { 17 } else { 16 }) as usize;
        let items_w      = content_w.saturating_sub(if needs_scroll { 3 } else { 2 });
        let items_area   = Rect::new(list_x, area.y, items_w, area.height);
        let offset       = if self.trending_selected >= visible_n {
            self.trending_selected - visible_n + 1
        } else { 0 };

        let items: Vec<ListItem> = self.trending_results
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, s)| {
                let active  = i == self.trending_selected;
                let prefix  = if active { "▶  " } else { "   " };
                let rank    = format!("{:>2}. ", offset + i + 1);
                let max_n   = name_w.saturating_sub(4);
                let name: String = if s.name.chars().count() > max_n {
                    s.name.chars().take(max_n.saturating_sub(1)).collect::<String>() + "…"
                } else {
                    format!("{:<width$}", s.name, width = max_n)
                };
                let bitrate = s.bitrate_kbps
                    .map(|b| format!("{b:>4}k"))
                    .unwrap_or_else(|| "    ".to_string());
                let votes = format_votes(s.votes);
                let (name_st, meta_st) = if active {
                    (Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD), Style::default().fg(theme::ACCENT))
                } else {
                    (Style::default().fg(theme::HIGHLIGHT), Style::default().fg(theme::MUTED))
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix,  name_st),
                    Span::styled(rank,    meta_st),
                    Span::styled(name,    name_st),
                    Span::styled(bitrate, meta_st),
                    Span::styled(votes,   meta_st),
                ]))
            })
            .collect();

        List::new(items).render(items_area, buf);

        if needs_scroll {
            self.render_scrollbar(items_area, self.trending_results.len(), self.trending_selected, buf);
        }
    }

    fn render_settings_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let items: &[(&str, &str)] = &[
            ("Auto-play última radio al iniciar", if self.autoplay_last { "ON" } else { "OFF" }),
            ("Overlay Windows",                   self.overlay_mode),
            ("Crossfade",                         self.crossfade),
            ("Teclas multimedia",                 if self.media_keys    { "ON" } else { "OFF" }),
            ("Icono en bandeja",                  if self.tray_icon     { "ON" } else { "OFF" }),
            ("Notificaciones",                    if self.notifications  { "ON" } else { "OFF" }),
        ];

        let list_x    = content_x + 2;
        let list_w    = content_w.saturating_sub(2);
        let [_gap, list_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
        ]).areas(area);

        for (i, (label, value)) in items.iter().enumerate() {
            let y = list_area.y + i as u16;
            if y >= list_area.y + list_area.height { break; }
            let active = i == self.settings_selected;
            let (label_st, val_st) = if active {
                (
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                    Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    Style::default().fg(theme::HIGHLIGHT),
                    Style::default().fg(theme::MUTED),
                )
            };
            let prefix = if active { "▶  " } else { "   " };
            let row = Rect::new(list_x, y, list_w, 1);
            Paragraph::new(Line::from(vec![
                Span::styled(prefix, label_st),
                Span::styled(*label, label_st),
                Span::styled("  [", Style::default().fg(theme::MUTED)),
                Span::styled(*value, val_st),
                Span::styled("]", Style::default().fg(theme::MUTED)),
            ]))
            .render(row, buf);
        }
    }

    fn render_country_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        if !self.results.is_empty() {
            let [header_row, list_area] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(area);
            let header = Rect::new(content_x, header_row.y, content_w, 1);
            Paragraph::new(Line::from(vec![
                Span::styled("< ", Style::default().fg(theme::MUTED)),
                Span::styled(self.genre_query, Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled("  >", Style::default().fg(theme::MUTED)),
            ]))
            .render(header, buf);
            self.render_results(list_area, content_x, content_w, buf);
            return;
        }

        let [_gap, input_row, cap_row, list_body] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        buf[(content_x, input_row.y)]
            .set_symbol("┃").set_fg(theme::ACCENT).set_bg(BG);

        let text_x    = content_x + 2;
        let text_w    = content_w.saturating_sub(2);
        let text_area = Rect::new(text_x, input_row.y, text_w, 1);

        if self.country_filter.is_empty() {
            Paragraph::new(Span::styled("Filtrar país…", Style::default().fg(theme::MUTED)))
                .render(text_area, buf);
        } else {
            Paragraph::new(Line::from(vec![
                Span::styled(self.country_filter, Style::default().fg(theme::HIGHLIGHT)),
                Span::styled("_", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            ]))
            .render(text_area, buf);
        }

        buf[(content_x, cap_row.y)]
            .set_symbol("╹").set_fg(theme::ACCENT).set_bg(BG);

        if self.loading {
            Paragraph::new(Span::styled(
                format!("{}  Buscando país…", spin_frame()),
                Style::default().fg(theme::MUTED),
            ))
            .render(Rect::new(text_x, list_body.y, text_w, 1), buf);
            return;
        }

        let filtered  = filtered_countries(self.country_filter);
        let list_x    = content_x + 2;
        let list_w    = content_w.saturating_sub(2);
        let list_area = Rect::new(list_x, list_body.y, list_w, list_body.height);
        let visible_n = list_area.height as usize;
        let offset    = if self.country_selected >= visible_n { self.country_selected - visible_n + 1 } else { 0 };

        if filtered.is_empty() {
            Paragraph::new(Span::styled("Sin coincidencias", Style::default().fg(theme::MUTED)))
                .render(list_area, buf);
            return;
        }

        let items: Vec<ListItem> = filtered
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, (_, label))| {
                let active = i == self.country_selected;
                let (prefix, style) = if active {
                    ("▶  ", Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD))
                } else {
                    ("   ", Style::default().fg(theme::HIGHLIGHT))
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(*label, style),
                ]))
            })
            .collect();

        List::new(items).render(list_area, buf);

        if filtered.len() > visible_n {
            self.render_scrollbar(list_area, filtered.len(), self.country_selected, buf);
        }
    }
}

fn filtered_genres(filter: &str) -> Vec<(&'static str, &'static str)> {
    if filter.is_empty() {
        return GENRES.iter().map(|&(t, l)| (t, l)).collect();
    }
    let f = filter.to_lowercase();
    GENRES.iter()
        .filter(|(_, label)| label.to_lowercase().contains(&f))
        .map(|&(t, l)| (t, l))
        .collect()
}

fn filtered_countries(filter: &str) -> Vec<(&'static str, &'static str)> {
    if filter.is_empty() {
        return COUNTRIES.iter().map(|&(t, l)| (t, l)).collect();
    }
    let f = filter.to_lowercase();
    COUNTRIES.iter()
        .filter(|(_, label)| label.to_lowercase().contains(&f))
        .map(|&(t, l)| (t, l))
        .collect()
}

fn key(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD))
}

fn sep(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::MUTED))
}

fn format_votes(v: u32) -> String {
    match v {
        0           => "     ".to_string(),
        1..=999     => format!("{:>4}v", v),
        1000..=9999 => format!("{:.1}kv", v as f32 / 1000.0),
        _           => format!("{:>3}kv", v / 1000),
    }
}
