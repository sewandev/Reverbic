
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::audio::{PlayerState, PlayerStatus};
use crate::ui::theme;

pub struct NowPlayingWidget<'a> {
    pub state: &'a PlayerState,
}

impl<'a> Widget for NowPlayingWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" NOW PLAYING ")
            .borders(Borders::ALL)
            .border_style(theme::BORDER_STYLE);

        let inner = block.inner(area);
        block.render(area, buf);

        let station_line = match &self.state.status {
            PlayerStatus::Idle => Line::from(Span::styled("(sin estación)", Style::default().fg(theme::MUTED))),
            PlayerStatus::Connecting => Line::from(vec![
                Span::styled("Conectando a ", Style::default().fg(theme::ACCENT)),
                Span::styled(
                    self.state.station.as_ref().map(|s| s.name).unwrap_or("…"),
                    theme::PLAYING_STYLE,
                ),
            ]),
            PlayerStatus::Playing => Line::from(vec![
                Span::styled(
                    self.state.station.as_ref().map(|s| s.name).unwrap_or(""),
                    theme::PLAYING_STYLE,
                ),
            ]),
            PlayerStatus::Paused => Line::from(vec![
                Span::styled("⏸ ", Style::default().fg(theme::ACCENT)),
                Span::styled(
                    self.state.station.as_ref().map(|s| s.name).unwrap_or(""),
                    theme::SELECTED_STYLE,
                ),
            ]),
            PlayerStatus::Error(msg) => Line::from(vec![
                Span::styled("Error: ", Style::default().fg(theme::ERROR)),
                Span::styled(msg.as_str(), Style::default().fg(theme::ERROR)),
            ]),
        };

        // Si hay show de API, lo mostramos en lugar del separador
        let middle_line = if let Some(show) = &self.state.api_show {
            Line::from(vec![
                Span::styled("Show: ", Style::default().fg(theme::ACCENT)),
                Span::styled(show.as_str(), Style::default().fg(theme::MUTED)),
            ])
        } else {
            Line::from(Span::styled(
                "═".repeat(inner.width as usize),
                Style::default().fg(theme::MUTED),
            ))
        };

        let title_line = if let Some(title) = &self.state.title {
            Line::from(vec![
                Span::styled("Track: ", Style::default().fg(theme::ACCENT)),
                Span::styled(title.as_str(), Style::default().fg(theme::HIGHLIGHT)),
            ])
        } else {
            Line::from(Span::styled("Track: —", Style::default().fg(theme::MUTED)))
        };

        let text = vec![station_line, middle_line, title_line];
        Paragraph::new(text).render(inner, buf);
    }
}
