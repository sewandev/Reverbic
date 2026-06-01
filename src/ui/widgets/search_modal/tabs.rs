use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget},
};

use crate::app::SearchMode;
use crate::i18n::t;
use crate::station::{filter_items, COUNTRIES, GENRES};
use crate::ui::theme;

use super::helpers::{placeholder_example, render_filter_input, spin_frame};
use super::{BG, SearchModalWidget};

impl<'a> SearchModalWidget<'a> {
    pub(super) fn render_tabs(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let tab_area = Rect::new(content_x, area.y, content_w, 1);
        let active   = Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD);
        let inactive = Style::default().fg(theme::MUTED);
        let (ns, gs, cs, ss, is) = match self.mode {
            SearchMode::Name         => (active, inactive, inactive, inactive, inactive),
            SearchMode::Genre        => (inactive, active, inactive, inactive, inactive),
            SearchMode::Country      => (inactive, inactive, active, inactive, inactive),
            SearchMode::Settings     => (inactive, inactive, inactive, active, inactive),
            SearchMode::Integrations => (inactive, inactive, inactive, inactive, active),
        };
        let line = Line::from(vec![
            Span::styled(t("modal.tab.name"),         ns),
            Span::styled("  ",                        Style::default()),
            Span::styled(t("modal.tab.genre"),        gs),
            Span::styled("  ",                        Style::default()),
            Span::styled(t("modal.tab.country"),      cs),
            Span::styled("  ",                        Style::default()),
            Span::styled(t("modal.tab.config"),       ss),
            Span::styled("  ",                        Style::default()),
            Span::styled(t("modal.tab.integrations"), is),
        ]);
        Paragraph::new(line).render(tab_area, buf);
    }

    pub(super) fn render_name_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
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

    pub(super) fn render_genre_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
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

        render_filter_input(self.genre_filter, &t("modal.genre.placeholder"), text_area, buf);

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
        let offset    = super::super::scroll_offset(self.genre_selected, visible_n);

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

    pub(super) fn render_country_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
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

        render_filter_input(self.country_filter, &t("modal.country.placeholder"), text_area, buf);

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
        let offset    = super::super::scroll_offset(self.country_selected, visible_n);

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

    pub(super) fn render_results(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
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

        let offset = super::super::scroll_offset(self.selected, visible_n);

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

    pub(super) fn render_scrollbar(&self, list_area: Rect, total: usize, selected: usize, buf: &mut Buffer) {
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
}
