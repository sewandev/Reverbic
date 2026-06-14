use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::audio::{PlayerState, PlayerStatus};
use crate::i18n::t;
use crate::ui::theme::{self, Palette, ThemeId};

pub(super) fn render_rename_overlay(
    frame: &mut Frame,
    input: &str,
    title: &str,
    palette: &Palette,
) {
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
                title.to_owned(),
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        )
        .title_bottom(
            Line::from(Span::styled(
                t("modal.rename.hint"),
                Style::default().fg(palette.muted),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accent))
        .style(Style::default().bg(palette.panel_bg));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let text_area =
        ratatui::layout::Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(input.to_owned(), Style::default().fg(palette.highlight)),
            Span::styled(
                "_",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        text_area,
    );
}

pub(super) fn render_device_picker_overlay(
    frame: &mut Frame,
    devices: &[crate::integrations::spotify::devices::SpotifyDevice],
    selected: usize,
    active_device_id: Option<&str>,
    palette: &Palette,
) {
    let area = frame.area();
    let w = area.width.clamp(40, 56);
    let h = (devices.len() as u16 + 4).clamp(5, area.height);
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let panel = Rect::new(x, y, w, h);

    frame.render_widget(Clear, panel);

    let block = Block::default()
        .title_top(
            Line::from(Span::styled(
                t("modal.device_picker.title"),
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        )
        .title_bottom(
            Line::from(Span::styled(
                t("modal.device_picker.hint"),
                Style::default().fg(palette.muted),
            ))
            .alignment(Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.spotify))
        .style(Style::default().bg(palette.panel_bg));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    for (i, device) in devices.iter().enumerate() {
        let row_y = inner.y + 1 + i as u16;
        if row_y >= inner.bottom() {
            break;
        }
        let focused = i == selected;
        let is_active =
            device.is_active || (device.id.is_some() && device.id.as_deref() == active_device_id);
        let marker = if focused { ">" } else { " " };
        let name_style = if focused {
            Style::default()
                .fg(palette.playing)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(palette.highlight)
        };
        let state_label = if is_active {
            t("modal.spotify.footer.active")
        } else {
            t("modal.spotify.footer.available")
        };
        let state_style = if is_active {
            Style::default().fg(palette.spotify)
        } else {
            Style::default().fg(palette.muted)
        };
        let name_w = inner.width.saturating_sub(6) as usize;
        let label = crate::ui::strings::truncate(
            &format!("{} · {}", device.name, device.device_type),
            name_w.saturating_sub(state_label.chars().count() + 3),
        )
        .to_string();
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!(" {marker} "), name_style),
                Span::styled(label, name_style),
                Span::styled(format!("  [{state_label}]"), state_style),
            ]))
            .style(Style::default().bg(palette.panel_bg)),
            Rect::new(inner.x + 1, row_y, inner.width.saturating_sub(2), 1),
        );
    }
}

