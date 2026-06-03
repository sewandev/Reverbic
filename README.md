<p align="center">
  <img src="assets/logo.svg" alt="Reverbic" width="340">
</p>

<p align="center">Terminal radio player &amp; Spotify remote for Windows.</p>

<p align="center">
  <img alt="Version" src="https://img.shields.io/badge/version-1.0.0-blueviolet?style=flat-square" />
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows-0078d4?style=flat-square" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-CE422B?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

[![Reverbic preview](assets/preview.png)](https://github.com/sewandev/Reverbic)

---

## Features

**Radio**
- Search internet radio stations by name, genre, or country via [radio-browser.info](https://www.radio-browser.info)
- Curated station list with rich metadata (codec, bitrate, tags, homepage)
- Favorites with rename support
- Recent tracks history
- Crossfade between stations (1–10 s)
- Save tracks to a local list
- On-demand show catalog

**Spotify**
- Remote control: search, play, pause, seek, volume
- Device transfer (Premium required for playback)
- Sub-tabs: Search and Devices
- Rate-limit handling with countdown

**Windows**
- Floating overlay — always on top, configurable position (4 corners) and transparency
- System tray icon with balloon notifications
- Media key support (Play/Pause, Stop)
- Audio ducking — auto-reduces volume when another app produces sound
- Game detection — switches overlay to game-info mode

**UI / UX**
- Screensaver mode with clock, station info, and track metadata
- Full mouse support (click, scroll, double-click)
- Fuzzy search in station list and modal
- Keyboard-first navigation
- i18n: English / Spanish

---

## Why a terminal app?

| | Reverbic | Browser + web radio |
|---|---|---|
| RAM usage | ~25 MB | 300–600 MB |
| CPU at idle | < 1 % | 3–8 % |
| Startup time | < 1 s | 3–8 s |
| Disk footprint | ~8 MB | 500 MB+ |
| Runs in background | Yes — stays in tray | Needs a window open |
| Media keys | Native support | Depends on the site |
| Audio ducking | Built-in | Not available |
| Ads / tracking | None | Present on most sites |
| Screensaver / overlay | Yes | Not available |
| Offline config | Local JSON | Account / cookies |

---

## Keyboard Shortcuts

### Station list

| Key | Action |
|-----|--------|
| `↑` / `k`, `↓` / `j` | Navigate |
| `Enter` | Play station |
| `Space` | Pause / Resume |
| `+` / `=`, `-` | Volume up / down |
| `F` | Add to favorites |
| `e` | Rename station |
| `r` | Random station |
| `s` | Save current track |
| `/` | Quick search |
| `→` | Focus recent tracks |
| `Tab` | Open search modal |
| `Alt+O` | Open settings |
| `q` | Quit |

### Search modal

| Key | Action |
|-----|--------|
| `↑↓` | Navigate results |
| `Enter` | Play / Confirm |
| `F` | Save to favorites |
| `R` | Random result |
| `Alt+G` | Search by genre |
| `Alt+C` | Search by country |
| `Tab` | Switch to Spotify tab |
| `?` | Help overlay |
| `Esc` | Back / Close |

### Spotify tab

| Key | Action |
|-----|--------|
| `←` / `→` | Switch sub-tabs (Search / Devices) |
| `Enter` | Play track / Transfer to device |
| `Space` | Pause / Resume |
| `Alt+D` | Disconnect |
| `Alt+R` | Reload devices |

---

## Installation

### Requirements

- Windows 10 or 11
- [Rust](https://rustup.rs/) (latest stable)
- Spotify Premium account (for Spotify features)

### Build from source

```powershell
git clone https://github.com/sewandev/Reverbic.git
cd Reverbic
cargo build --release
.\target\release\reverbic.exe
```

### Spotify setup

Spotify integration requires a client ID from the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard).

1. Create an app in the dashboard
2. Add `http://localhost:8888/callback` and `http://localhost:8898/login` as Redirect URIs
3. Open Reverbic, press `Alt+O` to open Settings, navigate to **Spotify Client ID** and press `Space`
4. Paste your Client ID and press `Enter` — no recompile needed

> Spotify playback requires a **Premium** account. Free accounts can use search and device listing only.

---

## Configuration

All settings are accessible inside the app via `Alt+O`. No config file editing required.

| Setting | Description |
|---------|-------------|
| Autoplay last station | Resume the last station on startup |
| Crossfade | Crossfade duration between stations |
| Overlay mode | Hidden / When playing / Always / Games only |
| Overlay position | Top-left / Top-right / Bottom-left / Bottom-right |
| Overlay transparency | 0–100 % |
| Audio ducking | Auto-reduce volume when other apps play audio |
| Duck volume | Target volume level when ducking |
| Media keys | Enable media key support |
| System tray | Show tray icon with notifications |
| Screensaver | Idle time before screensaver activates |
| Volume step | Volume change per keypress |
| Pre-buffer | Seconds to buffer before playback |
| Language | English / Spanish |

Config is stored at `%APPDATA%\reverbic\config.json`.

---

## Author

**Esteban Jaramillo** — Chile  
[github.com/sewandev/Reverbic](https://github.com/sewandev/Reverbic)
