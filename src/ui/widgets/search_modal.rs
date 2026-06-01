use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};

use crate::app::{SearchMode, SettingItem, settings_items};
use crate::i18n::{t, current_language, Language};
use crate::station::{filter_items, DynamicStation, GENRES, COUNTRIES};
use crate::ui::theme;

const BG:         Color = theme::PANEL_BG;
const OVERLAY_BG: Color = theme::OVERLAY_COLOR;

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
    pub settings_selected:  usize,
    pub autoplay_last:      bool,
    pub overlay_mode:       String,
    pub overlay_position:   String,
    pub crossfade:          String,
    pub media_keys:         bool,
    pub tray_icon:          bool,
    pub notifications:      bool,
    pub restore_volume:     bool,
    pub duck_enabled:              bool,
    pub duck_volume:               u8,
    pub overlay_alpha:             u8,
    pub screensaver_secs:          u16,
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
                key("[↵]"),    sep_s(format!(" {}  ", t("hint.play"))),
                key("[R]"),    sep_s(format!(" {}  ", t("hint.random"))),
                key("[↑↓]"),  sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Esc]"),  sep_s(format!(" {} ",  t("hint.back"))),
            ];
        }
        match self.mode {
            SearchMode::Name => vec![
                Span::raw(" "),
                key("[↵]"),    sep(" Play  "),
                key("[↑↓]"),  sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Tab]"),  sep_s(format!(" {}  ", t("hint.next_tab"))),
                key("[Esc]"),  sep_s(format!(" {} ",  t("hint.close"))),
            ],
            SearchMode::Genre | SearchMode::Country => vec![
                Span::raw(" "),
                key("[↵]"),    sep_s(format!(" {}  ", t("hint.search"))),
                key("[↑↓]"),  sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Tab]"),  sep_s(format!(" {}  ", t("hint.next_tab"))),
                key("[Esc]"),  sep_s(format!(" {} ",  t("hint.close"))),
            ],
            SearchMode::Settings => vec![
                Span::raw(" "),
                key("[Space]"), sep_s(format!(" {}  ", t("hint.change"))),
                key("[↑↓]"),  sep_s(format!(" {}  ", t("hint.nav"))),
                key("[Tab]"),  sep_s(format!(" {}  ", t("hint.next_tab"))),
                key("[Esc]"),  sep_s(format!(" {} ",  t("hint.close"))),
            ],
        }
    }

    fn render_tabs(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let tab_area = Rect::new(content_x, area.y, content_w, 1);
        let active   = Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD);
        let inactive = Style::default().fg(theme::MUTED);
        let (ns, gs, cs, ss) = match self.mode {
            SearchMode::Name     => (active, inactive, inactive, inactive),
            SearchMode::Genre    => (inactive, active, inactive, inactive),
            SearchMode::Country  => (inactive, inactive, active, inactive),
            SearchMode::Settings => (inactive, inactive, inactive, active),
        };
        let line = Line::from(vec![
            Span::styled(t("modal.tab.name"),    ns),
            Span::styled("  ",                   Style::default()),
            Span::styled(t("modal.tab.genre"),   gs),
            Span::styled("  ",                   Style::default()),
            Span::styled(t("modal.tab.country"), cs),
            Span::styled("  ",                   Style::default()),
            Span::styled(t("modal.tab.config"),  ss),
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
                placeholder_example(),
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

        self.render_results(list_area, content_x, content_w, buf);
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
                t("modal.genre.placeholder"),
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
                format!("{}  {}", spin_frame(), t("modal.loading.genre")),
                Style::default().fg(theme::MUTED),
            ))
            .render(area, buf);
            return;
        }

        let filtered = filter_items(GENRES, self.genre_filter);
        let list_x    = content_x + 2;
        let list_w    = content_w.saturating_sub(2);
        let list_area = Rect::new(list_x, list_body.y, list_w, list_body.height);
        let visible_n = list_area.height.saturating_sub(1) as usize;
        let offset    = super::scroll_offset(self.genre_selected, visible_n);

        if filtered.is_empty() {
            Paragraph::new(Span::styled(t("modal.empty.no_match"), Style::default().fg(theme::MUTED)))
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
        let visible_n = area.height.saturating_sub(1) as usize;
        let needs_scroll = self.results.len() > visible_n;
        let name_w  = content_w.saturating_sub(if needs_scroll { 9 } else { 8 }) as usize;
        let items_w = content_w.saturating_sub(if needs_scroll { 3 } else { 2 });
        let items_area = Rect::new(list_x, area.y, items_w, area.height);

        if self.loading {
            Paragraph::new(Span::styled(
                format!("{}  {}", spin_frame(), t("modal.loading")),
                Style::default().fg(theme::MUTED),
            ))
            .render(items_area, buf);
            return;
        }

        if self.results.is_empty() {
            if !self.query.is_empty() {
                Paragraph::new(Span::styled(t("modal.empty.no_results"), Style::default().fg(theme::MUTED)))
                    .render(items_area, buf);
            }
            return;
        }

        let offset = super::scroll_offset(self.selected, visible_n);

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

    fn item_value(&self, item: SettingItem) -> String {
        let on  = t("config.value.on");
        let off = t("config.value.off");
        match item {
            SettingItem::Autoplay        => if self.autoplay_last  { on } else { off },
            SettingItem::RestoreVolume   => if self.restore_volume { on } else { off },
            SettingItem::Crossfade       => self.crossfade.clone(),
            SettingItem::OverlayMode     => self.overlay_mode.clone(),
            SettingItem::OverlayAlpha    => format!("{}%", self.overlay_alpha),
            SettingItem::OverlayPosition => self.overlay_position.clone(),
            SettingItem::Screensaver     => screensaver_display(self.screensaver_secs),
            SettingItem::DuckEnabled     => if self.duck_enabled { on } else { off },
            SettingItem::DuckVolume      => format!("{}%", self.duck_volume),
            SettingItem::MediaKeys       => if self.media_keys    { on } else { off },
            SettingItem::TrayIcon        => if self.tray_icon     { on } else { off },
            SettingItem::Notifications   => if self.notifications { on } else { off },
            SettingItem::Language        => match current_language() {
                Language::Es => t("lang.display.es"),
                Language::En => t("lang.display.en"),
            },
        }
    }

    fn render_settings_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let mut rows: Vec<(String, Option<String>)> = Vec::new();
        let mut last_group = "";
        for item in settings_items(self.duck_enabled) {
            let gk = item.group_key();
            if gk != last_group {
                rows.push((t(gk), None));
                last_group = gk;
            }
            rows.push((item.label(), Some(self.item_value(item))));
        }
        let mut item_idx = 0usize;
        let mut selected_row = 0usize;
        for (ri, (_, val)) in rows.iter().enumerate() {
            if val.is_some() {
                if item_idx == self.settings_selected { selected_row = ri; }
                item_idx += 1;
            }
        }

        let list_x = content_x + 2;
        let list_w = content_w.saturating_sub(2);
        let [_gap, items_area, tooltip_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
        ]).areas(area);

        let visible_n = items_area.height.saturating_sub(1) as usize;
        let offset    = super::scroll_offset(selected_row, visible_n);

        item_idx = 0;
        for (ri, (label, val_opt)) in rows.iter().enumerate() {
            let display_y = ri as isize - offset as isize;
            if display_y < 0 || display_y >= visible_n as isize {
                if val_opt.is_some() { item_idx += 1; }
                continue;
            }
            let y = items_area.y + display_y as u16;

            if let Some(value) = val_opt {
                let active = item_idx == self.settings_selected;
                let (label_st, val_st) = if active {
                    (Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                     Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD))
                } else {
                    (Style::default().fg(theme::HIGHLIGHT),
                     Style::default().fg(theme::MUTED))
                };
                let prefix = if active { "▶  " } else { "   " };
                Paragraph::new(Line::from(vec![
                    Span::styled(prefix,        label_st),
                    Span::styled(label.clone(), label_st),
                    Span::styled("  [",         Style::default().fg(theme::MUTED)),
                    Span::styled(value.clone(), val_st),
                    Span::styled("]",           Style::default().fg(theme::MUTED)),
                ])).render(Rect::new(list_x, y, list_w, 1), buf);
                item_idx += 1;
            } else {
                Paragraph::new(Span::styled(
                    format!("── {} ", label),
                    Style::default().fg(theme::MUTED),
                )).render(Rect::new(list_x, y, list_w, 1), buf);
            }
        }
        let total_rows = rows.len();
        if total_rows > visible_n {
            let scroll_area = Rect::new(list_x, items_area.y, list_w, items_area.height);
            self.render_scrollbar(scroll_area, total_rows, selected_row, buf);
        }

        let sep = "─".repeat(content_w as usize);
        Paragraph::new(Span::styled(sep, Style::default().fg(theme::DIM)))
            .render(Rect::new(content_x, tooltip_area.y, content_w, 1), buf);

        let tooltip = settings_items(self.duck_enabled)
            .get(self.settings_selected)
            .map(|item| t(item.tooltip_key()))
            .unwrap_or_default();
        Paragraph::new(tooltip)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(theme::MUTED))
            .render(Rect::new(list_x, tooltip_area.y + 1, list_w, 2), buf);
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
            Paragraph::new(Span::styled(t("modal.country.placeholder"), Style::default().fg(theme::MUTED)))
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
                format!("{}  {}", spin_frame(), t("modal.loading.country")),
                Style::default().fg(theme::MUTED),
            ))
            .render(Rect::new(text_x, list_body.y, text_w, 1), buf);
            return;
        }

        let filtered  = filter_items(COUNTRIES, self.country_filter);
        let list_x    = content_x + 2;
        let list_w    = content_w.saturating_sub(2);
        let list_area = Rect::new(list_x, list_body.y, list_w, list_body.height);
        let visible_n = list_area.height.saturating_sub(1) as usize;
        let offset    = super::scroll_offset(self.country_selected, visible_n);

        if filtered.is_empty() {
            Paragraph::new(Span::styled(t("modal.empty.no_match"), Style::default().fg(theme::MUTED)))
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

fn screensaver_display(secs: u16) -> String {
    match secs {
        0          => "OFF".to_string(),
        s if s < 60 => format!("{}s", s),
        s           => format!("{}m", s / 60),
    }
}

fn key(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD))
}

fn sep(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::MUTED))
}

fn sep_s(s: String) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::MUTED))
}

