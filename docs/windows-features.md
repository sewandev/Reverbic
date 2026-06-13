# Windows-Specific Features

Reverbic provides deep integration with Windows to improve the background listening experience while working or gaming.

## Exclusive Windows Features

Currently, the following features are supported on **Windows only**:
- **Floating Overlay**: A transparent, always-on-top window that displays the current song, radio station, or metadata without focusing the terminal.
- **System Tray Icon**: Reverbic can minimize to the tray and be controlled via a context menu.
- **System Notifications**: Native toast notifications for track changes.
- **Media Keys Support**: Control playback (Play, Pause, Next, Prev) using the physical keys on your keyboard.
- **Audio Ducking**: Automatically lowers Reverbic's volume when specific games or processes (e.g., voice chat) are focused.
- **Game Detection**: Detects running processes matching known games in `assets/games.json` to trigger the overlay or audio ducking.

## Overlay Configuration

The overlay can be deeply customized in Settings:
- **Modes**: Disabled, Always On, or Game Only (triggers only when a full-screen game is detected).
- **Styles**: Choose between different visual sizes and metadata richness (e.g., Simple, Detailed, Cover Art).
- **Positions**: Snap to corners (Top Left, Top Right, Bottom Left, Bottom Right).
- **Transparency**: Adjust the alpha level (opacity) of the overlay.

## Platform Boundaries in Code

These features rely heavily on Win32 APIs (e.g., `windows-rs`, `wasapi`).
To maintain cross-platform compatibility for macOS and Linux, the codebase uses conditional compilation (`#[cfg(target_os = "windows")]`) in modules like `src/ui/renderer/overlays.rs` and `src/game_detect.rs`.
On non-Windows systems, these functions compile to no-ops or return default safe values, and the settings menu dynamically hides Windows-specific options.
