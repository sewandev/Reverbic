use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget},
};

use crate::app::SearchMode;
use crate::i18n::t;
use crate::station::{COUNTRIES, GENRES};
use crate::ui::strings;
use crate::ui::widgets::scroll_offset_for_selection;

use super::helpers::{placeholder_example, render_filter_list_body, spin_frame, FilterListParams};
use super::SearchModalWidget;
use super::{
    filter_list_layout, header_list_layout, radio_favorites_list_layout, radio_name_layout,
};

impl<'a> SearchModalWidget<'a> {
    pub(super) fn render_tabs(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let tab_area = Rect::new(content_x, area.y, content_w, 1);
        let radio_active = Style::default()
            .fg(self.palette.radio_accent)
            .add_modifier(Modifier::BOLD);
        let spotify_active = Style::default()
            .fg(self.palette.spotify)
            .add_modifier(Modifier::BOLD);
        let youtube_active = Style::default()
            .fg(self.palette.youtube)
            .add_modifier(Modifier::BOLD);
        let inactive = Style::default().fg(self.palette.muted);

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

        let radio_dot = self.palette.playing;
        let spotify_dot = match self.spotify_status {
            crate::app::SpotifyAuthStatus::LoggedIn if self.spotify_remote_blocked => {
                self.palette.warning
            }
            crate::app::SpotifyAuthStatus::LoggedIn => self.palette.playing,
            _ => self.palette.dim,
        };
        let youtube_dot = if !self.youtube_cookies_configured {
            self.palette.dim
        } else if self.youtube_session_health == Some(false) {
            self.palette.danger
        } else {
            self.palette.playing
        };

        Paragraph::new(Line::from(vec![
            Span::styled("\u{25CF} ", Style::default().fg(radio_dot)),
            Span::styled(t("modal.tab.radio"), radio_st),
            Span::styled("  ", Style::default()),
            Span::styled("\u{25CF} ", Style::default().fg(spotify_dot)),
            Span::styled(t("modal.tab.spotify"), spotify_st),
            Span::styled("  ", Style::default()),
            Span::styled("\u{25CF} ", Style::default().fg(youtube_dot)),
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

        let layout = radio_name_layout(area);

        self.render_radio_subtabs(layout.subtab, content_x, content_w, buf);

        match self.radio_sub_tab {
            RadioSubTab::Search => self.render_name_search(layout.body, content_x, content_w, buf),
            RadioSubTab::Favorites => {
                self.render_favorites_body(layout.body, content_x, content_w, buf)
            }
            RadioSubTab::Playlists => {
                self.render_playlists_body(layout.body, content_x, content_w, buf)
            }
        }
    }

    fn render_name_search(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let layout = filter_list_layout(area);

        buf[(content_x, layout.input.y)]
            .set_symbol("┃")
            .set_fg(self.palette.accent)
            .set_bg(self.palette.panel_bg);

        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);
        let text_area = Rect::new(text_x, layout.input.y, text_w, 1);

        if self.query.is_empty() {
            Paragraph::new(Span::styled(
                placeholder_example(),
                Style::default().fg(self.palette.muted),
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
                Span::styled(visible, Style::default().fg(self.palette.highlight)),
                Span::styled(
                    "_",
                    Style::default()
                        .fg(self.palette.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .render(text_area, buf);
        }

        buf[(content_x, layout.cap.y)]
            .set_symbol("╹")
            .set_fg(self.palette.accent)
            .set_bg(self.palette.panel_bg);

        self.render_results(layout.list, content_x, content_w, buf);
    }

    fn render_radio_subtabs(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        use crate::app::RadioSubTab;
        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);

        let active = Style::default()
            .fg(self.palette.radio_accent)
            .add_modifier(Modifier::BOLD);
        let inactive = Style::default().fg(self.palette.dim);

        let (search_st, fav_st, pl_st) = match self.radio_sub_tab {
            RadioSubTab::Search => (active, inactive, inactive),
            RadioSubTab::Favorites => (inactive, active, inactive),
            RadioSubTab::Playlists => (inactive, inactive, active),
        };

        Paragraph::new(Line::from(vec![
            Span::styled(t("modal.radio.subtab.search"), search_st),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!(
                    "[ {} ({}) ]",
                    t("modal.radio.subtab.favorites.label"),
                    self.favorites.len()
                ),
                fav_st,
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!(
                    "[ {} ({}) ]",
                    t("modal.radio.subtab.playlists.label"),
                    self.playlists.len()
                ),
                pl_st,
            ),
        ]))
        .render(Rect::new(text_x, area.y, text_w, 1), buf);
    }

    fn render_favorites_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);
        let list_area = radio_favorites_list_layout(area);

        if self.favorites.is_empty() {
            Paragraph::new(Span::styled(
                t("modal.favorites.empty"),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(text_x, list_area.y, text_w, 1), buf);
            return;
        }

        let visible_n = list_area.height as usize;
        let needs_scroll = self.favorites.len() > visible_n;
        let items_w = text_w.saturating_sub(if needs_scroll { 1 } else { 0 });
        let items_area = Rect::new(text_x, list_area.y, items_w, list_area.height);
        let offset = scroll_offset_for_selection(
            self.radio_fav_selected,
            visible_n,
            self.radio_fav_scroll_offset,
        );

        let items: Vec<ListItem> = self
            .favorites
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, fav)| {
                self.station_list_item(
                    fav,
                    i == self.radio_fav_selected,
                    self.playing_favorite_index == Some(i),
                    "★ ",
                    items_w,
                )
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

    fn station_list_item(
        &self,
        station: &crate::favorites::FavoriteStation,
        active: bool,
        is_playing: bool,
        icon: &str,
        items_w: u16,
    ) -> ListItem<'static> {
        let (prefix, name_st, star_st, meta_st) = if active {
            (
                "▶  ",
                Style::default()
                    .fg(self.palette.playing)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(self.palette.playing)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(self.palette.playing),
            )
        } else if is_playing {
            (
                "   ",
                Style::default().fg(self.palette.playing),
                Style::default().fg(self.palette.accent),
                Style::default().fg(self.palette.muted),
            )
        } else {
            (
                "   ",
                Style::default().fg(self.palette.highlight),
                Style::default().fg(self.palette.accent),
                Style::default().fg(self.palette.muted),
            )
        };
        let display_name = strings::title_case(&station.name);
        let mut meta_parts: Vec<String> = Vec::new();
        if !station.country.is_empty() {
            meta_parts.push(strings::title_case(&station.country));
        }
        if let Some(tag) = station.tags.first() {
            if !tag.is_empty() {
                meta_parts.push(strings::title_case(tag));
            }
        }
        if !station.homepage.is_empty() {
            meta_parts.push(station.homepage.clone());
        }

        let name_w = items_w.saturating_sub(5) as usize;
        let name_truncated = strings::truncate(&display_name, name_w);

        if meta_parts.is_empty() {
            ListItem::new(Line::from(vec![
                Span::styled(prefix, name_st),
                Span::styled(icon.to_string(), star_st),
                Span::styled(name_truncated.to_string(), name_st),
            ]))
        } else {
            let meta_str = format!("  ·  {}", meta_parts.join("  ·  "));
            let avail_w =
                items_w.saturating_sub(5 + name_truncated.chars().count() as u16) as usize;
            let meta_trunc = strings::truncate(&meta_str, avail_w);
            ListItem::new(Line::from(vec![
                Span::styled(prefix, name_st),
                Span::styled(icon.to_string(), star_st),
                Span::styled(name_truncated.to_string(), name_st),
                Span::styled(meta_trunc.to_string(), meta_st),
            ]))
        }
    }

    fn render_playlists_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);
        let list_area = radio_favorites_list_layout(area);

        if let Some(pl_idx) = self.radio_open_playlist {
            let Some(playlist) = self.playlists.get(pl_idx) else {
                return;
            };
            let layout = header_list_layout(list_area);

            Paragraph::new(Line::from(vec![
                Span::styled("« ", Style::default().fg(self.palette.muted)),
                Span::styled(
                    strings::title_case(&playlist.name),
                    Style::default()
                        .fg(self.palette.radio_accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "  ·  {} {}",
                        playlist.stations.len(),
                        t("modal.playlists.stations_label")
                    ),
                    Style::default().fg(self.palette.muted),
                ),
            ]))
            .render(Rect::new(text_x, layout.header.y, text_w, 1), buf);

            if playlist.stations.is_empty() {
                Paragraph::new(Span::styled(
                    t("modal.playlists.empty_stations"),
                    Style::default().fg(self.palette.muted),
                ))
                .render(Rect::new(text_x, layout.list.y, text_w, 1), buf);
                return;
            }

            let visible_n = layout.list.height as usize;
            let needs_scroll = playlist.stations.len() > visible_n;
            let items_w = text_w.saturating_sub(if needs_scroll { 1 } else { 0 });
            let items_area = Rect::new(text_x, layout.list.y, items_w, layout.list.height);
            let offset = scroll_offset_for_selection(
                self.radio_playlist_station_selected,
                visible_n,
                self.radio_playlist_station_scroll_offset,
            );

            let items: Vec<ListItem> = playlist
                .stations
                .iter()
                .enumerate()
                .skip(offset)
                .take(visible_n)
                .map(|(i, station)| {
                    self.station_list_item(
                        station,
                        i == self.radio_playlist_station_selected,
                        self.playing_playlist_station_index == Some(i),
                        "♪ ",
                        items_w,
                    )
                })
                .collect();

            List::new(items).render(items_area, buf);

            if needs_scroll {
                self.render_scrollbar(
                    items_area,
                    playlist.stations.len(),
                    self.radio_playlist_station_selected,
                    buf,
                );
            }
            return;
        }

        if self.playlists.is_empty() {
            Paragraph::new(Span::styled(
                t("modal.playlists.empty"),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(text_x, list_area.y, text_w, 1), buf);
            return;
        }

        let visible_n = list_area.height as usize;
        let needs_scroll = self.playlists.len() > visible_n;
        let items_w = text_w.saturating_sub(if needs_scroll { 1 } else { 0 });
        let items_area = Rect::new(text_x, list_area.y, items_w, list_area.height);
        let offset = scroll_offset_for_selection(
            self.radio_playlist_selected,
            visible_n,
            self.radio_playlist_scroll_offset,
        );

        let items: Vec<ListItem> = self
            .playlists
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, playlist)| {
                let active = i == self.radio_playlist_selected;
                let (prefix, name_st, meta_st) = if active {
                    (
                        "▶  ",
                        Style::default()
                            .fg(self.palette.playing)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(self.palette.playing),
                    )
                } else {
                    (
                        "   ",
                        Style::default().fg(self.palette.highlight),
                        Style::default().fg(self.palette.muted),
                    )
                };
                let display_name = strings::title_case(&playlist.name);
                let name_w = items_w.saturating_sub(5) as usize;
                let name_truncated = strings::truncate(&display_name, name_w);
                let meta_str = format!(
                    "  ·  {} {}",
                    playlist.stations.len(),
                    t("modal.playlists.stations_label")
                );
                let avail_w =
                    items_w.saturating_sub(5 + name_truncated.chars().count() as u16) as usize;
                let meta_trunc = strings::truncate(&meta_str, avail_w);
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, name_st),
                    Span::styled("♪ ", Style::default().fg(self.palette.accent)),
                    Span::styled(name_truncated.to_string(), name_st),
                    Span::styled(meta_trunc.to_string(), meta_st),
                ]))
            })
            .collect();

        List::new(items).render(items_area, buf);

        if needs_scroll {
            self.render_scrollbar(
                items_area,
                self.playlists.len(),
                self.radio_playlist_selected,
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
            let layout = header_list_layout(area);

            let header = Rect::new(content_x, layout.header.y, content_w, 1);
            Paragraph::new(Line::from(vec![
                Span::styled("< ", Style::default().fg(self.palette.muted)),
                Span::styled(
                    self.genre_query,
                    Style::default()
                        .fg(self.palette.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  >", Style::default().fg(self.palette.muted)),
            ]))
            .render(header, buf);

            self.render_results(layout.list, content_x, content_w, buf);
            return;
        }

        let (needs_scroll, list_area, total) = render_filter_list_body(
            FilterListParams {
                filter: self.genre_filter,
                placeholder: &t("modal.genre.placeholder"),
                items: GENRES,
                selected: self.genre_selected,
                scroll_offset: self.genre_filter_scroll_offset,
                loading: self.loading,
                loading_text: &t("modal.loading.genre"),
            },
            self.palette,
            area,
            content_x,
            content_w,
            buf,
        );

        if needs_scroll {
            self.render_scrollbar(
                list_area,
                total,
                self.genre_selected.min(total.saturating_sub(1)),
                buf,
            );
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
            let layout = header_list_layout(area);
            let header = Rect::new(content_x, layout.header.y, content_w, 1);
            Paragraph::new(Line::from(vec![
                Span::styled("< ", Style::default().fg(self.palette.muted)),
                Span::styled(
                    self.genre_query,
                    Style::default()
                        .fg(self.palette.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  >", Style::default().fg(self.palette.muted)),
            ]))
            .render(header, buf);
            self.render_results(layout.list, content_x, content_w, buf);
            return;
        }

        let (needs_scroll, list_area, total) = render_filter_list_body(
            FilterListParams {
                filter: self.country_filter,
                placeholder: &t("modal.country.placeholder"),
                items: COUNTRIES,
                selected: self.country_selected,
                scroll_offset: self.country_filter_scroll_offset,
                loading: self.loading,
                loading_text: &t("modal.loading.country"),
            },
            self.palette,
            area,
            content_x,
            content_w,
            buf,
        );

        if needs_scroll {
            self.render_scrollbar(
                list_area,
                total,
                self.country_selected.min(total.saturating_sub(1)),
                buf,
            );
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
                Style::default().fg(self.palette.muted),
            ))
            .render(items_area, buf);
            return;
        }

        if self.results.is_empty() {
            if !self.query.is_empty() {
                Paragraph::new(Span::styled(
                    t("modal.empty.no_results"),
                    Style::default().fg(self.palette.muted),
                ))
                .render(items_area, buf);
            }
            return;
        }

        let scroll_offset = match self.mode {
            SearchMode::Genre => self.radio_genre_results_scroll_offset,
            SearchMode::Country => self.radio_country_results_scroll_offset,
            _ => self.radio_search_scroll_offset,
        };
        let offset = scroll_offset_for_selection(self.selected, visible_n, scroll_offset);

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
                            .fg(self.palette.playing)
                            .add_modifier(Modifier::BOLD),
                        Style::default().fg(self.palette.accent),
                    )
                } else {
                    (
                        Style::default().fg(self.palette.highlight),
                        Style::default().fg(self.palette.muted),
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
                ("▲", self.palette.dim)
            } else if row == track_h - 1 && offset + track_h < total {
                ("▼", self.palette.dim)
            } else if row == thumb {
                let color = match self.mode {
                    crate::app::SearchMode::Spotify => self.palette.spotify,
                    crate::app::SearchMode::Youtube => self.palette.youtube,
                    _ => self.palette.accent,
                };
                ("┃", color)
            } else {
                ("│", self.palette.muted)
            };
            buf[(sb_x, sy)]
                .set_symbol(sym)
                .set_fg(fg)
                .set_bg(self.palette.panel_bg);
        }
    }
}
