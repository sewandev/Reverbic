use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget, Wrap},
};

use crate::app::SpotifyAuthStatus;
use crate::i18n::t;
use crate::integrations::spotify::devices::SpotifyDevice;
use crate::ui::strings;
use crate::ui::widgets::scroll_offset_for_selection;

use super::helpers::{render_filter_input, spin_frame};
use super::SearchModalWidget;

fn fmt_duration(ms: u32) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}

fn active_spotify_device<'a>(
    devices: &'a [SpotifyDevice],
    active_device_id: Option<&str>,
) -> Option<&'a SpotifyDevice> {
    active_device_id
        .and_then(|id| {
            devices
                .iter()
                .find(|device| device.id.as_deref() == Some(id))
        })
        .or_else(|| devices.iter().find(|device| device.is_active))
}

impl<'a> SearchModalWidget<'a> {
    pub(super) fn render_spotify_body(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        let lx = content_x + 2;
        let lw = content_w.saturating_sub(2);
        match self.spotify_status {
            SpotifyAuthStatus::LoggedIn => {
                self.render_spotify_logged_in(area, content_x, content_w, buf)
            }
            SpotifyAuthStatus::Connecting => self.render_spotify_connecting(area, lx, lw, buf),
            SpotifyAuthStatus::Error(msg) => {
                self.render_spotify_error(area, lx, lw, buf, msg.as_str())
            }
            SpotifyAuthStatus::Idle => self.render_spotify_connect(area, lx, lw, buf),
        }
    }

    fn render_spotify_logged_in(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        use crate::app::SpotifySubTab;

        let [_gap, subtab_row, _body_gap, body, footer_row] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);

        {
            let tab_style = |active: bool| {
                if active {
                    Style::default()
                        .fg(self.palette.spotify)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.palette.dim)
                }
            };
            let mut text = ratatui::text::Text::default();
            text.lines.push(Line::from(vec![
                Span::styled(
                    t("modal.spotify.subtab.search"),
                    tab_style(self.spotify_sub_tab == SpotifySubTab::Search),
                ),
                Span::raw("  "),
                Span::styled(
                    t("modal.spotify.subtab.liked"),
                    tab_style(self.spotify_sub_tab == SpotifySubTab::Liked),
                ),
                Span::raw("  "),
                Span::styled(
                    t("modal.spotify.subtab.playlists"),
                    tab_style(self.spotify_sub_tab == SpotifySubTab::Playlists),
                ),
                Span::raw("  "),
                Span::styled(
                    t("Top Tracks"),
                    tab_style(self.spotify_sub_tab == SpotifySubTab::TopTracks),
                ),
                Span::raw("  "),
                Span::styled(
                    t("Recent"),
                    tab_style(self.spotify_sub_tab == SpotifySubTab::Recent),
                ),
                Span::raw("  "),
                Span::styled(
                    t("Albums"),
                    tab_style(self.spotify_sub_tab == SpotifySubTab::Albums),
                ),
            ]));

