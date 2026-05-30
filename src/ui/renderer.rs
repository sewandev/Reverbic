use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, AppFocus};
use crate::audio::PlayerStatus;
use crate::audio::PlayerState;
use crate::ui::{
    theme,
    widgets::{
        countdown::CountdownWidget,
        local_time::LocalTimeWidget,
        now_playing::NowPlayingWidget,
        now_playing_overlay::NowPlayingOverlayWidget,
        on_demand_panel::OnDemandPanelWidget,
        recent_tracks::RecentTracksWidget,
        saved_tracks::SavedTracksWidget,
        station_list::StationListWidget,
        vu_meter::VuMeterWidget,
    },
};
/// Devuelve el `Rect` del widget NOW PLAYING dados los parámetros de layout.
/// Usado por `App::on_click` para detectar clicks sobre la barra de progreso.
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
        .and_then(|playing| app.stations.iter().position(|s| s.url == playing.url));

    let playing_dynamic_index = if let Some(ref station) = player_state.station {
        app.search_results.iter().position(|s| s.url == station.url)
    } else {
        None
    };

    let playing_favorite_index = player_state
        .station
        .as_ref()
        .and_then(|playing| app.favorites.iter().position(|f| f.url == playing.url));

    // ID del show on-demand en reproducción (clave con prefijo "ondemand_")
    let playing_ondemand_id: Option<String> = player_state
        .station
        .as_ref()
        .and_then(|s| s.key.strip_prefix("ondemand_"))
        .map(str::to_string);

    let has_recent = !player_state.recent_titles.is_empty();
    let has_saved = !app.saved_tracks.is_empty();
    let has_on_demand = !app.on_demand_shows.is_empty() || app.on_demand_loading;
    let show_countdown = player_state
        .station
        .as_ref()
        .map(|s| s.show_countdown)
        .unwrap_or(false);
    let layout = compute_layout(frame.area(), has_recent, has_saved, show_countdown, has_on_demand);

    frame.render_widget(
        StationListWidget {
            stations:              &app.stations,
            dynamic_stations:      &app.search_results,
            favorites:             &app.favorites,
            selected:              app.selected,
            playing_index,
            playing_dynamic_index,
            playing_favorite_index,
            player_status:         &player_state.status,
            search_query:          &app.search_query,
            search_loading:        app.search_loading,
            is_searching:          matches!(app.focus, AppFocus::StationSearch),
        },
        layout.stations,
    );

    if let Some(od_area) = layout.on_demand {
        let focused = matches!(app.focus, AppFocus::OnDemandList);
        frame.render_widget(
            OnDemandPanelWidget {
                shows: &app.on_demand_shows,
                selected: app.on_demand_selected,
                focused,
                loading: app.on_demand_loading,
                playing_id: playing_ondemand_id.as_deref(),
                program_name: crate::station::on_demand::PROGRAMS
                    .get(app.selected_program)
                    .map(|p| p.name)
                    .unwrap_or("Shows"),
            },
            od_area,
        );
    }

    if let Some(saved_area) = layout.saved_tracks {
        let station_name = player_state.station.as_ref().map(|s| s.name.as_str());
        frame.render_widget(
            SavedTracksWidget {
                tracks: &app.saved_tracks,
                station_name,
            },
            saved_area,
        );
    }

    if let Some(recent_area) = layout.recent_tracks {
        let focused = matches!(app.focus, AppFocus::RecentTracks);
        frame.render_widget(
            RecentTracksWidget {
                tracks: &player_state.recent_titles,
                selected: app.recent_selected,
                focused,
                preview_active: player_state.preview_title.is_some(),
                preview_loading_track: player_state.preview_loading_track.as_deref(),
                preview_playing_track: player_state.preview_playing_track.as_deref(),
                preview_unavailable: &player_state.preview_unavailable,
            },
            recent_area,
        );
    }

    if let Some(now_playing_area) = layout.now_playing {
        frame.render_widget(
            NowPlayingWidget {
                state: &player_state,
            },
            now_playing_area,
        );
    }

    if let Some(countdown_area) = layout.countdown {
        frame.render_widget(CountdownWidget, countdown_area);
    }

    if let Some(audio_area) = layout.audio {
        let buffer_fill_pct = if let PlayerStatus::Buffering(pct) = player_state.status {
            Some(pct)
        } else {
            None
        };
        frame.render_widget(
            VuMeterWidget {
                level_db: player_state.level_db,
                volume: player_state.volume,
                buffer_fill_pct,
            },
            audio_area,
        );
    }

    let overlay_width = 23;
    let overlay_height = 1;
    let overlay_x = frame.area().width.saturating_sub(overlay_width);
    let overlay_y = 0;
    if overlay_x > 1 && overlay_height > 0 {
        let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);
        frame.render_widget(LocalTimeWidget::new(), overlay_area);
    }

    render_help(
        frame,
        layout.help,
        &player_state.status,
        &app.focus,
        app.save_notice.as_deref(),
        player_state.preview_title.as_deref(),
        player_state.preview_searching,
        &app.seek_input,
    );

    render_now_playing_overlay(frame, app, &player_state);

    // Panel de configuración — overlay modal por encima de todo lo demás
    if app.show_settings {
        use crate::ui::widgets::settings_panel::{SettingsItem, SettingsPanelWidget};
        let items = [SettingsItem {
            label: "Auto-play última radio al iniciar",
            value: app.config.autoplay_last,
        }];
        frame.render_widget(
            SettingsPanelWidget { items: &items, selected: app.settings_selected },
            frame.area(),
        );
    }
}

