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
    let area = frame.area();
    let w = area.width.clamp(30, 50);
    let h: u16 = 5;
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let panel = ratatui::layout::Rect::new(x, y, w, h);

    frame.render_widget(Clear, panel);

    let block = Block::default()
        .title_top(
            Line::from(Span::styled(
                t("modal.rename.title"),
                Style::default()
                    .fg(theme::HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        )
        .title_bottom(
            Line::from(Span::styled(
                t("modal.rename.hint"),
                Style::default().fg(theme::MUTED),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::ACCENT))
        .style(Style::default().bg(theme::PANEL_BG));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let text_area =
        ratatui::layout::Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(input.to_owned(), Style::default().fg(theme::HIGHLIGHT)),
            Span::styled(
                "_",
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        text_area,
    );
}

pub(super) fn render_client_id_overlay(frame: &mut Frame, input: &str) {
    let area = frame.area();
    let w = area.width.clamp(40, 60);
    let h: u16 = 5;
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let panel = ratatui::layout::Rect::new(x, y, w, h);

    frame.render_widget(Clear, panel);

    let block = Block::default()
        .title_top(
            Line::from(Span::styled(
                t("modal.client_id.title"),
                Style::default()
                    .fg(theme::HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        )
        .title_bottom(
            Line::from(Span::styled(
                t("modal.client_id.hint"),
                Style::default().fg(theme::MUTED),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::ACCENT))
        .style(Style::default().bg(theme::PANEL_BG));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let text_area =
        ratatui::layout::Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(input.to_owned(), Style::default().fg(theme::HIGHLIGHT)),
            Span::styled(
                "_",
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        text_area,
    );
}

pub(super) fn render_game_strip(
    frame: &mut Frame,
    area: Rect,
    name: &str,
    genre: &str,
    border_tick: u32,
) {
    const H_PAD: u16 = 2;

    let border_color = theme::border_color(border_tick);
    let block = Block::default()
        .title_top(
            Line::from(Span::styled(
                format!(" {} ", t("overlay.playing_game")),
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Left),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::PANEL_BG));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cx = inner.x + H_PAD;
    let cw = inner.width.saturating_sub(H_PAD * 2);
    let mut spans: Vec<Span<'static>> = vec![Span::styled(name.to_owned(), theme::PLAYING_STYLE)];
    if !genre.is_empty() {
        spans.push(Span::styled("  ·  ", Style::default().fg(theme::MUTED)));
        spans.push(Span::styled(
            genre.to_owned(),
            Style::default().fg(theme::DIM),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::PANEL_BG)),
        Rect::new(cx, inner.y, cw, 1),
    );
}

pub(super) fn render_modal_np_strip(
    frame: &mut Frame,
    strip: Rect,
    state: &PlayerState,
    border_tick: u32,
) {
    const H_PAD: u16 = 2;

    if matches!(state.status, PlayerStatus::Idle | PlayerStatus::Error(_)) {
        return;
    }

    let content_w = strip.width.saturating_sub(2 + H_PAD * 2) as usize;
    if content_w == 0 {
        return;
    }

    let raw_title = match &state.status {
        PlayerStatus::Playing | PlayerStatus::Paused | PlayerStatus::Buffering(_) => {
            state.title.clone().unwrap_or_default()
        }
        _ => String::new(),
    };
    let title_lines = wrap_into_lines(&raw_title, content_w, 2);
    let title_line_count = title_lines.len() as u16;
    let has_progress = state.playback_pos_secs.is_some();
    let panel_h = 2 + 1 + title_line_count + 1;
    if panel_h > strip.height {
        return;
    }

    let panel = Rect::new(strip.x, strip.y, strip.width, panel_h);

    let border_color = match &state.status {
        PlayerStatus::Playing => theme::border_color(border_tick),
        PlayerStatus::Paused => theme::WARNING,
        PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_) => {
            ratatui::style::Color::Rgb(80, 80, 80)
        }
        _ => theme::MUTED,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::PANEL_BG));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx = inner.x + H_PAD;
    let cw = inner.width.saturating_sub(H_PAD * 2);
    let station_line = build_modal_station_line(state);
    frame.render_widget(
        Paragraph::new(station_line).style(Style::default().bg(theme::PANEL_BG)),
        Rect::new(cx, inner.y, cw, 1),
    );
    for (i, tline) in title_lines.into_iter().enumerate() {
        let row_y = inner.y + 1 + i as u16;
        if row_y < inner.bottom() {
            frame.render_widget(
                Paragraph::new(tline).style(Style::default().bg(theme::PANEL_BG)),
                Rect::new(cx, row_y, cw, 1),
            );
        }
    }
    let viz_row = inner.bottom().saturating_sub(1);
    if viz_row >= inner.y && viz_row < inner.bottom() {
        let vol_pct = (state.volume.clamp(0.0, 1.0) * 100.0).round() as u32;
        let vol_color = if state.volume > 0.85 {
            theme::WARNING
        } else {
            theme::ACCENT
        };
        let (filled, empty) = volume_bar_spans(state.volume, 8);
        let pct_str = format!("  {:>3}%", vol_pct);
        let vol_prefix = "  ";
        let vol_w =
            vol_prefix.len() + filled.chars().count() + empty.chars().count() + pct_str.len();
        let progress_text = if has_progress {
            playback_progress_text(state)
        } else {
            None
        };
        let progress_w = progress_text
            .as_ref()
            .map(|text| text.chars().count() + 2)
            .unwrap_or(0);
        let viz_w = (cw as usize).saturating_sub(progress_w + vol_w + 1);
        let mut spans = visualizer_spans(state.level_db, viz_w, theme::PANEL_BG);
        if let Some(text) = progress_text {
            if !spans.is_empty() {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                text,
                Style::default().fg(theme::MUTED).bg(theme::PANEL_BG),
            ));
        }
        spans.push(Span::raw(vol_prefix));
        spans.push(Span::styled(filled, Style::default().fg(vol_color)));
        spans.push(Span::styled(empty, Style::default().fg(theme::MUTED)));
        spans.push(Span::styled(pct_str, Style::default().fg(vol_color)));
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::PANEL_BG)),
            Rect::new(cx, viz_row, cw, 1),
        );
    }
}

fn volume_bar_spans(vol: f32, bar_width: usize) -> (String, String) {
    let filled = (vol.clamp(0.0, 1.0) * bar_width as f32).round() as usize;
    let filled = filled.min(bar_width);
    ("█".repeat(filled), "░".repeat(bar_width - filled))
}

fn visualizer_spans(level_db: f32, width: usize, bg: ratatui::style::Color) -> Vec<Span<'static>> {
    use ratatui::style::Color::Rgb;
    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    const SPECTRUM: [ratatui::style::Color; 8] = [
        Rgb(0, 240, 255),
        Rgb(40, 160, 255),
        Rgb(75, 80, 255),
        Rgb(112, 0, 255),
        Rgb(160, 0, 200),
        Rgb(200, 0, 140),
        Rgb(235, 0, 100),
        Rgb(255, 0, 85),
    ];
    if width == 0 {
        return vec![];
    }
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
            theme::MUTED
        } else {
            SPECTRUM[pos_idx]
        };
        spans.push(Span::styled(
            BLOCKS[idx].to_string(),
            Style::default().fg(color).bg(bg),
        ));
        if i + 1 < n_bars {
            spans.push(Span::styled(" ", Style::default().bg(bg)));
        }
    }
    spans
}

