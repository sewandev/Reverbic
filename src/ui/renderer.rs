use chrono::{Datelike, Local, Timelike};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, AppFocus};
use crate::audio::{PlayerState, PlayerStatus};
use crate::i18n::t;
use crate::ui::{
    theme,
    widgets::{
        countdown::CountdownWidget,
        now_playing::NowPlayingWidget,
        on_demand_panel::OnDemandPanelWidget,
        recent_tracks::RecentTracksWidget,
        saved_tracks::SavedTracksWidget,
        station_list::StationListWidget,
        vu_meter::VuMeterWidget,
    },
};

pub fn now_playing_rect(
    area: Rect,
    has_recent: bool,
    has_saved: bool,
    show_countdown: bool,
    has_on_demand: bool,
) -> Option<Rect> {
    compute_layout(area, has_recent, has_saved, show_countdown, has_on_demand).now_playing
}

pub fn render(frame: &mut Frame, app: &App) {
    let player_state = app.player_state();

    let playing_index = player_state
        .station
        .as_ref()
        .and_then(|p| app.stations.iter().position(|s| s.url == p.url));

    let playing_dynamic_index = player_state
        .station
        .as_ref()
        .and_then(|p| app.search_results.iter().position(|s| s.url == p.url));

    let playing_favorite_index = player_state
        .station
        .as_ref()
        .and_then(|p| app.favorites.iter().position(|f| f.url == p.url));

    let playing_ondemand_id: Option<&str> = player_state
        .station
        .as_ref()
        .and_then(|s| s.key.strip_prefix("ondemand_"));

    let has_recent    = !player_state.recent_titles.is_empty();
    let has_saved     = !app.saved_tracks.is_empty();
    let has_on_demand = !app.on_demand_shows.is_empty() || app.on_demand_loading;
    let show_countdown = player_state.station.as_ref().map(|s| s.show_countdown).unwrap_or(false);

    let layout = compute_layout(frame.area(), has_recent, has_saved, show_countdown, has_on_demand);

    if let Some(h) = layout.header {
        render_header(frame, h);
    }
    if let Some(s) = layout.sep_header {
        render_sep(frame, s);
    }

    frame.render_widget(
        StationListWidget {
            stations:               &app.stations,
            dynamic_stations:       &app.search_results,
            favorites:              &app.favorites,
            selected:               app.selected,
            playing_index,
            playing_dynamic_index,
            playing_favorite_index,
            player_status:          &player_state.status,
            search_query:           if app.show_search_modal { "" } else { &app.search_query },
            search_loading:         !app.show_search_modal && app.search_loading,
            is_searching:           !app.show_search_modal && matches!(app.focus, AppFocus::StationSearch),
            flash_index:            app.click_flash.and_then(|(i, t)| {
                if t.elapsed().as_millis() < 300 { Some(i) } else { None }
            }),
        },
        layout.stations,
    );

    if let Some(r) = layout.on_demand {
        let focused = matches!(app.focus, AppFocus::OnDemandList);
        frame.render_widget(
            OnDemandPanelWidget {
                shows:        &app.on_demand_shows,
                selected:     app.on_demand_selected,
                focused,
                loading:      app.on_demand_loading,
                playing_id:   playing_ondemand_id,
                program_name: crate::station::on_demand::PROGRAMS
                    .get(app.selected_program)
                    .map(|p| p.name)
                    .unwrap_or("Shows"),
            },
            r,
        );
    }

    if let Some(r) = layout.saved_tracks {
        let station_name = player_state.station.as_ref().map(|s| s.name.as_str());
        frame.render_widget(
            SavedTracksWidget { tracks: &app.saved_tracks, station_name },
            r,
        );
    }

    if let Some(r) = layout.recent_tracks {
        let focused = matches!(app.focus, AppFocus::RecentTracks);
        frame.render_widget(
            RecentTracksWidget {
                tracks:                &player_state.recent_titles,
                selected:              app.recent_selected,
                focused,
                preview_active:        player_state.preview_title.is_some(),
                preview_loading_track: player_state.preview_loading_track.as_deref(),
                preview_playing_track: player_state.preview_playing_track.as_deref(),
                preview_unavailable:   &player_state.preview_unavailable,
            },
            r,
        );
    }

    if let Some(s) = layout.sep_body {
        render_sep(frame, s);
    }

    if let Some(r) = layout.now_playing {
        if let Some((ref name, ref genre)) = crate::game_detect::get() {
            let area  = frame.area();
            let pw    = area.width.min(66).max(44);
            let px    = area.x + area.width.saturating_sub(pw) / 2;
            if r.y >= 1 {
                render_game_inline(frame, Rect::new(px, r.y, pw, 1), name, genre);
            }
        }
        frame.render_widget(NowPlayingWidget { state: &player_state }, r);
    }

    if let Some(r) = layout.vu {
        let buffer_fill_pct = if let PlayerStatus::Buffering(pct) = player_state.status {
            Some(pct)
        } else {
            None
        };
        frame.render_widget(
            VuMeterWidget {
                level_db:        player_state.level_db,
                volume:          player_state.volume,
                buffer_fill_pct,
            },
            r,
        );
    }

    if let Some(s) = layout.sep_footer {
        render_sep(frame, s);
    }

    if let Some(r) = layout.countdown {
        frame.render_widget(CountdownWidget, r);
    }

    render_help(
        frame,
        layout.help,
        &player_state.status,
        &app.focus,
        app.save_notice.as_deref(),
        app.save_notice_is_dup,
        player_state.preview_title.as_deref(),
        player_state.preview_searching,
        &app.seek_input,
    );

    if app.show_search_modal && app.screensaver_active() {
        let is_fav = player_state.station.as_ref()
            .map(|s| app.favorites.iter().any(|f| f.url == s.url))
            .unwrap_or(false);
        render_screensaver(frame, frame.area(), &player_state, app.station_details.as_ref(), is_fav);
        return;
    }

    if app.show_search_modal {
        use crate::ui::widgets::search_modal::SearchModalWidget;
        let full_area = frame.area();
        frame.render_widget(
            SearchModalWidget {
                query:             &app.search_query,
                results:           &app.search_results,
                loading:           app.search_loading,
                selected:          app.modal_selected,
                mode:              &app.modal_mode,
                genre_selected:    app.genre_selected,
                genre_filter:      &app.genre_filter,
                genre_query:       &app.genre_query,
                country_selected:  app.country_selected,
                country_filter:    &app.country_filter,
                settings_selected:  app.settings_selected,
                autoplay_last:      app.config.autoplay_last,
                overlay_mode:       app.config.overlay_mode.display(),
                overlay_position:   app.config.overlay_position.display(),
                crossfade:          app.config.crossfade_display(),
                media_keys:         app.config.media_keys,
                tray_icon:          app.config.tray_icon,
                notifications:      app.config.notifications,
                restore_volume:     app.config.restore_volume,
                duck_enabled:       app.config.duck_enabled,
                duck_volume:               app.config.duck_volume,
                overlay_alpha:             app.config.overlay_alpha,
                screensaver_secs:          app.config.screensaver_secs,
            },
            full_area,
        );

        let modal_w = full_area.width.min(66).max(44);
        let modal_h = full_area.height.min(14).max(10);
        let modal_x = full_area.x + full_area.width.saturating_sub(modal_w) / 2;
        let modal_y = full_area.y + full_area.height.saturating_sub(modal_h) / 2;
        if let Some((ref name, ref genre)) = crate::game_detect::get() {
            let panel_h: u16 = 3;
            let game_y = modal_y.saturating_sub(panel_h);
            render_game_strip(frame, Rect::new(modal_x, game_y, modal_w, panel_h), name, genre);
        }

        let strip_y     = modal_y + modal_h;
        let remaining_h = full_area.bottom().saturating_sub(strip_y);
        if remaining_h >= 3 {
            let strip = Rect::new(modal_x, strip_y, modal_w, remaining_h);
            render_modal_np_strip(frame, strip, &player_state);
        }
    }

    if let Some(_) = app.renaming_favorite {
        render_rename_overlay(frame, &app.rename_input);
    }

}