            Paragraph::new(text).render(
                Rect::new(text_x, subtab_row.y, text_w, subtab_row.height),
                buf,
            );
        }

        match self.spotify_sub_tab {
            SpotifySubTab::Search => {
                let [input_row, cap_row, list_area] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .areas(body);

                buf[(content_x, input_row.y)]
                    .set_symbol("┃")
                    .set_fg(self.palette.spotify)
                    .set_bg(self.palette.panel_bg);
                let text_area = Rect::new(text_x, input_row.y, text_w, 1);
                render_filter_input(
                    self.spotify_query,
                    &t("modal.spotify.placeholder"),
                    text_area,
                    self.palette,
                    buf,
                    self.palette.spotify,
                );

                buf[(content_x, cap_row.y)]
                    .set_symbol("╹")
                    .set_fg(self.palette.spotify)
                    .set_bg(self.palette.panel_bg);

                self.render_spotify_list(list_area, text_x, text_w, buf);
            }
            SpotifySubTab::Liked => {
                self.render_spotify_liked(body, text_x, text_w, buf);
            }
            SpotifySubTab::Playlists => {
                if self.spotify_open_playlist.is_some() {
                    self.render_spotify_playlist_tracks(body, text_x, text_w, buf);
                } else {
                    self.render_spotify_playlists(body, text_x, text_w, buf);
                }
            }
            SpotifySubTab::TopTracks => {
                self.render_spotify_top_tracks(body, text_x, text_w, buf);
            }
            SpotifySubTab::Recent => {
                self.render_spotify_recent_tracks(body, text_x, text_w, buf);
            }
            SpotifySubTab::Albums => {
                if self.spotify_open_album.is_some() {
                    self.render_spotify_album_tracks(body, text_x, text_w, buf);
                } else {
                    self.render_spotify_albums(body, text_x, text_w, buf);
                }
            }
        }

        let active_device =
            active_spotify_device(self.spotify_devices, self.spotify_active_device_id);
        let mode_text =
            if self.spotify_playback_mode_kind == crate::config::SpotifyPlaybackMode::Native {
                t("modal.spotify.footer.mode_native")
            } else {
                let dev_name = active_device
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| t("modal.spotify.footer.unknown_device"));
                let dev_type = active_device.map(|d| d.device_type.as_str()).unwrap_or("");
                let switch_hint = if self.spotify_devices.len() > 1 {
                    t("modal.spotify.footer.mode_remote_switch_hint")
                } else {
                    String::new()
                };
                let mode = t("modal.spotify.footer.mode_remote");
                let active = t("modal.spotify.footer.active");

                format!("{mode} {dev_name} * {dev_type} [{active}]{switch_hint}")
            };

        let footer_area = Rect::new(text_x, footer_row.y, text_w, 1);
        let mode_style = Style::default()
            .fg(ratatui::style::Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let badge_style = Style::default()
            .fg(self.palette.spotify)
            .add_modifier(Modifier::BOLD);

        let premium_badge = self
            .spotify_is_premium
            .then(|| t("integrations.spotify.account_premium"));
        let badge_len = premium_badge
            .as_ref()
            .map(|badge| badge.chars().count() as u16)
            .unwrap_or(0);

        if let Some(badge) = premium_badge.filter(|_| badge_len + 4 < text_w) {
            let [mode_area, _gap, badge_area] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(2),
                Constraint::Length(badge_len),
            ])
            .areas(footer_area);

            if mode_area.width > 0 {
                Paragraph::new(Span::styled(
                    strings::truncate(&mode_text, mode_area.width as usize),
                    mode_style,
                ))
                .render(mode_area, buf);
            }

            Paragraph::new(Span::styled(badge, badge_style)).render(badge_area, buf);
        } else {
            Paragraph::new(Span::styled(
                strings::truncate(&mode_text, footer_area.width as usize),
                mode_style,
            ))
            .render(footer_area, buf);
        }
    }

    fn render_spotify_connect(&self, area: Rect, lx: u16, lw: u16, buf: &mut Buffer) {
        let mut y = area.y + 1;
        if y >= area.bottom() {
            return;
        }
        Paragraph::new(Span::styled(
            "SPOTIFY",
            Style::default()
                .fg(self.palette.playing)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(ratatui::layout::Alignment::Center)
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 2;
        if y >= area.bottom() {
            return;
        }
        Paragraph::new(Span::styled(
            t("modal.spotify.remote_feature"),
            Style::default()
                .fg(self.palette.highlight)
                .add_modifier(Modifier::BOLD),
        ))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 1;

        if y >= area.bottom() {
            return;
        }
        Paragraph::new(Span::styled(
            t("modal.spotify.remote_subtitle"),
            Style::default().fg(self.palette.muted),
        ))
        .wrap(Wrap { trim: true })
        .render(
            Rect::new(lx, y, lw, 2.min(area.bottom().saturating_sub(y))),
            buf,
        );
        y += 3;
        if y >= area.bottom() {
            return;
        }
        Paragraph::new(Span::styled(
            "─".repeat(lw as usize),
            Style::default().fg(self.palette.dim),
        ))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 1;
        if y >= area.bottom() {
            return;
        }
        Paragraph::new(Line::from(vec![
            Span::styled(
                "[↵]  ",
                Style::default()
                    .fg(self.palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                t("modal.spotify.connect_action"),
                Style::default()
                    .fg(self.palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 2;
        if y >= area.bottom() {
            return;
        }
        Paragraph::new(Span::styled(
            t("modal.spotify.experimental"),
            Style::default().fg(self.palette.caution),
        ))
        .wrap(Wrap { trim: true })
        .render(Rect::new(lx, y, lw, area.bottom().saturating_sub(y)), buf);
    }

    fn render_spotify_connecting(&self, area: Rect, lx: u16, lw: u16, buf: &mut Buffer) {
        let mut y = area.y + 2;
        if y >= area.bottom() {
            return;
        }
        Paragraph::new(Line::from(vec![
            Span::styled(spin_frame(), Style::default().fg(self.palette.accent)),
            Span::styled(
                format!("  {}", t("integrations.spotify.web.waiting")),
                Style::default().fg(self.palette.muted),
            ),
        ]))
        .render(Rect::new(lx, y, lw, 1), buf);
        y += 1;
        if y < area.bottom() {
            Paragraph::new(Span::styled(
                t("integrations.spotify.web.waiting2"),
                Style::default().fg(self.palette.dim),
            ))
            .render(Rect::new(lx, y, lw, 1), buf);
        }
    }

    fn render_spotify_error(&self, area: Rect, lx: u16, lw: u16, buf: &mut Buffer, msg: &str) {
        let mut y = area.y + 2;
        if y >= area.bottom() {
            return;
        }

        Paragraph::new(Span::styled(
            strings::truncate(msg, lw as usize),
            Style::default().fg(self.palette.warning),
        ))
        .render(Rect::new(lx, y, lw, 1), buf);

        y += 1;
        if y < area.bottom() {
            Paragraph::new(Span::styled(
                t("integrations.spotify.error.hint"),
                Style::default().fg(self.palette.dim),
            ))
            .render(Rect::new(lx, y, lw, 1), buf);
        }

        y += 2;
        if y < area.bottom() {
            Paragraph::new(Line::from(vec![
                Span::styled(
                    "[↵]  ",
                    Style::default()
                        .fg(self.palette.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    t("modal.spotify.connect_action"),
                    Style::default()
                        .fg(self.palette.highlight)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
            .render(Rect::new(lx, y, lw, 1), buf);
        }
    }

    fn render_spotify_list(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        if self.spotify_loading {
            Paragraph::new(Span::styled(
                format!("{}  {}", spin_frame(), t("modal.loading.spotify")),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        if self.spotify_search_rate_limited {
            let mut y = area.y;
            let countdown = if self.spotify_rate_limited_secs > 0 {
                format!(
                    "{}  {}:{:02}",
                    t("modal.spotify.rate_limit_countdown"),
                    self.spotify_rate_limited_secs / 60,
                    self.spotify_rate_limited_secs % 60
                )
            } else {
                t("modal.spotify.rate_limit_countdown")
            };
            Paragraph::new(Span::styled(
                countdown,
                Style::default()
                    .fg(self.palette.warning)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(Rect::new(list_x, y, list_w, 1), buf);
            y += 1;
            if y < area.bottom() {
                use ratatui::widgets::Wrap;
                Paragraph::new(Span::styled(
                    t("modal.spotify.rate_limited"),
                    Style::default().fg(self.palette.warning),
                ))
                .wrap(Wrap { trim: true })
                .render(
                    Rect::new(list_x, y, list_w, area.bottom().saturating_sub(y)),
                    buf,
                );
            }
            return;
        }

        if self.spotify_results.is_empty() {
            if !self.spotify_query.is_empty() {
                Paragraph::new(Span::styled(
                    t("modal.empty.no_results"),
                    Style::default().fg(self.palette.muted),
                ))
                .render(Rect::new(list_x, area.y, list_w, 1), buf);
            }
            return;
        }

        const ITEM_HEIGHT: usize = 2;
        let visible_n = (area.height as usize) / ITEM_HEIGHT;
        let offset = scroll_offset_for_selection(
            self.spotify_selected,
            visible_n,
            self.spotify_scroll_offset,
        );

        let items: Vec<ListItem> = self
            .spotify_results
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, track)| {
                let active = i == self.spotify_selected;
                if active {
                    let name_max = list_w.saturating_sub(4) as usize;
                    let dur = fmt_duration(track.duration_ms);
                    let meta_max = list_w.saturating_sub(3 + dur.len() as u16) as usize;
                    let name = strings::truncate(&track.name, name_max);
                    let meta_raw = format!("{} · {}", track.artist, track.album);
                    let meta = strings::truncate(&meta_raw, meta_max);
                    let name_st = Style::default()
                        .fg(self.palette.spotify)
                        .add_modifier(Modifier::BOLD);
                    let meta_st = Style::default().fg(self.palette.spotify);
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled("▶  ", name_st),
                            Span::styled(name, name_st),
                            Span::styled(format!("  {}", dur), meta_st),
                        ]),
                        Line::from(vec![
                            Span::styled("   ", meta_st),
                            Span::styled(meta, meta_st),
                        ]),
                    ])
                } else {
                    let name_max = list_w.saturating_sub(3) as usize;
                    let name = strings::truncate(&track.name, name_max);
                    let artist_max = list_w.saturating_sub(3) as usize;
                    let artist = strings::truncate(&track.artist, artist_max);
                    let name_st = Style::default().fg(self.palette.highlight);
                    let artist_st = Style::default().fg(self.palette.muted);
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled("   ", name_st),
                            Span::styled(name, name_st),
                        ]),
                        Line::from(vec![
                            Span::styled("   ", artist_st),
                            Span::styled(artist, artist_st),
                        ]),
                    ])
                }
            })
            .collect();

        let list_area = Rect::new(list_x, area.y, list_w, area.height);
        List::new(items).render(list_area, buf);

        let needs_scroll = self.spotify_results.len() > visible_n;
        if needs_scroll {
            self.render_scrollbar(
                list_area,
                self.spotify_results.len(),
                self.spotify_selected,
                buf,
            );
        }

        if self.spotify_loading_more {
            let indicator_y = list_area.bottom().saturating_sub(1);
            if indicator_y > list_area.y {
                Paragraph::new(Span::styled(
                    format!(
                        "{}  {}",
                        super::helpers::spin_frame(),
                        t("modal.spotify.load_more")
                    ),
                    Style::default().fg(self.palette.dim),
                ))
                .render(Rect::new(list_area.x, indicator_y, list_area.width, 1), buf);
            }
        }
    }

    fn render_generic_track_list(
        &self,
        list_area: Rect,
        buf: &mut Buffer,
        tracks: &[crate::integrations::spotify::SpotifyTrack],
        selected: usize,
        scroll_offset: usize,
        loading: bool,
    ) {
        let lx = list_area.x;
        let lw = list_area.width;
        if loading {
            Paragraph::new(Span::styled(
                format!("{}  {}", spin_frame(), t("modal.loading.spotify")),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(lx, list_area.y, lw, 1), buf);
            return;
        }

        if tracks.is_empty() {
            Paragraph::new(Span::styled(
                t("modal.empty.no_results"),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(lx, list_area.y, lw, 1), buf);
            return;
        }

        const ITEM_HEIGHT: usize = 2;
        let visible_n = (list_area.height as usize) / ITEM_HEIGHT;
        let offset = scroll_offset_for_selection(selected, visible_n, scroll_offset);

        let items: Vec<ListItem> = tracks
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, track)| {
                let active = i == selected;
                if active {
                    let name_max = lw.saturating_sub(4) as usize;
                    let dur = fmt_duration(track.duration_ms);
                    let meta_max = lw.saturating_sub(3 + dur.len() as u16) as usize;
                    let name = strings::truncate(&track.name, name_max);
                    let meta_raw = format!("{} · {}", track.artist, track.album);
                    let meta = strings::truncate(&meta_raw, meta_max);
                    let name_st = Style::default()
                        .fg(self.palette.spotify)
                        .add_modifier(Modifier::BOLD);
                    let meta_st = Style::default().fg(self.palette.spotify);
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled("▶  ", name_st),
                            Span::styled(name, name_st),
                            Span::styled(format!("  {}", dur), meta_st),
                        ]),
                        Line::from(vec![
                            Span::styled("   ", meta_st),
                            Span::styled(meta, meta_st),
                        ]),
                    ])
                } else {
                    let name_max = lw.saturating_sub(3) as usize;
                    let name = strings::truncate(&track.name, name_max);
                    let artist_max = lw.saturating_sub(3) as usize;
                    let artist = strings::truncate(&track.artist, artist_max);
                    let name_st = Style::default().fg(self.palette.highlight);
                    let artist_st = Style::default().fg(self.palette.muted);
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled("   ", name_st),
                            Span::styled(name, name_st),
                        ]),
                        Line::from(vec![
                            Span::styled("   ", artist_st),
                            Span::styled(artist, artist_st),
                        ]),
                    ])
                }
            })
            .collect();

        List::new(items).render(list_area, buf);

        if tracks.len() > visible_n {
            self.render_scrollbar(list_area, tracks.len(), selected, buf);
        }
    }

    fn render_spotify_liked(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        self.render_generic_track_list(
            Rect::new(list_x, area.y, list_w, area.height),
            buf,
            self.spotify_liked_tracks,
            self.spotify_liked_selected,
            self.spotify_liked_scroll_offset,
            self.spotify_liked_loading,
        );
    }

    fn render_spotify_playlists(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        if self.spotify_playlists_loading {
            Paragraph::new(Span::styled(
                format!("{}  {}", spin_frame(), t("modal.loading.spotify")),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        if self.spotify_playlists.is_empty() {
            Paragraph::new(Span::styled(
                t("modal.empty.no_results"),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        const ITEM_HEIGHT: usize = 2;
        let visible_n = (area.height as usize) / ITEM_HEIGHT;
        let selected = self.spotify_playlists_selected;
        let offset =
            scroll_offset_for_selection(selected, visible_n, self.spotify_playlists_scroll_offset);

        let items: Vec<ListItem> = self
            .spotify_playlists
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, pl)| {
                let active = i == selected;
                let prefix = if active { "▶  " } else { "   " };
                let name_max = list_w.saturating_sub(3) as usize;
                let name = strings::truncate(&pl.name, name_max);
                let st = if active {
                    Style::default()
                        .fg(self.palette.spotify)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.palette.highlight)
                };
                let sub_st = Style::default().fg(self.palette.muted);
                let sub = strings::truncate(
                    &format!("{} tracks · {}", pl.tracks_total, pl.owner),
                    list_w.saturating_sub(3) as usize,
                );
                ListItem::new(vec![
                    Line::from(vec![Span::styled(prefix, st), Span::styled(name, st)]),
                    Line::from(vec![Span::styled("   ", sub_st), Span::styled(sub, sub_st)]),
                ])
            })
            .collect();

        let list_area = Rect::new(list_x, area.y, list_w, area.height);
        ratatui::widgets::List::new(items).render(list_area, buf);

        if self.spotify_playlists.len() > visible_n {
            self.render_scrollbar(list_area, self.spotify_playlists.len(), selected, buf);
        }
    }

    fn render_spotify_playlist_tracks(
        &self,
        area: Rect,
        list_x: u16,
        list_w: u16,
        buf: &mut Buffer,
    ) {
        if let Some(pl) = self.spotify_open_playlist {
            let esc_hint = "[Esc]";
            let sep = " <- ";
            let reserved = (esc_hint.len() + sep.len() + 1) as u16;
            let title = strings::truncate(&pl.name, list_w.saturating_sub(reserved) as usize);
            let track_count = if pl.tracks_total > 0 {
                format!("  ({} tracks)", pl.tracks_total)
            } else {
                String::new()
            };
            let line = Line::from(vec![
                Span::styled(esc_hint, Style::default().fg(self.palette.muted)),
                Span::styled(sep, Style::default().fg(self.palette.dim)),
                Span::styled(
                    title,
                    Style::default()
                        .fg(self.palette.spotify)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(track_count, Style::default().fg(self.palette.muted)),
            ]);
            Paragraph::new(line).render(Rect::new(list_x, area.y, list_w, 1), buf);
        }
        let inner = Rect::new(
            area.x,
            area.y + 1,
            area.width,
            area.height.saturating_sub(1),
        );
        self.render_generic_track_list(
            inner,
            buf,
            self.spotify_playlist_tracks,
            self.spotify_playlist_tracks_selected,
            self.spotify_playlist_tracks_scroll_offset,
            self.spotify_playlist_tracks_loading,
        );
    }

    fn render_spotify_top_tracks(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        self.render_generic_track_list(
            Rect::new(list_x, area.y, list_w, area.height),
            buf,
            self.spotify_top_tracks,
            self.spotify_top_tracks_selected,
            self.spotify_top_tracks_scroll_offset,
            self.spotify_top_tracks_loading,
        );
    }

    fn render_spotify_recent_tracks(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        self.render_generic_track_list(
            Rect::new(list_x, area.y, list_w, area.height),
            buf,
            self.spotify_recent_tracks,
            self.spotify_recent_tracks_selected,
            self.spotify_recent_tracks_scroll_offset,
            self.spotify_recent_tracks_loading,
        );
    }

    fn render_spotify_albums(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        if self.spotify_albums_loading {
            Paragraph::new(Span::styled(
                format!("{}  {}", spin_frame(), t("modal.loading.spotify")),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        if self.spotify_albums.is_empty() {
            Paragraph::new(Span::styled(
                t("modal.empty.no_results"),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        const ITEM_HEIGHT: usize = 2;
        let visible_n = (area.height as usize) / ITEM_HEIGHT;
        let selected = self.spotify_albums_selected;
        let offset =
            scroll_offset_for_selection(selected, visible_n, self.spotify_albums_scroll_offset);

        let items: Vec<ListItem> = self
            .spotify_albums
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, album)| {
                let active = i == selected;
                let prefix = if active { "▶  " } else { "   " };
                let name_max = list_w.saturating_sub(3) as usize;
                let name = strings::truncate(&album.name, name_max);
                let st = if active {
                    Style::default()
                        .fg(self.palette.spotify)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.palette.highlight)
                };
                let sub_st = Style::default().fg(self.palette.muted);
                let sub = strings::truncate(
                    &format!("{} · {} tracks", album.artist, album.total_tracks),
                    list_w.saturating_sub(3) as usize,
                );
                ListItem::new(vec![
                    Line::from(vec![Span::styled(prefix, st), Span::styled(name, st)]),
                    Line::from(vec![Span::styled("   ", sub_st), Span::styled(sub, sub_st)]),
                ])
            })
            .collect();

        let list_area = Rect::new(list_x, area.y, list_w, area.height);
        ratatui::widgets::List::new(items).render(list_area, buf);

        if self.spotify_albums.len() > visible_n {
            self.render_scrollbar(list_area, self.spotify_albums.len(), selected, buf);
        }
    }

    fn render_spotify_album_tracks(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        if let Some(album) = self.spotify_open_album {
            let esc_hint = "[Esc]";
            let sep = " <- ";
            let reserved = (esc_hint.len() + sep.len() + 1) as u16;
            let title = strings::truncate(&album.name, list_w.saturating_sub(reserved) as usize);
            let track_count = if album.total_tracks > 0 {
                format!("  ({} tracks)", album.total_tracks)
            } else {
                String::new()
            };
            let line = Line::from(vec![
                Span::styled(esc_hint, Style::default().fg(self.palette.muted)),
                Span::styled(sep, Style::default().fg(self.palette.dim)),
                Span::styled(
                    title,
                    Style::default()
                        .fg(self.palette.spotify)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(track_count, Style::default().fg(self.palette.muted)),
            ]);
            Paragraph::new(line).render(Rect::new(list_x, area.y, list_w, 1), buf);
        }
        let inner = Rect::new(
            area.x,
            area.y + 1,
            area.width,
            area.height.saturating_sub(1),
        );
        self.render_generic_track_list(
            inner,
            buf,
            self.spotify_album_tracks,
            self.spotify_album_tracks_selected,
            self.spotify_album_tracks_scroll_offset,
            self.spotify_album_tracks_loading,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spotify_device(id: Option<&str>, name: &str, is_active: bool) -> SpotifyDevice {
        SpotifyDevice {
            id: id.map(str::to_string),
            name: name.to_string(),
            device_type: "Computer".to_string(),
            is_active,
        }
    }

    #[test]
    fn active_spotify_device_uses_active_device_id_before_spotify_active_flag() {
        let devices = vec![
            spotify_device(Some("preserved"), "Preserved", false),
            spotify_device(Some("spotify-active"), "Spotify Active", true),
        ];

        let active = active_spotify_device(&devices, Some("preserved"));

        assert_eq!(active.map(|device| device.name.as_str()), Some("Preserved"));
    }

    #[test]
    fn active_spotify_device_falls_back_to_spotify_active_flag() {
        let devices = vec![
            spotify_device(Some("available"), "Available", false),
            spotify_device(Some("spotify-active"), "Spotify Active", true),
        ];

        let active = active_spotify_device(&devices, None);

        assert_eq!(
            active.map(|device| device.name.as_str()),
            Some("Spotify Active")
        );
    }
}