pub(super) fn playback_progress_line(
    state: &PlayerState,
    width: u16,
    active_color: ratatui::style::Color,
    bg: ratatui::style::Color,
) -> Option<Line<'static>> {
    let pos = state.playback_pos_secs?;
    let elapsed = fmt_secs(pos);
    let Some(duration) = state.playback_duration_secs.filter(|d| *d > 0.0) else {
        return Some(Line::from(Span::styled(
            elapsed,
            Style::default().fg(theme::MUTED).bg(bg),
        )));
    };

    let remaining = (duration - pos).max(0.0);
    let prefix = format!("{elapsed} ");
    let suffix = format!(" -{}", fmt_secs(remaining));
    let bar_w = (width as usize).saturating_sub(prefix.len() + suffix.len());
    if bar_w == 0 {
        return Some(Line::from(Span::styled(
            format!("{elapsed}  {}", suffix.trim()),
            Style::default().fg(theme::MUTED).bg(bg),
        )));
    }

    let ratio = (pos / duration).clamp(0.0, 1.0);
    let filled = (ratio * bar_w as f32).round() as usize;
    let empty = bar_w.saturating_sub(filled);
    Some(Line::from(vec![
        Span::styled(prefix, Style::default().fg(theme::MUTED).bg(bg)),
        Span::styled(
            "\u{2588}".repeat(filled),
            Style::default().fg(active_color).bg(bg),
        ),
        Span::styled(
            "\u{2591}".repeat(empty),
            Style::default().fg(theme::MUTED).bg(bg),
        ),
        Span::styled(suffix, Style::default().fg(theme::MUTED).bg(bg)),
    ]))
}

