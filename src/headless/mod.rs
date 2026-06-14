//! Headless command-line control for radio playback.
//!
//! When Reverbic is launched with a subcommand it does not open the TUI. Instead
//! a detached background process (the "daemon") owns the audio pipeline and is
//! controlled over a local named pipe, so playback keeps running after the
//! terminal that launched it is closed. Running `reverbic` with no subcommand
//! opens the TUI exactly as before.
//!
//! This first iteration is radio-only and Windows-only; Spotify, YouTube and
//! cross-platform IPC are intentionally out of scope.

use clap::{Parser, Subcommand};

#[cfg(target_os = "windows")]
mod client;
#[cfg(target_os = "windows")]
mod daemon;
#[cfg(target_os = "windows")]
mod ipc;
#[cfg(target_os = "windows")]
mod resolve;

#[derive(Parser)]
#[command(name = "reverbic", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// Start radio playback in the background.
    ///
    /// The station is matched first against your favorites (fuzzy) and then via
    /// an online search. With no argument it resumes the last played station.
    Play {
        #[arg(value_name = "STATION", trailing_var_arg = true)]
        query: Vec<String>,
    },
    /// Stop playback and shut down the background player.
    Stop,
    /// Print the current station, track title and playback state.
    Status,
    /// Set the playback volume (0-100).
    Volume {
        #[arg(value_name = "LEVEL", value_parser = clap::value_parser!(u8).range(0..=100))]
        level: u8,
    },
    /// Toggle between play and pause.
    Toggle,
    /// Internal: run the background player process. Not meant to be called directly.
    #[command(hide = true)]
    Daemon,
}

pub async fn run(command: Command) -> crate::error::Result<()> {
    #[cfg(target_os = "windows")]
    {
        match command {
            Command::Daemon => {
                crate::paths::migrate_legacy();
                let _log_guard = crate::init_logging();
                daemon::run().await
            }
            other => client::run(other).await,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = command;
        eprintln!("Reverbic headless mode is currently only available on Windows.");
        Ok(())
    }
}