const HEIGHT_NORMAL: u16 = 19; // 5 top mínimo + 5 now_playing + 3 countdown + 4 audio + 2 help
const HEIGHT_COMPACT: u16 = 10;
const HEIGHT_NOW_PLAYING_NORMAL: u16 = 5;
const HEIGHT_NOW_PLAYING_COMPACT: u16 = 3;
const HEIGHT_COUNTDOWN: u16 = 3;
const HEIGHT_AUDIO: u16 = 4;
const HEIGHT_AUDIO_COMPACT: u16 = 3;
const HEIGHT_HELP: u16 = 2;
const HEIGHT_HELP_MINIMAL: u16 = 1;

struct AppLayout {
    stations:     Rect,
    on_demand:    Option<Rect>,
    saved_tracks: Option<Rect>,
    recent_tracks: Option<Rect>,
    now_playing:  Option<Rect>,
    countdown:    Option<Rect>,
    audio:        Option<Rect>,
    help:         Rect,
}

// Reparte el área horizontal entre estaciones, panel on-demand (opcional) y columna derecha.
// Cuando hay on_demand, la columna derecha solo se muestra si hay espacio suficiente (>= 90 cols).
fn build_columns(
    top: Rect,
    has_on_demand: bool,
    has_recent: bool,
    has_saved: bool,
) -> (Rect, Option<Rect>, Option<Rect>, Option<Rect>) {
    if has_on_demand {
        // Muestra columna derecha solo cuando el terminal es suficientemente ancho
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
    right: Rect,
    has_recent: bool,
    has_saved: bool,
) -> (Option<Rect>, Option<Rect>) {
    match (has_saved, has_recent) {
        (true, true) => {
            let rows = Layout::vertical([Constraint::Percentage(58), Constraint::Percentage(42)])
                .split(right);
            (Some(rows[0]), Some(rows[1]))
        }
        (true, false) => (Some(right), None),
        (false, true) => (None, Some(right)),
        (false, false) => (None, None),
    }
}

fn compute_layout(
    area: Rect,
    has_recent: bool,
    has_saved: bool,
    show_countdown: bool,
    has_on_demand: bool,
) -> AppLayout {
    if area.height >= HEIGHT_NORMAL {
        let (chunks, countdown_slot) = if show_countdown {
            let c = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(HEIGHT_NOW_PLAYING_NORMAL),
                Constraint::Length(HEIGHT_COUNTDOWN),
                Constraint::Length(HEIGHT_AUDIO),
                Constraint::Length(HEIGHT_HELP),
            ])
            .split(area);
            let slot = c[2];
            (c, Some(slot))
        } else {
            let c = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(HEIGHT_NOW_PLAYING_NORMAL),
                Constraint::Length(HEIGHT_AUDIO),
                Constraint::Length(HEIGHT_HELP),
            ])
            .split(area);
            (c, None)
        };
        let (audio_idx, help_idx) = if show_countdown { (3, 4) } else { (2, 3) };

        let (stations, on_demand, saved_tracks, recent_tracks) =
            build_columns(chunks[0], has_on_demand, has_recent, has_saved);

        AppLayout {
            stations,
            on_demand,
            saved_tracks,
            recent_tracks,
            now_playing: Some(chunks[1]),
            countdown: countdown_slot,
            audio: Some(chunks[audio_idx]),
            help: chunks[help_idx],
        }
    } else if area.height >= HEIGHT_COMPACT {
        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(HEIGHT_NOW_PLAYING_COMPACT),
            Constraint::Length(HEIGHT_AUDIO_COMPACT),
            Constraint::Length(HEIGHT_HELP),
        ])
        .split(area);

        let (stations, on_demand, saved_tracks, recent_tracks) =
            build_columns(chunks[0], has_on_demand, has_recent, has_saved);

        AppLayout {
            stations,
            on_demand,
            saved_tracks,
            recent_tracks,
            now_playing: Some(chunks[1]),
            countdown: None,
            audio: Some(chunks[2]),
            help: chunks[3],
        }
    } else {
        let chunks =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(HEIGHT_HELP_MINIMAL)])
                .split(area);
        AppLayout {
            stations: chunks[0],
            on_demand: None,
            saved_tracks: None,
            recent_tracks: None,
            now_playing: None,
            countdown: None,
            audio: None,
            help: chunks[1],
        }
    }
}

