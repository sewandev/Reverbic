mod components;
mod layout;
mod overlays;
mod screensaver;

pub use layout::now_playing_rect;

use ratatui::{layout::Rect, Frame};

pub fn spotify_screensaver_progress_rect(
    area:         Rect,
    profile_rows: u16,
) -> Option<Rect> {
    let pw = (area.width * 85 / 100).clamp(60, 110);

    let ph_base: u16 = 2 + 1 + 5 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1;
    let ph = ph_base + if profile_rows > 0 { 1 + profile_rows } else { 0 };

    let px = area.x + area.width.saturating_sub(pw) / 2;
    let py = area.y + area.height.saturating_sub(ph) / 2;

    let inner_x = px + 1;
    let inner_y = py + 1;
    let inner_w = pw.saturating_sub(2);
    let cx      = inner_x + 2;
    let cw      = inner_w.saturating_sub(4);

    let progress_y = inner_y + 14;

    if progress_y >= area.bottom() { return None; }
    Some(Rect::new(cx, progress_y, cw, 1))
}

use crate::app::{App, AppFocus};
use crate::audio::PlayerStatus;
use crate::ui::widgets::{
    countdown::CountdownWidget,
    now_playing::NowPlayingWidget,
    on_demand_panel::OnDemandPanelWidget,
    recent_tracks::RecentTracksWidget,
    saved_tracks::SavedTracksWidget,
    station_list::StationListWidget,
    vu_meter::VuMeterWidget,
};

use components::{render_header, render_help, render_sep};
use layout::compute_layout;
use overlays::{render_game_inline, render_game_strip, render_modal_np_strip, render_rename_overlay};
use screensaver::{render_screensaver, render_spotify_screensaver};

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
            let pw    = area.width.clamp(44, 66);
            let px    = area.x + area.width.saturating_sub(pw) / 2;
            if r.y >= 1 {
                render_game_inline(frame, ratatui::layout::Rect::new(px, r.y, pw, 1), name, genre);
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
        if let Some(ref playback) = app.spotify.playback {
            render_spotify_screensaver(
                frame, frame.area(), playback,
                app.config.spotify.display_name.as_deref(),
                app.config.spotify.country.as_deref(),
                app.config.spotify.followers,
                app.spotify.is_premium,
            );
            return;
        }
        let is_fav = player_state.station.as_ref()
            .map(|s| app.favorites.iter().any(|f| f.url == s.url))
            .unwrap_or(false);
        render_screensaver(
            frame, frame.area(), &player_state, app.station_details.as_ref(), is_fav,
            app.config.spotify.display_name.as_deref(),
            app.spotify.is_premium,
        );
        return;
    }

    if app.show_search_modal {
        use crate::ui::widgets::search_modal::SearchModalWidget;
        let full_area = frame.area();
        frame.render_widget(
            SearchModalWidget::from(app),
            full_area,
        );

        let modal_w = (full_area.width * 78 / 100).clamp(52, 120);
        let modal_h = (full_area.height * 75 / 100).clamp(14, 30);
        let modal_x = full_area.x + full_area.width.saturating_sub(modal_w) / 2;
        let modal_y = full_area.y + full_area.height.saturating_sub(modal_h) / 2;
        if let Some((ref name, ref genre)) = crate::game_detect::get() {
            let panel_h: u16 = 3;
            let game_y = modal_y.saturating_sub(panel_h);
            render_game_strip(frame, ratatui::layout::Rect::new(modal_x, game_y, modal_w, panel_h), name, genre);
        }

        let strip_y     = modal_y + modal_h;
        let remaining_h = full_area.bottom().saturating_sub(strip_y);
        if remaining_h >= 3 {
            let strip = ratatui::layout::Rect::new(modal_x, strip_y, modal_w, remaining_h);
            render_modal_np_strip(frame, strip, &player_state);
        }
    }

    if app.renaming_favorite.is_some() {
        render_rename_overlay(frame, &app.rename_input);
    }
}
