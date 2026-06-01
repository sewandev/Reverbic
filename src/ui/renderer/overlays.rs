use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::audio::{PlayerState, PlayerStatus};
use crate::i18n::t;
use crate::ui::theme;

pub(super) fn render_rename_overlay(frame: &mut Frame, input: &str) {
    let area  = frame.area();
    let w     = area.width.min(50).max(30);
    let h: u16 = 5;
    let x     = area.width.saturating_sub(w) / 2;
    let y     = area.height.saturating_sub(h) / 2;
    let panel = ratatui::layout::Rect::new(x, y, w, h);

    frame.render_widget(Clear, panel);

    let block = Block::default()
        .title_top(Line::from(Span::styled(
            t("modal.rename.title"),
            Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD),
        )).alignment(ratatui::layout::Alignment::Center))
        .title_bottom(Line::from(Span::styled(
            t("modal.rename.hint"),
            Style::default().fg(theme::MUTED),
        )).alignment(ratatui::layout::Alignment::Center))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::ACCENT))
        .style(Style::default().bg(theme::PANEL_BG));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let text_area = ratatui::layout::Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(input.to_owned(), Style::default().fg(theme::HIGHLIGHT)),
            Span::styled("_", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ])),
        text_area,
    );
}

pub(super) fn render_game_inline(frame: &mut Frame, area: Rect, name: &str, genre: &str) {
    let label = t("overlay.playing_game");
    let mut spans = vec![
        Span::styled(format!("  {label}  "), Style::default().fg(theme::MUTED)),
        Span::styled(name.to_owned(), theme::PLAYING_STYLE),
    ];
    if !genre.is_empty() {
        spans.push(Span::styled("  ·  ", Style::default().fg(theme::MUTED)));
        spans.push(Span::styled(genre.to_owned(), Style::default().fg(theme::DIM)));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub(super) fn render_game_strip(frame: &mut Frame, area: Rect, name: &str, genre: &str) {
    use ratatui::style::Color;
    const BG: Color = Color::Rgb(13, 13, 13);
    const H_PAD: u16 = 2;

    let block = Block::default()
        .title_top(
            Line::from(Span::styled(
                format!(" {} ", t("overlay.playing_game")),
                Style::default().fg(theme::MUTED),
            ))
            .alignment(Alignment::Left),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::MUTED))
        .style(Style::default().bg(BG));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cx = inner.x + H_PAD;
    let cw = inner.width.saturating_sub(H_PAD * 2);
    let mut spans: Vec<Span<'static>> = vec![
        Span::styled(name.to_owned(), theme::PLAYING_STYLE),
    ];
    if !genre.is_empty() {
        spans.push(Span::styled("  ·  ", Style::default().fg(theme::MUTED)));
        spans.push(Span::styled(genre.to_owned(), Style::default().fg(theme::DIM)));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(BG)),
        Rect::new(cx, inner.y, cw, 1),
    );
}

pub(super) fn render_modal_np_strip(frame: &mut Frame, strip: Rect, state: &PlayerState) {
    use ratatui::style::Color;
    const STRIP_BG: Color = Color::Rgb(13, 13, 13);
    const H_PAD:    u16   = 2;

    if matches!(state.status, PlayerStatus::Idle | PlayerStatus::Error(_)) {
        return;
    }

    let content_w = strip.width.saturating_sub(2 + H_PAD * 2) as usize;
    if content_w == 0 { return; }

    let raw_title = match &state.status {
        PlayerStatus::Playing | PlayerStatus::Paused | PlayerStatus::Buffering(_) => {
            state.title.clone().unwrap_or_default()
        }
        _ => String::new(),
    };
    let title_lines = wrap_into_lines(&raw_title, content_w, 2);

    let panel_h = 2 + 1 + title_lines.len() as u16;
    if panel_h > strip.height { return; }

    let panel = Rect::new(strip.x, strip.y, strip.width, panel_h);

    let vol_pct   = (state.volume.clamp(0.0, 1.0) * 100.0).round() as u32;
    let vol_color = if state.volume > 0.85 { theme::WARNING } else { theme::ACCENT };

    let block = Block::default()
        .title_top(
            Line::from(Span::styled(
                format!(" {vol_pct:>3}% "),
                Style::default().fg(vol_color),
            ))
            .alignment(Alignment::Right),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::MUTED))
        .style(Style::default().bg(STRIP_BG));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx = inner.x + H_PAD;
    let cw = inner.width.saturating_sub(H_PAD * 2);
    let station_line = build_modal_station_line(state);
    frame.render_widget(
        Paragraph::new(station_line).style(Style::default().bg(STRIP_BG)),
        Rect::new(cx, inner.y, cw, 1),
    );
    for (i, tline) in title_lines.into_iter().enumerate() {
        let row_y = inner.y + 1 + i as u16;
        if row_y < inner.bottom() {
            frame.render_widget(
                Paragraph::new(tline).style(Style::default().bg(STRIP_BG)),
                Rect::new(cx, row_y, cw, 1),
            );
        }
    }
}

pub(super) fn build_modal_station_line(state: &PlayerState) -> Line<'static> {
    let name = state.station.as_ref().map(|s| s.name.clone()).unwrap_or_default();
    match &state.status {
        PlayerStatus::Connecting | PlayerStatus::Reconnecting(_) => Line::from(vec![
            Span::styled("…  ", Style::default().fg(theme::ACCENT)),
            Span::styled(name, Style::default().fg(theme::MUTED)),
        ]),
        PlayerStatus::Buffering(_) | PlayerStatus::Playing | PlayerStatus::Paused => {
            let icon = if matches!(state.status, PlayerStatus::Paused) { "⏸  " } else { ">>  " };
            Line::from(vec![
                Span::styled(icon, Style::default().fg(theme::ACCENT)),
                Span::styled(name, theme::PLAYING_STYLE),
            ])
        }
        _ => Line::default(),
    }
}

pub(super) fn wrap_into_lines(text: &str, width: usize, max_lines: usize) -> Vec<Line<'static>> {
    if text.is_empty() || width == 0 { return vec![]; }
    let chars: Vec<char> = text.chars().collect();
    let mut lines = Vec::new();
    let mut offset = 0;
    while offset < chars.len() && lines.len() < max_lines {
        let end   = (offset + width).min(chars.len());
        let slice: String = chars[offset..end].iter().collect();
        lines.push(Line::from(Span::styled(slice, Style::default().fg(theme::HIGHLIGHT))));
        offset = end;
    }
    lines
}
