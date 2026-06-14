# Data Sources

Reverbic pulls content and metadata from several external services. This page
documents the sources that exist in the code and how each is used.

## Radio Browser

Radio station search, genres, countries, trending, and stream URLs come from the
community [Radio Browser](https://www.radio-browser.info/) API
(`src/station/radio_browser.rs`). Requests rotate over a fixed set of mirror
hosts for resilience:

- `https://de1.api.radio-browser.info`
- `https://at1.api.radio-browser.info`
- `https://nl1.api.radio-browser.info`

Stations are deserialized into `RadioBrowserStation` (UUID, name, resolved URL,
bitrate, country, tags, homepage).

### Static station enrichment

Some well-known stations are augmented with hard-coded metadata, history, and
schedule endpoints defined in `src/station/enrichment.rs` (for example the
Tomorrowland One World Radio family and the OnlyHit network). These augment a
matched Radio Browser station with `metadata_api_url`, `history_api_url`,
`schedule_url`, a countdown flag, and a known bitrate. These are station-owned
endpoints, not a generic third-party service.

## ICY stream metadata

For live streams, the now-playing string is parsed directly from the ICY
metadata embedded in the audio stream (`StreamTitle='Artist - Title';`) in
`src/metadata/icy.rs`.

## Track enrichment: Deezer with iTunes fallback

When an ICY title is available, Reverbic enriches it into artist/title/album in
`src/metadata/track_enrichment.rs`:

1. **Deezer** — `https://api.deezer.com/search` (primary).
2. **iTunes Search API** — `https://itunes.apple.com/search` (fallback, used only
   if Deezer returns no acceptable match).

Both responses are fuzzy-matched against the parsed artist and title before
being accepted, and requests use an 8-second timeout.

## Spotify Web API

Spotify integration (`src/integrations/spotify/`) uses the Spotify Web API for
OAuth authentication, search, library/playlists/albums, recently-played and top
tracks, device discovery, and remote playback control. Native playback is
handled through Librespot. See the [Spotify Integration](spotify.md) guide for
user-facing details.

## YouTube via yt-dlp

YouTube support (`src/integrations/youtube/`) drives `yt-dlp` to search,
resolve audio stream URLs, and read playlists, backed by a bundled Deno runtime
and optional user cookies. SponsorBlock segments are fetched for skipping
non-music segments. See the [YouTube Integration](youtube.md) guide for details.

## Dota 2 Game State Integration

Dota 2 match state is received locally via Valve's Game State Integration
(`src/integrations/dota2/`). Reverbic installs a GSI config file into the Dota 2
`cfg` directory and runs a local HTTP listener on `127.0.0.1:7836` that receives
JSON payloads (map, player, hero data) pushed by the game. This is a
machine-local data source; no external Dota 2 service is contacted.

## Self-update

Release checks and downloads use the GitHub API (`src/update.rs`). This is an
update mechanism rather than a content source, but it is the remaining external
service the application contacts.

---
[Back to documentation index](README.md)
