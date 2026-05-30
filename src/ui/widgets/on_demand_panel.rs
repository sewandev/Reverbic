use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::station::on_demand::OnDemandShow;
use crate::ui::theme;

pub struct OnDemandPanelWidget<'a> {
    pub shows:        &'a [OnDemandShow],
    pub selected:     usize,
    pub focused:      bool,
    pub loading:      bool,
    pub playing_id:   Option<&'a str>,
    pub program_name: &'a str,
}

const CURSOR_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(theme::ACCENT)
    .add_modifier(Modifier::BOLD);

const PLAYING_STYLE: Style = Style::new()
    .fg(theme::PLAYING)
    .add_modifier(Modifier::BOLD);

const NORMAL_STYLE: Style = Style::new().fg(theme::MUTED);


impl<'a> Widget for OnDemandPanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::new().fg(theme::ACCENT)
        } else {
            theme::BORDER_STYLE
        };

        let title_str: String;
        let title = if self.loading {
            title_str = format!(" {}  {} ", self.program_name, super::spinner_frame());
            title_str.as_str()
        } else if self.focused {
            title_str = format!(" {}  [↑↓] Nav  [p] Show  [Enter] Play  [Esc] Volver ", self.program_name);
            title_str.as_str()
        } else {
            title_str = format!(" {}  [Tab] ", self.program_name);
            title_str.as_str()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        if self.shows.is_empty() {
            return;
        }

        let total = self.shows.len();
        let selected = self.selected.min(total.saturating_sub(1));
        // Cada item ocupa 2 líneas (título + fecha)
        let height = (inner.height as usize).saturating_div(2).max(1);

        let offset = if selected >= height {
            selected + 1 - height
        } else {
            0
        };
        let slice_end = (offset + height).min(total);

        let items: Vec<ListItem> = self.shows[offset..slice_end]
            .iter()
            .enumerate()
            .map(|(local_i, show)| {
                let abs_i = offset + local_i;
                let is_selected = abs_i == selected && self.focused;
                let is_playing = self.playing_id == Some(show.id.as_str());

                let (prefix, style) = if is_selected {
                    (if is_playing { ">> " } else { "   " }, CURSOR_STYLE)
                } else if is_playing {
                    (">> ", PLAYING_STYLE)
                } else {
                    ("   ", NORMAL_STYLE)
                };

                let date_style = if is_selected {
                    style
                } else {
                    Style::new().fg(crate::ui::theme::MUTED)
                };
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(show.title.as_str(), style),
                    ]),
                    Line::from(vec![
                        Span::styled("   ", date_style),
                        Span::styled(show.date.as_str(), date_style),
                    ]),
                ])
            })
            .collect();

        List::new(items).render(inner, buf);
    }
}