pub(super) fn render_playlist_picker_overlay(
    frame: &mut Frame,
    picker: &crate::app::PlaylistPicker,
    playlists: &[crate::playlists::RadioPlaylist],
    palette: &Palette,
) {
    let area = frame.area();

    if picker.creating {
        let w = area.width.clamp(30, 50);
        let h: u16 = 5;
        let x = area.width.saturating_sub(w) / 2;
        let y = area.height.saturating_sub(h) / 2;
        let panel = Rect::new(x, y, w, h);

        frame.render_widget(Clear, panel);

        let block = Block::default()
            .title_top(
                Line::from(Span::styled(
                    t("modal.playlist_picker.name_title"),
                    Style::default()
                        .fg(palette.highlight)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Center),
            )
            .title_bottom(
                Line::from(Span::styled(
                    t("modal.playlist_picker.name_hint"),
                    Style::default().fg(palette.muted),
                ))
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(palette.accent))
            .style(Style::default().bg(palette.panel_bg));

        let inner = block.inner(panel);
        frame.render_widget(block, panel);

        let text_area = Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    picker.input.to_owned(),
                    Style::default().fg(palette.highlight),
                ),
                Span::styled(
                    "_",
                    Style::default()
                        .fg(palette.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
            text_area,
        );
        return;
    }

    let item_count = playlists.len() + 1;
    let w = area.width.clamp(34, 50);
    let h = (item_count as u16 + 4).clamp(5, area.height);
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let panel = Rect::new(x, y, w, h);

    frame.render_widget(Clear, panel);

    let block = Block::default()
        .title_top(
            Line::from(Span::styled(
                t("modal.playlist_picker.title"),
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        )
        .title_bottom(
            Line::from(Span::styled(
                t("modal.playlist_picker.hint"),
                Style::default().fg(palette.muted),
            ))
            .alignment(Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accent))
        .style(Style::default().bg(palette.panel_bg));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    for i in 0..item_count {
        let row_y = inner.y + 1 + i as u16;
        if row_y >= inner.bottom() {
            break;
        }
        let active = i == picker.selected;
        let marker = if active { ">" } else { " " };
        let style = if active {
            Style::default()
                .fg(palette.playing)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(palette.highlight)
        };
        let label = match playlists.get(i) {
            Some(playlist) => format!(
                " {marker} {} ({})",
                crate::ui::strings::title_case(&playlist.name),
                playlist.stations.len()
            ),
            None => format!(" {marker} {}", t("modal.playlist_picker.new")),
        };
        frame.render_widget(
            Paragraph::new(Span::styled(label, style)).style(Style::default().bg(palette.panel_bg)),
            Rect::new(inner.x + 1, row_y, inner.width.saturating_sub(2), 1),
        );
    }
}

pub(super) fn render_client_id_overlay(frame: &mut Frame, input: &str, palette: &Palette) {
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
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        )
        .title_bottom(
            Line::from(Span::styled(
                t("modal.client_id.hint"),
                Style::default().fg(palette.muted),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accent))
        .style(Style::default().bg(palette.panel_bg));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let text_area =
        ratatui::layout::Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(input.to_owned(), Style::default().fg(palette.highlight)),
            Span::styled(
                "_",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        text_area,
    );
}

pub(super) fn render_cookies_path_overlay(
    frame: &mut Frame,
    input: &str,
    error: Option<&str>,
    palette: &Palette,
) {
    let area = frame.area();
    let w = area.width.clamp(50, 80);
    let h: u16 = 5;
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let panel = ratatui::layout::Rect::new(x, y, w, h);

    frame.render_widget(Clear, panel);

    let (hint, hint_style) = match error {
        Some(message) => (message.to_string(), Style::default().fg(palette.danger)),
        None => (
            t("modal.cookies_path.hint"),
            Style::default().fg(palette.muted),
        ),
    };

    let block = Block::default()
        .title_top(
            Line::from(Span::styled(
                t("modal.cookies_path.title"),
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(ratatui::layout::Alignment::Center),
        )
        .title_bottom(
            Line::from(Span::styled(hint, hint_style))
                .alignment(ratatui::layout::Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accent))
        .style(Style::default().bg(palette.panel_bg));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let text_area =
        ratatui::layout::Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), 1);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(input.to_owned(), Style::default().fg(palette.highlight)),
            Span::styled(
                "_",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        text_area,
    );
}

pub(super) fn render_theme_picker_overlay(
    frame: &mut Frame,
    current: ThemeId,
    selected: usize,
    palette: &Palette,
) {
    let themes = ThemeId::all();
    let area = frame.area();
    let w = area.width.clamp(34, 46);
    let h = (themes.len() as u16 + 4).clamp(5, area.height);
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let panel = Rect::new(x, y, w, h);

    frame.render_widget(Clear, panel);

    let block = Block::default()
        .title_top(
            Line::from(Span::styled(
                t("theme.picker.title"),
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        )
        .title_bottom(
            Line::from(Span::styled(
                t("theme.picker.hint"),
                Style::default().fg(palette.muted),
            ))
            .alignment(Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accent))
        .style(Style::default().bg(palette.panel_bg));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    for (i, theme) in themes.iter().copied().enumerate() {
        let y = inner.y + 1 + i as u16;
        if y >= inner.bottom() {
            break;
        }
        let active = i == selected;
        let is_current = theme == current;
        let marker = if active { ">" } else { " " };
        let current_marker = if is_current { "*" } else { " " };
        let style = if active {
            Style::default()
                .fg(palette.playing)
                .add_modifier(Modifier::BOLD)
        } else if is_current {
            Style::default().fg(palette.accent)
        } else {
            Style::default().fg(palette.highlight)
        };
        let label = format!(" {marker} {current_marker} {}", theme.display());
        frame.render_widget(
            Paragraph::new(Span::styled(label, style)).style(Style::default().bg(palette.panel_bg)),
            Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1),
        );
    }
}

pub(super) fn render_game_strip(
    frame: &mut Frame,
    area: Rect,
    name: &str,
    genre: &str,
    border_tick: u32,
    palette: &Palette,
) {
    const H_PAD: u16 = 2;

    let border_color = theme::border_color_for(palette, border_tick);
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
        .style(Style::default().bg(palette.panel_bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cx = inner.x + H_PAD;
    let cw = inner.width.saturating_sub(H_PAD * 2);
    let mut spans: Vec<Span<'static>> =
        vec![Span::styled(name.to_owned(), theme::playing_style(palette))];
    if !genre.is_empty() {
        spans.push(Span::styled("  ·  ", Style::default().fg(palette.muted)));
        spans.push(Span::styled(
            genre.to_owned(),
            Style::default().fg(palette.dim),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(palette.panel_bg)),
        Rect::new(cx, inner.y, cw, 1),
    );
}

pub(super) fn render_modal_np_strip(
    frame: &mut Frame,
    strip: Rect,
    state: &PlayerState,
    border_tick: u32,
    palette: &Palette,
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
    let title_lines = wrap_into_lines(&raw_title, content_w, 2, palette);
    let title_line_count = title_lines.len() as u16;
    let has_progress = state.playback_pos_secs.is_some();
    let panel_h = 2 + 1 + title_line_count + 1;
    if panel_h > strip.height {
        return;
    }

    let panel = Rect::new(strip.x, strip.y, strip.width, panel_h);

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
        .style(Style::default().bg(palette.panel_bg));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx = inner.x + H_PAD;
    let cw = inner.width.saturating_sub(H_PAD * 2);
    let station_line = build_modal_station_line(state, palette);
    frame.render_widget(
        Paragraph::new(station_line).style(Style::default().bg(palette.panel_bg)),
        Rect::new(cx, inner.y, cw, 1),
    );
    for (i, tline) in title_lines.into_iter().enumerate() {
        let row_y = inner.y + 1 + i as u16;
        if row_y < inner.bottom() {
            frame.render_widget(
                Paragraph::new(tline).style(Style::default().bg(palette.panel_bg)),
                Rect::new(cx, row_y, cw, 1),
            );
        }
    }
    let viz_row = inner.bottom().saturating_sub(1);
    if viz_row >= inner.y && viz_row < inner.bottom() {
        let vol_pct = (state.volume.clamp(0.0, 1.0) * 100.0).round() as u32;
        let vol_color = if state.volume > 0.85 {
            palette.warning
        } else {
            palette.accent
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
        let viz_source = crate::ui::widgets::visualizer::AudioSource::Live(state.level_db);
        let mut spans = crate::ui::widgets::visualizer::visualizer_spans(
            viz_source,
            viz_w,
            palette.panel_bg,
            palette,
        );
        if let Some(text) = progress_text {
            if !spans.is_empty() {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                text,
                Style::default().fg(palette.muted).bg(palette.panel_bg),
            ));
        }
        spans.push(Span::raw(vol_prefix));
        spans.push(Span::styled(filled, Style::default().fg(vol_color)));
        spans.push(Span::styled(empty, Style::default().fg(palette.muted)));
        spans.push(Span::styled(pct_str, Style::default().fg(vol_color)));
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(palette.panel_bg)),
            Rect::new(cx, viz_row, cw, 1),
        );
    }
}

pub(crate) fn volume_bar_spans(vol: f32, bar_width: usize) -> (String, String) {
    let filled = (vol.clamp(0.0, 1.0) * bar_width as f32).round() as usize;
    let filled = filled.min(bar_width);
    ("█".repeat(filled), "░".repeat(bar_width - filled))
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

pub(super) fn build_modal_station_line(state: &PlayerState, palette: &Palette) -> Line<'static> {
    let name = state
        .station
        .as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_default();
    match &state.status {
        PlayerStatus::Connecting | PlayerStatus::Reconnecting(_) => Line::from(vec![
            Span::styled("…  ", Style::default().fg(palette.accent)),
            Span::styled(name, Style::default().fg(palette.muted)),
        ]),
        PlayerStatus::Buffering(_) | PlayerStatus::Playing | PlayerStatus::Paused => {
            let icon = if matches!(state.status, PlayerStatus::Paused) {
                "⏸  "
            } else {
                "▶  "
            };
            Line::from(vec![
                Span::styled(icon, Style::default().fg(palette.accent)),
                Span::styled(name, theme::playing_style(palette)),
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
    palette: &Palette,
) {
    use crate::app::SpotifyPlayerStatus;
    const H_PAD: u16 = 2;

    let (is_playing, is_paused, is_loading) = if let Some(pb) = playback {
        (pb.is_playing, !pb.is_playing, false)
    } else {
        (
            matches!(player_status, SpotifyPlayerStatus::Playing),
            matches!(player_status, SpotifyPlayerStatus::Paused),
            matches!(player_status, SpotifyPlayerStatus::Loading),
        )
    };
    if !is_playing && !is_paused && !is_loading {
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
    let title_lines = wrap_into_lines(&track_meta, content_w, 2, palette);

    let panel_h = 2 + 1 + title_lines.len() as u16 + u16::from(has_progress) + 1;
    if panel_h > strip.height {
        return;
    }

    let panel = Rect::new(strip.x, strip.y, strip.width, panel_h);

    let border_color = if is_playing {
        theme::border_color_for(palette, border_tick)
    } else if is_paused {
        palette.warning
    } else {
        palette.buffering
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(palette.panel_bg));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx = inner.x + H_PAD;
    let cw = inner.width.saturating_sub(H_PAD * 2);
    let icon = if is_playing {
        "▶  "
    } else if is_paused {
        "⏸  "
    } else {
        "…  "
    };

    let artist_display = crate::ui::strings::truncate(artist, cw.saturating_sub(3) as usize);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(icon, Style::default().fg(palette.playing)),
            Span::styled(artist_display, theme::playing_style(palette)),
        ]))
        .style(Style::default().bg(palette.panel_bg)),
        Rect::new(cx, inner.y, cw, 1),
    );

    let mut last_row = inner.y;
    for (i, tline) in title_lines.into_iter().enumerate() {
        let row_y = inner.y + 1 + i as u16;
        if row_y < inner.bottom() {
            frame.render_widget(
                Paragraph::new(tline).style(Style::default().bg(palette.panel_bg)),
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
                    Span::styled(prefix, ratatui::style::Style::default().fg(palette.muted)),
                    Span::styled(
                        "█".repeat(filled),
                        ratatui::style::Style::default().fg(border_color),
                    ),
                    Span::styled(
                        "░".repeat(empty),
                        ratatui::style::Style::default().fg(palette.muted),
                    ),
                    Span::styled(suffix, ratatui::style::Style::default().fg(palette.muted)),
                ]))
                .style(ratatui::style::Style::default().bg(palette.panel_bg)),
                Rect::new(cx, prog_y, cw, 1),
            );
        }
    }
    let viz_row = inner.bottom().saturating_sub(1);
    if viz_row >= inner.y && viz_row < inner.bottom() {
        let vol_color = palette.playing;
        let vol_w = if let Some(v) = volume_pct {
            let (f, e) = volume_bar_spans(v as f32 / 100.0, 8);
            2 + f.chars().count() + e.chars().count() + format!("  {:>3}%", v).len()
        } else {
            0
        };
        let viz_w = (cw as usize).saturating_sub(vol_w + 1);
        let viz_source = crate::ui::widgets::visualizer::AudioSource::Simulated;
        let mut spans = crate::ui::widgets::visualizer::visualizer_spans(
            viz_source,
            viz_w,
            palette.panel_bg,
            palette,
        );
        if let Some(v) = volume_pct {
            let (filled, empty) = volume_bar_spans(v as f32 / 100.0, 8);
            spans.push(Span::raw("  "));
            spans.push(Span::styled(filled, Style::default().fg(vol_color)));
            spans.push(Span::styled(empty, Style::default().fg(palette.muted)));
            spans.push(Span::styled(
                format!("  {:>3}%", v),
                Style::default().fg(vol_color),
            ));
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(palette.panel_bg)),
            Rect::new(cx, viz_row, cw, 1),
        );
    }
}

pub(super) fn render_update_toast(
    frame: &mut Frame,
    version: &str,
    is_ready: bool,
    is_modal_open: bool,
    tick: u32,
    full_area: Rect,
    palette: &Palette,
) {
    use ratatui::widgets::{Block, BorderType, Borders};

    let (text, border_color) = if is_ready {
        (
            format!(" [i] v{} Ready (Restart app) ", version),
            palette.playing,
        )
    } else {
        let spinners = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let s = spinners[(tick as usize / 2) % spinners.len()];
        (format!(" {} Downloading v{} ", s, version), palette.warning)
    };

    let w = text.chars().count() as u16 + 2;
    let h = 3;

    let x = if is_modal_open {
        let modal_area = crate::ui::widgets::search_modal::modal_rect(full_area);
        modal_area.right().saturating_sub(w + 1)
    } else {
        full_area.right().saturating_sub(w + 2)
    };

    let y = if is_modal_open {
        let modal_area = crate::ui::widgets::search_modal::modal_rect(full_area);
        modal_area.y.saturating_sub(1)
    } else {
        full_area.y + 1
    };

    let area = Rect::new(x, y, w, h);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(palette.panel_bg));

    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(palette.highlight)),
        area,
    );
}

