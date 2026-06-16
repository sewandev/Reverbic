# Changelog

All notable changes to Reverbic are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
Versioning: [Semantic Versioning](https://semver.org/)

> Also available in [Español](CHANGELOG.es.md)

---

## [Unreleased]

### Added
- Next/previous track controls (Ctrl+Right / Ctrl+Left) for YouTube playback, moving through the current context (search results, playlist, liked, bookmarks or mix). Previous stops at the first item; next extends the mix or follows YouTube Radio at the end of a list, just like auto-advance.
- Next/previous (Ctrl+Right / Ctrl+Left) also work for radio: they move through the list the station was played from (Favorites or Search results), or within the active playlist. A short notice appears at the start/end of the list, or when there is nothing playing to navigate.
- The terminal window title now reflects what is currently playing (for example, "Reverbic v1.5.5 Radio", "Reverbic v1.5.5 YouTube" or "Reverbic v1.5.5 Spotify"), and goes back to just the version when nothing is playing.
- New "Public playlists" sub-tab in YouTube, right next to "Search", that searches public playlists by name (for example, typing "nier automata" lists matching playlists). Like "Search", it works without signing in; open a playlist to browse and play its videos.

### Changed
- The Ambient Mode settings now open in a dedicated pop-up (like the theme picker) instead of expanding inline in the settings list. Selecting "Ambient Mode" opens a small modal where you set the activation time and toggle every widget (clock, logo, visualizer, recent tracks, progress bar, station details, now playing).
- The Overlay settings now also open in a dedicated pop-up: selecting "Overlay" opens a small modal with the display mode, style, transparency and position. Ambient Mode and Overlay are now separate sections in Settings instead of being grouped together.

### Fixed
- Audio playback no longer risks a cascading crash if an internal stream lock is left in a poisoned state; the player now recovers instead of panicking.
- YouTube now recovers automatically within the same session if the bundled Deno runtime goes missing or corrupt: after a failed resolve it re-checks the runtime (at most once every few minutes) and reinstalls it if needed, instead of failing until the app is restarted.
- YouTube no longer stops working on long-standing installs: the bundled Deno runtime is now kept up to date automatically (like yt-dlp), so playback keeps resolving after a yt-dlp update that requires a newer Deno.
- Improved YouTube audio reliability so more videos resolve to a clean audio-only stream instead of falling back to a lower-quality combined format.

### Security
- YouTube playlist identifiers are now validated before being used to build a request URL, matching the existing check on video identifiers (defense in depth).
- Update downloads now use a per-user private directory instead of the shared system temp folder, closing a theoretical local symlink-hijack vector during self-update on multi-user systems.

## [1.5.5] - 2026-06-14

### Added
- New headless command-line mode for radio on Windows: `reverbic play <station>` starts playback in the background and returns the terminal, `reverbic stop`, `reverbic status`, `reverbic volume <0-100>` and `reverbic toggle` control the running player, and playback keeps going after the terminal is closed. The station is matched first against your favorites and then via an online search; `reverbic play` with no name resumes the last station. Running `reverbic` with no arguments still opens the full interface.

### Changed
- Reverbic now stores its files following each operating system's standard locations (configuration, data, and cache are kept separate) instead of a single `~/.reverbic` folder. Existing installs are migrated automatically on first launch, so no settings are lost.

## [1.5.4] - 2026-06-13

### Added
- Added a "Spotify Crossfade" setting (up to 12 seconds) that blends the end of each track into the start of the next when using the Native playback mode; the setting appears disabled until Native mode is selected.
- The YouTube search tab now shows a hint that you can press Ctrl+R on a video to start an infinite radio.
- Reverbic now self-updates on Linux (x86_64), just like on Windows and macOS: a Linux binary is published with every release and the app downloads, verifies, and installs the new version automatically.
- Reverbic now publishes macOS builds (Intel and Apple Silicon) and self-updates on macOS, matching the existing Windows auto-updater.

### Changed
- The "YouTube Radio" setting tooltip now states that it requires a configured cookies.txt, since YouTube blocks mixes for unauthenticated requests.
- On macOS and Linux, the options that only work on Windows (overlay, ducking, media keys, tray icon, notifications, and Discord Rich Presence) no longer appear in Settings or in the first-run wizard.
- Redesigned the Spotify profile block in Ambient Mode as a dedicated widget with centered text: the display name stands out and a single line shows Premium, country, and follower count (now formatted with thousands separators).
- Reorganized the Settings menu into clearer categories: Radio, Spotify and YouTube each get their own section, separate from Overlay, Ducking, System and Appearance.
- The Overlay and Ducking settings sections are now labeled "(Windows only)" so it is clear they do not apply on other platforms.
- The shortcuts overlay ([?]) now groups keys by scope with section headers (Radio, Spotify, YouTube, Global) instead of showing one flat list.

### Fixed
- Starting a YouTube radio (Ctrl+R or when a list ends with YouTube Radio enabled) without a configured cookies.txt now shows a clear message instead of announcing the mix and then failing silently.
- The shortcuts overlay showed the wrong action for [Tab] and ignored YouTube; it now reads "Switch source" consistently across every tab.
- Unified the label of the "Open Settings" shortcut, which was inconsistent between the Radio, Spotify and YouTube views.
- When the configured YouTube cookies file became invalid (removed, moved or unreadable), the Liked and Playlists tabs stayed silently empty; they now show a clear error and the same recovery guide as the unauthenticated state.
- On macOS, pasting text (for example into the search box or the cookies path field) now works correctly through the system clipboard.
- The Reverbic logo could overlap the game strip; the layout now reserves space so both stay visible.

### Security
- Disabling or removing the YouTube cookies file now immediately stops playback of cookie-backed restricted videos within the same session; the in-memory resolved-URL cache no longer serves a cookie-backed result once credentials are gone.
- Removed the vulnerable `rustls-webpki 0.102.8` from the dependency tree (Dependabot alerts RUSTSEC-2026-0049, 0098, 0099 and 0104). It was pulled only by `hyper-proxy2` through the Spotify integration; the app now builds it against the patched rustls 0.23 / hyper-rustls 0.27 stack already used elsewhere, keeping the `ring` crypto backend.

## [1.5.3] - 2026-06-13

### Added
- Added individual settings in Settings > Ambient Mode to toggle visualizer, recent tracks, progress bar, and logo within the Ambient Mode.
- Ambient Mode now labels the YouTube source and shows the current chapter of the playing video below the title.
- The recent tracks list in Ambient Mode now works for every source (radio, YouTube, and Spotify), keeping the last 5 tracks played during the session.
- The station details in Ambient Mode are now a dedicated, color-highlighted widget showing country, region, language, codec and bitrate, tags, popularity (votes and plays), and a clickable website.
- Added a setting in Settings > Ambient Mode to toggle the station details on or off.
- Added a setting in Settings > Ambient Mode to toggle the now playing block (source name plus current artist, title, and album) on or off.
- The window title and UI overlays now dynamically display the version detection and download progress status (e.g. "Downloading vX.Y.Z..." and "Update vX.Y.Z Ready").

### Changed
- Refactored Ambient Mode rendering to use modular widgets for clock, visualizer, progress bar, and logo.
- Redesigned the Ambient Mode shortcuts bar: emoji-free, centered, with highlighted keys (e.g. Space Pause · +/- Volume · Alt+S Stop · Key Exit).
- Long titles, artists, and album names in Ambient Mode now wrap onto a second line instead of being cut off with an ellipsis.
- Ambient Mode no longer activates when all of its widgets (clock, logo, visualizer, progress bar, recent tracks) are turned off.

### Fixed
- The playback source indicator (Radio / YouTube / Spotify) in the Windows overlay was nearly invisible in gray; it now uses a distinct color per source so it stands out.
- The Reverbic logo in Ambient Mode could be pushed off-screen when the panel grew tall; space is now reserved so it always stays visible above the panel.
- The installation script now correctly overwrites and updates the persisted binary when the installer is re-run.

## [1.5.2] - 2026-06-12

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
- Animated playback dot on the main tabs: an intense green dot pulses next to the tab whose source is actually playing right now (it stays there even while you browse other tabs, and stops pulsing while paused); the active tab additionally shows an amber dot on [Spotify] in Remote mode with no device or a red one on [YouTube] when the cookies session expired
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

[Unreleased]: https://github.com/sewandev/Reverbic/compare/v1.5.3...HEAD
[1.5.3]: https://github.com/sewandev/Reverbic/compare/v1.5.2...v1.5.3
[1.5.2]: https://github.com/sewandev/Reverbic/compare/v1.5.1...v1.5.2
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
