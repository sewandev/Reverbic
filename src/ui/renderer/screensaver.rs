use chrono::{Local, Timelike};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::audio::{PlayerState, PlayerStatus};
use crate::i18n::t;
use crate::station::StationDetails;
use crate::ui::strings;
use crate::ui::theme::{self, Palette};

pub(super) struct ScreensaverCtx<'a> {
    pub palette: &'a Palette,
    pub state: &'a PlayerState,
    pub details: Option<&'a StationDetails>,
    pub is_favorite: bool,
    pub spotify_name: Option<&'a str>,
    pub spotify_premium: Option<bool>,
    pub enriched_track: Option<&'a crate::metadata::EnrichedTrack>,
    pub show_clock: bool,
    pub border_tick: u32,
}

pub(super) fn render_screensaver(frame: &mut Frame, area: Rect, ctx: ScreensaverCtx<'_>) {
    let ScreensaverCtx {
        palette,
        state,
        details,
        is_favorite,
        spotify_name,
        spotify_premium,
        enriched_track,
        show_clock,
        border_tick,
    } = ctx;
    let overlay = palette.overlay_color;
    let bg = palette.panel_bg;

    frame.render_widget(Clear, area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            frame.buffer_mut()[(x, y)].set_bg(overlay);
        }
    }

    let has_recent = !state.recent_titles.is_empty();
    let n_recent = state.recent_titles.len().min(5) as u16;

    let (has_meta, has_tags, has_url) = details
        .map(|d| {
            (
                !d.country.is_empty() || !d.language.is_empty() || !d.codec.is_empty(),
                !d.tags.is_empty(),
                !d.homepage.is_empty(),
            )
        })
        .unwrap_or((false, false, false));

    let detail_rows: u16 = u16::from(has_meta) + u16::from(has_tags) + u16::from(has_url);
    let has_details = detail_rows > 0;
    let pw = (area.width * 85 / 100).clamp(74, 120).min(area.width);
    let cw_est = pw.saturating_sub(6);
    let title = state.title.as_deref().unwrap_or("—");
    let title_rows: u16 = if title.chars().count() > cw_est as usize {
        2
    } else {
        1
    };

    let title_block_h: u16 = if enriched_track.is_some() {
        3
    } else {
        title_rows
    };
    let has_playback_progress = state.playback_pos_secs.is_some();
    let recent_rows: u16 = if has_recent { 1 + n_recent } else { 0 };

    let clock_h: u16 = if show_clock { 5 + 1 } else { 0 };
    let ph = 2u16                                           // borders
        + 1                                                 // top margin
        + clock_h                                           // clock + gap
        + 1                                                 // station name
        + title_block_h                                     // song title (or artist+title+album)
        + u16::from(spotify_name.is_some())                 // spotify
        + u16::from(has_playback_progress)                  // progreso on-demand
        + 1                                                 // gap
        + 1                                                 // visualizer
        + 1                                                 // gap
        + if has_details { 1 + detail_rows } else { 0 }    // separator + details
        + if has_recent  { 1 + recent_rows } else { 0 }    // separator + recent
        + 1                                                 // gap
        + 1; // bottom bar

    let px = area.x + area.width.saturating_sub(pw) / 2;
    let py = area.y + area.height.saturating_sub(ph) / 2;
    let panel = Rect::new(px, py, pw, ph.min(area.height));

    if py >= 3 {
        render_logo_above(
            frame,
            area.x,
            area.width,
            py - 1,
            overlay,
            border_tick,
            palette,
        );
    }

    if let Some((ref name, ref genre)) = crate::game_detect::get() {
        if py >= 3 {
            let panel_h: u16 = 3;
            super::overlays::render_game_strip(
                frame,
                Rect::new(px, py.saturating_sub(panel_h), pw, panel_h),
                name,
                genre,
                border_tick,
                palette,
            );
        }
    }

    let border_color = match &state.status {
        PlayerStatus::Playing => theme::border_color_for(palette, border_tick),
        PlayerStatus::Paused => palette.warning,
        PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_) => palette.buffering,
        _ => palette.muted,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx = inner.x + 2;
    let cw = inner.width.saturating_sub(4);
    let mut row = inner.y + 1;

    let now_t = Local::now();
    if show_clock {
        let h1 = (now_t.hour() / 10) as u8;
        let h2 = (now_t.hour() % 10) as u8;
        let m1 = (now_t.minute() / 10) as u8;
        let m2 = (now_t.minute() % 10) as u8;
        let colon_on = now_t.second().is_multiple_of(2);
        let clock_w: u16 = 19;
        let clock_x = cx + cw.saturating_sub(clock_w) / 2;
        for r in 0..5usize {
            frame.render_widget(
                Paragraph::new(build_clock_row(
                    r,
                    h1,
                    h2,
                    m1,
                    m2,
                    colon_on,
                    border_color,
                    bg,
                ))
                .style(Style::default().bg(bg)),
                Rect::new(clock_x, row, clock_w, 1).intersection(inner),
            );
            row += 1;
        }
        row += 1;
    }
    let raw_name = state
        .station
        .as_ref()
        .map(|s| s.name.as_str())
        .unwrap_or("—");
    let prefix = if is_favorite { "★  " } else { "" };
    let station_str =
        strings::truncate(&format!("{prefix}{}", raw_name.to_uppercase()), cw as usize);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            station_str,
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .style(Style::default().bg(bg)),
        Rect::new(cx, row, cw, 1).intersection(inner),
    );
    row += 1;
    if let Some(et) = enriched_track {
        frame.render_widget(
            Paragraph::new(Span::styled(
                strings::truncate(&et.artist, cw as usize),
                Style::default().fg(palette.muted),
            ))
            .alignment(Alignment::Center)
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        row += 1;
        frame.render_widget(
            Paragraph::new(Span::styled(
                strings::truncate(&et.title, cw as usize),
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center)
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        row += 1;
        let duration_str = if et.duration_secs > 0 {
            format!("{}:{:02}", et.duration_secs / 60, et.duration_secs % 60)
        } else {
            String::new()
        };
        let album_line = match (et.year, duration_str.is_empty()) {
            (Some(y), false) => format!("{}  ·  {}  ·  {}", et.album, y, duration_str),
            (Some(y), true) => format!("{}  ·  {}", et.album, y),
            (None, false) => format!("{}  ·  {}", et.album, duration_str),
            (None, true) => et.album.clone(),
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                strings::truncate(&album_line, cw as usize),
                Style::default().fg(palette.dim),
            ))
            .alignment(Alignment::Center)
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        row += 1;
    } else {
        frame.render_widget(
            Paragraph::new(Span::styled(
                title.to_owned(),
                Style::default().fg(palette.highlight),
            ))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, title_rows).intersection(inner),
        );
        row += title_rows;
    }
    if let Some(name) = spotify_name {
        let name_tc = strings::title_case(name);
        let label = if spotify_premium.is_some_and(|is_premium| is_premium) {
            format!("★  {}  ·  {}", name_tc, t("integrations.spotify.premium"))
        } else {
            name_tc
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                label,
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center)
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        row += 1;
    }
    if has_playback_progress {
        if let Some(progress_line) =
            super::overlays::playback_progress_line(state, cw, border_color, bg, palette)
        {
            frame.render_widget(
                Paragraph::new(progress_line).style(Style::default().bg(bg)),
                Rect::new(cx, row, cw, 1).intersection(inner),
            );
            row += 1;
        }
    }
    row += 1;
    frame.render_widget(
        Paragraph::new(visualizer_line(state.level_db, cw as usize, bg, palette))
            .style(Style::default().bg(bg)),
        Rect::new(cx, row, cw, 1).intersection(inner),
    );
    row += 1;
    row += 1;
    if has_details {
        let sep = "─".repeat(cw as usize);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                sep,
                Style::default().fg(palette.dim),
            )))
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        row += 1;

        if let Some(d) = details {
            if has_meta {
                let mut spans: Vec<Span<'static>> = Vec::new();
                for val in [&d.country, &d.language] {
                    if !val.is_empty() {
                        if !spans.is_empty() {
                            spans.push(Span::styled("  ·  ", Style::default().fg(palette.muted)));
                        }
                        spans.push(Span::styled(
                            strings::title_case(val),
                            Style::default().fg(palette.dim),
                        ));
                    }
                }
                if !d.codec.is_empty() {
                    if !spans.is_empty() {
                        spans.push(Span::styled("  ·  ", Style::default().fg(palette.muted)));
                    }
                    let s = if d.bitrate > 0 {
                        format!("{}  {}k", d.codec.to_uppercase(), d.bitrate)
                    } else {
                        d.codec.to_uppercase()
                    };
                    spans.push(Span::styled(s, Style::default().fg(palette.dim)));
                }
                frame.render_widget(
                    Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
                    Rect::new(cx, row, cw, 1).intersection(inner),
                );
                row += 1;
            }
            if has_tags && !d.tags.is_empty() {
                let raw = d
                    .tags
                    .iter()
                    .map(|t| strings::title_case(t))
                    .collect::<Vec<_>>()
                    .join("  ·  ");
                let display = strings::truncate(&raw, cw as usize);
                frame.render_widget(
                    Paragraph::new(Span::styled(display, Style::default().fg(palette.muted)))
                        .style(Style::default().bg(bg)),
                    Rect::new(cx, row, cw, 1).intersection(inner),
                );
                row += 1;
            }
            if has_url && !d.homepage.is_empty() {
                let url = strings::truncate(
                    d.homepage.trim_end_matches('/'),
                    cw.saturating_sub(5) as usize,
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled("[o]  ", Style::default().fg(palette.accent)),
                        Span::styled(
                            url,
                            Style::default()
                                .fg(palette.muted)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                    ]))
                    .style(Style::default().bg(bg)),
                    Rect::new(cx, row, cw, 1).intersection(inner),
                );
                row += 1;
            }
        }
    }
    if has_recent {
        let sep = "─".repeat(cw as usize);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                sep,
                Style::default().fg(palette.dim),
            )))
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        row += 1;

        frame.render_widget(
            Paragraph::new(Span::styled(
                t("screensaver.recent_tracks"),
                Style::default().fg(palette.muted),
            ))
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        row += 1;

        let now_live = t("screensaver.now_live");
        let badge_w = now_live.chars().count() as u16 + 4;
        let max_cur = cw.saturating_sub(3 + badge_w) as usize;

        if let Some(current) = state.recent_titles.first() {
            let display = strings::truncate(current, max_cur);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("▶  ", Style::default().fg(palette.accent)),
                    Span::styled(
                        display,
                        Style::default()
                            .fg(palette.highlight)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("  {now_live}"), Style::default().fg(palette.accent)),
                ]))
                .style(Style::default().bg(bg)),
                Rect::new(cx, row, cw, 1).intersection(inner),
            );
            row += 1;
        }

        let max_prev = cw.saturating_sub(4) as usize;
        for track in state.recent_titles.iter().skip(1).take(4) {
            let display = strings::truncate(track, max_prev);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("↳  ", Style::default().fg(palette.dim)),
                    Span::styled(display, Style::default().fg(palette.highlight)),
                ]))
                .style(Style::default().bg(bg)),
                Rect::new(cx, row, cw, 1).intersection(inner),
            );
            row += 1;
        }
    }
    row += 1;
    if row < inner.bottom() {
        let vol_pct = (state.volume.clamp(0.0, 1.0) * 100.0).round() as u32;
        let vol_color = if state.volume > 0.85 {
            palette.warning
        } else {
            palette.accent
        };
        let (filled, empty) = volume_bar_spans(state.volume, 10);
        let time_str = now_t.format("%H:%M").to_string();
        let shortcuts = "[Space] ⏸/▶  [+/-] Vol  [Alt+S] ■  [any] →";
        frame.render_widget(
            Paragraph::new(Span::styled(shortcuts, Style::default().fg(palette.dim)))
                .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{}  ", time_str),
                    Style::default().fg(palette.muted),
                ),
                Span::styled(filled, Style::default().fg(vol_color)),
                Span::styled(empty, Style::default().fg(palette.muted)),
                Span::styled(format!("  {:>3}%", vol_pct), Style::default().fg(vol_color)),
            ]))
            .alignment(Alignment::Right)
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
    }
}

