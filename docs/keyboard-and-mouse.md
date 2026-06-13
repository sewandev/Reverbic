# Keyboard and Mouse

This is a complete reference of the keyboard shortcuts and mouse interactions in Reverbic, extracted from the event handlers in `src/app/input.rs`. Letter keys are matched case-insensitively unless a table states a specific case (for example `Shift+R` vs `r`).

The search modal is the main screen of Reverbic. Most navigation happens inside it, organized into three top tabs (Radio, Spotify, YouTube) plus a Settings view.

## Main Screen (station list)

The main screen shows the station list (favorites first, then built-in stations), an optional recent-tracks column, and an optional on-demand column.

| Key | Action |
| --- | --- |
| `Up` / `k` | Move selection up |
| `Down` / `j` | Move selection down |
| `Enter` | Play the selected station |
| `Space` | Pause / resume playback |
| `+` / `=` | Increase volume by one step |
| `-` | Decrease volume |
| `r` | Restart the currently playing station |
| `s` | Stop playback |
| `/` | Open the search modal (Radio tab) |
| `e` | Rename the selected favorite |
| `Alt+F` | Toggle favorite for the selected (or playing) station |
| `Shift+Up` | Move the selected favorite up in the list |
| `Shift+Down` | Move the selected favorite down in the list |
| `Tab` | Cycle focus between station list, on-demand list and recent tracks |
| `Right` | Move focus to the on-demand list (if available) |
| `q` | Save state and quit |
| `Esc` | Clear active search, stop a failed/reconnecting stream, or quit |
| Any alphanumeric, space or `-` | Start an inline station search |

When an inline search is active, `Esc` or clearing the query (via `Backspace`) returns focus to the station list.

### Recent Tracks column

Reached with `Tab`. Lists the recently announced track titles for the current station.

| Key | Action |
| --- | --- |
| `Up` / `k` | Move selection up |
| `Down` / `j` | Move selection down |
| `Enter` | Save the selected track to the local library |
| `p` | Toggle a 35-second Deezer preview of the selected track |
| `Esc` | Return to the station list |

### On-Demand column

Reached with `Tab` or `Right` when on-demand shows are available.

| Key | Action |
| --- | --- |
| `Up` / `k` | Move selection up |
| `Down` / `j` | Move selection down |
| `Enter` | Play the selected episode (or seek if a seek input is typed) |
| `p` | Switch to the next on-demand program |
| `[` | Seek backward 60 seconds |
| `]` | Seek forward 60 seconds |
| digits / `:` | Type a seek target (for example `12:30`), then `Enter` to seek |
| `Backspace` | Delete the last character of the seek input |
| `Left` / `Esc` | Clear the seek input, or return to the station list |

## Search Modal (global)

These keys work across the modal regardless of the active tab.

| Key | Action |
| --- | --- |
| `Tab` | Cycle the top tab: Radio -> Spotify -> YouTube -> Radio |
| `?` | Show the help overlay (any key closes it) |
| `+` / `=` | Increase volume |
| `-` | Decrease volume |
| `Ctrl+Shift+Right` | Jump to the next track in the active playback queue |
| `Ctrl+Shift+Left` | Jump to the previous track in the active playback queue |
| `[` / `]` | Jump between YouTube chapters (only outside the modal, on the main screen) |

## Radio Tab

The Radio tab has three sub-tabs: Search, Favorites and Playlists. Switch between them with `Left` / `Right`.

### Radio Search

| Key | Action |
| --- | --- |
| Type text | Live search by station name on Radio Browser |
| `Backspace` | Delete the last character |
| `Up` / `k` | Move selection up (wraps) |
| `Down` / `j` | Move selection down (wraps) |
| `Enter` | Play the selected result |
| `Shift+R` | Play a random result |
| `Alt+R` | Play a random result |
| `Alt+F` | Toggle favorite for the selected result |
| `Alt+P` | Add the selected result to a playlist |
| `Space` | Pause / resume (only while results are showing) |
| `Left` / `Right` | Switch radio sub-tab |
| `Esc` | Clear the query/results, or quit if both are empty |

### Genre and Country filters

Open with `Alt+G` (genre) or `Alt+C` (country) from the Radio tab. They show a fuzzy-filtered list; once a search runs, the list is replaced by results.

| Key | Action |
| --- | --- |
| Type text | Fuzzy-filter the genre/country list |
| `Backspace` | Delete the last character |
| `Up` / `k` | Move selection up |
| `Down` / `j` | Move selection down |
| `Enter` | Run the search for the selected genre/country |
| `Esc` | Return to the Radio name search |

While results are showing in Genre/Country mode:

| Key | Action |
| --- | --- |
| `Enter` | Play the selected result |
| `Shift+R` | Play a random result |
| `Space` | Pause / resume |
| `Alt+F` | Toggle favorite |
| `Alt+P` | Add to a playlist |
| `Up` / `Down` / `k` / `j` | Navigate results |
| `Esc` | Clear results and return to the filter list |

### Favorites sub-tab

| Key | Action |
| --- | --- |
| `Up` / `k` | Move selection up (wraps) |
| `Down` / `j` | Move selection down (wraps) |
| `Enter` | Play the selected favorite |
| `Shift+R` | Rename the selected favorite |
| `Alt+F` | Remove the selected favorite |
| `Alt+P` | Add the selected favorite to a playlist |
| `Shift+Up` | Move the favorite up |
| `Shift+Down` | Move the favorite down |
| `Space` | Pause / resume |
| `Esc` | Return to the Search sub-tab |

