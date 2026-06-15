//! Anonymous online-presence telemetry.
//!
//! Its only purpose is to estimate how many people are using Reverbic at the
//! same time. While enabled, the app sends a small heartbeat every couple of
//! minutes; a session is counted as "online" if its heartbeat was seen in the
//! last few minutes.
//!
//! Privacy by design:
//! - **Opt-in.** Disabled by default; the user must turn it on, and can turn it
//!   off again at any time from Settings (the live flag below reflects that).
//! - **Ephemeral identity.** The session id is random and kept only in memory,
//!   regenerated on every launch, so a person can never be correlated across
//!   sessions, let alone identified.
//! - **No personal data.** Only the ephemeral id and the app version are sent.
//!   No IP is stored by the receiver, no location, no usage details.
//! - **Fire-and-forget.** Failures are ignored and never affect playback or the
//!   UI; an offline user simply is not counted.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use rand::RngCore;

/// Receiver for the presence heartbeats. Deploy the open-source worker in
/// `telemetry/worker.js` and point this at its URL. Until it is set to a real
/// endpoint, heartbeats simply fail silently (telemetry is best-effort).
const HEARTBEAT_ENDPOINT: &str = "https://reverbic-presence.sewandev.workers.dev/beat";

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(120);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Live on/off switch shared with the heartbeat task, so the Settings toggle can
/// enable or disable telemetry without restarting the app.
static ENABLED: OnceLock<Arc<AtomicBool>> = OnceLock::new();

/// Starts the heartbeat loop once, honoring the initial opt-in state. Safe to
/// call a single time during startup.
pub fn spawn(initial_enabled: bool) {
    let flag = ENABLED
        .get_or_init(|| Arc::new(AtomicBool::new(initial_enabled)))
        .clone();
    flag.store(initial_enabled, Ordering::Relaxed);

    tokio::spawn(async move {
        let Ok(client) = reqwest::Client::builder().timeout(REQUEST_TIMEOUT).build() else {
            return;
        };
        let session_id = session_id();
        let mut ticker = tokio::time::interval(HEARTBEAT_INTERVAL);
        loop {
            ticker.tick().await;
            if flag.load(Ordering::Relaxed) {
                send_heartbeat(&client, &session_id).await;
            }
        }
    });
}

/// Reflects a Settings toggle change onto the running heartbeat task.
pub fn set_enabled(enabled: bool) {
    if let Some(flag) = ENABLED.get() {
        flag.store(enabled, Ordering::Relaxed);
    }
}

async fn send_heartbeat(client: &reqwest::Client, session_id: &str) {
    let body = serde_json::json!({
        "session": session_id,
        "version": env!("CARGO_PKG_VERSION"),
    });
    let _ = client.post(HEARTBEAT_ENDPOINT).json(&body).send().await;
}

/// Random per-process id, kept only in memory, so it cannot link sessions.
fn session_id() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
