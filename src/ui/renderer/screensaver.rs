use chrono::{Local, Timelike};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::audio::{PlayerState, PlayerStatus};
use crate::i18n::t;
use crate::station::StationDetails;
use crate::ui::theme;

pub(super) fn render_screensaver(
    frame:       &mut Frame,
    area:        Rect,
    state:       &PlayerState,
    details:     Option<&StationDetails>,
    is_favorite: bool,
) {
    const OVERLAY: ratatui::style::Color = theme::OVERLAY_COLOR;
    const BG:      ratatui::style::Color = theme::PANEL_BG;

    frame.render_widget(Clear, area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            frame.buffer_mut()[(x, y)].set_bg(OVERLAY);
        }
    }

    let has_game    = crate::game_detect::get().is_some();
    let has_recent  = !state.recent_titles.is_empty();
    let n_prev      = state.recent_titles.len().saturating_sub(1).min(4) as u16;

    let (has_meta, has_tags, has_url) = details.map(|d| (
        !d.country.is_empty() || !d.language.is_empty() || !d.codec.is_empty(),
        !d.tags.is_empty(),
        !d.homepage.is_empty(),
    )).unwrap_or((false, false, false));

    let detail_rows: u16 = u16::from(has_meta) + u16::from(has_tags) + u16::from(has_url) + u16::from(has_game);
    let recent_rows: u16 = if has_recent { 2 + n_prev } else { 0 };
    let two_col_rows     = detail_rows.max(recent_rows);
    let has_two_col      = two_col_rows > 0;

    let ph = 2u16
        + 5 + 1
        + 1
        + 1
        + 1
        + 1
        + 1
        + if has_two_col { 1 + two_col_rows } else { 0 }
        + 1
        + 1;

    let pw    = area.width.clamp(50, 72);
    let px    = area.x + area.width.saturating_sub(pw) / 2;
    let py    = area.y + area.height.saturating_sub(ph) / 2;
    let panel = Rect::new(px, py, pw, ph.min(area.height));

    let border_color = match &state.status {
        PlayerStatus::Playing                             => theme::ACCENT,
        PlayerStatus::Paused                              => theme::WARNING,
        PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_) => ratatui::style::Color::Rgb(80, 80, 80),
        _                                                 => theme::MUTED,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx      = inner.x + 2;
    let cw      = inner.width.saturating_sub(4);
    let mut row = inner.y;


    let now_t    = Local::now();
    let h1       = (now_t.hour()   / 10) as u8;
    let h2       = (now_t.hour()   % 10) as u8;
    let m1       = (now_t.minute() / 10) as u8;
    let m2       = (now_t.minute() % 10) as u8;
    let colon_on = now_t.second().is_multiple_of(2);
    let clock_w: u16 = 19;
    let clock_x      = cx + cw.saturating_sub(clock_w) / 2;
    for r in 0..5usize {
        frame.render_widget(
            Paragraph::new(build_clock_row(r, h1, h2, m1, m2, colon_on, border_color, BG))
                .style(Style::default().bg(BG)),
            Rect::new(clock_x, row, clock_w, 1),
        );
        row += 1;
    }
    row += 1;

    let raw_name    = state.station.as_ref().map(|s| s.name.as_str()).unwrap_or("—");
    let prefix      = if is_favorite { "★  " } else { "" };
    let station_str = format!("{prefix}{}", raw_name.to_uppercase());
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            station_str,
            Style::default().fg(border_color).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .style(Style::default().bg(BG)),
        Rect::new(cx, row, cw, 1),
    );
    row += 1;

    let title = state.title.as_deref().unwrap_or("—");
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            title.to_owned(),
            Style::default().fg(theme::HIGHLIGHT),
        )))
        .alignment(Alignment::Center)
        .style(Style::default().bg(BG)),
        Rect::new(cx, row, cw, 1),
    );
    row += 1;

    row += 1;
    frame.render_widget(
        Paragraph::new(visualizer_line(state.level_db, cw as usize, BG))
            .style(Style::default().bg(BG)),
        Rect::new(cx, row, cw, 1),
    );
    row += 1;
    row += 1;

    if has_two_col {
        let sep = "─".repeat(cw as usize);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(sep, Style::default().fg(theme::DIM))))
                .style(Style::default().bg(BG)),
            Rect::new(cx, row, cw, 1),
        );
        row += 1;

        let left_w  = cw.saturating_sub(2) / 2;
        let right_w = cw.saturating_sub(left_w + 2);
        let right_x = cx + left_w + 2;
        let col_top = row;
        let mut left_row  = col_top;
        let mut right_row = col_top;

        if let Some(d) = details {
            if has_meta {
                let mut spans: Vec<Span<'static>> = Vec::new();
                for val in [&d.country, &d.language] {
                    if !val.is_empty() {
                        if !spans.is_empty() { spans.push(Span::styled("  ·  ", Style::default().fg(theme::MUTED))); }
                        spans.push(Span::styled(val.clone(), Style::default().fg(theme::DIM)));
                    }
                }
                if !d.codec.is_empty() {
                    if !spans.is_empty() { spans.push(Span::styled("  ·  ", Style::default().fg(theme::MUTED))); }
                    let s = if d.bitrate > 0 { format!("{}  {}k", d.codec, d.bitrate) } else { d.codec.clone() };
                    spans.push(Span::styled(s, Style::default().fg(theme::DIM)));
                }
                frame.render_widget(
                    Paragraph::new(Line::from(spans)).style(Style::default().bg(BG)),
                    Rect::new(cx, left_row, left_w, 1),
                );
                left_row += 1;
            }
            if has_tags && !d.tags.is_empty() {
                let raw = d.tags.join("  ·  ");
                let display = truncate_str(&raw, left_w as usize);
                frame.render_widget(
                    Paragraph::new(Span::styled(display, Style::default().fg(theme::MUTED)))
                        .style(Style::default().bg(BG)),
                    Rect::new(cx, left_row, left_w, 1),
                );
                left_row += 1;
            }
            if has_url && !d.homepage.is_empty() {
                let url = truncate_str(d.homepage.trim_end_matches('/'), left_w.saturating_sub(5) as usize);
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled("[o]  ", Style::default().fg(theme::ACCENT)),
                        Span::styled(url, Style::default().fg(theme::MUTED).add_modifier(Modifier::UNDERLINED)),
                    ])).style(Style::default().bg(BG)),
                    Rect::new(cx, left_row, left_w, 1),
                );
                left_row += 1;
            }
        }
        if let Some((ref name, ref genre)) = crate::game_detect::get() {
            let text = if genre.is_empty() { format!("  {name}") } else { format!("  {name}  ·  {genre}") };
            let display = truncate_str(&text, left_w as usize);
            frame.render_widget(
                Paragraph::new(Span::styled(display, Style::default().fg(theme::DIM)))
                    .style(Style::default().bg(BG)),
                Rect::new(cx, left_row, left_w, 1),
            );
            left_row += 1;
        }

        if has_recent {
            frame.render_widget(
                Paragraph::new(Span::styled(
                    t("screensaver.recent_tracks"),
                    Style::default().fg(theme::MUTED),
                )).style(Style::default().bg(BG)),
                Rect::new(right_x, right_row, right_w, 1),
            );
            right_row += 1;

            let now_live  = t("screensaver.now_live");
            let label_w   = now_live.chars().count() + 3;
            let max_cur   = right_w.saturating_sub(5 + label_w as u16) as usize;
            if let Some(current) = state.recent_titles.first() {
                let display = truncate_str(current, max_cur);
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled("▶  ", Style::default().fg(theme::ACCENT)),
                        Span::styled(display, Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("  {now_live}"), Style::default().fg(theme::ACCENT)),
                    ])).style(Style::default().bg(BG)),
                    Rect::new(right_x, right_row, right_w, 1),
                );
                right_row += 1;
            }

            let max_prev = right_w.saturating_sub(4) as usize;
            for track in state.recent_titles.iter().skip(1).take(4) {
                let display = truncate_str(track, max_prev);
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled("↳  ", Style::default().fg(theme::DIM)),
                        Span::styled(display, Style::default().fg(theme::HIGHLIGHT)),
                    ])).style(Style::default().bg(BG)),
                    Rect::new(right_x, right_row, right_w, 1),
                );
                right_row += 1;
            }
        }

        row = left_row.max(right_row);
    }

    row += 1;
    let vol_pct   = (state.volume.clamp(0.0, 1.0) * 100.0).round() as u32;
    let vol_color = if state.volume > 0.85 { theme::WARNING } else { theme::ACCENT };
    let vol_bar   = volume_bar(state.volume, 10);
    let time_str  = now_t.format("%H:%M").to_string();
    let right_str = format!("{}  {}  {:>3}%", time_str, vol_bar, vol_pct);

    frame.render_widget(
        Paragraph::new(Span::styled(t("screensaver.prompt"), Style::default().fg(theme::MUTED)))
            .style(Style::default().bg(BG)),
        Rect::new(cx, row, cw, 1),
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(right_str, Style::default().fg(vol_color))))
            .alignment(Alignment::Right)
            .style(Style::default().bg(BG)),
        Rect::new(cx, row, cw, 1),
    );
}

