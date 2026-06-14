# Game Integrations

Reverbic can detect running games and react to live match state. Game detection and the Dota 2 Game State Integration (GSI) are Windows-only features, implemented in `src/game_detect.rs` and `src/integrations/dota2/`.

## Game detection

On startup Reverbic loads a game database that maps process executable names (lowercase, without the `.exe` suffix) to a display name and a genre. The database has two layers:

1. An embedded `assets/games.json` shipped with the binary (Dota 2, CS2, CS:GO, League of Legends, Apex Legends, Valorant, Fortnite, Overwatch 2, Hearthstone, StarCraft II, and more).
2. An optional user override file at `~/.reverbic/games.json`.

If the user file exists, its entries are merged on top of the embedded database, so you can add games or override existing entries without losing your changes when Reverbic updates. The key is the process name as it appears in Task Manager (Image Name column), in lowercase and without `.exe`. Each entry has the form:

```json
"dota2": { "name": "Dota 2", "genre": "MOBA" }
```

When a known process is running, its display name and genre become available to the rest of the app (for example, the overlay and audio-ducking behavior).

## Dota 2 Game State Integration

Dota 2 can push live match data to Reverbic through Valve's Game State Integration. Reverbic runs a local listener and reads the data Dota 2 sends.

### Local listener

Reverbic listens on `127.0.0.1:7836` (TCP). It parses the HTTP requests Dota 2 sends, reads the JSON body, and updates an in-memory match state: game phase, hero, team, game clock, kills/deaths/assists and net worth. It always replies `HTTP/1.1 200 OK`.

### Config file

For Dota 2 to send data, a config file named `gamestate_integration_reverbic.cfg` must exist in the Dota 2 `cfg` directory. Reverbic can install it automatically. The generated file points back at the local listener:

```
"Reverbic"
{
    "uri"       "http://127.0.0.1:7836"
    "timeout"   "5.0"
    "buffer"    "0.1"
    "throttle"  "0.1"
    "heartbeat" "30.0"
    "data"
    {
        "map"    "1"
        "player" "1"
        "hero"   "1"
    }
}
```

### Discovery paths

To install the config, Reverbic locates Steam, then Dota 2 inside it:

1. Steam is found by reading the registry value `HKCU\SOFTWARE\Valve\Steam\SteamPath`. If that fails, it falls back to `C:\Program Files (x86)\Steam` and `C:\Program Files\Steam`.
2. The Dota 2 config directory is resolved as `<Steam>\steamapps\common\dota 2 beta\game\dota\cfg`.

The config is written to `<that cfg dir>\gamestate_integration_reverbic.cfg`.

### Install outcomes

Installing the config can result in:

- Installed — the file was written. If Dota 2 was already running at install time, a restart is required (see below).
- Already installed — the file already exists; nothing is changed.
- Steam not found — Steam could not be located.
- Dota 2 not found — the Dota 2 `cfg` directory does not exist.
- Write error — the file could not be written.

### Restart requirement

Dota 2 only reads GSI config files at launch. If the config is installed while Dota 2 is running (detected via `tasklist` for `dota2.exe`), the install reports that a restart is required. Restart Dota 2 for the integration to take effect.

## Troubleshooting

- No data arriving — confirm Dota 2 was restarted after the config was installed; it loads GSI configs only at launch.
- Steam not found — Reverbic could not read the Steam path from the registry or the default install folders. Verify Steam is installed in a standard location.
- Dota 2 not found — the `dota 2 beta\game\dota\cfg` directory was missing. Make sure Dota 2 is installed under that Steam library.
- Config not installed — if install reports "Already installed", the file is present; delete it and reinstall if you suspect it is stale or corrupt.
- Port unavailable — if `127.0.0.1:7836` cannot be bound (for example another process is using it), the listener logs an error and does not start. Free the port and restart Reverbic.

---
[Back to documentation index](README.md)
