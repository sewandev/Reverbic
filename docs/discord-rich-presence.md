# Discord Rich Presence

Reverbic can show what you are listening to on your Discord profile through Rich Presence. This is a Windows-only feature, implemented in `src/integrations/discord/`. The module is compiled only on Windows (`#![cfg(target_os = "windows")]`) and communicates with the Discord desktop client over its local IPC named pipes.

## Enabling it

Discord Rich Presence is controlled by the `Discord RPC` toggle in Settings (the `DiscordRpc` setting). When the toggle is off, Reverbic clears any presence state and does not connect to Discord. Turning it back on triggers a reconnect and republishes the current activity.

## Requirements

- Windows.
- The Discord desktop client running locally. Reverbic connects by opening one of the Discord IPC named pipes (`\\.\pipe\discord-ipc-0` through `\\.\pipe\discord-ipc-9`), trying each in order until one succeeds.

If no pipe can be opened (Discord is not running), Reverbic does not show presence and retries later.

## Activity fields

The presence is built from the current player state:

- While playing, buffering, reconnecting or connecting: the station name is shown as the activity details, the current track title (when known) as the activity state, and a start timestamp is included so Discord shows elapsed time.
- While paused: the station name is shown as the details and the state reads `Paused`, with no timestamp.
- While idle or in an error state: no presence is shown (it is cleared).

The presence always includes a large image asset labeled `Reverbic`. The start timestamp is reset whenever the station changes, so the elapsed-time counter restarts with each new station.

Reverbic avoids redundant updates: if the computed activity is unchanged and the connection is alive, it does not resend it.

## Reconnection behavior

The integration runs a background task that watches for player-state and config changes. Connection handling works as follows:

- On startup (or after the toggle is turned on), it connects and performs a handshake with the Reverbic Discord application client ID.
- If the connection or handshake fails, it schedules a retry after a fixed delay of 15 seconds.
- If the pipe breaks while sending an update, it drops the connection, clears the cached activity and schedules a reconnect after the same 15-second delay.

This means a Discord restart or a transient pipe failure is recovered automatically without restarting Reverbic.

---
[Back to documentation index](README.md)