fn playback_progress_text(state: &PlayerState) -> Option<String> {
    let pos = state.playback_pos_secs?;
    let elapsed = fmt_secs(pos);
    state
        .playback_duration_secs
        .filter(|d| *d > 0.0)
        .map(|duration| format!("{elapsed} -{}", fmt_secs((duration - pos).max(0.0))))
        .or(Some(elapsed))
}

fn fmt_secs(secs: f32) -> String {
    let secs = secs.max(0.0).round() as u32;
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub(super) fn build_modal_station_line(state: &PlayerState) -> Line<'static> {
    let name = state
        .station
        .as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_default();
    match &state.status {
        PlayerStatus::Connecting | PlayerStatus::Reconnecting(_) => Line::from(vec![
            Span::styled("…  ", Style::default().fg(theme::ACCENT)),
            Span::styled(name, Style::default().fg(theme::MUTED)),
        ]),
        PlayerStatus::Buffering(_) | PlayerStatus::Playing | PlayerStatus::Paused => {
            let icon = if matches!(state.status, PlayerStatus::Paused) {
                "⏸  "
            } else {
                "▶  "
            };
            Line::from(vec![
                Span::styled(icon, Style::default().fg(theme::ACCENT)),
                Span::styled(name, theme::PLAYING_STYLE),
            ])
        }
        _ => Line::default(),
    }
}

