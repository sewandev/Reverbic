# Changelog

All notable changes to Reverbic are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
Versioning: [Semantic Versioning](https://semver.org/)

> Also available in [Español](CHANGELOG.es.md)

---

## [Unreleased]

### Added
- Added a setting to configure a YouTube cookies.txt file, allowing access to age-restricted, region-locked, or members-only videos
- Added [Liked] and [Playlists] sub-tabs to the YouTube tab, mirroring Spotify, to browse and play your liked videos and personal playlists (requires a configured cookies.txt)
- Reverbic now automatically downloads and verifies the Deno runtime, used by yt-dlp to solve YouTube's signature challenges with near-instant resolution times (the binary stays on disk and is not loaded into memory at startup)
- Continuous YouTube playback: when a video ends, the next one in the active list (search results, Liked, or playlist) plays automatically, pre-resolving the upcoming video ahead of time
- Added a "YouTube Crossfade" setting to blend the end of each video into the start of the next when playing YouTube lists
- Added a "Validate YouTube session" action in Settings to instantly check whether the configured cookies.txt is still valid
- Resolved YouTube audio URLs are now reused for 4 hours, making replays of recent videos near-instant; the cache now survives restarts (cookie-authenticated resolves are never written to disk)
- YouTube Mix: press Ctrl+R on any video to start an "infinite radio" of similar songs that extends itself as the queue nears its end
- yt-dlp now updates itself automatically (daily check against GitHub with SHA256 verification), preventing YouTube changes from breaking the integration over time
- The highlighted video is pre-resolved in the background, making Enter playback near-instant
- YouTube tracks now download to a temporary file at full speed, enabling precise seeking and playback immune to network drops
- YouTube chapters: in long videos the current chapter shows next to the title, and the [ and ] keys jump between chapters
- New optional "SponsorBlock (YouTube)" setting that automatically skips non-music sections using the community database (off by default)
- New "YouTube Radio" setting, on by default: when the playing list ends, playback continues automatically with a mix of similar songs
- The game overlay now shows a countdown with the time remaining in the current track (Spotify and YouTube), in both Full and Compact styles; Radio has no duration so nothing changes there
- In Spotify Remote mode, when no Connect device is detected the Spotify tab now locks with a clear notice explaining to open Spotify on a device (phone, computer or web player); it rescans automatically every few seconds and unlocks itself as soon as a device appears (Ctrl+D forces an immediate scan)
- Radio playlists: new [ Playlists ] sub-tab in the Radio tab to group your stations into named collections; press Alt+P on any station (in Search, Genre, Country or Favorites) to add it to an existing playlist or create a new one, and playlists persist on disk between sessions
- Inside the [ Playlists ] sub-tab: N creates a named empty playlist, R renames the selected one, Shift+↑/↓ reorders stations within a playlist and Alt+F removes the station or deletes the playlist depending on the level
- Ctrl+Shift+→/← jumps to the next or previous station of the active playlist without opening any list, ideal for switching vibes without leaving what you are doing
- Local YouTube bookmarks: new [ Bookmarks ] sub-tab in the YouTube tab; press Alt+F on any video (search results, Liked or a playlist) to save it locally and play it later — no Google account, cookies or authentication needed
- Status dots on the main tabs: [Spotify] shows green when connected, amber when in Remote mode with no device and gray when disconnected; [YouTube] shows green when the cookies session is valid, red when it expired and gray when not configured
- The YouTube session is now validated automatically in the background at startup and whenever the cookies file changes; the result shows next to the "Validate YouTube session" setting ("Session valid" / "Cookies expired") and drives the [YouTube] tab status dot
- Spotify device picker: Ctrl+D now opens a list of every Connect device (name, type, active/available) and Enter transfers playback to the chosen one, instead of blindly cycling to the next device
- Ongoing live streams now show a red LIVE badge in YouTube search results, and trying to play one explains immediately that live broadcasts are not supported yet, before any resolving starts
- Notice panels now include an [O] shortcut that jumps straight to the relevant setting (YouTube cookies file, Spotify Client ID or the Spotify playback mode), with the item preselected
- New "Open logs folder" action in Settings to reach Reverbic's log files without using the terminal; each session now logs the app version and the Spotify playback mode at startup

### Changed
- The YouTube tab now uses YouTube's red consistently across all its elements (selected video, search input, typing cursor, scrollbar), mirroring the green pattern of the Spotify tab so it is always clear which tab is active
- The YouTube [Liked] and [Playlists] sub-tabs now show a clear notice panel when no cookies.txt is configured: it explains that authentication is needed, recommends using a secondary account and links to the step-by-step guide with the risks; the sub-tab labels also render as disabled (the old message overflowed the panel and was easy to miss)
- The Spotify tab now shows the same style of notice panel when the account is not connected: it explains that signing in is mandatory, that a Premium account and a Spotify Developer Dashboard app are required, and links to the step-by-step guide (clickable, includes the legal notes); Enter still starts the sign-in flow
- Bottom notices are now color-coded by severity (errors in red, warnings in amber, info in the source color) and queue up instead of overwriting each other, so an error can no longer be hidden by a routine message
- All notice panels (Spotify connect, Spotify no-device, YouTube authentication) now share one consistent component; the no-device panel gained the clickable guide link the others already had
- Connecting a non-Premium Spotify account now shows a clear warning that playback will not work, instead of failing later with confusing errors
- The YouTube authentication panel now mentions the local [ Bookmarks ] alternative for saving videos without an account

### Fixed
- Videos from recently ended live streams no longer hang in an endless retry loop; Reverbic now explains that YouTube is still processing the recording and to try again later
- Trying to play a YouTube stream that is live right now no longer shows the generic "no compatible format" error; Reverbic now explains it is an ongoing live broadcast and that it can be played once the stream ends
- The Spotify footer no longer claims "Mode: Remote Listening on Unknown [active]" when using Auto mode with no device; it now shows the real mode (Auto or Remote) and "no Spotify device detected" when there is none
- The Spotify footer now distinguishes between a device that is really playing ([active]) and one that is merely listed by Spotify ([available])
- When a Spotify device does not respond on playback (e.g. a phone whose app was closed but Spotify still lists it), Reverbic now discards it, explains what happened, and rescans instead of keeping it as the target
- The [?] help overlay now has a dedicated YouTube section (it previously showed generic hints) and every listed shortcut was audited against real behavior
- Space now pauses/resumes in every list without a text input (radio Favorites and Playlists, Genre/Country results, Spotify and YouTube library sub-tabs); it previously only worked in radio Favorites while the help claimed otherwise
- Alt+F and Alt+R no longer act on leftover radio search results while browsing the Spotify or YouTube tabs

### Security
- Updated dependencies (OpenSSL, ratatui, crossterm and others) to resolve known security advisories reported by Dependabot
- The Windows install script now verifies the SHA256 checksum of the downloaded binary before running it, and removes the "downloaded from the internet" mark only after verification succeeds
- The Windows install script now aborts if the release asset has no SHA256 digest to verify against, instead of running an unverified binary; this can be overridden at the user's own risk via the `REVERBIC_SKIP_VERIFY` environment variable

### Changed
- The Crossfade setting now offers 1, 3, 5 and 7 second steps (previously 1, 2 and 3)
- The Windows install script no longer overwrites the current session's PATH; it only appends Reverbic's install folder if missing
- The Windows install script now shows a clearer message before launching Reverbic, since the terminal stays occupied until the app is closed with `q`

### Fixed
- The Windows install script now handles network failures and GitHub API rate limits with friendly messages instead of raw errors, removes the temporary installer file afterwards, and supports ARM64 (via x86_64 emulation) and pre-release builds (via the `REVERBIC_PRERELEASE` environment variable)
- Fixed "Requested format is not available" errors when searching, resolving, or browsing YouTube videos and playlists, caused by yt-dlp now requiring a JavaScript runtime to solve YouTube's signature challenges
- On-demand playback (YouTube and replays) now reconnects and resumes from the exact byte if the connection drops mid-song
- Fixed YouTube songs cutting off mid-track or going silent: YouTube only served a combined video format whose HE-AAC audio the decoder cannot handle; playback now uses yt-dlp's android_vr client, which serves higher-quality audio-only AAC-LC

---

## [1.5.1] — 2026-06-10

### Fixed
- Fixed a version mismatch where the application reported v1.4.2 instead of v1.5.0, which caused the auto-updater to repeatedly suggest updating to the version already installed

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

[Unreleased]: https://github.com/sewandev/Reverbic/compare/v1.5.1...HEAD
[1.5.1]: https://github.com/sewandev/Reverbic/compare/v1.5.0...v1.5.1
[1.5.0]: https://github.com/sewandev/Reverbic/compare/v1.4.2...v1.5.0
[1.4.2]: https://github.com/sewandev/Reverbic/compare/v1.4.1...v1.4.2
[1.4.1]: https://github.com/sewandev/Reverbic/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/sewandev/Reverbic/compare/v1.3.1...v1.4.0
[1.3.1]: https://github.com/sewandev/Reverbic/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/sewandev/Reverbic/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/sewandev/Reverbic/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/sewandev/Reverbic/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/sewandev/Reverbic/releases/tag/v1.0.0
