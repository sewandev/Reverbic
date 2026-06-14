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

```bash
# Quick install (Windows)
irm https://raw.githubusercontent.com/sewandev/Reverbic/main/install.ps1 | iex

# Quick install (macOS / Linux)
brew install sewandev/reverbic/reverbic

# Package managers
scoop bucket add reverbic https://github.com/sewandev/scoop-reverbic; scoop install reverbic   # Windows (Scoop)
cargo install --git https://github.com/sewandev/Reverbic.git --locked                          # Any OS (Rust)

# Build from source
git clone https://github.com/sewandev/Reverbic.git
cd Reverbic
cargo build --release
./target/release/reverbic
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

## Documentation

- **[Spotify guide](docs/spotify.md)** — playback modes, Client ID setup, shortcuts, and known limitations
- **[YouTube guide](docs/youtube.md)** — features (Mix, chapters, SponsorBlock), cookies setup, and known limitations
- **[Legal notes](LEGAL.md)** — third-party services, terms of service, and risk disclosures

> [!WARNING]
> If you configure YouTube cookies, **use a secondary ("burner") account** — never your main Google account. Full instructions in the [YouTube guide](docs/youtube.md).

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
