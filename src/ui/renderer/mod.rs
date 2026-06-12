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

use crate::app::App;
use crate::ui::theme;
use overlays::{
    render_client_id_overlay, render_cookies_path_overlay, render_game_strip, render_help_overlay,
    render_modal_np_strip, render_modal_spotify_strip, render_rename_overlay,
    render_theme_picker_overlay, render_update_toast,
};
use screensaver::{render_screensaver, render_spotify_screensaver, ScreensaverCtx};

pub(crate) use screensaver::{render_logo_above, LOGO_W};

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

    let palette = theme::palette(app.config.theme);
    let player_state = app.player_state();

    if app.screensaver_active() {
        if app.active_source_is_spotify() {
            let native_playback = if app.spotify.playback.is_none() {
                app.spotify.now_playing.as_ref().map(|track| {
                    let is_playing = matches!(
                        app.spotify.player_status,
                        crate::app::SpotifyPlayerStatus::Playing
                            | crate::app::SpotifyPlayerStatus::Loading
                    );
                    crate::integrations::spotify::SpotifyPlaybackState {
                        is_playing,
                        progress_ms: 0,
                        duration_ms: track.duration_ms,
                        track_name: track.name.clone(),
                        artist: track.artist.clone(),
                        album: track.album.clone(),
                        device_name: "Reverbic".to_string(),
                        volume_pct: (app.config.volume * 100.0).round().clamp(0.0, 100.0) as u8,
                    }
                })
            } else {
                None
            };

            if let Some(playback) = app.spotify.playback.as_ref().or(native_playback.as_ref()) {
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
                    palette,
                );
                if let Some(ref version) = app.update_available {
                    render_update_toast(
                        frame,
                        version,
                        app.update_path.is_some(),
                        app.show_search_modal,
                        area,
                        palette,
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
                palette,
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
                palette,
            );
        }
        return;
    }

    use crate::ui::widgets::search_modal::SearchModalWidget;
    let full_area = frame.area();
    frame.render_widget(SearchModalWidget::from_app(app, palette), full_area);

    let modal = crate::ui::widgets::search_modal::modal_rect(full_area);

    if modal.y >= 3 {
        render_logo_above(
            frame,
            modal.x,
            modal.width.max(LOGO_W),
            modal.y - 1,
            palette.overlay_color,
            app.border_tick,
            palette,
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
            palette,
        );
    }

    let strip_y = modal.y + modal.height;
    let remaining_h = full_area.bottom().saturating_sub(strip_y);
    if remaining_h >= 3 {
        let strip = Rect::new(modal.x, strip_y, modal.width, remaining_h);
        if app.active_source_is_spotify()
            && (app.spotify.playback.is_some() || app.spotify.now_playing.is_some())
        {
            render_modal_spotify_strip(
                frame,
                strip,
                app.spotify.playback.as_ref(),
                app.spotify.now_playing.as_ref(),
                &app.spotify.player_status,
                app.border_tick,
                palette,
            );
        } else {
            render_modal_np_strip(frame, strip, &player_state, app.border_tick, palette);
        }
    }

    if app.renaming_favorite.is_some() {
        render_rename_overlay(frame, &app.rename_input, palette);
    }

    if app.editing_client_id {
        render_client_id_overlay(frame, &app.client_id_input, palette);
    }

    if app.editing_cookies_path {
        render_cookies_path_overlay(
            frame,
            &app.cookies_path_input,
            app.cookies_path_error.as_deref(),
            palette,
        );
    }

    if app.theme_picker_open {
        render_theme_picker_overlay(frame, app.config.theme, app.theme_picker_selected, palette);
    }

    if let Some(ref version) = app.update_available {
        render_update_toast(
            frame,
            version,
            app.update_path.is_some(),
            app.show_search_modal,
            full_area,
            palette,
        );
    }

    if app.show_help {
        use crate::app::SpotifyAuthStatus;
        let spotify_logged_in = matches!(app.spotify.status, SpotifyAuthStatus::LoggedIn);
        let spotify_can_cycle_device = spotify_logged_in
            && app.config.spotify.playback_mode != crate::config::SpotifyPlaybackMode::Native;
        render_help_overlay(
            frame,
            &app.modal_mode,
            spotify_logged_in,
            spotify_can_cycle_device,
            app.update_available.as_deref(),
            palette,
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