pub(crate) const LOGO_W: u16 = 39;

fn eq_bar_level(tick: u32, period: u32, phase: u32, min_l: usize, max_l: usize) -> usize {
    let t = tick.wrapping_add(phase) % period;
    let half = period / 2;
    let range = max_l - min_l;
    let level = if t < half {
        (t as usize * range) / (half as usize).max(1) + min_l
    } else {
        ((period - t) as usize * range) / (half as usize).max(1) + min_l
    };
    level.clamp(min_l, max_l)
}

fn eq_bar_char(level: usize) -> &'static str {
    match level {
        1 => "▁▁",
        2 => "▂▂",
        3 => "▃▃",
        4 => "▄▄",
        5 => "▅▅",
        6 => "▆▆",
        7 => "▇▇",
        _ => "██",
    }
}

fn logo_lines(bg: ratatui::style::Color, tick: u32, palette: &Palette) -> [Line<'static>; 2] {
    const LETTERS: &[char] = &['R', 'E', 'V', 'E', 'R', 'B', 'I', 'C'];
    let cyan = palette.logo_letters[0];
    let violet = palette.logo_letters[3];
    let pink = palette.logo_letters[7];

    let b1 = eq_bar_level(tick, 36, 0, 1, 8);
    let b2 = eq_bar_level(tick, 42, 12, 2, 8);
    let b3 = eq_bar_level(tick, 30, 22, 1, 7);

    let top1 = if b1 == 8 { "▄▄" } else { "  " };
    let top2 = match b2 {
        8 => "▄▄",
        7 => "▁▁",
        _ => "  ",
    };
    let top3 = if b3 == 7 { "▁▁" } else { "  " };

    let s = Style::default().bg(bg);

    let top = Line::from(vec![
        Span::styled("    ", s),
        Span::styled(top1, s.fg(cyan)),
        Span::styled("  ", s),
        Span::styled(top2, s.fg(violet)),
        Span::styled("  ", s),
        Span::styled(top3, s.fg(pink)),
    ]);

    let mut spans: Vec<Span<'static>> = vec![
        Span::styled("▶", s.fg(cyan)),
        Span::styled("   ", s),
        Span::styled(eq_bar_char(b1), s.fg(cyan)),
        Span::styled("  ", s),
        Span::styled(eq_bar_char(b2), s.fg(violet)),
        Span::styled("  ", s),
        Span::styled(eq_bar_char(b3), s.fg(pink)),
        Span::styled("   ", s),
    ];
    for (i, &ch) in LETTERS.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", s));
        }
        spans.push(Span::styled(
            ch.to_string(),
            s.fg(palette.logo_letters[i]).add_modifier(Modifier::BOLD),
        ));
    }
    [top, Line::from(spans)]
}

