mod overlays;
mod screensaver;

use ratatui::{layout::Rect, Frame};

const UNICODE_VISUALIZER_GLYPHS: [char; 8] = [
    '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}',
];
const ASCII_VISUALIZER_GLYPHS: [char; 8] = ['.', ':', '-', '=', '+', '*', '#', '@'];

fn visualizer_glyphs() -> &'static [char; 8] {
    visualizer_glyphs_for_legacy_console(uses_legacy_windows_console())
}

fn visualizer_glyphs_for_legacy_console(legacy_console: bool) -> &'static [char; 8] {
    if legacy_console {
        &ASCII_VISUALIZER_GLYPHS
    } else {
        &UNICODE_VISUALIZER_GLYPHS
    }
}

fn uses_legacy_windows_console() -> bool {
    cfg!(windows)
        && std::env::var_os("WT_SESSION").is_none()
        && std::env::var_os("TERM_PROGRAM").is_none()
        && std::env::var_os("TERM").is_none()
        && std::env::var_os("ConEmuANSI").is_none()
        && std::env::var_os("ANSICON").is_none()
}

pub fn spotify_screensaver_progress_rect(
    area: Rect,
    profile_rows: u16,
    show_clock: bool,
) -> Option<Rect> {
    let pw = (area.width * 85 / 100).clamp(60, 110).min(area.width);

    let clock_rows: u16 = if show_clock { 6 } else { 0 };
    let ph_base: u16 = 2 + 1 + clock_rows + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1;
    let ph = ph_base
        + if profile_rows > 0 {
            1 + profile_rows
        } else {
            0
        };

    let px = area.x + area.width.saturating_sub(pw) / 2;
    let py = area.y + area.height.saturating_sub(ph) / 2;

    let inner_x = px + 1;
    let inner_y = py + 1;
    let inner_w = pw.saturating_sub(2);
    let cx = inner_x + 2;
    let cw = inner_w.saturating_sub(4);

    let progress_y = inner_y + 7 + clock_rows + 1;

    if progress_y >= area.bottom() {
        return None;
    }
    Some(Rect::new(cx, progress_y, cw, 1))
}

use crate::app::App;
use overlays::{
    render_client_id_overlay, render_game_strip, render_help_overlay, render_modal_np_strip,
    render_modal_spotify_strip, render_rename_overlay, render_update_toast,
};
use screensaver::{
    render_logo_above, render_screensaver, render_spotify_screensaver, ScreensaverCtx, LOGO_W,
};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width < crate::ui::widgets::search_modal::MODAL_MIN_WIDTH
        || area.height < crate::ui::widgets::search_modal::MODAL_MIN_HEIGHT
    {
        use ratatui::{
            style::{Color, Style},
            text::{Line, Span},
            widgets::Paragraph,
        };
        let msg = Paragraph::new(Line::from(Span::styled(
            "[ terminal too small ]",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(msg, area);
        return;
    }

    let player_state = app.player_state();

    if app.screensaver_active() {
        if app.active_source_is_spotify() {
            if let Some(ref playback) = app.spotify.playback {
                render_spotify_screensaver(
                    frame,
                    area,
                    playback,
                    app.config.spotify.display_name.as_deref(),
                    app.config.spotify.country.as_deref(),
                    app.config.spotify.followers,
                    app.spotify.is_premium,
                    app.config.screensaver_clock,
                    app.border_tick,
                );
                if let Some(ref version) = app.update_available {
                    render_update_toast(
                        frame,
                        version,
                        app.update_path.is_some(),
                        app.show_search_modal,
                        area,
                    );
                }
                return;
            }
        }
        let is_fav = player_state
            .station
            .as_ref()
            .map(|s| app.favorites.iter().any(|f| f.url == s.url))
            .unwrap_or(false);
        render_screensaver(
            frame,
            area,
            ScreensaverCtx {
                state: &player_state,
                details: app.station_details.as_ref(),
                is_favorite: is_fav,
                spotify_name: app.config.spotify.display_name.as_deref(),
                spotify_premium: app.spotify.is_premium,
                enriched_track: app.radio_enriched_track.as_ref(),
                show_clock: app.config.screensaver_clock,
                border_tick: app.border_tick,
            },
        );
        if let Some(ref version) = app.update_available {
            render_update_toast(
                frame,
                version,
                app.update_path.is_some(),
                app.show_search_modal,
                area,
            );
        }
        return;
    }

    use crate::ui::widgets::search_modal::SearchModalWidget;
    let full_area = frame.area();
    frame.render_widget(SearchModalWidget::from(app), full_area);

    let modal = crate::ui::widgets::search_modal::modal_rect(full_area);

    if modal.y >= 3 {
        render_logo_above(
            frame,
            modal.x,
            modal.width.max(LOGO_W),
            modal.y - 1,
            crate::ui::theme::OVERLAY_COLOR,
            app.border_tick,
        );
    }

    if let Some((ref name, ref genre)) = crate::game_detect::get() {
        let panel_h: u16 = 3;
        let game_y = modal.y.saturating_sub(panel_h);
        render_game_strip(
            frame,
            Rect::new(modal.x, game_y, modal.width, panel_h),
            name,
            genre,
            app.border_tick,
        );
    }

    let strip_y = modal.y + modal.height;
    let remaining_h = full_area.bottom().saturating_sub(strip_y);
    if remaining_h >= 3 {
        let strip = Rect::new(modal.x, strip_y, modal.width, remaining_h);
        if matches!(app.modal_mode, crate::app::SearchMode::Spotify)
            && (app.spotify.playback.is_some() || app.spotify.now_playing.is_some())
        {
            render_modal_spotify_strip(
                frame,
                strip,
                app.spotify.playback.as_ref(),
                app.spotify.now_playing.as_ref(),
                &app.spotify.player_status,
                app.border_tick,
            );
        } else {
            render_modal_np_strip(frame, strip, &player_state, app.border_tick);
        }
    }

    if app.renaming_favorite.is_some() {
        render_rename_overlay(frame, &app.rename_input);
    }

    if app.editing_client_id {
        render_client_id_overlay(frame, &app.client_id_input);
    }

    if let Some(ref version) = app.update_available {
        render_update_toast(
            frame,
            version,
            app.update_path.is_some(),
            app.show_search_modal,
            full_area,
        );
    }

    if app.show_help {
        use crate::app::SpotifyAuthStatus;
        let spotify_logged_in = matches!(app.spotify.status, SpotifyAuthStatus::LoggedIn);
        render_help_overlay(
            frame,
            &app.modal_mode,
            spotify_logged_in,
            app.update_available.as_deref(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::visualizer_glyphs_for_legacy_console;

    #[test]
    fn legacy_console_visualizer_uses_ascii_fallback() {
        assert_eq!(
            visualizer_glyphs_for_legacy_console(true),
            &['.', ':', '-', '=', '+', '*', '#', '@']
        );
    }

    #[test]
    fn modern_terminal_visualizer_uses_unicode_blocks() {
        assert_eq!(
            visualizer_glyphs_for_legacy_console(false),
            &[
                '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
                '\u{2588}'
            ]
        );
    }
}
