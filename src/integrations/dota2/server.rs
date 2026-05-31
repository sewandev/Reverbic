use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use super::state::{Dota2State, hero_display};

pub async fn run(port: u16, state: Arc<Mutex<Dota2State>>) {
    let listener = match TcpListener::bind(format!("127.0.0.1:{port}")).await {
        Ok(l)  => l,
        Err(e) => { tracing::error!("Dota2 GSI: puerto {port} no disponible: {e}"); return; }
    };
    tracing::info!("Dota2 GSI escuchando en 127.0.0.1:{port}");
    loop {
        match listener.accept().await {
            Ok((stream, _)) => { tokio::spawn(handle(stream, Arc::clone(&state))); }
            Err(e)          => tracing::warn!("Dota2 GSI accept: {e}"),
        }
    }
}

async fn handle(mut stream: tokio::net::TcpStream, state: Arc<Mutex<Dota2State>>) {
    let mut buf = vec![0u8; 65_536];
    let n = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        stream.read(&mut buf),
    ).await {
        Ok(Ok(n)) if n > 0 => n,
        _                  => return,
    };

    if let Some(off) = buf[..n].windows(4).position(|w| w == b"\r\n\r\n") {
        let body = &buf[off + 4..n];
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body) {
            apply(&json, &state);
        }
    }

    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").await;
}

fn apply(json: &serde_json::Value, state: &Arc<Mutex<Dota2State>>) {
    let Ok(mut s) = state.lock() else { return };

    if let Some(map) = json.get("map") {
        let gs = map["game_state"].as_str().unwrap_or("");
        s.in_game = matches!(
            gs,
            "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS" | "DOTA_GAMERULES_STATE_PRE_GAME"
        );
        if let Some(t) = map["clock_time"].as_i64() {
            s.game_time_secs = t as i32;
        }
    } else {
        s.in_game = false;
    }

    if let Some(p) = json.get("player") {
        s.kills     = p["kills"].as_u64().unwrap_or(0) as u32;
        s.deaths    = p["deaths"].as_u64().unwrap_or(0) as u32;
        s.assists   = p["assists"].as_u64().unwrap_or(0) as u32;
        s.net_worth = p["net_worth"].as_u64().unwrap_or(0) as u32;
    }

    if let Some(hero) = json.get("hero") {
        if let Some(name) = hero["name"].as_str().filter(|n| !n.is_empty()) {
            s.hero = hero_display(name);
        }
    }
}
