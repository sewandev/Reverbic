# Architecture Overview

Reverbic is a terminal-based music and radio player built in Rust using Tokio for the async runtime and Ratatui for the terminal UI.

## High-Level Module Map

- **`src/app/`**: Core application state (`App`), input event handling, modal flows (settings, search), and coordination between UI and integrations. The state is split across submodules (`search`, `playlists`, `spotify_state`, `youtube_state`, `metadata`, `notice`, `on_demand`, `player_ctrl`, `update_ctrl`, etc.).
- **`src/audio/`**: Core audio engine. Handles streaming playback, buffering, audio meter levels, the YouTube stream cache, and output device monitoring.
- **`src/integrations/`**: Connectors for external platforms:
  - `spotify/`: Librespot for native playback, Web API for remote control. Submodules cover OAuth (`oauth.rs`), the player, device control (`devices.rs`), library, playlists, search, albums, and radio.
  - `youtube/`: `yt-dlp` process management, a bundled Deno runtime (`deno.rs`), cookie handling, search, stream resolution, playlists, install/update, and SponsorBlock.
  - `discord/`: Rich Presence state pushing over IPC (Windows-only; the module is gated with `#[cfg(target_os = "windows")]`).
  - `dota2/`: Game State Integration (GSI) listener.
- **`src/station/`**: Radio registry, interaction with the Radio Browser API (`radio_browser.rs`), static station enrichment (`enrichment.rs`), and on-demand stream sources.
- **`src/metadata/`**: ICY stream metadata parsing (`icy.rs`) and active track enrichment (`track_enrichment.rs`), which fetches artist/title/album from Deezer with an iTunes fallback.
- **`src/ui/`**: TUI rendering using Ratatui. Contains components (`widgets`), themes, localized strings mapping, and the overlay rendering loop.
- **`src/onboarding/`**: The first-run setup wizard.
- **`src/update.rs`**: The self-updater module, which talks to the GitHub API for releases. `main` also calls `update::cleanup_stale` on startup and `update::apply_update` on exit when an update was downloaded.
- **`src/install.rs`**: Windows self-installation logic (`maybe_self_install`), invoked at the start of `main`.

Other top-level modules include `config` (persisted settings), `favorites`, `playlists`, `library`, `youtube_bookmarks`, `game_detect`, `schedule`, `preview`, `http`, `terminal`, and the Windows-only `overlay`.

## Async Task and Polling Model

The application uses an asynchronous architecture driven by `tokio` (`#[tokio::main]`). The main render/event loop lives in `run()` in `src/main.rs`:

1. **Render + poll loop**: A single loop ticks every 50 ms via a `tokio::time::interval`, draws the UI through Ratatui, and uses `tokio::select!` over the interval tick, `ctrl_c`, and the `crossterm` `EventStream` for keyboard/mouse input. Each iteration calls a large set of `App::poll_*` methods that drain results from background work (search, enrichment, Spotify, YouTube, updates, etc.) into the `App` state without blocking the render.
2. **Tokio background tasks**: Integration work runs on `tokio::spawn` / `spawn_blocking` tasks — for example update checks, YouTube session health checks, the bundled yt-dlp updater, and clearing the YouTube cache. The Dota 2 GSI HTTP listener also runs as a spawned task. Background tasks publish read-only state to the UI through shared state and `tokio::sync::watch` channels (e.g. config and tab dots pushed to the overlay).
3. **Dedicated `std::thread` workers (Windows)**: Blocking Win32 / WASAPI work runs on dedicated OS threads, never on the tokio runtime or the render loop:
   - The overlay runs its own Win32 message loop on a thread named `overlay` (`src/overlay.rs`).
   - A `wasapi-monitor` thread polls per-process audio activity for game detection.
   - An `audio-device-monitor` thread (`src/audio/device_monitor.rs`) blocks on WASAPI device-change notifications.
   - The audio engine spawns a dedicated playback thread (`src/audio/player.rs`).

Shared, read-only UI state is exposed through `Arc<Mutex<T>>` and `OnceLock` globals (for example the Dota 2 state and the i18n tables) and `watch` channels, in preference to command channels.

## Persistence and Configuration Ownership

Path resolution is centralized in the `paths` module, which follows the XDG Base
Directory specification (and the equivalent conventions on Windows and macOS via
the `directories` crate) to split state across three category directories:
config, data, and cache. On the first startup after upgrading, `migrate_legacy()`
moves any pre-existing `~/.reverbic` layout into these directories.

Configuration is owned by the `Config` struct and persisted to `config.json` in
the config directory. Other persisted state is grouped by category:

- Config: `config.json` — application settings
- Data: `favorites.json`, `playlists.json`, `youtube_bookmarks.json` — saved content
- Data: `games.json` — user game-detection database
- Data: `library/`, `bin/` — track history and managed binaries (yt-dlp, Deno)
- Cache: `youtube_url_cache.json`, `youtube/` — resolved YouTube stream cache
- Cache: `librespot/` — native Spotify playback working directory
- Cache: `logs/reverbic.log` — application log

The `App` owns the in-memory `Config`; UI components read settings from the
central `App` state rather than the disk. Changes made in the settings modal
update the in-memory config and trigger a save to disk. On Windows the current
config is also broadcast to the overlay thread through a `watch` channel.

---
[Back to documentation index](README.md)
