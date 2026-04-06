use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::ui::theme;

pub struct RecentTracksWidget<'a> {
    pub tracks: &'a [String],
    pub selected: usize,
    pub focused: bool,
    pub preview_active: bool,
    pub preview_loading_track: Option<&'a str>,
    pub preview_playing_track: Option<&'a str>,
    pub preview_unavailable: &'a HashSet<String>,
}

const NOW_PLAYING_STYLE: Style = Style::new().fg(theme::PLAYING).add_modifier(Modifier::BOLD);

const CURSOR_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(theme::ACCENT)
    .add_modifier(Modifier::BOLD);

const NORMAL_STYLE: Style = Style::new().fg(theme::MUTED);
const UNAVAILABLE_STYLE: Style = Style::new().fg(Color::Yellow);
const PREVIEW_PLAYING_STYLE: Style = Style::new().fg(theme::PLAYING).add_modifier(Modifier::BOLD);
const SPINNER_STYLE: Style = Style::new().fg(theme::FESTIVAL_ACCENT);

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const FRAME_MS: u128 = 120;

fn spinner_frame() -> &'static str {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let idx = ((ms / FRAME_MS) as usize) % SPINNER_FRAMES.len();
    SPINNER_FRAMES[idx]
}

impl<'a> Widget for RecentTracksWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::new().fg(theme::ACCENT)
        } else {
            theme::BORDER_STYLE
        };

        // Título responsivo según el ancho disponible
        let focused_title_long;
        let focused_title_short;
        let title = if self.focused {
            let preview_hint = if self.preview_active {
                "[p] Parar"
            } else {
                "[p] Preview"
            };
            if area.width >= 60 {
                focused_title_long = format!(
                    " RECENT  [↑↓] Nav  [Enter] Guardar  {preview_hint}  [Esc] Volver "
                );
                focused_title_long.as_str()
            } else {
                focused_title_short = format!(" RECENT  {preview_hint} ");
                focused_title_short.as_str()
            }
        } else if area.width >= 40 {
            " RECENT TRACKS  [Tab] "
        } else {
            " RECENT "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        let total = self.tracks.len();
        if total == 0 {
            return;
        }

        let height = inner.height as usize;
        let selected = self.selected.min(total.saturating_sub(1));

        // Scroll offset: mantiene el cursor siempre visible
        let offset = if selected >= height {
            selected + 1 - height
        } else {
            0
        };
        let slice_end = (offset + height).min(total);
        let spinner = spinner_frame();

        let items: Vec<ListItem> = self.tracks[offset..slice_end]
            .iter()
            .enumerate()
            .map(|(local_i, track)| {
                let abs_i = offset + local_i;
                let is_now_playing = abs_i == 0;
                let is_selected = abs_i == selected && self.focused;
                let is_loading = self.preview_loading_track == Some(track.as_str());
                let is_previewing = self.preview_playing_track == Some(track.as_str());
                let is_unavailable = self.preview_unavailable.contains(track);

                let row_style = if is_selected {
                    CURSOR_STYLE
                } else if is_now_playing {
                    NOW_PLAYING_STYLE
                } else {
                    NORMAL_STYLE
                };

                let prefix = format!("{:>2}. ", abs_i + 1);
                let mut spans = vec![
                    Span::styled(prefix, row_style),
                    Span::styled(track.as_str(), row_style),
                ];

                if is_loading {
                    let style = if is_selected {
                        Style::new()
                            .fg(Color::Black)
                            .bg(theme::ACCENT)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        SPINNER_STYLE
                    };
                    spans.push(Span::styled(format!("  {spinner}"), style));
                } else if is_previewing {
                    let style = if is_selected {
                        Style::new()
                            .fg(Color::Black)
                            .bg(theme::ACCENT)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        PREVIEW_PLAYING_STYLE
                    };
                    spans.push(Span::styled("  Playing preview", style));
                } else if is_unavailable {
                    let style = if is_selected {
                        Style::new().fg(Color::Black).bg(theme::ACCENT)
                    } else {
                        UNAVAILABLE_STYLE
                    };
                    spans.push(Span::styled("  Preview no disponible", style));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        List::new(items).render(inner, buf);
    }
}
