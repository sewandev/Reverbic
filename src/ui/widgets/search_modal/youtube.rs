use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget, Wrap},
};

use crate::app::YoutubeStatus;
use crate::i18n::t;
use crate::ui::strings;

use super::helpers::{render_filter_input, spin_frame};
use super::SearchModalWidget;

fn fmt_duration(secs: u32) -> String {
    if secs == 0 {
        "--:--".to_string()
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

impl<'a> SearchModalWidget<'a> {
    pub(super) fn render_youtube_body(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        let [_gap, input_row, cap_row, list_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        let text_x = content_x + 2;
        let text_w = content_w.saturating_sub(2);

        buf[(content_x, input_row.y)]
            .set_symbol("┃")
            .set_fg(self.palette.accent)
            .set_bg(self.palette.panel_bg);

        render_filter_input(
            self.youtube_query,
            &t("modal.youtube.placeholder"),
            Rect::new(text_x, input_row.y, text_w, 1),
            self.palette,
            buf,
            self.palette.danger,
        );

        buf[(content_x, cap_row.y)]
            .set_symbol("╹")
            .set_fg(self.palette.accent)
            .set_bg(self.palette.panel_bg);

        match self.youtube_status {
            YoutubeStatus::Installing => {
                self.render_youtube_message(
                    list_area,
                    text_x,
                    text_w,
                    &format!("{}  {}", spin_frame(), t("modal.youtube.installing")),
                    self.palette.muted,
                    buf,
                );
            }
            YoutubeStatus::Resolving => {
                self.render_youtube_message(
                    list_area,
                    text_x,
                    text_w,
                    &format!("{}  {}", spin_frame(), t("modal.youtube.resolving")),
                    self.palette.muted,
                    buf,
                );
            }
            YoutubeStatus::Error(msg) => {
                self.render_youtube_error(list_area, text_x, text_w, msg, buf);
            }
            YoutubeStatus::Idle | YoutubeStatus::Ready => {
                self.render_youtube_results(list_area, text_x, text_w, buf);
            }
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

    fn render_youtube_results(&self, area: Rect, list_x: u16, list_w: u16, buf: &mut Buffer) {
        if self.youtube_loading {
            Paragraph::new(Span::styled(
                format!("{}  {}", spin_frame(), t("modal.loading.youtube")),
                Style::default().fg(self.palette.muted),
            ))
            .render(Rect::new(list_x, area.y, list_w, 1), buf);
            return;
        }

        if self.youtube_results.is_empty() {
            if !self.youtube_query.is_empty() {
                Paragraph::new(Span::styled(
                    t("modal.youtube.no_results"),
                    Style::default().fg(self.palette.muted),
                ))
                .render(Rect::new(list_x, area.y, list_w, 1), buf);
            }
            return;
        }

        const ITEM_HEIGHT: usize = 2;
        let visible_n = (area.height as usize) / ITEM_HEIGHT;
        let offset = super::super::scroll_offset(self.youtube_selected, visible_n);

        let items: Vec<ListItem> = self
            .youtube_results
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_n)
            .map(|(i, video)| {
                let active = i == self.youtube_selected;
                let duration = fmt_duration(video.duration_secs);
                let title_w = list_w.saturating_sub(4 + duration.len() as u16) as usize;
                let title = strings::truncate(&video.title, title_w);
                let channel = strings::truncate(&video.channel, list_w.saturating_sub(3) as usize);

                if active {
                    let title_st = Style::default()
                        .fg(self.palette.playing)
                        .add_modifier(Modifier::BOLD);
                    let meta_st = Style::default().fg(self.palette.accent);
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

        if self.youtube_results.len() > visible_n {
            self.render_scrollbar(
                list_area,
                self.youtube_results.len(),
                self.youtube_selected,
                buf,
            );
        }
    }
}
