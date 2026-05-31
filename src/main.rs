
#![deny(warnings)]

use std::panic;
use std::time::{Duration, Instant};

use crossterm::event::{Event, EventStream, KeyEventKind, MouseEventKind};
use futures_util::{FutureExt, StreamExt};
use tracing_subscriber::{fmt, fmt::time::ChronoLocal, EnvFilter};

mod app;
mod audio;
mod game_detect;
mod config;
mod error;
mod favorites;
mod i18n;
mod library;
mod metadata;
#[cfg(target_os = "windows")]
mod overlay;
mod preview;
mod schedule;
mod station;
mod terminal;
mod ui;

use app::App;
use audio::PlayerCommand;
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        terminal::restore();
        original_hook(info);
    }));
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
fn init_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    std::fs::create_dir_all("logs")
        .unwrap_or_else(|e| eprintln!("No se pudo crear directorio logs: {e}"));

    match std::fs::File::create("logs/reverbic.log") {
        Ok(file) => {
            let (non_blocking, guard) = tracing_appender::non_blocking(file);
            fmt()
                .with_timer(ChronoLocal::new("%Y-%m-%dT%H:%M:%S%.3f%z".to_string()))
                .with_env_filter(
                    EnvFilter::from_default_env()
                        .add_directive(tracing::Level::DEBUG.into()),
                )
                .with_writer(non_blocking)
                .with_ansi(false)
                .init();
            Some(guard)
        }
        Err(e) => {
            eprintln!("No se pudo crear log file: {e}. Logging a stderr.");
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

    #[cfg(target_os = "windows")]
    {
        let (config_tx, config_rx) = tokio::sync::watch::channel(app.config.clone());
        overlay::spawn(app.player.subscribe(), config_rx, app.player.clone_sender());
        app.windows_tx = Some(config_tx);
    }
    if app.config.autoplay_last {
        if let Some(saved) = app.config.last_station.clone() {
            use crate::station::{enrich, find_enrichment, Station};
            let mut station = Station {
                key:              saved.key.clone(),
                name:             saved.name.clone(),
                url:              saved.url.clone(),
                metadata_api_url: None,
                history_api_url:  None,
                schedule_url:     None,
                show_countdown:   false,
                bitrate_kbps:     saved.bitrate_kbps,
            };
            if let Some(enrichment) = find_enrichment(&saved.name) {
                enrich(&mut station, enrichment);
            }
            app.player.send(PlayerCommand::Play(station)).await;
        }
    }
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    let mut events = EventStream::new();
    let mut last_click: Option<Instant> = None;
    let mut click_count: u8 = 0;

    loop {
        app.poll_search_results();
        app.poll_on_demand_results();
        app.poll_station_details();
        let mut last_area = app.terminal_area;
        tui.draw(|frame| {
            last_area = frame.area();
            ui::render(frame, &app);
        })
        .map_err(|e| error::AppError::Terminal(e.to_string()))?;
        app.terminal_area = last_area;
        tokio::select! {
            _ = ticker.tick() => {
                app.poll_search_results();
                app.poll_on_demand_results();
                app.poll_station_details();
            }
            maybe_event = events.next() => {
                let now = Instant::now();
                if let Some(prev) = last_click {
                    if now.duration_since(prev).as_millis() < 300 {
                        click_count += 1;
                    } else {
                        click_count = 1;
                    }
                } else {
                    click_count = 1;
                }
                last_click = Some(now);
                handle_event(&mut app, maybe_event, click_count).await;
            }
        }
        loop {
            match events.next().now_or_never() {
                Some(Some(maybe_event)) => handle_event(&mut app, Some(maybe_event), click_count).await,
                _ => break,
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn handle_event(app: &mut App, maybe_event: Option<std::io::Result<Event>>, click_count: u8) {
    match maybe_event {
        Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
            app.on_key_event(key).await;
        }
        Some(Ok(Event::Paste(text))) => {
            app.on_paste(text);
        }
        Some(Ok(Event::Mouse(mouse))) => {
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    app.on_mouse_scroll(if app.show_search_modal { -1 } else { -3 }).await;
                }
                MouseEventKind::ScrollDown => {
                    app.on_mouse_scroll(if app.show_search_modal { 1 } else { 3 }).await;
                }
                MouseEventKind::Down(_) if click_count >= 2 => {
                    app.on_double_click().await;
                }
                MouseEventKind::Down(_) => {
                    app.on_click(mouse.column, mouse.row).await;
                }
                _ => {}
            }
        }
        Some(Ok(Event::Resize(_, _))) => {
            tracing::debug!("terminal resized");
        }
        Some(Err(e)) => tracing::error!("Error leyendo evento: {e}"),
        _ => {}
    }
}