pub(super) fn render_modal_spotify_strip(
    frame: &mut Frame,
    strip: Rect,
    playback: Option<&crate::integrations::spotify::SpotifyPlaybackState>,
    now_playing: Option<&crate::integrations::spotify::SpotifyTrack>,
    player_status: &crate::app::SpotifyPlayerStatus,
    border_tick: u32,
) {
    use crate::app::SpotifyPlayerStatus;
    const H_PAD: u16 = 2;

    let (is_playing, is_paused) = if let Some(pb) = playback {
        (pb.is_playing, !pb.is_playing)
    } else {
        (
            matches!(player_status, SpotifyPlayerStatus::Playing),
            matches!(player_status, SpotifyPlayerStatus::Paused),
        )
    };
    if !is_playing && !is_paused {
        return;
    }

    let (artist, track_name, album, volume_pct) = if let Some(pb) = playback {
        (
            pb.artist.as_str(),
            pb.track_name.as_str(),
            pb.album.as_str(),
            Some(pb.volume_pct),
        )
    } else if let Some(np) = now_playing {
        (
            np.artist.as_str(),
            np.name.as_str(),
            np.album.as_str(),
            None,
        )
    } else {
        return;
    };

    if artist.is_empty() && track_name.is_empty() {
        return;
    }

    let content_w = strip.width.saturating_sub(2 + H_PAD * 2) as usize;
    if content_w == 0 {
        return;
    }

    let (progress_ms, duration_ms) = playback
        .map(|pb| (pb.progress_ms, pb.duration_ms))
        .unwrap_or((0, 0));
    let has_progress = duration_ms > 0 && content_w >= 12;

    let track_meta = if album.is_empty() {
        track_name.to_owned()
    } else {
        format!("{track_name} · {album}")
    };
    let title_lines = wrap_into_lines(&track_meta, content_w, 2);

    let panel_h = 2 + 1 + title_lines.len() as u16 + u16::from(has_progress) + 1;
    if panel_h > strip.height {
        return;
    }

    let panel = Rect::new(strip.x, strip.y, strip.width, panel_h);

    let border_color = if is_playing {
        theme::border_color(border_tick)
    } else {
        theme::WARNING
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme::PANEL_BG));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx = inner.x + H_PAD;
    let cw = inner.width.saturating_sub(H_PAD * 2);
    let icon = if is_playing { "▶  " } else { "⏸  " };

    let artist_display = crate::ui::strings::truncate(artist, cw.saturating_sub(3) as usize);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(icon, Style::default().fg(theme::PLAYING)),
            Span::styled(artist_display, theme::PLAYING_STYLE),
        ]))
        .style(Style::default().bg(theme::PANEL_BG)),
        Rect::new(cx, inner.y, cw, 1),
    );

    let mut last_row = inner.y;
    for (i, tline) in title_lines.into_iter().enumerate() {
        let row_y = inner.y + 1 + i as u16;
        if row_y < inner.bottom() {
            frame.render_widget(
                Paragraph::new(tline).style(Style::default().bg(theme::PANEL_BG)),
                Rect::new(cx, row_y, cw, 1),
            );
            last_row = row_y;
        }
    }

    if has_progress {
        let prog_y = last_row + 1;
        if prog_y < inner.bottom() {
            let ratio = (progress_ms as f32 / duration_ms as f32).clamp(0.0, 1.0);
            let fmt_ms = |ms: u32| {
                let s = ms / 1000;
                format!("{}:{:02}", s / 60, s % 60)
            };
            let prefix = format!("{} ", fmt_ms(progress_ms));
            let suffix = format!(" {}", fmt_ms(duration_ms));
            let bar_w = (cw as usize).saturating_sub(prefix.len() + suffix.len());
            let filled = (ratio * bar_w as f32).round() as usize;
            let empty = bar_w.saturating_sub(filled);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(prefix, ratatui::style::Style::default().fg(theme::MUTED)),
                    Span::styled(
                        "█".repeat(filled),
                        ratatui::style::Style::default().fg(border_color),
                    ),
                    Span::styled(
                        "░".repeat(empty),
                        ratatui::style::Style::default().fg(theme::MUTED),
                    ),
                    Span::styled(suffix, ratatui::style::Style::default().fg(theme::MUTED)),
                ]))
                .style(ratatui::style::Style::default().bg(theme::PANEL_BG)),
                Rect::new(cx, prog_y, cw, 1),
            );
        }
    }
    let viz_row = inner.bottom().saturating_sub(1);
    if viz_row >= inner.y && viz_row < inner.bottom() {
        let vol_color = theme::PLAYING;
        let vol_w = if let Some(v) = volume_pct {
            let (f, e) = volume_bar_spans(v as f32 / 100.0, 8);
            2 + f.chars().count() + e.chars().count() + format!("  {:>3}%", v).len()
        } else {
            0
        };
        let viz_w = (cw as usize).saturating_sub(vol_w + 1);
        let mut spans = visualizer_spans(-60.0, viz_w, theme::PANEL_BG);
        if let Some(v) = volume_pct {
            let (filled, empty) = volume_bar_spans(v as f32 / 100.0, 8);
            spans.push(Span::raw("  "));
            spans.push(Span::styled(filled, Style::default().fg(vol_color)));
            spans.push(Span::styled(empty, Style::default().fg(theme::MUTED)));
            spans.push(Span::styled(
                format!("  {:>3}%", v),
                Style::default().fg(vol_color),
            ));
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::PANEL_BG)),
            Rect::new(cx, viz_row, cw, 1),
        );
    }
}

