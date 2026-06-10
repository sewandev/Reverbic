# Changelog

All notable changes to Reverbic are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
Versioning: [Semantic Versioning](https://semver.org/)

> Also available in [Español](CHANGELOG.es.md)

---

## [Unreleased]

### Added
- First-launch welcome stepper with logo animation, ambient music and initial setup options (overlay, autoplay, volume restore)
- "Show welcome again" option in Settings to replay the first-launch experience
- Spotify continuous playback: after a track ends, the next track in the loaded context plays automatically (batch-load via Spirc for gapless advance)
- Spotify radio mode: when the queue is exhausted, similar tracks from the same artist play automatically; can be toggled in Settings
- Spotify "Liked Songs" tab: browse and play saved tracks with pagination
- Spotify Playlists tab: browse user playlists, open them, and play tracks with sequential continuation

---

## [1.5.0] — 2026-06-09

### Added
- First-launch welcome stepper with logo animation, ambient music and initial setup options (overlay, autoplay, volume restore)
- "Show welcome again" option in Settings to replay the first-launch experience
- Spotify continuous playback: after a track ends, the next track in the loaded context plays automatically (batch-load via Spirc for gapless advance)
- Spotify radio mode: when the queue is exhausted, similar tracks from the same artist play automatically; can be toggled in Settings
- Spotify "Liked Songs" tab: browse and play saved tracks with pagination
- Spotify Playlists tab: browse user playlists, open them, and play tracks with sequential continuation

---

## [1.4.2] — 2026-06-06

### Added
- Extract UI theme into a modular palette system allowing dynamic themes
- New compact overlay style mode (`compact`)
- Require CI testing and strict GitHub Actions branch protection on `develop`

### Fixed
- Fixed broken unit test assertion for modal layout width

---

## [1.4.1] — 2026-06-06

### Added
- Harden updater payload validation against security risks

---

## [1.4.0] — 2026-06-05

### Added
- First-class YouTube tab powered by `yt-dlp` (search, resolve, playback)
- Auto-provision `yt-dlp` on first use
- On-demand streaming support for YouTube (resumes instead of restarting after a reconnect)

### Fixed
- Fixed PowerShell path injection vulnerability during self-installation
- Spotify token persistence now correctly uses the Windows Credential Manager without panicking on missing config
- `on_demand` bug correctly classifying YouTube streams to allow seeking and resume
- Improved macOS cross-platform path handling for `yt-dlp` binaries

---

## [1.3.1] — 2026-06-04

### Changed
- Overlay: song title and recent tracks are now brighter and more legible
- Overlay: clock and bitrate use the brand font (bold, larger) for better visibility
- Overlay: station name and now-playing title show more characters before truncating
- Overlay: DUCK indicator added to show auto-duck state at a glance
- Favorites sub-tab now shows the total count next to the label

### Fixed
- Removed leftover debug log from Spotify device auto-selection

---

## [1.3.0] — 2026-06-04

### Added
- Gaming mode panel shown above the radio screensaver when a game is detected
- Favorites subtab now shows country, tags, and homepage URL for each saved station
- Automatic enrichment of saved favorites with missing metadata (country, tags, URL) on startup

### Changed
- Win32 overlay redesigned: larger window (380×145 px), 9 animated VU bars with sine-wave per bar, real-time clock, bitrate indicator, volume bar, and last 2 played tracks instead of the redundant game name
- "Gaming Mode" label in the gaming strip uses the animated border color (bold)

### Fixed
- Fixed panic "A Tokio 1.x context was found, but it is being shutdown" when pausing Spotify on radio switch or screensaver activation

---

## [1.2.0] — 2026-06-03

### Added
- Animated modal border cycling through the brand logo colors (cyan → purple → crimson)
- Animated now-playing strip border when a radio station is playing
- Animated equalizer bars in the app logo: SVG version with CSS keyframes (README/browser) and TUI version with Unicode block characters

### Changed
- Spotify main tab uses Spotify brand green (#1ED760)

### Fixed
- Clicking anywhere in the modal no longer interrupted or restarted the currently playing radio
- Spacing between Spotify sub-tabs and search input now matches the radio tab layout

---

## [1.1.0] — 2026-06-03

### Added
- Progress bar in the Spotify overlay strip with elapsed and total time
- App logo visible in the main view when the terminal has enough vertical space
- Contextual keybinding hints at the modal footer that adapt to the current state (results, mode, tab)
- Setting to show or hide the digital clock in the screensaver (on by default)
- Dead URL detection for radio stations (HTTP 404): no retries, immediate error
- Visual `!` indicator on favorites for stations with a not-found URL (404)
- YouTube tab in the modal (coming soon placeholder)
- CI with `cargo check`, `cargo clippy` and `cargo fmt` on PRs targeting `main` and `develop`
- `develop` branch as integration branch (GitFlow)

### Changed
- Favorites shortcut changed from `F` to `Alt+F`, consistent with other modal shortcuts
- Section separators change color based on the player state (playing, paused, buffering, etc.)

### Fixed
- Keyring feature flag now correctly targets the Windows native credential store (`windows-native`)

---

## [1.0.0] — 2026-06-03

### Added
- Online radio playback via RadioBrowser API (search by name, genre, and country)
- Full Spotify integration: OAuth 2.0 PKCE auth, playback control, history, and queue
- Secure Spotify refresh token storage in Windows Credential Manager
- Win32 always-on-top overlay with WASAPI audio ducking
- Screensaver after 10 s of inactivity: bar visualizer, clock, recent tracks, and station details
- Dota 2 GSI integration: match phase detection and automatic cfg installation
- Self-installation to PATH on first launch (no admin rights required)
- System tray icon with window restore on double-click
- Real crossfade between stations using two simultaneous streams
- English / Spanish i18n with automatic Windows language detection
- Media key support (Play/Pause, Stop)
- Favorites, recent tracks, genres, and countries tabs
- Volume control with immediate persistence
- Configurable screensaver from the Config tab
- Configurable overlay transparency
- Configurable overlay position
- Automatic reconnection with exponential backoff on network failures
- Distribution via Scoop (`sewandev/scoop-reverbic`) and winget (`Sewandev.Reverbic`)
- GitHub Actions: automated release builds with binary and SHA256
- Issue templates (bug, feature, question)
- Logo and assets embedded in the executable (no external dependencies)

[Unreleased]: https://github.com/sewandev/Reverbic/compare/v1.5.0...HEAD
[1.5.0]: https://github.com/sewandev/Reverbic/compare/v1.4.2...v1.5.0
[1.4.2]: https://github.com/sewandev/Reverbic/compare/v1.4.1...v1.4.2
[1.4.1]: https://github.com/sewandev/Reverbic/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/sewandev/Reverbic/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/sewandev/Reverbic/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/sewandev/Reverbic/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/sewandev/Reverbic/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/sewandev/Reverbic/releases/tag/v1.0.0