fn big_digit_rows(d: u8) -> [&'static str; 5] {
    match d {
        0 => ["███", "█ █", "█ █", "█ █", "███"],
        1 => [" █ ", "██ ", " █ ", " █ ", "███"],
        2 => ["███", "  █", "███", "█  ", "███"],
        3 => ["███", "  █", " ██", "  █", "███"],
        4 => ["█ █", "█ █", "███", "  █", "  █"],
        5 => ["███", "█  ", "███", "  █", "███"],
        6 => ["███", "█  ", "███", "█ █", "███"],
        7 => ["███", "  █", "  █", "  █", "  █"],
        8 => ["███", "█ █", "███", "█ █", "███"],
        9 => ["███", "█ █", "███", "  █", "███"],
        _ => ["   ", "   ", "   ", "   ", "   "],
    }
}

#[expect(clippy::too_many_arguments)]
fn build_clock_row(
    row:      usize,
    h1: u8, h2: u8,
    m1: u8, m2: u8,
    colon_on: bool,
    color:    ratatui::style::Color,
    bg:       ratatui::style::Color,
) -> Line<'static> {
    let colon_ch = if colon_on { match row { 1 | 3 => "█", _ => " " } } else { " " };
    let s = format!(
        "{} {}  {}  {} {}",
        big_digit_rows(h1)[row],
        big_digit_rows(h2)[row],
        colon_ch,
        big_digit_rows(m1)[row],
        big_digit_rows(m2)[row],
    );
    Line::from(Span::styled(s, Style::default().fg(color).bg(bg)))
}

