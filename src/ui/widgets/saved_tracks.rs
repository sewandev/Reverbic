use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

use crate::ui::theme;

pub struct SavedTracksWidget<'a> {
    pub tracks:       &'a [String],
    pub station_name: Option<&'a str>,
}

impl<'a> Widget for SavedTracksWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::new().fg(theme::ACCENT));
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height == 0 {
            return;
        }
        let count = self.tracks.len();
        let title = match (self.station_name, count) {
            (Some(name), 0) => format!("LIBRARY — {name}"),
            (Some(name), n) => format!("LIBRARY — {name} ({n})"),
            (None, 0)       => "LIBRARY".to_string(),
            (None, n)       => format!("LIBRARY ({n})"),
        };
        Paragraph::new(Line::from(Span::styled(
            title,
            Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
        )))
        .render(Rect::new(inner.x, inner.y, inner.width, 1), buf);

        if inner.height < 2 || count == 0 {
            return;
        }

        let list_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height - 1);
        let visible   = (list_area.height as usize).min(count);

        let items: Vec<ListItem> = self.tracks[..visible]
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let (num_style, title_style) = if i == 0 {
                    (
                        Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                        Style::new().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD),
                    )
                } else {
                    (Style::new().fg(theme::ACCENT), Style::new().fg(theme::MUTED))
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:>3}. ", i + 1), num_style),
                    Span::styled(t.as_str(), title_style),
                ]))
            })
            .collect();

        List::new(items).render(list_area, buf);
    }
}
