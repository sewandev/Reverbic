use chrono::{Datelike, Local};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, AppFocus};
use crate::audio::PlayerStatus;
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

    let playing_ondemand_id: Option<String> = player_state
        .station
        .as_ref()
        .and_then(|s| s.key.strip_prefix("ondemand_"))
        .map(str::to_string);

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
                playing_id:   playing_ondemand_id.as_deref(),
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

    if app.show_search_modal {
        use crate::ui::widgets::search_modal::SearchModalWidget;
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
                history:           &app.config.search_history,
                settings_selected:  app.settings_selected,
                autoplay_last:      app.config.autoplay_last,
                overlay_mode:       app.config.overlay_mode.display(),
                crossfade:          app.config.crossfade_display(),   // String
                media_keys:         app.config.media_keys,
                tray_icon:          app.config.tray_icon,
                notifications:      app.config.notifications,
                trending_results:   &app.trending_results,
                trending_loading:   app.trending_loading,
                trending_selected:  app.trending_selected,
            },
            frame.area(),
        );
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
            Constraint::Length(1),              // header
            Constraint::Length(1),              // sep_header
            Constraint::Fill(1),                // content
            Constraint::Length(1),              // sep_body
            Constraint::Length(1),              // now_playing
            Constraint::Length(1),              // vu
            Constraint::Length(1),              // sep_footer
            Constraint::Length(countdown_h),    // countdown (0 si no hay)
            Constraint::Length(1),              // help
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
        .style(Style::default().bg(ratatui::style::Color::Rgb(13, 13, 13)));

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

fn month_es(m: u32) -> &'static str {
    ["ene","feb","mar","abr","may","jun","jul","ago","sep","oct","nov","dic"]
        [(m.saturating_sub(1) as usize).min(11)]
}
