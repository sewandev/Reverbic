# Reverbic presence counter

This is the receiver for Reverbic's **optional, anonymous online counter**
(see [`../PRIVACY.md`](../PRIVACY.md)). Its only job is to answer one question:
*how many people are using Reverbic right now?*

It is published here so anyone can verify it stores **no IP, no location, no
identity** — only an ephemeral `session -> version` entry that expires on its
own after a few minutes.

## How it works

- The app sends `POST /beat` with `{"session": "<random hex>", "version": "x.y.z"}`
  every couple of minutes while the user has opted in.
- The worker stores that under a short TTL in a Cloudflare KV namespace.
- `GET /count?token=...` returns `{"online": N}` — the number of distinct
  sessions seen in the last 5 minutes.

`worker.js` is the entire implementation.

## Deploy (Cloudflare Workers, free tier)

1. Install [Wrangler](https://developers.cloudflare.com/workers/wrangler/).
2. Create the KV namespace and copy its id into `wrangler.toml`:
   ```sh
   wrangler kv namespace create PRESENCE
   ```
3. Set a read token so only you can see the count:
   ```sh
   wrangler secret put READ_TOKEN
   ```
4. Deploy:
   ```sh
   wrangler deploy
   ```
5. Point the app at it: set `HEARTBEAT_ENDPOINT` in
   [`../src/telemetry.rs`](../src/telemetry.rs) to `https://<your-worker>/beat`.

## Read the count

```sh
curl "https://<your-worker>/count?token=<READ_TOKEN>"
# {"online": 7}
```

## Notes

- KV `list` returns up to 1000 keys per call; if usage ever exceeds that,
  paginate with the returned cursor (or move to a Durable Object for exact,
  real-time counts).
- KV is eventually consistent, so the number is a close estimate, not a
  to-the-second figure — which is all this needs to be.
