use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget},
};

use crate::app::SearchMode;
use crate::i18n::t;
use crate::station::{COUNTRIES, GENRES};
use crate::ui::strings;
use crate::ui::theme;

use super::helpers::{placeholder_example, render_filter_list_body, spin_frame, FilterListParams};
use super::{SearchModalWidget, BG};

impl<'a> SearchModalWidget<'a> {
    pub(super) fn render_tabs(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let tab_area = Rect::new(content_x, area.y, content_w, 1);
        let radio_active = Style::default()
            .fg(theme::RADIO_ACCENT)
            .add_modifier(Modifier::BOLD);
        let spotify_active = Style::default()
            .fg(theme::SPOTIFY_GREEN)
            .add_modifier(Modifier::BOLD);
        let youtube_active = Style::default()
            .fg(theme::DANGER)
            .add_modifier(Modifier::BOLD);
        let inactive = Style::default().fg(theme::MUTED);

        let radio_st = match self.mode {
            SearchMode::Name | SearchMode::Genre | SearchMode::Country => radio_active,
            _ => inactive,
        };
        let spotify_st = match self.mode {
            SearchMode::Spotify => spotify_active,
            _ => inactive,
        };
        let youtube_st = match self.mode {
            SearchMode::Youtube => youtube_active,
            _ => inactive,
        };

        Paragraph::new(Line::from(vec![
            Span::styled(t("modal.tab.radio"), radio_st),
            Span::styled("  ", Style::default()),
            Span::styled(t("modal.tab.spotify"), spotify_st),
            Span::styled("  ", Style::default()),
            Span::styled(t("modal.tab.youtube"), youtube_st),
        ]))
        .render(tab_area, buf);
    }

    pub(super) fn render_name_body(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        use crate::app::RadioSubTab;

        let [_gap, subtab_row, body] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        self.render_radio_subtabs(subtab_row, content_x, content_w, buf);

        match self.radio_sub_tab {
            RadioSubTab::Search => self.render_name_search(body, content_x, content_w, buf),
            RadioSubTab::Favorites => self.render_favorites_body(body, content_x, content_w, buf),
        }
    }