const HEIGHT_NORMAL:  u16 = 11;
const HEIGHT_COMPACT: u16 = 5;

struct AppLayout {
    header:       Option<Rect>,
    sep_header:   Option<Rect>,
    stations:     Rect,
    on_demand:    Option<Rect>,
    saved_tracks: Option<Rect>,
    recent_tracks: Option<Rect>,
    sep_body:     Option<Rect>,
    now_playing:  Option<Rect>,
    vu:           Option<Rect>,
    sep_footer:   Option<Rect>,
    countdown:    Option<Rect>,
    help:         Rect,
}

fn compute_layout(
    area:          Rect,
    has_recent:    bool,
    has_saved:     bool,
    show_countdown: bool,
    has_on_demand: bool,
) -> AppLayout {
    let countdown_h: u16 = u16::from(show_countdown);

    if area.height >= HEIGHT_NORMAL + countdown_h {
        let rows = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(countdown_h),
            Constraint::Length(1),
        ])
        .split(area);

        let countdown = if show_countdown { Some(rows[7]) } else { None };
        let (stations, on_demand, saved_tracks, recent_tracks) =
            build_columns(rows[2], has_on_demand, has_recent, has_saved);

        AppLayout {
            header:       Some(rows[0]),
            sep_header:   Some(rows[1]),
            stations,
            on_demand,
            saved_tracks,
            recent_tracks,
            sep_body:     Some(rows[3]),
            now_playing:  Some(rows[4]),
            vu:           Some(rows[5]),
            sep_footer:   Some(rows[6]),
            countdown,
            help:         rows[8],
        }
    } else if area.height >= HEIGHT_COMPACT {
        let rows = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

        let (stations, on_demand, saved_tracks, recent_tracks) =
            build_columns(rows[0], has_on_demand, has_recent, has_saved);

        AppLayout {
            header: None, sep_header: None,
            stations, on_demand, saved_tracks, recent_tracks,
            sep_body: None,
            now_playing:  Some(rows[1]),
            vu:           None,
            sep_footer:   None,
            countdown:    None,
            help:         rows[2],
        }
    } else {
        let rows = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);
        let (stations, on_demand, saved_tracks, recent_tracks) =
            build_columns(rows[0], false, false, false);

        AppLayout {
            header: None, sep_header: None,
            stations, on_demand, saved_tracks, recent_tracks,
            sep_body: None, now_playing: None, vu: None, sep_footer: None, countdown: None,
            help: rows[1],
        }
    }
}