pub(super) fn render_update_badge(frame: &mut Frame, version: &str, area: Rect) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let blink = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() % 1000 < 500)
        .unwrap_or(false);
    let dot = if blink { "• " } else { "  " };
    let text = format!("{dot}v{version}");
    let w = (text.chars().count() as u16 + 1).min(area.width);
    let x = area.x + area.width.saturating_sub(w);
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(theme::WARNING)),
        Rect::new(x, area.y, w, 1),
    );
}

pub(super) fn render_help_overlay(
    frame: &mut Frame,
    mode: &crate::app::SearchMode,
    spotify_logged_in: bool,
    update_available: Option<&str>,
) {
    use crate::app::SearchMode;

    let lines: Vec<(&str, String)> = match mode {
        SearchMode::Name => vec![
            ("[↵]", t("help.shortcut.play_station")),
            ("[↑↓]", t("help.shortcut.nav_list")),
            ("[Alt+F]", t("help.shortcut.save_fav")),
            ("[Space]", t("help.shortcut.pause_resume")),
            ("[Alt+R]", t("help.shortcut.random_station")),
            ("[Alt+S]", t("help.shortcut.stop_radio")),
            ("[Tab]", t("help.shortcut.go_spotify")),
            ("[Alt+G]", t("help.shortcut.by_genre")),
            ("[Alt+C]", t("help.shortcut.by_country")),
            ("[Alt+O]", t("help.shortcut.open_config")),
            ("[Esc]", t("help.shortcut.close_quit")),
        ],
        SearchMode::Spotify => {
            if spotify_logged_in {
                vec![
                    ("[←→]", t("help.shortcut.switch_subtab")),
                    ("[↵]", t("help.shortcut.transfer_play")),
                    ("[↑↓]", t("help.shortcut.navigate")),
                    ("[Space]", t("help.shortcut.pause_resume")),
                    ("[Alt+O]", t("help.shortcut.settings")),
                    ("[Alt+D]", t("integrations.spotify.hint_disconnect")),
                    ("[Alt+R]", t("help.shortcut.reload_devices")),
                    ("[Esc]", t("hint.close")),
                ]
            } else {
                vec![
                    ("[↵]", t("help.shortcut.connect_spotify")),
                    ("[Tab]", t("help.shortcut.go_radio")),
                    ("[Esc]", t("hint.close")),
                ]
            }
        }
        SearchMode::Settings => vec![
            ("[Space]", t("help.shortcut.change_value")),
            ("[↑↓]", t("help.shortcut.nav_options")),
            ("[Esc]", t("hint.back")),
        ],
        _ => vec![
            ("[↑↓]", t("help.shortcut.navigate")),
            ("[↵]", t("help.shortcut.confirm")),
            ("[Esc]", t("hint.back")),
        ],
    };

    const CREDITS: &[&str] = &["Esteban Jaramillo — Chile", "github.com/sewandev/Reverbic"];

    let area = frame.area();
    let w = 46u16.min(area.width);
    let line_count = lines.len();
    let update_row = update_available.is_some() as u16;
    let h = (line_count as u16 + 3 + CREDITS.len() as u16 + 2 + update_row).min(area.height);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let rect = ratatui::layout::Rect::new(x, y, w, h);

    frame.render_widget(Clear, rect);
    let block = Block::default()
        .title(format!(" {} ", t("help.overlay.title")))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::ACCENT))
        .style(Style::default().bg(theme::PANEL_BG));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    for (i, (key_str, desc)) in lines.into_iter().enumerate() {
        let row_y = inner.y + i as u16;
        if row_y >= inner.bottom() {
            break;
        }
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("  {:9}", key_str),
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(desc, Style::default().fg(theme::HIGHLIGHT)),
            ]))
            .style(Style::default().bg(theme::PANEL_BG)),
            ratatui::layout::Rect::new(inner.x, row_y, inner.width, 1),
        );
    }
    let sep_y = inner.y + line_count as u16 + 1;
    if sep_y < inner.bottom() {
        let sep = "─".repeat(inner.width as usize);
        frame.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(theme::DIM)))
                .style(Style::default().bg(theme::PANEL_BG)),
            ratatui::layout::Rect::new(inner.x, sep_y, inner.width, 1),
        );
    }

    if let Some(version) = update_available {
        let row_y = sep_y + 1;
        if row_y < inner.bottom() {
            let notice = format!("  {} v{version}", t("update.available"));
            frame.render_widget(
                Paragraph::new(Span::styled(
                    notice,
                    Style::default()
                        .fg(theme::WARNING)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(theme::PANEL_BG)),
                ratatui::layout::Rect::new(inner.x, row_y, inner.width, 1),
            );
        }
    }
    let credits_offset = update_row;
    for (i, line) in CREDITS.iter().enumerate() {
        let row_y = sep_y + 1 + credits_offset + i as u16;
        if row_y >= inner.bottom() {
            break;
        }
        frame.render_widget(
            Paragraph::new(Span::styled(*line, Style::default().fg(theme::MUTED)))
                .style(Style::default().bg(theme::PANEL_BG)),
            ratatui::layout::Rect::new(inner.x + 2, row_y, inner.width.saturating_sub(2), 1),
        );
    }
}

