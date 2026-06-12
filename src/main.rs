#![deny(warnings)]

use std::panic;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crossterm::event::{Event, EventStream, KeyEventKind, MouseButton, MouseEventKind};
use futures_util::{FutureExt, StreamExt};
use tracing_subscriber::{fmt, fmt::time::ChronoLocal, EnvFilter};

mod app;
mod audio;
mod config;
mod error;
mod favorites;
mod game_detect;
mod http;
mod i18n;
mod install;
mod integrations;
mod library;
mod metadata;
mod onboarding;
#[cfg(target_os = "windows")]
mod overlay;
mod playlists;
mod preview;
mod schedule;
mod shell;
mod station;
mod terminal;
mod ui;
mod update;

use app::App;
use audio::PlayerCommand;
use error::Result;

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> windows::Win32::Foundation::BOOL {
    const CTRL_C_EVENT: u32 = 0;
    const CTRL_BREAK_EVENT: u32 = 1;
    const CTRL_CLOSE_EVENT: u32 = 2;
    if ctrl_type == CTRL_C_EVENT || ctrl_type == CTRL_BREAK_EVENT || ctrl_type == CTRL_CLOSE_EVENT {
        SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        windows::Win32::Foundation::BOOL(1)
    } else {
        windows::Win32::Foundation::BOOL(0)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        terminal::restore();
        original_hook(info);
    }));
    install::maybe_self_install();
    update::cleanup_stale();
    let _log_guard = init_logging();
    i18n::init(config::Config::load().language);
    game_detect::init_game_db();

    tracing::info!("reverbic starting");

    let mut tui = terminal::init()?;
    let result = run(&mut tui).await;
    terminal::restore();

    if let Err(ref e) = result {
        eprintln!("Error: {e}");
    }

    tracing::info!("reverbic stopping");
    result
}
fn log_file_path() -> PathBuf {
    config::reverbic_dir().join("logs").join("reverbic.log")
}

fn init_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_path = log_file_path();
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|e| eprintln!("Failed to create logs directory: {e}"));
    }

    match std::fs::File::create(&log_path) {
        Ok(file) => {
            let (non_blocking, guard) = tracing_appender::non_blocking(file);
            fmt()
                .with_timer(ChronoLocal::new("%Y-%m-%dT%H:%M:%S%.3f%z".to_string()))
                .with_env_filter(
                    EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()),
                )
                .with_writer(non_blocking)
                .with_ansi(false)
                .init();
            Some(guard)
        }
        Err(e) => {
            eprintln!(
                "Failed to create log file at {}: {e}. Logging to stderr.",
                log_path.display()
            );
            fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .with_writer(std::io::stderr)
                .init();
            None
        }
    }
}

