use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

use crate::ui::theme;

pub struct RecentTracksWidget<'a> {
    pub tracks:                &'a [String],
    pub selected:              usize,
    pub focused:               bool,
    pub preview_active:        bool,
    pub preview_loading_track: Option<&'a str>,
    pub preview_playing_track: Option<&'a str>,
    pub preview_unavailable:   &'a HashSet<String>,
}

fn tag_style(is_sel: bool, normal: Style) -> Style {
    if is_sel { Style::new().fg(Color::Black).bg(theme::ACCENT).add_modifier(Modifier::BOLD) } else { normal }
}

const NOW_PLAYING_STYLE:  Style = Style::new().fg(theme::PLAYING).add_modifier(Modifier::BOLD);
const CURSOR_STYLE:       Style = Style::new().fg(Color::Black).bg(theme::ACCENT).add_modifier(Modifier::BOLD);
const NORMAL_STYLE:       Style = Style::new().fg(theme::MUTED);
const UNAVAILABLE_STYLE:  Style = Style::new().fg(Color::Yellow);
const PREVIEW_PLAY_STYLE: Style = Style::new().fg(theme::PLAYING).add_modifier(Modifier::BOLD);
const SPINNER_STYLE:      Style = Style::new().fg(theme::FESTIVAL_ACCENT);

impl<'a> Widget for RecentTracksWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::new().fg(theme::ACCENT)
        } else {
            theme::BORDER_STYLE
        };

        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(border_style);
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 {
            return;
        }

        let preview_hint = if self.preview_active { "  [p] Parar" } else { "  [p] Preview" };
        let title_text = if self.focused {
            format!("RECENT{preview_hint}  [↵] Guardar  [Esc] Volver")
        } else {
            "RECENT  [Tab]".to_string()
        };
        let title_style = if self.focused {
            Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::DIM)
        };
        Paragraph::new(Line::from(Span::styled(title_text, title_style)))
            .render(Rect::new(inner.x, inner.y, inner.width, 1), buf);

        if inner.height < 2 || self.tracks.is_empty() {
            return;
        }

        let list_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height - 1);
        let total     = self.tracks.len();
        let height    = list_area.height as usize;
        let selected  = self.selected.min(total.saturating_sub(1));
        let offset    = super::scroll_offset(selected, height);
        let spinner   = super::spinner_frame();

        let items: Vec<ListItem> = self.tracks[offset..(offset + height).min(total)]
            .iter()
            .enumerate()
            .map(|(li, track)| {
                let abs_i        = offset + li;
                let is_playing   = abs_i == 0;
                let is_sel       = abs_i == selected && self.focused;
                let is_loading   = self.preview_loading_track == Some(track.as_str());
                let is_previewing = self.preview_playing_track == Some(track.as_str());
                let is_unavail   = self.preview_unavailable.contains(track);

                let row_style = if is_sel {
                    CURSOR_STYLE
                } else if is_playing {
                    NOW_PLAYING_STYLE
                } else {
                    NORMAL_STYLE
                };

                let mut spans = vec![
                    Span::styled(format!("{:>2}. ", abs_i + 1), row_style),
                    Span::styled(track.as_str(), row_style),
                ];

                if is_loading {
                    spans.push(Span::styled(format!("  {spinner}"), tag_style(is_sel, SPINNER_STYLE)));
                } else if is_previewing {
                    spans.push(Span::styled("  >> preview", tag_style(is_sel, PREVIEW_PLAY_STYLE)));
                } else if is_unavail {
                    spans.push(Span::styled("  no disponible", tag_style(is_sel, UNAVAILABLE_STYLE)));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        List::new(items).render(list_area, buf);
    }
}