fn visualizer_line(level_db: f32, width: usize, bg: ratatui::style::Color) -> Line<'static> {
    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let base = ((level_db + 60.0) / 60.0).clamp(0.0, 1.0) as f64;
    let ms   = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0) as f64;

    let n_bars = (width / 2).max(1);
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(n_bars * 2);
    for i in 0..n_bars {
        let freq  = 0.0025 + (i as f64) * 0.00025;
        let phase = i as f64 * 1.1;
        let wave  = (ms * freq + phase).sin() * 0.35 + 0.35;
        let h     = (base * 0.65 + wave * 0.35).clamp(0.0, 1.0);
        let idx   = ((h * 7.0) as usize).min(7);
        let color = if h > 0.85 {
            ratatui::style::Color::Red
        } else if h > 0.6 {
            theme::WARNING
        } else if h > 0.3 {
            theme::ACCENT
        } else {
            theme::MUTED
        };
        spans.push(Span::styled(BLOCKS[idx].to_string(), Style::default().fg(color).bg(bg)));
        if i + 1 < n_bars {
            spans.push(Span::styled(" ", Style::default().bg(bg)));
        }
    }
    Line::from(spans)
}

fn volume_bar(vol: f32, bar_width: usize) -> String {
    let filled = (vol.clamp(0.0, 1.0) * bar_width as f32).round() as usize;
    format!("{}{}", "█".repeat(filled), "░".repeat(bar_width - filled))
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        format!("{}…", s.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}
