
#![deny(warnings)]

use std::panic;
use std::time::{Duration, Instant};

use crossterm::event::{Event, EventStream, KeyEventKind, MouseEventKind};
use futures_util::{FutureExt, StreamExt};
use tracing_subscriber::{fmt, fmt::time::ChronoLocal, EnvFilter};

mod app;
mod audio;
mod config;
mod error;
mod favorites;
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

    tracing::info!("reverbic iniciando");

    let mut tui = terminal::init()?;
    let result = run(&mut tui).await;
    terminal::restore();

    if let Err(ref e) = result {
        eprintln!("Error: {e}");
    }

    tracing::info!("reverbic terminando");
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
    overlay::spawn(app.player.subscribe());

    // Auto-play de la última radio si está habilitado
    if app.config.autoplay_last && !app.stations.is_empty() {
        let idx = app.config.last_selected.min(app.stations.len() - 1);
        let station = app.stations[idx].clone();
        app.player.send(PlayerCommand::Play(station)).await;
    }
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    let mut events = EventStream::new();
    let mut last_click: Option<Instant> = None;
    let mut click_count: u8 = 0;

    loop {
        app.poll_search_results();
        app.poll_on_demand_results();
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
            tracing::debug!("Terminal redimensionado");
        }
        Some(Err(e)) => tracing::error!("Error leyendo evento: {e}"),
        _ => {}
    }
}