fn render_now_playing_overlay(frame: &mut Frame, app: &App, state: &PlayerState) {
    if !app.overlay.is_visible() {
        return;
    }
    let Some(station) = &state.station else { return };

    const W: u16 = 36;
    const H: u16 = 4;

    let area = frame.area();
    if area.width < W || area.height < H + HEIGHT_HELP {
        return;
    }

    let overlay_area = Rect::new(area.width - W, area.height - H - HEIGHT_HELP, W, H);
    frame.render_widget(
        NowPlayingOverlayWidget {
            station_name: &station.name,
            track_title:  state.title.as_deref(),
        },
        overlay_area,
    );
}

fn render_help(
    frame: &mut Frame,
    area: Rect,
    status: &PlayerStatus,
    focus: &AppFocus,
    save_notice: Option<&str>,
    preview_title: Option<&str>,
    preview_searching: bool,
    seek_input: &str,
) {
    let (text, color) = if let Some(title) = preview_title {
        (format!("  >> PREVIEW: {title}  [p] Parar"), theme::PLAYING)
    } else if preview_searching {
        (
            "  Buscando en Deezer...  [p] Cancelar".to_string(),
            theme::ACCENT,
        )
    } else if let Some(msg) = save_notice {
        let color = if msg.starts_with("Ya guardada") {
            theme::ACCENT
        } else {
            theme::PLAYING
        };
        (format!("  {msg}"), color)
    } else {
        let hint = match focus {
            AppFocus::RecentTracks => {
                "[↑↓/jk] Nav  [Enter] Guardar  [Esc] Volver  [Tab] Panel  [q] Salir".to_string()
            }
            AppFocus::StationSearch => {
                "[Backspace] Borrar  [Enter] Play  [Esc] Cancelar".to_string()
            }
            AppFocus::OnDemandList => {
                if !seek_input.is_empty() {
                    format!("  Saltar a: {seek_input}_   [Enter] Go  [Backspace] Borrar  [Esc] Cancelar")
                } else {
                    "[↑↓/jk] Nav  [p] Show  [Enter] Play  [0-9] Min  [[] -1min  []] +1min  [Esc] Volver".to_string()
                }
            }
            AppFocus::Stations => {
                let is_active = matches!(status, PlayerStatus::Playing | PlayerStatus::Paused);
                let pause_hint = if matches!(status, PlayerStatus::Paused) {
                    "[Space] Resume"
                } else {
                    "[Space] Pause"
                };
                if is_active {
                    format!("[↑↓/jk] Nav  [Enter] Play  {}  [s] Stop  [f] ★  [o] Config  [+/-] Vol  [Tab] Panel  [Esc] Salir", pause_hint)
                } else {
                    "[↑↓/jk] Nav  [Enter] Play  [f] ★  [o] Config  [+/-] Vol  [Tab] Panel  [Esc] Salir".to_string()
                }
            }
        };
        (hint, theme::MUTED)
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme::BORDER_STYLE);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text, Style::default().fg(color)))).block(block),
        area,
    );
}