fn build_columns(
    top:           Rect,
    has_on_demand: bool,
    has_recent:    bool,
    has_saved:     bool,
) -> (Rect, Option<Rect>, Option<Rect>, Option<Rect>) {
    if has_on_demand {
        let show_right = top.width >= 90 && (has_recent || has_saved);
        if show_right {
            let cols = Layout::horizontal([
                Constraint::Max(38),
                Constraint::Max(35),
                Constraint::Fill(1),
            ])
            .split(top);
            let (saved, recent) = split_saved_recent(cols[2], has_recent, has_saved);
            (cols[0], Some(cols[1]), saved, recent)
        } else {
            let cols = Layout::horizontal([Constraint::Max(38), Constraint::Fill(1)]).split(top);
            (cols[0], Some(cols[1]), None, None)
        }
    } else if has_recent || has_saved {
        let cols = Layout::horizontal([Constraint::Max(40), Constraint::Fill(1)]).split(top);
        let (saved, recent) = split_saved_recent(cols[1], has_recent, has_saved);
        (cols[0], None, saved, recent)
    } else {
        (top, None, None, None)
    }
}

fn split_saved_recent(
    right:      Rect,
    has_recent: bool,
    has_saved:  bool,
) -> (Option<Rect>, Option<Rect>) {
    match (has_saved, has_recent) {
        (true, true) => {
            let rows = Layout::vertical([
                Constraint::Percentage(58),
                Constraint::Percentage(42),
            ])
            .split(right);
            (Some(rows[0]), Some(rows[1]))
        }
        (true, false) => (Some(right), None),
        (false, true) => (None, Some(right)),
        (false, false) => (None, None),
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let now      = Local::now();
    let time_str = format!("{}  {:02} {}", now.format("%H:%M"), now.day(), month_es(now.month()));
    let brand    = " REVERBIC";
    let brand_w  = brand.chars().count();
    let time_w   = time_str.chars().count();
    let pad      = (area.width as usize).saturating_sub(brand_w + time_w + 1);

    let line = Line::from(vec![
        Span::styled(brand, Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(pad)),
        Span::styled(time_str, Style::new().fg(theme::MUTED)),
        Span::raw(" "),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_sep(frame: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(line, Style::default().fg(theme::MUTED))),
        area,
    );
}

fn render_help(
    frame:              &mut Frame,
    area:               Rect,
    status:             &PlayerStatus,
    focus:              &AppFocus,
    save_notice:        Option<&str>,
    save_notice_is_dup: bool,
    preview_title:      Option<&str>,
    preview_searching:  bool,
    seek_input:         &str,
) {
    let (text, color) = if let Some(title) = preview_title {
        (format!(" PREVIEW: {title}  {}", t("help.stop_preview")), theme::PLAYING)
    } else if preview_searching {
        (format!(" {}", t("help.searching_deezer")), theme::ACCENT)
    } else if let Some(msg) = save_notice {
        let color = if save_notice_is_dup { theme::ACCENT } else { theme::PLAYING };
        (format!(" {msg}"), color)
    } else {
        let hint = match focus {
            AppFocus::RecentTracks  => t("help.recent"),
            AppFocus::StationSearch => t("help.station_search"),
            AppFocus::OnDemandList  => {
                if !seek_input.is_empty() {
                    format!(" {}  {seek_input}_  {}", t("help.seek_prefix"), t("help.seek_suffix"))
                } else {
                    t("help.demand.hint")
                }
            }
            AppFocus::Stations => {
                let active = matches!(status, PlayerStatus::Playing | PlayerStatus::Paused);
                if active {
                    if matches!(status, PlayerStatus::Paused) {
                        t("help.stations_paused")
                    } else {
                        t("help.stations_playing")
                    }
                } else {
                    t("help.stations_idle")
                }
            }
        };
        (format!(" {hint}"), theme::MUTED)
    };

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text, Style::default().fg(color)))),
        area,
    );
}