async fn run(tui: &mut terminal::Tui) -> Result<()> {
    let mut app = App::new().await;

    if !app.config.onboarding_completed {
        onboarding::run(tui, &mut app.config, &app.player).await?;
        app.config.onboarding_completed = true;
        app.config.save();
    }

    #[cfg(target_os = "windows")]
    unsafe {
        let _ = windows::Win32::System::Console::SetConsoleCtrlHandler(
            Some(console_ctrl_handler),
            windows::Win32::Foundation::BOOL(1),
        );
    }

    app.init_integrations();
    app.start_update_check();
    tokio::spawn(crate::integrations::youtube::install::update_if_outdated());
    tokio::task::spawn_blocking(crate::audio::stream::clear_youtube_cache);

    #[cfg(target_os = "windows")]
    {
        let (config_tx, config_rx) = tokio::sync::watch::channel(app.config.clone());
        let discord_config_rx = config_rx.clone();
        overlay::spawn(app.player.subscribe(), config_rx, app.player.clone_sender());
        crate::integrations::discord::spawn(app.player.subscribe(), discord_config_rx);
        app.windows_tx = Some(config_tx);
    }
    if app.config.autoplay_last {
        if let Some(saved) = app.config.last_station.clone() {
            use crate::station::{enrich, find_enrichment, Station};
            let mut station = Station {
                key: saved.key.clone(),
                name: saved.name.clone(),
                url: saved.url.clone(),
                metadata_api_url: None,
                history_api_url: None,
                schedule_url: None,
                show_countdown: false,
                bitrate_kbps: saved.bitrate_kbps,
                custom_headers: None,
            };
            if let Some(enrichment) = find_enrichment(&saved.name) {
                enrich(&mut station, enrichment);
            }
            app.player.send(PlayerCommand::Play(station)).await;
        }
    }
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut events = EventStream::new();
    let mut double_clicks = DoubleClickTracker::default();

    loop {
        app.poll_dead_url();
        app.poll_favorites_enrichment();
        app.poll_update_check();
        app.poll_update_download();
        app.poll_search_results();
        app.poll_on_demand_results();
        app.poll_station_details();
        app.poll_track_enrichment();
        app.poll_spotify_auth();
        app.poll_token_refresh();
        app.poll_spotify_play_result();
        app.poll_spotify_search();
        app.poll_spotify_search_more();
        app.poll_spotify_player_events();
        app.poll_spotify_radio();
        app.poll_liked_tracks();
        app.poll_save_track();
        app.poll_playlists();
        app.poll_playlist_tracks();
        app.poll_top_tracks();
        app.poll_recent_tracks();
        app.poll_albums();
        app.poll_album_tracks();
        app.poll_spotify_devices();
        app.ensure_spotify_device_rescan();
        app.poll_remote_playback();
        app.poll_youtube_install();
        app.poll_youtube_search_debounce();
        app.poll_youtube_search();
        app.poll_youtube_resolve().await;
        app.poll_youtube_liked();
        app.poll_youtube_playlists();
        app.poll_youtube_playlist_videos();
        app.poll_youtube_playback();
        app.poll_youtube_validate();
        app.poll_youtube_preresolve();
        app.poll_youtube_mix();
        app.poll_youtube_sponsorblock();
        app.poll_youtube_chapters();
        if app
            .notice_until
            .map(|t| std::time::Instant::now() >= t)
            .unwrap_or(false)
        {
            app.advance_notice_queue();
        }
        if app
            .click_flash
            .map(|(_, t)| t.elapsed() >= std::time::Duration::from_millis(300))
            .unwrap_or(false)
        {
            app.click_flash = None;
        }
        if let Some(title) = app.player.state().title.clone() {
            if app.radio_enriched_for.as_deref() != Some(title.as_str()) {
                app.trigger_track_enrichment(title);
            }
        }
        let mut last_area = app.terminal_area;
        tui.draw(|frame| {
            last_area = frame.area();
            ui::render(frame, &app);
        })
        .map_err(|e| error::AppError::Terminal(e.to_string()))?;
        app.terminal_area = last_area;
        tokio::select! {
            _ = ticker.tick() => {
                app.border_tick = app.border_tick.wrapping_add(1);
                app.poll_search_results();
                app.poll_on_demand_results();
                app.poll_station_details();
            }
            _ = tokio::signal::ctrl_c() => {
                app.should_quit = true;
            }
            maybe_event = events.next() => {
                handle_event(&mut app, maybe_event, &mut double_clicks).await;
            }
        }
        while let Some(Some(maybe_event)) = events.next().now_or_never() {
            handle_event(&mut app, Some(maybe_event), &mut double_clicks).await;
        }

        if app.replay_onboarding {
            app.replay_onboarding = false;
            onboarding::run(tui, &mut app.config, &app.player).await?;
            if let Some(ref tx) = app.windows_tx {
                let _ = tx.send(app.config.clone());
            }
        }

        if SHUTDOWN_REQUESTED.load(Ordering::Relaxed) {
            app.should_quit = true;
        }
        if app.should_quit {
            break;
        }
    }

    if app.config.spotify.stop_on_quit {
        if let (Some(token), Some(device_id)) = (
            app.spotify.access_token.as_deref(),
            app.spotify.active_device_id.as_deref(),
        ) {
            let _ = crate::integrations::spotify::devices::pause_device(token, device_id).await;
        }
    }

    if let Some(ref path) = app.update_path.clone() {
        update::apply_update(path);
    }
    app.abort_all_tasks();
    Ok(())
}

const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(300);

#[derive(Default)]
struct DoubleClickTracker {
    last_mouse_down: Option<LastMouseDown>,
}

struct LastMouseDown {
    at: Instant,
    button: MouseButton,
    column: u16,
    row: u16,
}