pub(crate) fn render_logo_above(
    frame: &mut Frame,
    area_x: u16,
    area_width: u16,
    y: u16,
    bg: ratatui::style::Color,
    tick: u32,
    palette: &Palette,
) {
    if y < 2 {
        return;
    }
    let logo_x = area_x + area_width.saturating_sub(LOGO_W) / 2;
    let [l1, l2] = logo_lines(bg, tick, palette);
    let st = Style::default().bg(bg);
    frame.render_widget(
        Paragraph::new(l1).style(st),
        Rect::new(logo_x, y - 2, LOGO_W, 1),
    );
    frame.render_widget(
        Paragraph::new(l2).style(st),
        Rect::new(logo_x, y - 1, LOGO_W, 1),
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
    row: usize,
    h1: u8,
    h2: u8,
    m1: u8,
    m2: u8,
    colon_on: bool,
    color: ratatui::style::Color,
    bg: ratatui::style::Color,
) -> Line<'static> {
    let colon_ch = if colon_on {
        match row {
            1 | 3 => "█",
            _ => " ",
        }
    } else {
        " "
    };
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

pub(crate) fn visualizer_line(
    level_db: f32,
    width: usize,
    bg: ratatui::style::Color,
    palette: &Palette,
) -> Line<'static> {
    let glyphs = super::visualizer_glyphs();
    let base = ((level_db + 60.0) / 60.0).clamp(0.0, 1.0) as f64;
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0) as f64;

    let n_bars = (width / 2).max(1);
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(n_bars * 2);
    for i in 0..n_bars {
        let freq = 0.0025 + (i as f64) * 0.00025;
        let phase = i as f64 * 1.1;
        let wave = (ms * freq + phase).sin() * 0.35 + 0.35;
        let h = (base * 0.65 + wave * 0.35).clamp(0.0, 1.0);
        let idx = ((h * 7.0) as usize).min(7);
        let pos_idx = (i * 7 / n_bars.saturating_sub(1).max(1)).min(7);
        let color = if h < 0.05 {
            palette.muted
        } else {
            palette.spectrum[pos_idx]
        };
        spans.push(Span::styled(
            glyphs[idx].to_string(),
            Style::default().fg(color).bg(bg),
        ));
        if i + 1 < n_bars {
            spans.push(Span::styled(" ", Style::default().bg(bg)));
        }
    }
    Line::from(spans)
}

