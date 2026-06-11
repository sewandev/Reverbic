<p align="center">
  <img src="assets/logo.svg" alt="Reverbic" width="265">
</p>

<p align="center">All-in-one terminal player — Radio, Spotify &amp; YouTube, for Windows, macOS and Linux.</p>

<p align="center">
  <a href="https://github.com/sewandev/Reverbic/actions/workflows/ci.yml"><img alt="Build" src="https://github.com/sewandev/Reverbic/actions/workflows/ci.yml/badge.svg" /></a>
  <a href="https://github.com/sewandev/Reverbic/actions/workflows/codeql.yml"><img alt="CodeQL" src="https://github.com/sewandev/Reverbic/actions/workflows/codeql.yml/badge.svg" /></a>
  <img alt="Version" src="https://img.shields.io/github/v/release/sewandev/Reverbic?style=flat-square&label=version&color=blueviolet" />
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-0078d4?style=flat-square" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-CE422B?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

<p align="center">
  <img src="assets/Preview-Reverbic.gif" alt="Reverbic preview" width="100%">
</p>

---

## Installation

```powershell
# Quick install (Windows)
irm https://raw.githubusercontent.com/sewandev/Reverbic/main/install.ps1 | iex

# Package managers
scoop bucket add reverbic https://github.com/sewandev/scoop-reverbic; scoop install reverbic   # Windows (Scoop)
cargo install --git https://github.com/sewandev/Reverbic.git --locked                          # Any OS (Rust)

# Build from source
git clone https://github.com/sewandev/Reverbic.git
cd Reverbic
cargo build --release
.\target\release\reverbic.exe
```

> [!TIP]
> Recommended: run Reverbic in [Windows Terminal](https://apps.microsoft.com/detail/9n0dx20hk701?hl) with [PowerShell 7+](https://apps.microsoft.com/detail/9mz1snwt0n5d?hl) for the best visual experience.

> [!WARNING]
> **Windows SmartScreen** may show a warning for unsigned binaries. Click "More info" → "Run anyway".

---

## Features

- **Radio** — Search and play thousands of internet radio stations by name, genre, or country
- **Spotify** — Remote control: search, play, pause, seek, volume, and device transfer (Premium required)
- **YouTube** — Search and stream audio directly from YouTube
- **Lightweight** — ~25 MB RAM and < 1% CPU at idle, starts in under a second
- **Floating overlay** — always on top, with automatic game detection
- **Discord Rich Presence** — shows your current station and track on your profile
- **Favorites & crossfade** — save your favorite stations with smooth crossfade between them
- **Screensaver mode** — clock, station info, and track metadata when idle

> [!NOTE]
> Spotify's 2026 policy changes could restrict native playback (librespot) at any time. Remote Control mode (search and playback control via the official Spotify API) does not depend on librespot and is a reasonable fallback for that risk, though it has its own requirements (your own Spotify Premium account and Developer app). See [LEGAL.md](LEGAL.md) for details.

---

## YouTube Authentication (optional)

Some YouTube videos require signing in (age-restricted, region-locked, or members-only content). Reverbic can use a `cookies.txt` file to access these.

> [!WARNING]
> **Use a secondary ("burner") account** for this — never your main Google account. The cookies file grants Reverbic access to that account's YouTube session, and yt-dlp may rewrite the file as cookies rotate.

To set it up:

1. Open a **private/incognito window** and sign in to YouTube with your secondary account.
2. Install [Get cookies.txt LOCALLY](https://github.com/kairi003/Get-cookies.txt-LOCALLY), an open-source extension that never sends your cookies anywhere.
3. On youtube.com, export your cookies in Netscape format and save the file somewhere private.
4. In Reverbic, open Settings and set **YouTube Cookies File** to the saved file's path.

For file permissions: on Linux/macOS, restrict access with `chmod 600 cookies.txt`; on Windows, avoid storing the file in a cloud-synced folder (OneDrive, Dropbox, etc.).

> [!NOTE]
> Cookies help with sign-in-required videos, but they don't guarantee fixing every "Sign in to confirm you're not a bot" error — YouTube's anti-bot checks (PO Tokens) can still block playback in some cases.

Reverbic only reads the path you provide and passes it to yt-dlp; it never transmits or caches the cookie file's contents. See [LEGAL.md](LEGAL.md) for the legal notes on the YouTube integration.

---

## Screenshots

<table align="center">
  <tr>
    <td align="center">
      <img src="assets/spotify.PNG" alt="Spotify remote control" width="380"><br>
      <sub>Spotify remote control</sub>
    </td>
    <td align="center">
      <img src="assets/youtube.PNG" alt="YouTube search" width="380"><br>
      <sub>YouTube search</sub>
    </td>
    <td align="center">
      <img src="assets/Overlay.gif" alt="Gaming overlay" width="380"><br>
      <sub>Gaming overlay</sub>
    </td>
  </tr>
  <tr>
    <td align="center">
      <img src="assets/screensaver.PNG" alt="Screensaver mode" width="380"><br>
      <sub>Screensaver mode</sub>
    </td>
    <td align="center">
      <img src="assets/configs.PNG" alt="Settings" width="380"><br>
      <sub>Settings</sub>
    </td>
    <td align="center">
      <img src="assets/Discord-Rich-Presence.gif" alt="Discord Rich Presence" width="380"><br>
      <sub>Discord Rich Presence</sub>
    </td>
  </tr>
</table>

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release notes and version history. ([Español](CHANGELOG.es.md))

---

## Contributors

<a href="https://github.com/sewandev/Reverbic/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=sewandev/Reverbic" />
</a>