### Playlists sub-tab

When viewing the list of playlists:

| Key | Action |
| --- | --- |
| `Up` / `k` | Move selection up (wraps) |
| `Down` / `j` | Move selection down (wraps) |
| `Enter` | Open the selected playlist |
| `n` | Create a new playlist |
| `Shift+R` | Rename the selected playlist |
| `Alt+F` | Remove the selected playlist entry |
| `Space` | Pause / resume |
| `Esc` | Return to the Search sub-tab |

When a playlist is open (viewing its stations):

| Key | Action |
| --- | --- |
| `Up` / `k` | Move selection up (wraps) |
| `Down` / `j` | Move selection down (wraps) |
| `Enter` | Play the selected station |
| `Shift+Up` | Move the station up in the playlist |
| `Shift+Down` | Move the station down in the playlist |
| `Esc` | Close the playlist and return to the playlist list |

## Spotify Tab

### When logged out

| Key | Action |
| --- | --- |
| `Enter` | Start the OAuth login flow (unless already connecting) |
| `o` | Open Settings at the Spotify Client ID field (when idle) |
| `Esc` | Cancel an in-progress connection, or quit |

### When logged in

Switch sub-tabs (Search, Liked, Playlists, Top Tracks, Recent, Albums) with `Left` / `Right`.

| Key | Action |
| --- | --- |
| `Left` / `Right` | Cycle Spotify sub-tabs |
| `Up` | Move selection up |
| `Down` | Move selection down (loads more when reaching the end of paginated lists) |
| `Enter` | Play the selected track / open the selected playlist or album |
| `Space` | Pause / resume (outside the Search sub-tab) |
| `Alt+L` | Like the selected (or now-playing) track |
| `Ctrl+D` | Open the device picker (non-native playback modes) |
| `Alt+D` | Log out of Spotify |
| `Alt+R` | Refresh the device list |
| `Esc` | Close an open playlist/album, clear the search, or quit |
| `Backspace` | In track lists, close the open playlist/album; in search, delete a character |
| Type text | Search Spotify (Search sub-tab only) |

Device picker:

| Key | Action |
| --- | --- |
| `Up` / `k` | Move selection up |
| `Down` / `j` | Move selection down |
| `Enter` | Transfer playback to the selected device |
| `Esc` | Close the picker |

When playback is blocked (Remote mode with no active device):

| Key | Action |
| --- | --- |
| `o` | Open Settings at the Spotify playback mode |
| `Esc` | Quit |

## YouTube Tab

Sub-tabs: Search, Bookmarks, Liked, Playlists. Switch with `Left` / `Right`.

| Key | Action |
| --- | --- |
| `Left` / `Right` | Cycle YouTube sub-tabs |
| `Up` | Move selection up |
| `Down` | Move selection down |
| `Enter` | Play the selected video / run the typed search / open a playlist |
| `Space` | Pause / resume (outside the Search sub-tab) |
| `Ctrl+R` | Start a YouTube mix from the current context |
| `Alt+F` | Toggle a bookmark for the selected video |
| `o` | Open Settings at the YouTube cookies path (Liked/Playlists when no cookies set) |
| `Esc` | Close an open playlist, clear the search, or quit |
| `Backspace` | Close an open playlist, or delete a search character |
| Type text | Search YouTube (Search sub-tab only) |

## Settings

Open with `Alt+O` from anywhere, or via the in-context shortcuts above.

| Key | Action |
| --- | --- |
| `Up` / `k` | Move selection up (wraps) |
| `Down` / `j` | Move selection down (wraps) |
| `Enter` | Activate / toggle the selected setting |
| `Space` | Activate / toggle the selected setting |
| `Esc` | Close Settings and return to the Radio tab |

Text-input settings (Spotify Client ID, YouTube cookies path):

| Key | Action |
| --- | --- |
| Type text | Edit the value |
| `Backspace` | Delete the last character |
| `Enter` | Save the value |
| `Esc` | Cancel without saving |

Theme picker:

| Key | Action |
| --- | --- |
| `Up` / `k` | Previous theme |
| `Down` / `j` | Next theme |
| `Enter` | Apply the selected theme |
| `Esc` / `Left` | Close without applying |

## Screensaver / Ambient mode

When the screensaver is active, most input only wakes it. A few keys act directly:

| Key | Action |
| --- | --- |
| `+` / `=` | Increase volume |
| `-` | Decrease volume |
| `Space` | Pause / resume |
| `o` | Open the current station homepage (if any) |
| `Enter` / `Up` / `Down` | Wake the screensaver and act normally |
| Any other key | Wake the screensaver |

## Mouse

| Interaction | Action |
| --- | --- |
| Single click on a list row | Select that row (and play it on click in the modal) |
| Double click on a station (main screen) | Play the station |
| Click on a top tab | Switch to Radio / Spotify / YouTube |
| Click on a sub-tab | Switch the active sub-tab |
| Click on a setting row | Select and toggle that setting |
| Click on an auth/device notice | Open the related help URL |
| Click on the homepage link (screensaver) | Open the station homepage |
| Scroll wheel | Scroll the active list (selects items; loads more on paginated Spotify lists) |

In the modal, a single click on a list item both selects and activates it (plays the track/station or opens the playlist/album). On the main screen a single click only selects; a double click plays.

---
[Back to documentation index](README.md)
