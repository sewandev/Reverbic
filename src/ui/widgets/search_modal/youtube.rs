use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget, Wrap},
};

use crate::app::{YoutubeStatus, YoutubeSubTab};
use crate::i18n::t;
use crate::integrations::youtube::{YoutubePlaylist, YoutubeVideo};
use crate::ui::strings;
use crate::ui::widgets::scroll_offset_for_selection;

use super::helpers::{render_filter_input, spin_frame};
use super::SearchModalWidget;
use super::{spotify_titled_track_list_layout, youtube_layout, youtube_search_layout};

fn fmt_duration(secs: u32) -> String {
    if secs == 0 {
        "--:--".to_string()
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

struct VideoListState<'a> {
    videos: &'a [YoutubeVideo],
    selected: usize,
    scroll_offset: usize,
    loading: bool,
    empty_message: &'a str,
}

impl<'a> SearchModalWidget<'a> {
    pub(super) fn render_youtube_body(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        let layout = youtube_layout(area);

        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);

        self.render_youtube_subtabs(layout.subtab, text_x, text_w, buf);

        match self.youtube_status {
            YoutubeStatus::Installing => {
                self.render_youtube_message(
                    layout.body,
                    text_x,
                    text_w,
                    &format!("{}  {}", spin_frame(), t("modal.youtube.installing")),
                    self.palette.muted,
                    buf,
                );
            }
            YoutubeStatus::Resolving => {
                self.render_youtube_message(
                    layout.body,
                    text_x,
                    text_w,
                    &format!("{}  {}", spin_frame(), t("modal.youtube.resolving")),
                    self.palette.muted,
                    buf,
                );
            }
            YoutubeStatus::Error(msg) => {
                self.render_youtube_error(layout.body, text_x, text_w, msg, buf);
            }
            YoutubeStatus::Idle | YoutubeStatus::Ready => match self.youtube_sub_tab {
                YoutubeSubTab::Search => {
                    self.render_youtube_search_body(layout.body, content_x, text_x, text_w, buf)
                }
                YoutubeSubTab::Liked => {
                    self.render_youtube_liked_body(layout.body, text_x, text_w, buf)
                }
                YoutubeSubTab::Playlists => {
                    self.render_youtube_playlists_body(layout.body, text_x, text_w, buf)
                }
            },
        }
    }

    fn render_youtube_subtabs(&self, area: Rect, text_x: u16, text_w: u16, buf: &mut Buffer) {
        let tab_style = |active: bool| {
            if active {
                Style::default()
                    .fg(self.palette.youtube)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.palette.dim)
            }
        };
        let library_tab_style = |active: bool| {
            if !self.youtube_cookies_configured {
                let st = Style::default().fg(self.palette.muted);
                if active {
                    st.add_modifier(Modifier::BOLD)
                } else {
                    st
                }
            } else {
                tab_style(active)
            }
        };

        let line = Line::from(vec![
            Span::styled(
                t("modal.youtube.subtab.search"),
                tab_style(self.youtube_sub_tab == YoutubeSubTab::Search),
            ),
            Span::raw("  "),
            Span::styled(
                t("modal.youtube.subtab.liked"),
                library_tab_style(self.youtube_sub_tab == YoutubeSubTab::Liked),
            ),
            Span::raw("  "),
            Span::styled(
                t("modal.youtube.subtab.playlists"),
                library_tab_style(self.youtube_sub_tab == YoutubeSubTab::Playlists),
            ),
        ]);

        Paragraph::new(line).render(Rect::new(text_x, area.y, text_w, area.height), buf);
    }

    fn render_youtube_search_body(
        &self,
        area: Rect,
        content_x: u16,
        text_x: u16,
        text_w: u16,
        buf: &mut Buffer,
    ) {
        let layout = youtube_search_layout(area);

        buf[(content_x, layout.input.y)]
            .set_symbol("┃")
            .set_fg(self.palette.youtube)
            .set_bg(self.palette.panel_bg);

        render_filter_input(
            self.youtube_query,
            &t("modal.youtube.placeholder"),
            Rect::new(text_x, layout.input.y, text_w, 1),
            self.palette,
            buf,
            self.palette.youtube,
        );

        buf[(content_x, layout.cap.y)]
            .set_symbol("╹")
            .set_fg(self.palette.youtube)
            .set_bg(self.palette.panel_bg);

        if !self.youtube_loading && self.youtube_results.is_empty() && self.youtube_query.is_empty()
        {
            return;
        }

        self.render_video_list(
            layout.list,
            text_x,
            text_w,
            buf,
            VideoListState {
                videos: self.youtube_results,
                selected: self.youtube_selected,
                scroll_offset: self.youtube_scroll_offset,
                loading: self.youtube_loading,
                empty_message: &t("modal.youtube.no_results"),
            },
        );
    }

    fn render_youtube_liked_body(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        if !self.youtube_cookies_configured {
            self.render_youtube_auth_notice(area, buf);
            return;
        }

        self.render_video_list(
            area,
            list_x,
            list_w,
            buf,
            VideoListState {
                videos: self.youtube_liked_videos,
                selected: self.youtube_liked_selected,
                scroll_offset: self.youtube_liked_scroll_offset,
                loading: self.youtube_liked_loading,
                empty_message: &t("modal.youtube.liked_empty"),
            },
        );
    }

    fn render_youtube_playlists_body(
        &self,
        area: Rect,
        list_x: u16,
        list_w: u16,
        buf: &mut Buffer,
    ) {
        if let Some(playlist) = self.youtube_open_playlist {
            self.render_youtube_playlist_videos(area, list_x, list_w, playlist, buf);
            return;
        }

        if !self.youtube_cookies_configured {
            self.render_youtube_auth_notice(area, buf);
            return;
        }

        if self.youtube_playlists_loading {
            Paragraph::new(Span::styled(
                format!("{}  {}", spin_frame(), t("modal.loading.youtube")),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        if self.youtube_playlists.is_empty() {
            Paragraph::new(Span::styled(
                t("modal.youtube.playlists_empty"),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        const ITEM_HEIGHT: usize = 2;
        let visible_n = (area.height as usize) / ITEM_HEIGHT;
        let selected = self.youtube_playlists_selected;
        let offset =
            scroll_offset_for_selection(selected, visible_n, self.youtube_playlists_scroll_offset);

        let items: Vec<ListItem> = self
            .youtube_playlists
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, playlist)| {
                let active = i == selected;
                let prefix = if active { "▶  " } else { "   " };
                let name_max = list_w.saturating_sub(3) as usize;
                let name = strings::truncate(&playlist.title, name_max);
                let st = if active {
                    Style::default()
                        .fg(self.palette.youtube)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.palette.highlight)
                };
                let sub_st = Style::default().fg(self.palette.muted);
                let sub = strings::truncate(
                    &format!("{} videos", playlist.video_count),
                    list_w.saturating_sub(3) as usize,
                );
                ListItem::new(vec![
                    Line::from(vec![Span::styled(prefix, st), Span::styled(name, st)]),
                    Line::from(vec![Span::styled("   ", sub_st), Span::styled(sub, sub_st)]),
                ])
            })
            .collect();

        let list_area = Rect::new(list_x, area.y, list_w, area.height);
        List::new(items).render(list_area, buf);

        if self.youtube_playlists.len() > visible_n {
            self.render_scrollbar(list_area, self.youtube_playlists.len(), selected, buf);
        }
    }

    fn render_youtube_playlist_videos(
        &self,
        area: Rect,
        list_x: u16,
        list_w: u16,
        playlist: &YoutubePlaylist,
        buf: &mut Buffer,
    ) {
        let esc_hint = "[Esc]";
        let sep = " <- ";
        let reserved = (esc_hint.len() + sep.len() + 1) as u16;
        let title = strings::truncate(&playlist.title, list_w.saturating_sub(reserved) as usize);
        let video_count = if playlist.video_count > 0 {
            format!("  ({} videos)", playlist.video_count)
        } else {
            String::new()
        };
        let line = Line::from(vec![
            Span::styled(esc_hint, Style::default().fg(self.palette.muted)),
            Span::styled(sep, Style::default().fg(self.palette.dim)),
            Span::styled(
                title,
                Style::default()
                    .fg(self.palette.youtube)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(video_count, Style::default().fg(self.palette.muted)),
        ]);
        Paragraph::new(line).render(Rect::new(list_x, area.y, list_w, 1), buf);

        let inner = spotify_titled_track_list_layout(area);
        self.render_video_list(
            inner,
            list_x,
            list_w,
            buf,
            VideoListState {
                videos: self.youtube_playlist_videos,
                selected: self.youtube_playlist_videos_selected,
                scroll_offset: self.youtube_playlist_videos_scroll_offset,
                loading: self.youtube_playlist_videos_loading,
                empty_message: &t("modal.empty.no_results"),
            },
        );
    }

    fn render_video_list(
        &self,
        area: Rect,
        list_x: u16,
        list_w: u16,
        buf: &mut Buffer,
        state: VideoListState,
    ) {
        if state.loading {
            Paragraph::new(Span::styled(
                format!("{}  {}", spin_frame(), t("modal.loading.youtube")),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        if state.videos.is_empty() {
            Paragraph::new(Span::styled(
                state.empty_message,
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        const ITEM_HEIGHT: usize = 2;
        let visible_n = (area.height as usize) / ITEM_HEIGHT;
        let offset = scroll_offset_for_selection(state.selected, visible_n, state.scroll_offset);

        let items: Vec<ListItem> = state
            .videos
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, video)| {
                let active = i == state.selected;
                let duration = fmt_duration(video.duration_secs);
                let title_w = list_w.saturating_sub(4 + duration.len() as u16) as usize;
                let title = strings::truncate(&video.title, title_w);
                let channel = strings::truncate(&video.channel, list_w.saturating_sub(3) as usize);

                if active {
                    let title_st = Style::default()
                        .fg(self.palette.youtube)
                        .add_modifier(Modifier::BOLD);
                    let meta_st = Style::default().fg(self.palette.youtube);
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled("▶  ", title_st),
                            Span::styled(title, title_st),
                            Span::styled(format!("  {}", duration), meta_st),
                        ]),
                        Line::from(vec![
                            Span::styled("   ", meta_st),
                            Span::styled(channel, meta_st),
                        ]),
                    ])
                } else {
                    let title_st = Style::default().fg(self.palette.highlight);
                    let meta_st = Style::default().fg(self.palette.muted);
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled("   ", title_st),
                            Span::styled(title, title_st),
                        ]),
                        Line::from(vec![
                            Span::styled("   ", meta_st),
                            Span::styled(channel, meta_st),
                        ]),
                    ])
                }
            })
            .collect();

        let list_area = Rect::new(list_x, area.y, list_w, area.height);
        List::new(items).render(list_area, buf);

        if state.videos.len() > visible_n {
            self.render_scrollbar(list_area, state.videos.len(), state.selected, buf);
        }
    }

    fn render_youtube_auth_notice(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::{Block, BorderType, Borders, Clear};

        let box_w = area.width.saturating_sub(4).min(70);
        let box_h = 13.min(area.height);
        if box_w < 20 || box_h < 6 {
            return;
        }
        let box_x = area.x + (area.width - box_w) / 2;
        let box_y = area.y + (area.height.saturating_sub(box_h)) / 2;
        let box_area = Rect::new(box_x, box_y, box_w, box_h);

        Clear.render(box_area, buf);
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.palette.warning))
            .style(Style::default().bg(self.palette.panel_bg))
            .render(box_area, buf);

        let inner_x = box_x + 2;
        let inner_w = box_w.saturating_sub(4);
        let mut y = box_y + 1;

        Paragraph::new(Span::styled(
            t("modal.youtube.auth_notice.title"),
            Style::default()
                .fg(self.palette.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .render(Rect::new(inner_x, y, inner_w, 1), buf);
        y += 2;

        Paragraph::new(Span::styled(
            t("modal.youtube.auth_notice.body"),
            Style::default().fg(self.palette.highlight),
        ))
        .wrap(Wrap { trim: true })
        .render(Rect::new(inner_x, y, inner_w, 3), buf);
        y += 4;

        Paragraph::new(Span::styled(
            t("modal.youtube.auth_notice.risk"),
            Style::default().fg(self.palette.caution),
        ))
        .wrap(Wrap { trim: true })
        .render(Rect::new(inner_x, y, inner_w, 2), buf);
        y += 3;

        if y < box_y + box_h - 1 {
            Paragraph::new(Span::styled(
                t("modal.youtube.auth_notice.guide"),
                Style::default()
                    .fg(self.palette.accent)
                    .add_modifier(Modifier::BOLD),
            ))
            .wrap(Wrap { trim: true })
            .render(
                Rect::new(inner_x, y, inner_w, (box_y + box_h - 1).saturating_sub(y)),
                buf,
            );
        }
    }

    fn render_youtube_message(
        &self,
        area: Rect,
        text_x: u16,
        text_w: u16,
        message: &str,
        color: ratatui::style::Color,
        buf: &mut Buffer,
    ) {
        Paragraph::new(Span::styled(message, Style::default().fg(color)))
            .alignment(Alignment::Left)
            .render(Rect::new(text_x, area.y, text_w, 1), buf);
    }

    fn render_youtube_error(
        &self,
        area: Rect,
        text_x: u16,
        text_w: u16,
        message: &str,
        buf: &mut Buffer,
    ) {
        let mut y = area.y;
        let msg_len = message.chars().count() as u16;
        let msg_height = (msg_len / text_w.max(1)) + 1;
        let msg_height = msg_height.min(area.bottom().saturating_sub(y));

        Paragraph::new(Span::styled(
            message,
            Style::default().fg(self.palette.warning),
        ))
        .wrap(Wrap { trim: true })
        .render(Rect::new(text_x, y, text_w, msg_height), buf);

        y += msg_height + 1;
        if y < area.bottom() {
            Paragraph::new(Span::styled(
                t("modal.youtube.retry_hint"),
                Style::default().fg(self.palette.dim),
            ))
            .wrap(Wrap { trim: true })
            .render(
                Rect::new(text_x, y, text_w, area.bottom().saturating_sub(y)),
                buf,
            );
        }
    }
}