fn render_rename_overlay(frame: &mut Frame, input: &str) {
    use ratatui::widgets::{Block, BorderType, Borders, Clear};
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
            Span::styled(input, Style::default().fg(theme::HIGHLIGHT)),
            Span::styled("_", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ])),
        text_area,
    );
}
fn render_screensaver(
    frame:       &mut Frame,
    area:        Rect,
    state:       &PlayerState,
    details:     Option<&crate::station::StationDetails>,
    is_favorite: bool,
) {
    use ratatui::{layout::Alignment, widgets::{Block, BorderType, Borders, Clear}};
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
        + 5 + 1            // reloj grande + vacío
        + 1                // nombre estación
        + 1                // título canción
        + 1                // vacío
        + 1                // visualizador
        + 1                // vacío
        + if has_two_col { 1 + two_col_rows } else { 0 }
        + 1                // vacío
        + 1;               // pie

    let pw    = area.width.min(72).max(50);
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

    macro_rules! put {
        ($line:expr) => {{
            frame.render_widget(
                Paragraph::new($line).style(Style::default().bg(BG)),
                Rect::new(cx, row, cw, 1),
            );
            row += 1;
        }};
    }

    // ── Reloj grande ───────────────────────────────────────────────────
    let now_t    = Local::now();
    let h1       = (now_t.hour()   / 10) as u8;
    let h2       = (now_t.hour()   % 10) as u8;
    let m1       = (now_t.minute() / 10) as u8;
    let m2       = (now_t.minute() % 10) as u8;
    let colon_on = now_t.second() % 2 == 0;
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

    // ── Nombre de estación (centrado, mayúsculas, ★ si favorita) ───────
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

    // ── Título de canción (centrado) ────────────────────────────────────
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

    // ── Visualizador con gradiente ──────────────────────────────────────
    row += 1;
    frame.render_widget(
        Paragraph::new(visualizer_line(state.level_db, cw as usize, BG))
            .style(Style::default().bg(BG)),
        Rect::new(cx, row, cw, 1),
    );
    row += 1;
    row += 1;

    // ── Separador + dos columnas ────────────────────────────────────────
    if has_two_col {
        let sep = "─".repeat(cw as usize);
        put!(Line::from(Span::styled(sep, Style::default().fg(theme::DIM))));

        let left_w  = cw.saturating_sub(2) / 2;
        let right_w = cw.saturating_sub(left_w + 2);
        let right_x = cx + left_w + 2;
        let col_top = row;
        let mut left_row  = col_top;
        let mut right_row = col_top;

        // Columna izquierda: metadata
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

        // Columna derecha: canciones recientes
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

    // ── Pie: prompt + hora + barra de volumen ───────────────────────────
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

fn render_game_inline(frame: &mut Frame, area: Rect, name: &str, genre: &str) {
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

fn render_game_strip(frame: &mut Frame, area: Rect, name: &str, genre: &str) {
    use ratatui::{layout::Alignment, style::Color, widgets::{Block, BorderType, Borders}};
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

fn render_modal_np_strip(frame: &mut Frame, strip: Rect, state: &PlayerState) {
    use ratatui::{layout::Alignment, style::Color, widgets::{Block, BorderType, Borders}};
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

    let panel_h = 2 + 1 + title_lines.len() as u16; // borders + station + title rows
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

fn build_modal_station_line(state: &PlayerState) -> Line<'static> {
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

fn wrap_into_lines(text: &str, width: usize, max_lines: usize) -> Vec<Line<'static>> {
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

fn month_es(m: u32) -> &'static str {
    ["ene","feb","mar","abr","may","jun","jul","ago","sep","oct","nov","dic"]
        [(m.saturating_sub(1) as usize).min(11)]
}
