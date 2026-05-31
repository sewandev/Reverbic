use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use super::state::{Dota2State, DotaPhase, hero_display};

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
    let timeout = std::time::Duration::from_secs(5);

    // Lee hasta encontrar el fin de cabeceras \r\n\r\n (puede requerir varios reads)
    let mut raw = Vec::with_capacity(8192);
    let header_end = loop {
        let mut chunk = [0u8; 4096];
        let n = match tokio::time::timeout(timeout, stream.read(&mut chunk)).await {
            Ok(Ok(n)) if n > 0 => n,
            _                  => return,
        };
        raw.extend_from_slice(&chunk[..n]);
        if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
        if raw.len() > 65_536 { return; }
    };

    // Parsea Content-Length de las cabeceras
    let headers = String::from_utf8_lossy(&raw[..header_end]);
    let content_length: usize = headers
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
        .and_then(|l| l.splitn(2, ':').nth(1)?.trim().parse().ok())
        .unwrap_or(0);

    // El body puede haber llegado parcialmente junto con las cabeceras
    let mut body = raw[header_end..].to_vec();

    // Lee el resto del body hasta completar content_length
    while body.len() < content_length {
        let need = (content_length - body.len()).min(4096);
        let mut chunk = vec![0u8; need];
        let n = match tokio::time::timeout(timeout, stream.read(&mut chunk)).await {
            Ok(Ok(n)) if n > 0 => n,
            _                  => break,
        };
        body.extend_from_slice(&chunk[..n]);
    }

    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
        apply(&json, &state);
    }

    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").await;
}

fn apply(json: &serde_json::Value, state: &Arc<Mutex<Dota2State>>) {
    let Ok(mut s) = state.lock() else { return };

    if let Some(map) = json.get("map") {
        let gs = map["game_state"].as_str().unwrap_or("");
        s.phase = DotaPhase::from_str(gs);
        if let Some(t) = map["clock_time"].as_i64() {
            s.game_time_secs = t as i32;
        }
    } else {
        s.phase = DotaPhase::None;
    }

    if let Some(p) = json.get("player") {
        s.kills     = p["kills"].as_u64().unwrap_or(0) as u32;
        s.deaths    = p["deaths"].as_u64().unwrap_or(0) as u32;
        s.assists   = p["assists"].as_u64().unwrap_or(0) as u32;
        s.net_worth = p["net_worth"].as_u64().unwrap_or(0) as u32;
        if let Some(team) = p["team_name"].as_str().filter(|t| !t.is_empty()) {
            s.team = team.to_string();
        }
    }

    if let Some(hero) = json.get("hero") {
        if let Some(name) = hero["name"].as_str().filter(|n| !n.is_empty()) {
            s.hero = hero_display(name);
        }
    }
}