pub(super) fn wrap_into_lines(text: &str, width: usize, max_lines: usize) -> Vec<Line<'static>> {
    if text.is_empty() || width == 0 {
        return vec![];
    }
    let chars: Vec<char> = text.chars().collect();
    let mut lines = Vec::new();
    let mut offset = 0;
    while offset < chars.len() && lines.len() < max_lines {
        let end = (offset + width).min(chars.len());
        let slice: String = chars[offset..end].iter().collect();
        lines.push(Line::from(Span::styled(
            slice,
            Style::default().fg(theme::HIGHLIGHT),
        )));
        offset = end;
    }
    lines
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use super::{fmt_secs, playback_progress_line, playback_progress_text};
    use crate::audio::{PlayerState, PlayerStatus};

    fn line_text(line: &ratatui::text::Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
    }

    #[test]
    fn formats_seconds_as_minutes_and_seconds() {
        assert_eq!(fmt_secs(65.0), "1:05");
    }

    #[test]
    fn playback_progress_saturates_remaining_time() {
        let state = PlayerState {
            status: PlayerStatus::Playing,
            playback_pos_secs: Some(70.0),
            playback_duration_secs: Some(65.0),
            ..Default::default()
        };

        let line =
            playback_progress_line(&state, 20, Color::Green, Color::Black).expect("progress line");

        assert!(line_text(&line).contains("-0:00"));
    }

    #[test]
    fn playback_progress_uses_unicode_block_glyphs() {
        let state = PlayerState {
            status: PlayerStatus::Playing,
            playback_pos_secs: Some(65.0),
            playback_duration_secs: Some(185.0),
            ..Default::default()
        };

        let line =
            playback_progress_line(&state, 20, Color::Green, Color::Black).expect("progress line");
        let text = line_text(&line);

        assert!(text.contains('\u{2588}'));
        assert!(text.contains('\u{2591}'));
        assert!(!text.contains('\u{00e2}'));
    }

    #[test]
    fn compact_playback_progress_includes_remaining_time() {
        let state = PlayerState {
            status: PlayerStatus::Playing,
            playback_pos_secs: Some(65.0),
            playback_duration_secs: Some(185.0),
            ..Default::default()
        };

        assert_eq!(
            playback_progress_text(&state),
            Some("1:05 -2:00".to_string())
        );
    }

    #[test]
    fn playback_progress_without_duration_shows_elapsed_only() {
        let state = PlayerState {
            status: PlayerStatus::Playing,
            playback_pos_secs: Some(65.0),
            playback_duration_secs: None,
            ..Default::default()
        };

        let line =
            playback_progress_line(&state, 20, Color::Green, Color::Black).expect("progress line");

        assert_eq!(line_text(&line), "1:05");
    }
}