fn volume_bar_spans(vol: f32, bar_width: usize) -> (String, String) {
    let filled = (vol.clamp(0.0, 1.0) * bar_width as f32).round() as usize;
    let filled = filled.min(bar_width);
    ("█".repeat(filled), "░".repeat(bar_width - filled))
}

#[expect(clippy::too_many_arguments)]
pub(super) fn render_spotify_screensaver(
    frame: &mut Frame,
    area: Rect,
    playback: &crate::integrations::spotify::SpotifyPlaybackState,
    profile_name: Option<&str>,
    country: Option<&str>,
    followers: Option<u32>,
    is_premium: Option<bool>,
    show_clock: bool,
    border_tick: u32,
    palette: &Palette,
) {
    let overlay = palette.overlay_color;
    let bg = palette.panel_bg;
    let green = palette.playing;

    frame.render_widget(Clear, area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            frame.buffer_mut()[(x, y)].set_bg(overlay);
        }
    }

    let has_name_row = profile_name.is_some() || country.is_some();
    let has_plan_row = is_premium.is_some_and(|is_premium| is_premium) || followers.is_some();
    let profile_rows = u16::from(has_name_row) + u16::from(has_plan_row);
    let has_profile = profile_rows > 0;

    let pw = (area.width * 85 / 100).clamp(60, 110);
    let clock_rows: u16 = if show_clock { 6 } else { 0 };
    let ph_base: u16 = 2 + 1 + clock_rows + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1;
    let ph_with_profile = ph_base + 1 + profile_rows;
    let ph = if has_profile && ph_with_profile <= area.height {
        ph_with_profile
    } else {
        ph_base
    };
    let show_profile = has_profile && ph == ph_with_profile;

    let px = area.x + area.width.saturating_sub(pw) / 2;
    let py = area.y + area.height.saturating_sub(ph) / 2;
    let panel = Rect::new(px, py, pw, ph.min(area.height));

    if py >= 3 {
        render_logo_above(
            frame,
            area.x,
            area.width,
            py - 1,
            overlay,
            border_tick,
            palette,
        );
    }

    let border_color = if playback.is_playing {
        theme::border_color_for(palette, border_tick)
    } else {
        palette.warning
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx = inner.x + 2;
    let cw = inner.width.saturating_sub(4);
    let mut row = inner.y + 1;

    let now_t = Local::now();
    if show_clock {
        let h1 = (now_t.hour() / 10) as u8;
        let h2 = (now_t.hour() % 10) as u8;
        let m1 = (now_t.minute() / 10) as u8;
        let m2 = (now_t.minute() % 10) as u8;
        let colon_on = now_t.second().is_multiple_of(2);
        let clock_w: u16 = 19;
        let clock_x = cx + cw.saturating_sub(clock_w) / 2;
        for r in 0..5usize {
            frame.render_widget(
                Paragraph::new(build_clock_row(
                    r,
                    h1,
                    h2,
                    m1,
                    m2,
                    colon_on,
                    border_color,
                    bg,
                ))
                .style(Style::default().bg(bg)),
                Rect::new(clock_x, row, clock_w, 1).intersection(inner),
            );
            row += 1;
        }
        row += 1;
    }

    frame.render_widget(
        Paragraph::new(Span::styled(
            "SPOTIFY",
            Style::default().fg(green).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .style(Style::default().bg(bg)),
        Rect::new(cx, row, cw, 1).intersection(inner),
    );
    row += 1;

    frame.render_widget(
        Paragraph::new(Span::styled(
            strings::truncate(&playback.device_name, cw as usize),
            Style::default().fg(palette.dim),
        ))
        .alignment(Alignment::Center)
        .style(Style::default().bg(bg)),
        Rect::new(cx, row, cw, 1).intersection(inner),
    );
    row += 1;
    row += 1;

    frame.render_widget(
        Paragraph::new(Span::styled(
            strings::truncate(&playback.artist, cw as usize),
            Style::default().fg(palette.muted),
        ))
        .alignment(Alignment::Center)
        .style(Style::default().bg(bg)),
        Rect::new(cx, row, cw, 1).intersection(inner),
    );
    row += 1;

    frame.render_widget(
        Paragraph::new(Span::styled(
            strings::truncate(&playback.track_name, cw as usize),
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .style(Style::default().bg(bg)),
        Rect::new(cx, row, cw, 1).intersection(inner),
    );
    row += 1;

    frame.render_widget(
        Paragraph::new(Span::styled(
            strings::truncate(&playback.album, cw as usize),
            Style::default().fg(palette.dim),
        ))
        .alignment(Alignment::Center)
        .style(Style::default().bg(bg)),
        Rect::new(cx, row, cw, 1).intersection(inner),
    );
    row += 1;
    row += 1;

    let progress_ratio = if playback.duration_ms > 0 {
        (playback.progress_ms as f32 / playback.duration_ms as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let time_cur = fmt_ms(playback.progress_ms);
    let time_tot = fmt_ms(playback.duration_ms);

    let time_prefix = format!("{} ", time_cur);
    let time_suffix = format!(" {}", time_tot);
    let bar_w = cw.saturating_sub(time_prefix.len() as u16 + time_suffix.len() as u16);
    let filled = (progress_ratio * bar_w as f32).round() as usize;
    let empty = (bar_w as usize).saturating_sub(filled);
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(time_prefix, Style::default().fg(palette.muted)),
            Span::styled(bar, Style::default().fg(green)),
            Span::styled(time_suffix, Style::default().fg(palette.muted)),
        ]))
        .style(Style::default().bg(bg)),
        Rect::new(cx, row, cw, 1).intersection(inner),
    );
    row += 1;

    frame.render_widget(
        Paragraph::new(visualizer_line(-60.0, cw as usize, bg, palette))
            .style(Style::default().bg(bg)),
        Rect::new(cx, row, cw, 1).intersection(inner),
    );
    row += 1;

    if show_profile {
        let sep = "─".repeat(cw as usize);
        frame.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(palette.dim)))
                .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        row += 1;

        if has_name_row {
            let name_tc = profile_name.map(strings::title_case);
            let country_str = country.unwrap_or("");
            let mut spans: Vec<Span<'static>> = Vec::new();
            if let Some(name) = name_tc {
                spans.push(Span::styled(
                    name,
                    Style::default()
                        .fg(palette.highlight)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            if !country_str.is_empty() {
                let used = spans
                    .iter()
                    .map(|s| s.content.chars().count() as u16)
                    .sum::<u16>();
                let pad = cw.saturating_sub(used + country_str.chars().count() as u16);
                spans.push(Span::styled(" ".repeat(pad as usize), Style::default()));
                spans.push(Span::styled(
                    country_str.to_string(),
                    Style::default().fg(palette.muted),
                ));
            }
            if !spans.is_empty() {
                frame.render_widget(
                    Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
                    Rect::new(cx, row, cw, 1).intersection(inner),
                );
                row += 1;
            }
        }

        if has_plan_row {
            let plan_str = if is_premium.is_some_and(|is_premium| is_premium) {
                t("integrations.spotify.premium")
            } else {
                String::new()
            };
            let follow_str = followers
                .map(|f| format!("{f} {}", t("screensaver.followers")))
                .unwrap_or_default();
            let mut spans: Vec<Span<'static>> = Vec::new();
            if !plan_str.is_empty() {
                spans.push(Span::styled(
                    plan_str,
                    Style::default().fg(green).add_modifier(Modifier::BOLD),
                ));
            }
            if !follow_str.is_empty() {
                let used = spans
                    .iter()
                    .map(|s| s.content.chars().count() as u16)
                    .sum::<u16>();
                let pad = cw.saturating_sub(used + follow_str.chars().count() as u16);
                spans.push(Span::styled(" ".repeat(pad as usize), Style::default()));
                spans.push(Span::styled(follow_str, Style::default().fg(palette.muted)));
            }
            if !spans.is_empty() {
                frame.render_widget(
                    Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
                    Rect::new(cx, row, cw, 1).intersection(inner),
                );
                row += 1;
            }
        }
    }

    row += 1;

    if row < inner.bottom() {
        let time_str = now_t.format("%H:%M").to_string();
        let state_str = if playback.is_playing { "▶" } else { "⏸" };
        let vol_color = if playback.volume_pct > 85 {
            palette.warning
        } else {
            green
        };
        let (filled, empty) = volume_bar_spans(playback.volume_pct as f32 / 100.0, 10);
        let shortcuts = "[Space] ⏸/▶  [+/-] Vol  [any] →";
        frame.render_widget(
            Paragraph::new(Span::styled(shortcuts, Style::default().fg(palette.dim)))
                .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{}  ", time_str),
                    Style::default().fg(palette.muted),
                ),
                Span::styled(filled, Style::default().fg(vol_color)),
                Span::styled(empty, Style::default().fg(palette.muted)),
                Span::styled(
                    format!("  {:>3}%  {}", playback.volume_pct, state_str),
                    Style::default().fg(vol_color),
                ),
            ]))
            .alignment(Alignment::Right)
            .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
    }
}

fn fmt_ms(ms: u32) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}
