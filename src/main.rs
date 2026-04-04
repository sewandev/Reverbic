
#![deny(warnings)]

use std::panic;
use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::{FutureExt, StreamExt};
use tracing_subscriber::{fmt, fmt::time::ChronoLocal, EnvFilter};

mod app;
mod audio;
mod config;
mod error;
mod library;
mod metadata;
mod station;
mod terminal;
mod ui;

use app::App;
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
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    let mut events = EventStream::new();

    loop {
        tui.draw(|frame| ui::render(frame, &app))
            .map_err(|e| error::AppError::Terminal(e.to_string()))?;
        tokio::select! {
            _ = ticker.tick() => {}
            maybe_event = events.next() => {
                handle_event(&mut app, maybe_event).await;
            }
        }
        loop {
            match events.next().now_or_never() {
                Some(Some(maybe_event)) => handle_event(&mut app, Some(maybe_event)).await,
                _ => break,
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn handle_event(app: &mut App, maybe_event: Option<std::io::Result<Event>>) {
    match maybe_event {
        Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
            app.on_key(key.code).await;
        }
        Some(Ok(Event::Resize(_, _))) => {
            tracing::debug!("Terminal redimensionado");
        }
        Some(Err(e)) => tracing::error!("Error leyendo evento: {e}"),
        _ => {}
    }
}
