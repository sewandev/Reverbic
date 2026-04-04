
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, AppFocus};
use crate::audio::PlayerStatus;
use crate::ui::{
    theme,
    widgets::{
        countdown::CountdownWidget,
        now_playing::NowPlayingWidget,
        recent_tracks::RecentTracksWidget,
        saved_tracks::SavedTracksWidget,
        station_list::StationListWidget,
        vu_meter::VuMeterWidget,
    },
};
pub fn render(frame: &mut Frame, app: &App) {
    let player_state = app.player_state();

    let playing_index = player_state.station.as_ref().and_then(|playing| {
        app.stations.iter().position(|s| s.url == playing.url)
    });

    let has_recent      = !player_state.recent_titles.is_empty();
    let has_saved       = !app.saved_tracks.is_empty();
    let show_countdown  = player_state.station.as_ref().map(|s| s.show_countdown).unwrap_or(false);
    let layout = compute_layout(frame.area(), has_recent, has_saved, show_countdown);

    frame.render_widget(
        StationListWidget {
            stations:      app.stations,
            selected:      app.selected,
            playing_index,
            player_status: &player_state.status,
        },
        layout.stations,
    );

    if let Some(saved_area) = layout.saved_tracks {
        let station_name = player_state.station.as_ref().map(|s| s.name);
        frame.render_widget(
            SavedTracksWidget { tracks: &app.saved_tracks, station_name },
            saved_area,
        );
    }

    if let Some(recent_area) = layout.recent_tracks {
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
            recent_area,
        );
    }

    if let Some(now_playing_area) = layout.now_playing {
        frame.render_widget(NowPlayingWidget { state: &player_state }, now_playing_area);
    }

    if let Some(countdown_area) = layout.countdown {
        frame.render_widget(CountdownWidget, countdown_area);
    }

    if let Some(audio_area) = layout.audio {
        frame.render_widget(
            VuMeterWidget {
                level_db: player_state.level_db,
                volume:   player_state.volume,
            },
            audio_area,
        );
    }

    render_help(
        frame, layout.help, &player_state.status, &app.focus,
        app.save_notice.as_deref(),
        player_state.preview_title.as_deref(),
        player_state.preview_searching,
    );
}
const HEIGHT_NORMAL:  u16 = 19; // 5 top mínimo + 5 now_playing + 3 countdown + 4 audio + 2 help
const HEIGHT_COMPACT: u16 = 10;
const HEIGHT_NOW_PLAYING_NORMAL:  u16 = 5;
const HEIGHT_NOW_PLAYING_COMPACT: u16 = 3;
const HEIGHT_COUNTDOWN:           u16 = 3;
const HEIGHT_AUDIO:               u16 = 4;
const HEIGHT_AUDIO_COMPACT:       u16 = 3;
const HEIGHT_HELP:                u16 = 2;
const HEIGHT_HELP_MINIMAL:        u16 = 1;

struct AppLayout {
    stations:      Rect,
    saved_tracks:  Option<Rect>,
    recent_tracks: Option<Rect>,
    now_playing:   Option<Rect>,
    countdown:     Option<Rect>,
    audio:         Option<Rect>,
    help:          Rect,
}
fn build_right_column(
    top: Rect,
    has_recent: bool,
    has_saved: bool,
) -> (Rect, Option<Rect>, Option<Rect>) {
    let cols = Layout::horizontal([
        Constraint::Percentage(45),
        Constraint::Percentage(55),
    ])
    .split(top);

    let left  = cols[0];
    let right = cols[1];

    let (saved, recent) = match (has_saved, has_recent) {
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
    };

    (left, saved, recent)
}

fn compute_layout(area: Rect, has_recent: bool, has_saved: bool, show_countdown: bool) -> AppLayout {
    let has_right = has_recent || has_saved;

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

        let (stations, saved_tracks, recent_tracks) = if has_right {
            build_right_column(chunks[0], has_recent, has_saved)
        } else {
            (chunks[0], None, None)
        };

        AppLayout {
            stations,
            saved_tracks,
            recent_tracks,
            now_playing: Some(chunks[1]),
            countdown:   countdown_slot,
            audio:       Some(chunks[audio_idx]),
            help:        chunks[help_idx],
        }
    } else if area.height >= HEIGHT_COMPACT {
        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(HEIGHT_NOW_PLAYING_COMPACT),
            Constraint::Length(HEIGHT_AUDIO_COMPACT),
            Constraint::Length(HEIGHT_HELP),
        ])
        .split(area);

        let (stations, saved_tracks, recent_tracks) = if has_right {
            build_right_column(chunks[0], has_recent, has_saved)
        } else {
            (chunks[0], None, None)
        };

        AppLayout {
            stations,
            saved_tracks,
            recent_tracks,
            now_playing: Some(chunks[1]),
            countdown:   None,
            audio:       Some(chunks[2]),
            help:        chunks[3],
        }
    } else {
        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(HEIGHT_HELP_MINIMAL),
        ])
        .split(area);
        AppLayout {
            stations:      chunks[0],
            saved_tracks:  None,
            recent_tracks: None,
            now_playing:   None,
            countdown:     None,
            audio:         None,
            help:          chunks[1],
        }
    }
}

fn render_help(
    frame: &mut Frame,
    area: Rect,
    status: &PlayerStatus,
    focus: &AppFocus,
    save_notice: Option<&str>,
    preview_title: Option<&str>,
    preview_searching: bool,
) {
    let (text, color) = if let Some(title) = preview_title {
        (format!("  >> PREVIEW: {title}  [p] Parar"), theme::PLAYING)
    } else if preview_searching {
        ("  Buscando en Deezer...  [p] Cancelar".to_string(), theme::ACCENT)
    } else if let Some(msg) = save_notice {
        let color = if msg.starts_with("Ya guardada") { theme::ACCENT } else { theme::PLAYING };
        (format!("  {msg}"), color)
    } else {
        let hint = match focus {
            AppFocus::RecentTracks => {
                "[↑↓/jk] Nav  [Enter] Guardar  [Esc] Volver  [Tab] Panel  [q] Salir".to_string()
            }
            AppFocus::Stations => {
                let is_active = matches!(status, PlayerStatus::Playing | PlayerStatus::Paused);
                let pause_hint = if matches!(status, PlayerStatus::Paused) {
                    "[Space] Resume"
                } else {
                    "[Space] Pause"
                };
                if is_active {
                    format!("[↑↓/jk] Nav  [Enter] Play  {}  [s] Stop  [+/-] Vol  [Tab] Recent  [q] Salir", pause_hint)
                } else {
                    "[↑↓/jk] Nav  [Enter] Play  [+/-] Vol  [Tab] Recent  [q] Salir".to_string()
                }
            }
        };
        (hint, theme::MUTED)
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme::BORDER_STYLE);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text, Style::default().fg(color))))
            .block(block),
        area,
    );
}
