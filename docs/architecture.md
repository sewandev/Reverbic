# Architecture Overview

Reverbic is a terminal-based music and radio player built in Rust using Tokio for the async runtime and Ratatui for the terminal UI.

## High-Level Module Map

- **`src/app/`**: Core application state (`App`), input event handling, modal flows (settings, search), and coordination between UI and integrations.
- **`src/audio/`**: Core audio engine. Handles streaming playback, buffering, audio meter levels, and output device monitoring.
- **`src/integrations/`**: Connectors for external platforms:
  - `spotify/`: Librespot for native playback, Web API for remote control.
  - `youtube/`: `yt-dlp` process management and session handling.
  - `discord/`: Rich Presence state pushing.
  - `dota2/`: Game State Integration listener.
- **`src/station/`**: Radio registry, interaction with the Radio Browser API, station enrichment, and on-demand stream sources.
- **`src/metadata/`**: ICY stream metadata parsing and active track enrichment (e.g., fetching covers from Deezer/iTunes).
- **`src/ui/`**: TUI rendering using Ratatui. Contains components (`widgets`), themes, localized strings mapping, and the overlay rendering loop.
- **`src/onboarding/`**: The first-run setup wizard.
- **`src/update.rs`**: The self-updater module, which talks to the GitHub API for releases.
- **`src/install.rs`**: Windows self-installation logic (e.g., creating shortcuts, registering protocols).

## Async Task and Polling Model

The application uses an asynchronous architecture driven by `tokio`.
In `src/main.rs`, the program boots the UI loop alongside several background tasks:
1. **Audio Player Task**: Runs the audio sink and fetches streams over HTTP.
2. **Event Polling Loop**: Uses `crossterm` to process keyboard and mouse input without blocking the main render loop.
3. **Integration Workers**: Services like Discord RPC and Spotify connection run in background tasks and communicate with the main `App` state via `mpsc` channels and shared memory (`Arc<Mutex<T>>`).

## Persistence and Configuration Ownership boundaries

Configuration is owned by the `Config` struct and persisted to disk at `~/.reverbic/config.json`.
When a component needs to read a configuration (e.g., volume step), it reads it from the central `App` state. Changes made in the UI modal trigger an immediate save to disk. State boundaries are strictly enforced to avoid data races between the render thread and background workers.
