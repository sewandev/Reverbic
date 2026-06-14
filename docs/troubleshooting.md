# Troubleshooting

This guide covers common failures and how to diagnose them. Most issues can be traced through Reverbic's log file.

## Logs

Reverbic writes a rolling log to its cache directory:

```
~/.cache/reverbic/logs/reverbic.log
```

On Windows this resolves to `%LOCALAPPDATA%\Reverbic\cache\logs\reverbic.log`. The Settings menu also exposes an "Open logs folder" action that opens this directory without using the terminal. Each session logs the app version and the active Spotify playback mode at startup.

To read the log and filter for a specific topic:

```powershell
Get-Content "$env:LOCALAPPDATA\Reverbic\cache\logs\reverbic.log" | Select-String "your_filter"
```

When reporting a bug, include the relevant log lines.

## Spotify authentication and native playback

- **Authentication / refresh token issues**: the Spotify refresh token is stored in the OS keyring (entry `reverbic` / `spotify_refresh_token`), not in `config.json`. If the keyring backend is unavailable, the log shows warnings such as `Failed to load Spotify refresh token from keyring`. In that case re-authenticate from the Spotify settings. The startup log records the resolved playback mode, which is useful here.
- **Playback mode**: Reverbic supports `Auto`, `Remote`, and `Native` playback modes. Native playback requires a Spotify Premium account; if native playback does not start, confirm the account is Premium and try forcing `Remote` mode (which controls an existing Spotify client/device) to isolate whether the problem is the native engine or the account.
- See [Spotify Integration](spotify.md) for full setup details.

## yt-dlp download failures

YouTube playback resolves audio through `yt-dlp`. If tracks fail to resolve or end early, filter the log for the resolution pipeline:

```powershell
Get-Content "$env:LOCALAPPDATA\Reverbic\cache\logs\reverbic.log" | Select-String "yt-dlp|youtube:|track finished|ended early"
```

A healthy resolution logs `resolved YouTube audio format` with `format_id=140` (audio-only AAC). Failures usually mean `yt-dlp` is outdated or a video requires authentication. For age- or login-restricted videos, configure a cookies file (see [YouTube Integration](youtube.md)); public videos are always resolved without cookies.

## Radio stream and network failures

- Buffering and reconnection are surfaced both in the TUI and the overlay status indicator (`Buffering`, `Reconnecting`). Persistent reconnects usually indicate an unreachable or rate-limited station endpoint.
- A station that never starts is typically an offline or relocated stream URL; try another station to confirm connectivity.
- The prebuffer duration is configurable in Settings; increasing it can help on unstable connections at the cost of a longer startup delay.

## Overlay, tray, and media keys (Windows)

These features are Windows-only and run on a dedicated overlay thread. If they misbehave, look for log lines prefixed `overlay:`.

- **Overlay not appearing**: the overlay only shows when something is playing, and its visibility also depends on the overlay mode (see below). In `Games` mode it appears only when a full-screen application is in the foreground.
- **Tray icon missing**: the tray icon is opt-in; enable it in Settings. Double-clicking the tray icon restores and focuses the Reverbic terminal window.
- **Notifications**: native tray balloon notifications for track changes require both notifications and the tray icon to be enabled.
- **Media keys not working**: media-key support is opt-in and installs a low-level keyboard hook. Only the play/pause and stop hardware keys are handled. If another application captures these keys first, Reverbic will not receive them.

## Dota 2 GSI setup

Game State Integration relies on a config file placed in the Dota 2 installation:

- Reverbic locates Steam via the `HKCU\SOFTWARE\Valve\Steam` registry key, falling back to the default `Program Files (x86)\Steam` / `Program Files\Steam` paths.
- It writes `gamestate_integration_reverbic.cfg` into `steamapps/common/dota 2 beta/game/dota/cfg`. Common results:
  - `SteamNotFound`: Steam could not be located.
  - `Dota2NotFound`: the Dota 2 `cfg` directory does not exist (game not installed).
  - `AlreadyInstalled`: the config file is already present.
- After installation, Dota 2 must be restarted if it was already running for the integration to load.
- The integration listens on `127.0.0.1:7836`. If that port is already in use, the log shows `Dota2 GSI: puerto 7836 no disponible` and no data will be received.
- Diagnose by filtering the log for `Dota2 GSI`. A working setup logs incoming `game_state`, `player`, and `hero` updates.

## Terminal rendering and Unicode

Reverbic is a TUI and uses Unicode glyphs (status dots, arrows, the visualizer, etc.). If characters render as boxes or misalign:

- Use a terminal and font with good Unicode coverage (for example Windows Terminal with a font such as Cascadia Mono).
- Avoid the legacy `conhost.exe` console where possible, as its Unicode and color support is limited.
- Ensure the terminal is wide enough; very narrow windows can truncate or wrap the layout.

---
[Back to documentation index](README.md)
