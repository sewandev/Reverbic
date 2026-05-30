use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::ui::theme;

pub struct NowPlayingOverlayWidget<'a> {
    pub station_name: &'a str,
    pub track_title:  Option<&'a str>,
}

impl Widget for NowPlayingOverlayWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" ♪ NOW PLAYING ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::PLAYING));

        let inner = block.inner(area);
        block.render(area, buf);

        let w = inner.width as usize;
        let lines = vec![
            Line::from(Span::styled(truncated(self.station_name, w), theme::PLAYING_STYLE)),
            Line::from(Span::styled(
                self.track_title
                    .map(|t| truncated(t, w))
                    .unwrap_or_else(|| "—".to_owned()),
                Style::default().fg(theme::HIGHLIGHT),
            )),
        ];

        Paragraph::new(lines).render(inner, buf);
    }
}

fn truncated(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_owned();
    }
    let head: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{head}…")
}
