use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::ui::theme;

pub struct SavedTracksWidget<'a> {
    pub tracks: &'a [String],
    pub station_name: Option<&'a str>,
}

impl<'a> Widget for SavedTracksWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let count = self.tracks.len();
        let title = match (self.station_name, count) {
            (Some(name), 0) => format!(" LIBRARY — {name} "),
            (Some(name), n) => format!(" LIBRARY — {name} ({n}) "),
            (None, 0) => " LIBRARY ".to_string(),
            (None, n) => format!(" LIBRARY ({n}) "),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme::ACCENT));

        let inner = block.inner(area);
        block.render(area, buf);

        if count == 0 {
            return;
        }
        let visible = (inner.height as usize).min(count);

        let items: Vec<ListItem> = self.tracks[..visible]
            .iter()
            .enumerate()
            .map(|(i, title)| {
                let (num_style, title_style) = if i == 0 {
                    (
                        Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                        Style::new()
                            .fg(theme::HIGHLIGHT)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    (
                        Style::new().fg(theme::ACCENT),
                        Style::new().fg(theme::MUTED),
                    )
                };

                let line = Line::from(vec![
                    Span::styled(format!("{:>3}. ", i + 1), num_style),
                    Span::styled(title.as_str(), title_style),
                ]);
                ListItem::new(line)
            })
            .collect();

        List::new(items).render(inner, buf);
    }
}
