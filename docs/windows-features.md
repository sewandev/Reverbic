# Windows-Specific Features

Reverbic provides deep integration with Windows to improve the background listening experience while working or gaming.

## Exclusive Windows Features

Currently, the following features are supported on **Windows only**:

- **Floating Overlay**: A transparent, always-on-top, click-through window that displays the current station or track, playback status, bitrate, recent titles, a live VU meter, and volume without focusing the terminal.
- **System Tray Icon**: An opt-in tray icon whose tooltip reflects the current track. Double-clicking it restores and focuses the Reverbic terminal window.
- **System Notifications**: Native tray balloon notifications on track changes. These require both notifications and the tray icon to be enabled.
- **Media Keys Support**: Opt-in support for the physical play/pause and stop media keys via a low-level keyboard hook.
- **Audio Ducking**: Automatically lowers Reverbic's volume while another application is producing audio, and restores it after a short quiet period.
- **Game Detection**: Inspects active audio sessions and running processes against the embedded `assets/games.json` (and an optional user `games.json` in the data directory, `%LOCALAPPDATA%\Reverbic\data\games.json`) to identify the foreground game.
- **Dota 2 Game State Integration**: A local GSI server (`127.0.0.1:7836`) that receives match phase, player stats, and hero data. See [Troubleshooting](troubleshooting.md) for setup.

## Overlay

The overlay window is fixed at 380px wide. Its height depends on the selected style, and it is rendered directly with Win32 GDI on a dedicated thread.

### Modes

Set via `OverlayMode`:

- **When Playing**: shown whenever playback is active.
- **Always**: shown while playback is active (does not force-show when idle).
- **Hidden**: never shown.
- **Games**: shown only while playback is active and a full-screen window is in the foreground.

### Styles

Set via `OverlayStyle`:

- **Full**: the tall layout (145px) with station, show, title, remaining time, recent tracks, VU meter, volume bar, and duck indicator.
- **Compact**: a short layout (52px) with status, station, title, source, and remaining time.

### Positions

Set via `OverlayPosition`, snapping to a screen corner with a 16px margin:

- Top Left
- Top Right
- Bottom Right
- Bottom Left

### Transparency

The overlay opacity is controlled by `overlay_alpha`, a 0-100 value (default 90) that is mapped to the layered-window alpha channel.

## Audio Ducking

When ducking is enabled, the overlay thread polls other applications' audio peak roughly every 500ms. When another source exceeds the activity threshold, Reverbic lowers its volume to the configured duck target (`duck_volume`, default 40%). Once the other audio has been quiet for about 2 seconds, the previous volume is restored.

## Platform Boundaries in Code

These features rely on Win32 APIs (via the `windows` crate and WASAPI). Cross-platform compatibility is preserved through conditional compilation:

- `src/overlay.rs` is entirely gated with a crate-level `#![cfg(target_os = "windows")]` and the module is only declared on Windows in `src/main.rs`. It hosts the overlay window, tray icon, notifications, media-key hook, audio ducking, and the WASAPI audio-session monitor.
- `src/game_detect.rs` gates its types and functions per-item with `#[cfg(target_os = "windows")]`; on other platforms `init_game_db` is a no-op and `get` returns `None`.
- `src/install.rs` exposes `maybe_self_install`, which copies the executable into `%LOCALAPPDATA%\Programs\reverbic` and adds it to the user `PATH` on Windows; on other platforms it is a no-op.
- The Dota 2 integration's installer relies on the Steam registry key and Windows process listing, so it is effectively Windows-only.
- `src/config.rs` detects the system language from the Windows locale on Windows and defaults to English elsewhere.

On non-Windows systems these functions compile to no-ops or safe defaults, and Windows-only options are hidden from the settings menu.

---
[Back to documentation index](README.md)