impl DoubleClickTracker {
    fn register_mouse_down(
        &mut self,
        at: Instant,
        button: MouseButton,
        column: u16,
        row: u16,
    ) -> bool {
        let is_double_click = self.last_mouse_down.as_ref().is_some_and(|last| {
            last.button == button
                && last.column == column
                && last.row == row
                && at.saturating_duration_since(last.at) < DOUBLE_CLICK_THRESHOLD
        });
        self.last_mouse_down = Some(LastMouseDown {
            at,
            button,
            column,
            row,
        });
        is_double_click
    }
}

async fn handle_event(
    app: &mut App,
    maybe_event: Option<std::io::Result<Event>>,
    double_clicks: &mut DoubleClickTracker,
) {
    match maybe_event {
        Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
            app.on_key_event(key).await;
        }
        Some(Ok(Event::Paste(text))) => {
            app.on_paste(text);
        }
        Some(Ok(Event::Mouse(mouse))) => match mouse.kind {
            MouseEventKind::ScrollUp => {
                app.on_mouse_scroll(if app.show_search_modal { -1 } else { -3 })
                    .await;
            }
            MouseEventKind::ScrollDown => {
                app.on_mouse_scroll(if app.show_search_modal { 1 } else { 3 })
                    .await;
            }
            MouseEventKind::Down(button) => {
                if double_clicks.register_mouse_down(
                    Instant::now(),
                    button,
                    mouse.column,
                    mouse.row,
                ) {
                    app.on_double_click().await;
                } else {
                    app.on_click(mouse.column, mouse.row).await;
                }
            }
            _ => {}
        },
        Some(Ok(Event::Resize(_, _))) => {
            tracing::debug!("terminal resized");
        }
        Some(Err(e)) => tracing::error!("Error leyendo evento: {e}"),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent};
    use ratatui::layout::Rect;

    #[test]
    fn log_file_path_uses_reverbic_dir() {
        assert_eq!(
            log_file_path(),
            config::reverbic_dir().join("logs").join("reverbic.log")
        );
    }

    #[test]
    fn log_file_path_is_not_relative_to_working_directory() {
        assert_ne!(log_file_path(), PathBuf::from("logs").join("reverbic.log"));
    }

    #[test]
    fn double_click_tracker_triggers_for_same_button_row_and_column_within_threshold() {
        let start = Instant::now();
        let mut double_clicks = DoubleClickTracker::default();

        assert!(!double_clicks.register_mouse_down(start, MouseButton::Left, 10, 4));
        assert!(double_clicks.register_mouse_down(
            start + DOUBLE_CLICK_THRESHOLD / 2,
            MouseButton::Left,
            10,
            4
        ));
    }

    #[test]
    fn double_click_tracker_ignores_same_row_with_different_column() {
        let start = Instant::now();
        let mut double_clicks = DoubleClickTracker::default();

        assert!(!double_clicks.register_mouse_down(start, MouseButton::Left, 10, 4));
        assert!(!double_clicks.register_mouse_down(
            start + DOUBLE_CLICK_THRESHOLD / 2,
            MouseButton::Left,
            11,
            4
        ));
    }

    #[test]
    fn double_click_tracker_ignores_clicks_outside_threshold() {
        let start = Instant::now();
        let mut double_clicks = DoubleClickTracker::default();

        assert!(!double_clicks.register_mouse_down(start, MouseButton::Left, 10, 4));
        assert!(!double_clicks.register_mouse_down(
            start + DOUBLE_CLICK_THRESHOLD,
            MouseButton::Left,
            10,
            4
        ));
    }

    #[tokio::test]
    async fn non_click_events_before_mouse_down_do_not_trigger_double_click() {
        let mut app = App::new().await;
        app.show_search_modal = false;
        app.terminal_area = Rect::new(0, 0, 80, 24);
        let mut double_clicks = DoubleClickTracker::default();

        handle_event(
            &mut app,
            Some(Ok(Event::Resize(80, 24))),
            &mut double_clicks,
        )
        .await;
        handle_event(
            &mut app,
            Some(Ok(Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 10,
                row: 4,
                modifiers: KeyModifiers::empty(),
            }))),
            &mut double_clicks,
        )
        .await;

        handle_event(
            &mut app,
            Some(Ok(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 10,
                row: 4,
                modifiers: KeyModifiers::empty(),
            }))),
            &mut double_clicks,
        )
        .await;

        assert!(app.click_flash.is_none());
    }
}
