// Anonymous online-presence counter for Reverbic.
//
// Purpose: estimate how many people are using Reverbic at the same time.
// It counts how many distinct *ephemeral* session ids sent a heartbeat in the
// last ONLINE_WINDOW seconds. It deliberately stores NOTHING that could identify
// a person: no IP, no headers, no location — only `session -> version` with a
// short TTL, so the data evaporates on its own.
//
// Endpoints:
//   POST /beat   body: {"session":"<hex>","version":"x.y.z"}  -> 204
//   GET  /count?token=<READ_TOKEN>                            -> {"online": N}
//
// Bindings (see wrangler.toml):
//   PRESENCE    KV namespace
//   READ_TOKEN  secret guarding the private /count read (optional but recommended)

const ONLINE_WINDOW = 300; // seconds (5 minutes)

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (request.method === "POST" && url.pathname === "/beat") {
      let body;
      try {
        body = await request.json();
      } catch {
        return new Response("bad request", { status: 400 });
      }
      const session = typeof body.session === "string" ? body.session : "";
      if (!/^[a-f0-9]{1,64}$/.test(session)) {
        return new Response("bad session", { status: 400 });
      }
      const version =
        typeof body.version === "string" ? body.version.slice(0, 32) : "";

      // The only thing persisted. No IP, no request headers, no timestamp beyond
      // the TTL that KV manages for us.
      await env.PRESENCE.put(`s:${session}`, version, {
        expirationTtl: ONLINE_WINDOW,
      });
      return new Response(null, { status: 204 });
    }

    if (request.method === "GET" && (url.pathname === "/count" || url.pathname === "/")) {
      if (env.READ_TOKEN && url.searchParams.get("token") !== env.READ_TOKEN) {
        return new Response("forbidden", { status: 403 });
      }
      // Note: KV list returns up to 1000 keys per call; paginate with the
      // returned cursor if you ever grow past that.
      const list = await env.PRESENCE.list({ prefix: "s:" });
      return Response.json({ online: list.keys.length });
    }

    return new Response("not found", { status: 404 });
  },
};