    fn render_name_search(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let [_gap, input_row, cap_row, list_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        buf[(content_x, input_row.y)]
            .set_symbol("┃")
            .set_fg(theme::ACCENT)
            .set_bg(BG);

        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);
        let text_area = Rect::new(text_x, input_row.y, text_w, 1);

        if self.query.is_empty() {
            Paragraph::new(Span::styled(
                placeholder_example(),
                Style::default().fg(theme::MUTED),
            ))
            .render(text_area, buf);
        } else {
            let max_q = text_w.saturating_sub(2) as usize;
            let visible: String = if self.query.chars().count() > max_q {
                self.query
                    .chars()
                    .rev()
                    .take(max_q)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect()
            } else {
                self.query.to_owned()
            };
            Paragraph::new(Line::from(vec![
                Span::styled(visible, Style::default().fg(theme::HIGHLIGHT)),
                Span::styled(
                    "_",
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .render(text_area, buf);
        }

        buf[(content_x, cap_row.y)]
            .set_symbol("╹")
            .set_fg(theme::ACCENT)
            .set_bg(BG);

        self.render_results(list_area, content_x, content_w, buf);
    }

    fn render_radio_subtabs(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        use crate::app::RadioSubTab;
        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);

        let active = Style::default()
            .fg(theme::RADIO_ACCENT)
            .add_modifier(Modifier::BOLD);
        let inactive = Style::default().fg(theme::DIM);

        let (search_st, fav_st) = match self.radio_sub_tab {
            RadioSubTab::Search => (active, inactive),
            RadioSubTab::Favorites => (inactive, active),
        };

        Paragraph::new(Line::from(vec![
            Span::styled(t("modal.radio.subtab.search"), search_st),
            Span::styled("  ", Style::default()),
            Span::styled(t("modal.radio.subtab.favorites"), fav_st),
        ]))
        .render(Rect::new(text_x, area.y, text_w, 1), buf);
    }

    fn render_favorites_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);
        let [_gap, list_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        if self.favorites.is_empty() {
            Paragraph::new(Span::styled(
                t("modal.favorites.empty"),
                Style::default().fg(theme::MUTED),
            ))
            .render(Rect::new(text_x, list_area.y, text_w, 1), buf);
            return;
        }

        let visible_n = list_area.height as usize;
        let needs_scroll = self.favorites.len() > visible_n;
        let items_w = text_w.saturating_sub(if needs_scroll { 1 } else { 0 });
        let items_area = Rect::new(text_x, list_area.y, items_w, list_area.height);
        let offset = crate::ui::widgets::scroll_offset(self.radio_fav_selected, visible_n);

        let items: Vec<ListItem> = self
            .favorites
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, fav)| {
                let active = i == self.radio_fav_selected;
                let is_playing = self.playing_favorite_index == Some(i);
                let (prefix, name_st, star_st, meta_st) = if active {
                    (
                        "▶  ",
                        Style::default()
                            .fg(theme::PLAYING)
                            .add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(theme::PLAYING)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(theme::PLAYING),
                    )
                } else if is_playing {
                    (
                        "   ",
                        Style::default().fg(theme::PLAYING),
                        Style::default().fg(theme::ACCENT),
                        Style::default().fg(theme::MUTED),
                    )
                } else {
                    (
                        "   ",
                        Style::default().fg(theme::HIGHLIGHT),
                        Style::default().fg(theme::ACCENT),
                        Style::default().fg(theme::MUTED),
                    )
                };
                let display_name = strings::title_case(&fav.name);
                let mut meta_parts: Vec<String> = Vec::new();
                if !fav.country.is_empty() {
                    meta_parts.push(strings::title_case(&fav.country));
                }
                if let Some(tag) = fav.tags.first() {
                    if !tag.is_empty() {
                        meta_parts.push(strings::title_case(tag));
                    }
                }
                if !fav.homepage.is_empty() {
                    meta_parts.push(fav.homepage.clone());
                }

                let name_w = items_w.saturating_sub(5) as usize;
                let name_truncated = strings::truncate(&display_name, name_w);

                if meta_parts.is_empty() {
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, name_st),
                        Span::styled("★ ", star_st),
                        Span::styled(name_truncated.to_string(), name_st),
                    ]))
                } else {
                    let meta_str = format!("  ·  {}", meta_parts.join("  ·  "));
                    let avail_w =
                        items_w.saturating_sub(5 + name_truncated.chars().count() as u16) as usize;
                    let meta_trunc = strings::truncate(&meta_str, avail_w);
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, name_st),
                        Span::styled("★ ", star_st),
                        Span::styled(name_truncated.to_string(), name_st),
                        Span::styled(meta_trunc.to_string(), meta_st),
                    ]))
                }
            })
            .collect();

        List::new(items).render(items_area, buf);

        if needs_scroll {
            self.render_scrollbar(
                items_area,
                self.favorites.len(),
                self.radio_fav_selected,
                buf,
            );
        }
    }

    pub(super) fn render_genre_body(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        if !self.results.is_empty() {
            let [header_row, list_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

            let header = Rect::new(content_x, header_row.y, content_w, 1);
            Paragraph::new(Line::from(vec![
                Span::styled("< ", Style::default().fg(theme::MUTED)),
                Span::styled(
                    self.genre_query,
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  >", Style::default().fg(theme::MUTED)),
            ]))
            .render(header, buf);

            self.render_results(list_area, content_x, content_w, buf);
            return;
        }

        let (needs_scroll, list_area, total) = render_filter_list_body(
            FilterListParams {
                filter: self.genre_filter,
                placeholder: &t("modal.genre.placeholder"),
                items: GENRES,
                selected: self.genre_selected,
                loading: self.loading,
                loading_text: &t("modal.loading.genre"),
            },
            area,
            content_x,
            content_w,
            buf,
        );

        if needs_scroll {
            self.render_scrollbar(list_area, total, self.genre_selected, buf);
        }
    }

    pub(super) fn render_country_body(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        if !self.results.is_empty() {
            let [header_row, list_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
            let header = Rect::new(content_x, header_row.y, content_w, 1);
            Paragraph::new(Line::from(vec![
                Span::styled("< ", Style::default().fg(theme::MUTED)),
                Span::styled(
                    self.genre_query,
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  >", Style::default().fg(theme::MUTED)),
            ]))
            .render(header, buf);
            self.render_results(list_area, content_x, content_w, buf);
            return;
        }

        let (needs_scroll, list_area, total) = render_filter_list_body(
            FilterListParams {
                filter: self.country_filter,
                placeholder: &t("modal.country.placeholder"),
                items: COUNTRIES,
                selected: self.country_selected,
                loading: self.loading,
                loading_text: &t("modal.loading.country"),
            },
            area,
            content_x,
            content_w,
            buf,
        );

        if needs_scroll {
            self.render_scrollbar(list_area, total, self.country_selected, buf);
        }
    }

    pub(super) fn render_results(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        let list_x = content_x + 2;
        let visible_n = area.height.saturating_sub(1) as usize;
        let needs_scroll = self.results.len() > visible_n;
        let name_w = content_w.saturating_sub(if needs_scroll { 9 } else { 8 }) as usize;
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
                Paragraph::new(Span::styled(
                    t("modal.empty.no_results"),
                    Style::default().fg(theme::MUTED),
                ))
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
                let active = i == self.selected;
                let prefix = if active { "▶  " } else { "   " };
                let name: String = {
                    let t = strings::truncate(&s.name, name_w);
                    format!("{:<width$}", t, width = name_w)
                };
                let bitrate = s
                    .bitrate_kbps
                    .map(|b| format!("{b:>4}k"))
                    .unwrap_or_else(|| "    ".to_string());
                let (name_st, meta_st) = if active {
                    (
                        Style::default()
                            .fg(theme::PLAYING)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(theme::ACCENT),
                    )
                } else {
                    (
                        Style::default().fg(theme::HIGHLIGHT),
                        Style::default().fg(theme::MUTED),
                    )
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, name_st),
                    Span::styled(name, name_st),
                    Span::styled(bitrate, meta_st),
                ]))
            })
            .collect();

        List::new(items).render(items_area, buf);

        if needs_scroll {
            self.render_scrollbar(items_area, self.results.len(), self.selected, buf);
        }
    }

    pub(super) fn render_radio_body(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        match self.mode {
            SearchMode::Genre => self.render_genre_body(area, content_x, content_w, buf),
            SearchMode::Country => self.render_country_body(area, content_x, content_w, buf),
            _ => self.render_name_body(area, content_x, content_w, buf),
        }
    }

    pub(super) fn render_scrollbar(
        &self,
        list_area: Rect,
        total: usize,
        selected: usize,
        buf: &mut Buffer,
    ) {
        let sb_x = list_area.x + list_area.width + 1;
        let track_h = list_area.height as usize;
        let offset = if selected >= track_h {
            selected - track_h + 1
        } else {
            0
        };
        let thumb = if total <= 1 {
            0
        } else {
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