enum HelpRow {
    Header(String),
    Entry(&'static str, String),
}

fn help_header(key: &str) -> HelpRow {
    HelpRow::Header(t(key))
}

fn help_rows(
    mode: &crate::app::SearchMode,
    spotify_logged_in: bool,
    spotify_can_cycle_device: bool,
) -> Vec<HelpRow> {
    use crate::app::SearchMode;
    use HelpRow::Entry;

    let global = || {
        vec![
            help_header("help.group.global"),
            Entry("[Tab]", t("help.shortcut.switch_source")),
            Entry("[Alt+O]", t("help.shortcut.open_config")),
            Entry("[Esc]", t("help.shortcut.close_quit")),
        ]
    };

    match mode {
        SearchMode::Name => {
            let mut rows = vec![
                help_header("help.group.radio"),
                Entry("[↵]", t("help.shortcut.play_station")),
                Entry("[↑↓]", t("help.shortcut.nav_list")),
                Entry("[Alt+F]", t("help.shortcut.save_fav")),
                Entry("[Alt+P]", t("help.shortcut.add_playlist")),
                Entry("[Ctrl+Shift+←→]", t("help.shortcut.playlist_jump")),
                Entry("[Space]", t("help.shortcut.pause_resume")),
                Entry("[Alt+R]", t("help.shortcut.random_station")),
                Entry("[Alt+S]", t("help.shortcut.stop_radio")),
                Entry("[Alt+G]", t("help.shortcut.by_genre")),
                Entry("[Alt+C]", t("help.shortcut.by_country")),
            ];
            rows.extend(global());
            rows
        }
        SearchMode::Spotify => {
            let mut rows = vec![help_header("help.group.spotify")];
            if spotify_logged_in {
                rows.extend([
                    Entry("[←→]", t("help.shortcut.switch_subtab")),
                    Entry("[↵]", t("help.shortcut.transfer_play")),
                    Entry("[↑↓]", t("help.shortcut.navigate")),
                    Entry("[Space]", t("help.shortcut.pause_resume")),
                    Entry("[Alt+D]", t("integrations.spotify.hint_disconnect")),
                    Entry("[Alt+R]", t("help.shortcut.reload_devices")),
                ]);
                if spotify_can_cycle_device {
                    rows.push(Entry("[Ctrl+D]", t("help.shortcut.switch_device")));
                }
            } else {
                rows.push(Entry("[↵]", t("help.shortcut.connect_spotify")));
            }
            rows.extend(global());
            rows
        }
        SearchMode::Youtube => {
            let mut rows = vec![
                help_header("help.group.youtube"),
                Entry("[↵]", t("help.shortcut.play_video")),
                Entry("[↑↓]", t("help.shortcut.nav_list")),
                Entry("[←→]", t("help.shortcut.switch_subtab")),
                Entry("[Ctrl+R]", t("help.shortcut.youtube_mix")),
                Entry("[Alt+F]", t("help.shortcut.toggle_bookmark")),
                Entry("[Space]", t("help.shortcut.pause_resume")),
                Entry("[Alt+S]", t("help.shortcut.stop_playback")),
            ];
            rows.extend(global());
            rows
        }
        SearchMode::Settings => vec![
            help_header("help.group.settings"),
            Entry("[Space]", t("help.shortcut.change_value")),
            Entry("[↑↓]", t("help.shortcut.nav_options")),
            Entry("[Esc]", t("hint.back")),
        ],
        _ => vec![
            Entry("[↑↓]", t("help.shortcut.navigate")),
            Entry("[↵]", t("help.shortcut.confirm")),
            Entry("[Esc]", t("hint.back")),
        ],
    }
}

pub(super) fn render_help_overlay(
    frame: &mut Frame,
    mode: &crate::app::SearchMode,
    spotify_logged_in: bool,
    spotify_can_cycle_device: bool,
    update_available: Option<&str>,
    palette: &Palette,
) {
    let rows = help_rows(mode, spotify_logged_in, spotify_can_cycle_device);

    const CREDITS: &[&str] = &["Esteban Jaramillo — Chile", "github.com/sewandev/Reverbic"];

    let area = frame.area();
    let w = 46u16.min(area.width);
    let line_count = rows.len();
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
        .border_style(Style::default().fg(palette.accent))
        .style(Style::default().bg(palette.panel_bg));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    for (i, row) in rows.into_iter().enumerate() {
        let row_y = inner.y + i as u16;
        if row_y >= inner.bottom() {
            break;
        }
        let line = match row {
            HelpRow::Header(title) => Line::from(Span::styled(
                title,
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            HelpRow::Entry(key_str, desc) => Line::from(vec![
                Span::styled(
                    format!("  {:9}", key_str),
                    Style::default()
                        .fg(palette.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(desc, Style::default().fg(palette.highlight)),
            ]),
        };
        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(palette.panel_bg)),
            ratatui::layout::Rect::new(inner.x, row_y, inner.width, 1),
        );
    }
    let sep_y = inner.y + line_count as u16 + 1;
    if sep_y < inner.bottom() {
        let sep = "─".repeat(inner.width as usize);
        frame.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(palette.dim)))
                .style(Style::default().bg(palette.panel_bg)),
            ratatui::layout::Rect::new(inner.x, sep_y, inner.width, 1),
        );
    }

    if let Some(version) = update_available {
        let row_y = sep_y + 1;
        if row_y < inner.bottom() {
            let notice = format!("  [i] Update v{version} is ready. Restart Reverbic to apply.");
            frame.render_widget(
                Paragraph::new(Span::styled(
                    notice,
                    Style::default()
                        .fg(palette.playing)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(palette.panel_bg)),
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
            Paragraph::new(Span::styled(*line, Style::default().fg(palette.muted)))
                .style(Style::default().bg(palette.panel_bg)),
            ratatui::layout::Rect::new(inner.x + 2, row_y, inner.width.saturating_sub(2), 1),
        );
    }
}

pub(super) fn wrap_into_lines(
    text: &str,
    width: usize,
    max_lines: usize,
    palette: &Palette,
) -> Vec<Line<'static>> {
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
            Style::default().fg(palette.highlight),
        )));
        offset = end;
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::{fmt_secs, playback_progress_text};
    use crate::audio::{PlayerState, PlayerStatus};

    #[test]
    fn formats_seconds_as_minutes_and_seconds() {
        assert_eq!(fmt_secs(65.0), "1:05");
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
}
