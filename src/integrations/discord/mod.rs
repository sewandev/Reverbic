#![cfg(target_os = "windows")]

use std::time::Duration;

use tokio::sync::watch;

use crate::audio::PlayerState;
use crate::config::Config;

mod activity;
mod ipc;

const CLIENT_ID: &str = "1512183864100126810";
const RECONNECT_DELAY: Duration = Duration::from_secs(15);

pub fn spawn(mut player_rx: watch::Receiver<PlayerState>, mut config_rx: watch::Receiver<Config>) {
    tokio::spawn(async move {
        run(&mut player_rx, &mut config_rx).await;
    });
}

async fn run(
    player_rx: &mut watch::Receiver<PlayerState>,
    config_rx: &mut watch::Receiver<Config>,
) {
    let pid = std::process::id();
    let mut conn: Option<ipc::DiscordIpc> = None;
    let mut last_activity_json: Option<String> = None;
    let mut station_start_ts: Option<u64> = None;
    let mut current_station_name: Option<String> = None;
    let mut reconnect_deadline: Option<tokio::time::Instant> = None;
    let mut first_run = true;

    loop {
        if first_run {
            first_run = false;
        } else {
            wait_for_change(player_rx, config_rx, reconnect_deadline).await;
            reconnect_deadline = None;
        }

        let config = config_rx.borrow().clone();
        let state = player_rx.borrow().clone();

        if !config.discord_rpc {
            conn = None;
            last_activity_json = None;
            station_start_ts = None;
            current_station_name = None;
            continue;
        }

        // Update station start timestamp when the station changes
        let new_station_name = state.station.as_ref().map(|s| s.name.clone());
        if new_station_name != current_station_name {
            current_station_name = new_station_name;
            station_start_ts = current_station_name.as_ref().map(|_| unix_now());
        }

        let new_json = activity::build(&state).map(|a| a.to_json(station_start_ts));

        // Skip update if nothing changed and we're already connected
        if new_json == last_activity_json && conn.is_some() {
            continue;
        }

        // Connect if needed
        if conn.is_none() {
            match ipc::DiscordIpc::connect() {
                Some(mut c) => {
                    if c.handshake(CLIENT_ID).await {
                        tracing::info!("Discord RPC: conectado");
                        conn = Some(c);
                    } else {
                        tracing::debug!("Discord RPC: handshake fallido, reintentando...");
                        reconnect_deadline = Some(tokio::time::Instant::now() + RECONNECT_DELAY);
                        continue;
                    }
                }
                None => {
                    reconnect_deadline = Some(tokio::time::Instant::now() + RECONNECT_DELAY);
                    continue;
                }
            }
        }

        let c = conn.as_mut().expect("conn is Some here");
        let ok = match &new_json {
            Some(json) => c.set_activity(pid, json).await,
            None => c.clear_activity(pid).await,
        };

        if ok {
            last_activity_json = new_json;
        } else {
            tracing::warn!("Discord RPC: pipe roto, reconectando...");
            conn = None;
            last_activity_json = None;
            reconnect_deadline = Some(tokio::time::Instant::now() + RECONNECT_DELAY);
        }
    }
}

async fn wait_for_change(
    player_rx: &mut watch::Receiver<PlayerState>,
    config_rx: &mut watch::Receiver<Config>,
    reconnect_deadline: Option<tokio::time::Instant>,
) {
    tokio::select! {
        _ = player_rx.changed() => {}
        _ = config_rx.changed() => {}
        _ = async {
            match reconnect_deadline {
                Some(t) => tokio::time::sleep_until(t).await,
                None => std::future::pending::<()>().await,
            }
        } => {}
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
