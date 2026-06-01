use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
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

impl<'a> Widget for OnDemandPanelWidget<'a> {
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
        let title_hint = if self.focused { "  [p] cambiar" } else { "  [Tab]" };
        let prog_style = if self.focused {
            Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme::DIM)
        };
        let title_line = if self.loading {
            Line::from(vec![
                Span::styled(self.program_name, prog_style),
                Span::styled(
                    format!("  {}", super::spinner_frame()),
                    Style::default().fg(theme::ACCENT),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(self.program_name, prog_style),
                Span::styled(title_hint, Style::default().fg(theme::MUTED)),
            ])
        };
        Paragraph::new(title_line).render(
            Rect::new(inner.x, inner.y, inner.width, 1),
            buf,
        );

        if inner.height < 2 || self.shows.is_empty() {
            return;
        }

        let list_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height - 1);
        let total    = self.shows.len();
        let selected = self.selected.min(total.saturating_sub(1));
        let height  = (list_area.height as usize).saturating_div(2).max(1);
        let offset  = if selected >= height { selected + 1 - height } else { 0 };
        let end     = (offset + height).min(total);

        let items: Vec<ListItem> = self.shows[offset..end]
            .iter()
            .enumerate()
            .map(|(li, show)| {
                let abs_i      = offset + li;
                let is_sel     = abs_i == selected && self.focused;
                let is_playing = self.playing_id == Some(show.id.as_str());

                let (prefix, style) = if is_sel {
                    (if is_playing { ">> " } else { "   " }, theme::CURSOR_STYLE)
                } else if is_playing {
                    (">> ", theme::PLAYING_STYLE)
                } else {
                    ("   ", theme::NORMAL_STYLE)
                };

                let date_style = if is_sel { style } else { Style::new().fg(theme::MUTED) };
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

        List::new(items).render(list_area, buf);
    }
}
