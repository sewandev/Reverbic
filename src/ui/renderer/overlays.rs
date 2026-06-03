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
    let w     = area.width.clamp(30, 50);
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

pub(super) fn render_modal_spotify_strip(
    frame:         &mut Frame,
    strip:         Rect,
    playback:      Option<&crate::integrations::spotify::SpotifyPlaybackState>,
    now_playing:   Option<&crate::integrations::spotify::SpotifyTrack>,
    player_status: &crate::app::SpotifyPlayerStatus,
) {
    use ratatui::style::Color;
    use crate::app::SpotifyPlayerStatus;
    const STRIP_BG: Color = Color::Rgb(13, 13, 13);
    const H_PAD:    u16   = 2;

    let (is_playing, is_paused) = if let Some(pb) = playback {
        (pb.is_playing, !pb.is_playing)
    } else {
        (
            matches!(player_status, SpotifyPlayerStatus::Playing),
            matches!(player_status, SpotifyPlayerStatus::Paused),
        )
    };
    if !is_playing && !is_paused { return; }

    let (artist, track_name, album, volume_pct) = if let Some(pb) = playback {
        (pb.artist.as_str(), pb.track_name.as_str(), pb.album.as_str(), Some(pb.volume_pct))
    } else if let Some(np) = now_playing {
        (np.artist.as_str(), np.name.as_str(), np.album.as_str(), None)
    } else {
        return;
    };

    if artist.is_empty() && track_name.is_empty() { return; }

    let content_w = strip.width.saturating_sub(2 + H_PAD * 2) as usize;
    if content_w == 0 { return; }

    let track_meta = if album.is_empty() {
        track_name.to_owned()
    } else {
        format!("{track_name} · {album}")
    };
    let title_lines = wrap_into_lines(&track_meta, content_w, 2);

    let panel_h = 2 + 1 + title_lines.len() as u16;
    if panel_h > strip.height { return; }

    let panel = Rect::new(strip.x, strip.y, strip.width, panel_h);

    let vol_str = volume_pct
        .map(|v| format!(" {v}% "))
        .unwrap_or_default();

    let block = Block::default()
        .title_top(
            Line::from(Span::styled(vol_str, Style::default().fg(theme::ACCENT)))
                .alignment(Alignment::Right),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::MUTED))
        .style(Style::default().bg(STRIP_BG));

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx   = inner.x + H_PAD;
    let cw   = inner.width.saturating_sub(H_PAD * 2);
    let icon = if is_playing { "▶  " } else { "⏸  " };

    let artist_display = crate::ui::strings::truncate(artist, cw.saturating_sub(3) as usize);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(icon,           Style::default().fg(theme::PLAYING)),
            Span::styled(artist_display, theme::PLAYING_STYLE),
        ])).style(Style::default().bg(STRIP_BG)),
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

pub(super) fn render_help_overlay(frame: &mut Frame, mode: &crate::app::SearchMode, spotify_logged_in: bool) {
    use crate::app::SearchMode;

    let lines: &[(&str, &str)] = match mode {
        SearchMode::Name => &[
            ("[↵]",     "Reproducir estacion"),
            ("[↑↓]",   "Navegar lista"),
            ("[F]",     "Guardar en favoritas"),
            ("[R]",     "Estacion aleatoria"),
            ("[Tab]",   "Ir a Spotify"),
            ("[Alt+G]", "Buscar por Genero"),
            ("[Alt+C]", "Buscar por Pais"),
            ("[Alt+O]", "Abrir Configuracion"),
            ("[Esc]",   "Cerrar / Salir"),
        ],
        SearchMode::Spotify => {
            if spotify_logged_in {
                &[
                    ("[←→]",   "Cambiar sub-tab"),
                    ("[↵]",     "Transferir / Play"),
                    ("[↑↓]",   "Navegar"),
                    ("[Space]", "Pausar / Reanudar"),
                    ("[Alt+O]", "Configuracion"),
                    ("[Alt+D]", "Desconectar"),
                    ("[Alt+R]", "Recargar dispositivos"),
                    ("[Esc]",   "Cerrar"),
                ]
            } else {
                &[
                    ("[↵]",   "Conectar Spotify"),
                    ("[Tab]", "Ir a Radio"),
                    ("[Esc]", "Cerrar"),
                ]
            }
        }
        SearchMode::Settings => &[
            ("[Space]", "Cambiar valor"),
            ("[↑↓]",   "Navegar opciones"),
            ("[Esc]",   "Volver"),
        ],
        _ => &[
            ("[↑↓]",  "Navegar"),
            ("[↵]",   "Confirmar"),
            ("[Esc]", "Volver"),
        ],
    };

    const CREDITS: &[&str] = &[
        "Esteban Jaramillo — Chile",
        "github.com/sewandev/Reverbic",
    ];

    let area    = frame.area();
    let w       = 46u16.min(area.width);
    let h       = (lines.len() as u16 + 3 + CREDITS.len() as u16 + 2).min(area.height);
    let x       = area.x + area.width.saturating_sub(w) / 2;
    let y       = area.y + area.height.saturating_sub(h) / 2;
    let rect    = ratatui::layout::Rect::new(x, y, w, h);

    frame.render_widget(Clear, rect);
    let block = Block::default()
        .title(" Atajos ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::ACCENT))
        .style(Style::default().bg(theme::PANEL_BG));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    for (i, (key_str, desc)) in lines.iter().enumerate() {
        let row_y = inner.y + i as u16;
        if row_y >= inner.bottom() { break; }
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("  {:9}", key_str),
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(*desc, Style::default().fg(theme::HIGHLIGHT)),
            ]))
            .style(Style::default().bg(theme::PANEL_BG)),
            ratatui::layout::Rect::new(inner.x, row_y, inner.width, 1),
        );
    }
    let sep_y = inner.y + lines.len() as u16 + 1;
    if sep_y < inner.bottom() {
        let sep = "─".repeat(inner.width as usize);
        frame.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(theme::DIM)))
                .style(Style::default().bg(theme::PANEL_BG)),
            ratatui::layout::Rect::new(inner.x, sep_y, inner.width, 1),
        );
    }

    for (i, line) in CREDITS.iter().enumerate() {
        let row_y = sep_y + 1 + i as u16;
        if row_y >= inner.bottom() { break; }
        frame.render_widget(
            Paragraph::new(Span::styled(*line, Style::default().fg(theme::MUTED)))
                .style(Style::default().bg(theme::PANEL_BG)),
            ratatui::layout::Rect::new(inner.x + 2, row_y, inner.width.saturating_sub(2), 1),
        );
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
