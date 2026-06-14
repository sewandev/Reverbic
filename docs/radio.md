# Radio Features

Reverbic streams internet radio backed by the [Radio Browser](https://www.radio-browser.info/) directory, plus a small set of curated, metadata-enriched stations and an on-demand catalog. This guide describes how radio search, favorites, playlists, the local track library and on-demand content work, based on `src/station/` and `src/favorites.rs`.

## Searching Radio Browser

The Radio tab offers three ways to find stations, all querying the Radio Browser API.

### By name

In the Radio Search sub-tab, type a station name. The query runs against the `name` parameter, ordered by votes (most-voted first), with broken streams hidden (`hidebroken=true`). Results show the station name and bitrate.

### By genre

Press `Alt+G` to open the genre filter. It shows a fuzzy-filtered list of common tags (Pop, Rock, Jazz, Electronic, Lo-Fi, Salsa, and many more). Selecting one searches Radio Browser by `tag`.

### By country

Press `Alt+C` to open the country filter. It shows a fuzzy-filtered list of countries. Selecting one searches Radio Browser by `country`.

All three searches request results ordered by votes, reverse (most popular first), with broken streams hidden. Queries are tried against several Radio Browser mirror servers in turn, so a single mirror being down does not break search. Each result keeps up to three tags and its bitrate when available.

## Curated and enriched stations

A small set of stations is "enriched" with extra metadata sources. When a station name matches a known entry (matching is done by comparing the words in the names, in either direction), Reverbic attaches:

- A dedicated metadata API for now-playing track info.
- An optional track-history API.
- An optional schedule URL with a countdown.
- A known bitrate, when the stream does not report one.

Enriched stations currently include the Tomorrowland One World Radio family (One World Radio, Anthems, Daybreak Sessions) and several OnlyHit channels (OnlyHits, Top Hits, Kpop, Japan). Enrichment is applied automatically when a favorite is converted into a playable station, so favoriting a matching station gives you richer now-playing data.

## Favorites

Favorites are stored in `favorites.json` in the Reverbic data directory and persist across sessions.

### Adding and removing

- On the main screen, `Alt+F` toggles the selected (or currently playing) station as a favorite.
- In the Radio Search results, `Alt+F` toggles the selected result.
- In the Favorites sub-tab, `Alt+F` removes the selected favorite.

Toggling is by stream URL: if a favorite with the same URL exists it is removed, otherwise it is added.

### Renaming

Rename a favorite with `e` on the main screen, or `Shift+R` in the Favorites sub-tab. Type the new name and press `Enter` to save (empty names are ignored), or `Esc` to cancel.

### Reordering

Reorder favorites with `Shift+Up` / `Shift+Down`, both on the main screen and in the Favorites sub-tab. The change is saved immediately.

### Metadata enrichment

Favorites can be enriched in the background with country, tags and homepage. Only favorites missing all three fields are enriched. For each, Reverbic fetches station details from Radio Browser (by UUID if the key looks like a UUID, otherwise by name) and updates the stored favorite. The enriched country, first tag and homepage appear next to the station name in the Favorites list.

## Recent tracks and the local library

When a station announces track titles, they appear in the Recent Tracks column (reached with `Tab`). From there:

- `Enter` saves the selected title to the local library. Saved tracks are stored per station as a plain-text file under the `library/` folder in the Reverbic data directory; the filename is derived from the station key with unsafe characters percent-escaped and Windows reserved names guarded. Duplicate titles are not saved twice.
- `p` plays a 35-second Deezer preview of the selected title.

Saved tracks for the current station are loaded into memory and shown; stopping playback (`s`) clears the in-memory saved-track list.

## On-demand catalog

The on-demand column (reached with `Tab` or `Right` when available) lists episodes from a catalog of programs served via the Omny.fm API. Each program is a playlist; `p` switches to the next program and fetches up to 20 of its most recent episodes. Selecting an episode with `Enter` plays its audio stream, with seek support (`[` / `]` for 60-second jumps, or typing a `mm:ss` target).

## Pre-buffer setting

Streaming playback uses a configurable pre-buffer, controlled by the `Prebuffer` setting in Settings. Cycling the value updates the player live by sending the new pre-buffer duration (in seconds) to the audio engine. A larger pre-buffer trades a slightly longer start delay for smoother playback on unstable connections.

---
[Back to documentation index](README.md)
