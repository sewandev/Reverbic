use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::audio::{PlayerState, PlayerStatus};
use crate::ui::theme;

pub fn fmt_duration(secs: f32) -> String {
    let total = secs.max(0.0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

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

        let bitrate_tag = self
            .state
            .station
            .as_ref()
            .and_then(|s| s.bitrate_kbps)
            .map(|k| format!("{k}k"))
            .unwrap_or_default();

        let station_line = self.build_station_line(inner.width, &bitrate_tag);
        let middle_line = self.build_middle_line(inner.width);
        let title_line = self.build_title_line();

        Paragraph::new(vec![station_line, middle_line, title_line]).render(inner, buf);
    }
}

impl<'a> NowPlayingWidget<'a> {
    fn build_station_line(&self, width: u16, bitrate_tag: &str) -> Line<'a> {
        let name = self.state.station.as_ref().map(|s| s.name.as_str()).unwrap_or("…");

        // Right-align the bitrate tag on the same line as the station name
        let right_pad = if bitrate_tag.is_empty() {
            String::new()
        } else {
            let used = match &self.state.status {
                PlayerStatus::Idle => 0,
                PlayerStatus::Connecting => "Conectando a ".len() + name.len(),
                PlayerStatus::Playing => name.len(),
                PlayerStatus::Paused => "⏸ ".len() + name.len(),
                PlayerStatus::Buffering(_) => name.len(),
                PlayerStatus::Error(_) => 0,
                PlayerStatus::Reconnecting(n) => {
                    "Reconectando (".len() + n.to_string().len() + ")… ".len() + name.len()
                }
            };
            let tag_len = bitrate_tag.len();
            let pad = (width as usize).saturating_sub(used + tag_len);
            " ".repeat(pad)
        };

        match &self.state.status {
            PlayerStatus::Idle => Line::from(Span::styled(
                "(sin estación)",
                Style::default().fg(theme::MUTED),
            )),
            PlayerStatus::Connecting => Line::from(vec![
                Span::styled("Conectando a ", Style::default().fg(theme::ACCENT)),
                Span::styled(name.to_owned(), theme::PLAYING_STYLE),
            ]),
            PlayerStatus::Playing => Line::from(vec![
                Span::styled(name.to_owned(), theme::PLAYING_STYLE),
                Span::styled(right_pad, Style::default()),
                Span::styled(bitrate_tag.to_owned(), Style::default().fg(theme::MUTED)),
            ]),
            PlayerStatus::Paused => Line::from(vec![
                Span::styled("⏸ ", Style::default().fg(theme::ACCENT)),
                Span::styled(name.to_owned(), theme::SELECTED_STYLE),
                Span::styled(right_pad, Style::default()),
                Span::styled(bitrate_tag.to_owned(), Style::default().fg(theme::MUTED)),
            ]),
            PlayerStatus::Buffering(_) => Line::from(vec![
                Span::styled(
                    format!("{} ", crate::ui::widgets::spinner_frame()),
                    Style::default().fg(theme::ACCENT),
                ),
                Span::styled(name.to_owned(), theme::SELECTED_STYLE),
            ]),
            PlayerStatus::Error(msg) => Line::from(vec![
                Span::styled("Error: ", Style::default().fg(theme::ERROR)),
                Span::styled(msg.clone(), Style::default().fg(theme::ERROR)),
            ]),
            PlayerStatus::Reconnecting(attempt) => Line::from(vec![
                Span::styled("Reconectando (", Style::default().fg(theme::ACCENT)),
                Span::styled(attempt.to_string(), Style::default().fg(theme::HIGHLIGHT)),
                Span::styled(")… ", Style::default().fg(theme::ACCENT)),
                Span::styled(name.to_owned(), theme::SELECTED_STYLE),
            ]),
        }
    }

    fn build_middle_line(&self, width: u16) -> Line<'a> {
        // Si hay duración on-demand, muestra la barra de progreso
        if let Some(duration) = self.state.playback_duration_secs {
            let pos = self.state.playback_pos_secs.unwrap_or(0.0);
            let ratio = (pos / duration).clamp(0.0, 1.0);
            let time_str = format!(" {} / {} ", fmt_duration(pos), fmt_duration(duration));
            let bar_width = (width as usize).saturating_sub(time_str.len() + 2);
            let filled = (ratio * bar_width as f32) as usize;
            let empty  = bar_width.saturating_sub(filled);
            let bar = format!("[{}{}]{time_str}", "█".repeat(filled), "░".repeat(empty));
            return Line::from(Span::styled(bar, Style::default().fg(theme::ACCENT)));
        }
        if let Some(show) = &self.state.api_show {
            Line::from(vec![
                Span::styled("Show: ", Style::default().fg(theme::ACCENT)),
                Span::styled(show.clone(), Style::default().fg(theme::MUTED)),
            ])
        } else {
            Line::from(Span::styled(
                "═".repeat(width as usize),
                Style::default().fg(theme::MUTED),
            ))
        }
    }

    fn build_title_line(&self) -> Line<'a> {
        // Para on-demand (tiene duración pero no track API), muestra el tiempo solo
        if self.state.playback_duration_secs.is_some() {
            let pos      = self.state.playback_pos_secs.unwrap_or(0.0);
            let duration = self.state.playback_duration_secs.unwrap_or(1.0);
            let pct      = (pos / duration * 100.0).clamp(0.0, 100.0);
            return Line::from(Span::styled(
                format!("[0-9+Enter] Saltar a min  [[] -1min  []] +1min   {pct:.0}%"),
                Style::default().fg(theme::MUTED),
            ));
        }
        if let Some(title) = &self.state.title {
            Line::from(vec![
                Span::styled("Track: ", Style::default().fg(theme::ACCENT)),
                Span::styled(title.clone(), Style::default().fg(theme::HIGHLIGHT)),
            ])
        } else {
            Line::from(Span::styled("Track: —", Style::default().fg(theme::MUTED)))
        }
    }
}
