# Privacy

Reverbic does not track you. It has no accounts, no ads, and no analytics SDKs.
The only data that ever leaves your machine — beyond the functional requests
needed to play radio, talk to Spotify/YouTube if you use them, and check for
updates — is the **optional, anonymous online counter** described below.

## Anonymous online counter (opt-in)

The author wants a rough idea of how many people use Reverbic at the same time.
That is the **only** purpose of this feature.

- **Off by default.** Nothing is sent unless you explicitly turn it on in
  **Settings → Privacy → "Anonymous online counter"**. After using the app for a
  while you may see a one-time, non-blocking hint pointing you to that toggle —
  it never enables anything on its own.
- **You can turn it off at any time**, from the same toggle. It takes effect
  immediately.

### What is sent

While enabled, the app sends a small heartbeat every couple of minutes
containing exactly two fields:

- a **random session id** generated in memory at launch, and
- the **app version**.

That's it.

### What is NOT sent or stored

- **No IP address is stored.** The receiver discards it; it is never logged or
  used for geolocation.
- **No persistent identifier.** The session id lives only in memory and is
  regenerated every launch, so two sessions from the same person cannot be
  linked, and no one can be identified.
- **No location, no country, no operating system, no device fingerprint.**
- **No usage details** — not which stations you play, not your searches, not
  your favorites, nothing about what you do in the app.

### How "online now" is computed

The receiver simply counts how many distinct session ids sent a heartbeat in the
last few minutes. Old entries expire automatically. There is no database of
users to mine, because there are no users to identify.

### The receiver is open source

The endpoint that receives heartbeats is a tiny worker whose full source lives
in [`telemetry/`](telemetry/). You can read exactly what it does (and what it
refuses to store) instead of taking our word for it.

## Functional network requests

For completeness, these requests happen as part of normal use and are not
telemetry:

- **Radio**: streaming and station search via the public Radio Browser API.
- **Updates**: checking GitHub Releases for a newer version (can be disabled in
  Settings).
- **Spotify / YouTube**: only if you connect those integrations.

## Questions

Open an issue if anything here is unclear or if you'd like a change.
