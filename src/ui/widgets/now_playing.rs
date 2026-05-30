use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::audio::{PlayerState, PlayerStatus};
use crate::ui::theme;

pub fn fmt_duration(secs: f32) -> String {
    let total = secs.max(0.0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 { format!("{h}:{m:02}:{s:02}") } else { format!("{m}:{s:02}") }
}

pub struct NowPlayingWidget<'a> {
    pub state: &'a PlayerState,
}

impl<'a> Widget for NowPlayingWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let line = build_line(self.state, area.width);
        Paragraph::new(line).render(area, buf);
    }
}

fn build_line(state: &PlayerState, width: u16) -> Line<'static> {
    // On-demand: barra de progreso clickeable ([ ] al inicio para on_click)
    if let Some(duration) = state.playback_duration_secs {
        let pos   = state.playback_pos_secs.unwrap_or(0.0);
        let ratio = (pos / duration).clamp(0.0, 1.0);
        let pct   = (ratio * 100.0) as u8;
        let time_str = format!(" {} / {} ", fmt_duration(pos), fmt_duration(duration));
        // El bar empieza con un espacio + "[" → inner_x = area.x + 1 en on_click
        let bar_width = (width as usize).saturating_sub(time_str.len() + 2 + 4 + 1);
        let filled = (ratio * bar_width as f32) as usize;
        let empty  = bar_width.saturating_sub(filled);
        let bar = format!(
            " [{}{}]{time_str}{pct:>3}%",
            "█".repeat(filled),
            "░".repeat(empty),
        );
        return Line::from(Span::styled(bar, Style::default().fg(theme::ACCENT)));
    }

    let station = state.station.as_ref().map(|s| s.name.clone()).unwrap_or_default();

    match &state.status {
        PlayerStatus::Idle => Line::from(Span::styled(
            " ♪  —",
            Style::default().fg(theme::MUTED),
        )),

        PlayerStatus::Connecting => Line::from(vec![
            Span::styled(" ♪  Conectando… ", Style::default().fg(theme::ACCENT)),
            Span::styled(station, Style::default().fg(theme::MUTED)),
        ]),

        PlayerStatus::Buffering(_) => Line::from(vec![
            Span::styled(
                format!(" {}  ", super::spinner_frame()),
                Style::default().fg(theme::ACCENT),
            ),
            Span::styled(station, theme::PLAYING_STYLE),
        ]),

        PlayerStatus::Reconnecting(n) => Line::from(vec![
            Span::styled(format!(" ↻ ×{n}  "), Style::default().fg(theme::WARNING)),
            Span::styled(station, theme::PLAYING_STYLE),
        ]),

        PlayerStatus::Playing | PlayerStatus::Paused => {
            let icon = if matches!(state.status, PlayerStatus::Paused) { " ⏸  " } else { " ♪  " };
            let show  = state.api_show.clone().unwrap_or_default();
            let title = state.title.clone().unwrap_or_else(|| "—".to_owned());

            let mut spans: Vec<Span<'static>> = vec![
                Span::styled(icon, Style::default().fg(theme::ACCENT)),
                Span::styled(station, theme::PLAYING_STYLE),
            ];
            if !show.is_empty() {
                spans.push(Span::styled("  ·  ", Style::default().fg(theme::MUTED)));
                spans.push(Span::styled(show, Style::default().fg(theme::DIM)));
            }
            spans.push(Span::styled("  ·  ", Style::default().fg(theme::MUTED)));
            spans.push(Span::styled(title, Style::default().fg(theme::HIGHLIGHT)));
            Line::from(spans)
        }

        PlayerStatus::Error(msg) => Line::from(vec![
            Span::styled(" ✕  ", Style::default().fg(theme::DANGER)),
            Span::styled(msg.clone(), Style::default().fg(theme::DANGER)),
        ]),
    }
}
