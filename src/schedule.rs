use chrono::{
    DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone,
    Timelike, Utc,
};
use tokio::sync::mpsc;

use crate::audio::PlayerCommand;

fn format_timed_track(time: &str, artist: &str, title: &str) -> String {
    if artist.is_empty() {
        format!("{time}  {title}")
    } else {
        format!("{time}  {artist} - {title}")
    }
}

pub fn brussels_offset_secs() -> i32 {
    let now = Local::now();
    let (y, m, d, h) = (now.year(), now.month(), now.day(), now.hour());
    if !(3..=10).contains(&m) {
        return 3600;
    }
    if m > 3 && m < 10 {
        return 7200;
    }
    let last_sun = last_sunday_of_month(y, m);
    if m == 3 {
        if d > last_sun || (d == last_sun && h >= 2) {
            7200
        } else {
            3600
        }
    } else {
        if d > last_sun || (d == last_sun && h >= 3) {
            3600
        } else {
            7200
        }
    }
}

fn last_sunday_of_month(year: i32, month: u32) -> u32 {
    let (nm, ny) = if month == 12 {
        (1, year + 1)
    } else {
        (month + 1, year)
    };
    let last = NaiveDate::from_ymd_opt(ny, nm, 1)
        .and_then(|d| d.pred_opt())
        .unwrap_or_else(|| panic!("invalid date: year={ny}, month={nm}"));
    last.day()
        .saturating_sub(last.weekday().num_days_from_sunday())
}

pub fn brussels_hhmm_to_local(hhmm: &str) -> String {
    let offset = match FixedOffset::east_opt(brussels_offset_secs()) {
        Some(o) => o,
        None => return hhmm.to_string(),
    };
    let time = match NaiveTime::parse_from_str(hhmm, "%H:%M") {
        Ok(t) => t,
        Err(_) => return hhmm.to_string(),
    };
    let naive_dt = Local::now().date_naive().and_time(time);
    match offset.from_local_datetime(&naive_dt).single() {
        Some(dt) => dt.with_timezone(&Local).format("%H:%M").to_string(),
        None => hhmm.to_string(),
    }
}

pub fn utc_to_local_hhmm(utc_str: &str) -> String {
    match DateTime::parse_from_rfc3339(utc_str) {
        Ok(dt) => dt.with_timezone(&Local).format("%H:%M").to_string(),
        Err(e) => {
            tracing::warn!("Invalid RFC3339 timestamp '{utc_str}': {e}");
            "??:??".to_string()
        }
    }
}

pub async fn fetch_schedule(client: &reqwest::Client, url: &str) -> Option<serde_json::Value> {
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    serde_json::from_str(&resp.text().await.ok()?).ok()
}

pub fn current_show_time(schedule: &serde_json::Value) -> Option<String> {
    let offset = FixedOffset::east_opt(brussels_offset_secs())?;
    let now_brussels = Local::now()
        .with_timezone(&offset)
        .format("%H:%M")
        .to_string();
    let day = Local::now().format("%A").to_string().to_lowercase();
    let shows = schedule[&day].as_array()?;
    let pos = shows.iter().rposition(|e| {
        e["startTime"]
            .as_str()
            .map(|t| t <= now_brussels.as_str())
            .unwrap_or(false)
    })?;

    let start = shows[pos]["startTime"].as_str()?;
    let end = shows.get(pos + 1).and_then(|e| e["startTime"].as_str());

    let start_local = brussels_hhmm_to_local(start);
    let end_local = end
        .map(brussels_hhmm_to_local)
        .unwrap_or_else(|| "…".to_string());

    Some(format!("{start_local} - {end_local}"))
}

pub fn parse_history(body: &str) -> Vec<String> {
    let arr: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    arr.as_array()
        .map(|entries| {
            entries
                .iter()
                .take(10)
                .filter_map(|e| {
                    let artist = e["artist"].as_str().unwrap_or("");
                    let title = e["title"].as_str()?;
                    let ts = e["timestamp"].as_str().unwrap_or("");
                    let time = NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
                        .map(|ndt| {
                            Utc.from_utc_datetime(&ndt)
                                .with_timezone(&Local)
                                .format("%H:%M")
                                .to_string()
                        })
                        .unwrap_or_else(|_| "??:??".to_string());
                    Some(format_timed_track(&time, artist, title))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn parse_api_response(
    body: &str,
    history_body: Option<&str>,
    schedule: Option<&serde_json::Value>,
) -> Option<PlayerCommand> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let title = v["title"].as_str()?.to_string();
    let artist = v["artist"].as_str().unwrap_or("").to_string();
    let show_name = if v["show"].is_object() {
        v["show"]["name"].as_str().unwrap_or("").to_string()
    } else {
        v["show"].as_str().unwrap_or("").to_string()
    };
    let show = match schedule.and_then(current_show_time) {
        Some(time) => format!("{show_name}  {time}"),
        None => show_name,
    };

    let recent = if let Some(hist) = history_body {
        parse_history(hist)
    } else {
        v["tracklog"]
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .take(10)
                    .filter_map(|e| {
                        let t = e["title"].as_str()?;
                        let a = e["artist"].as_str().unwrap_or("");
                        let start = e["startTime"].as_str().unwrap_or("");
                        let time = utc_to_local_hhmm(start);
                        Some(format_timed_track(&time, a, t))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    Some(PlayerCommand::ApiMetadata {
        title,
        artist,
        show,
        recent,
    })
}

pub async fn poll_metadata_loop(
    url: String,
    history_url: Option<String>,
    schedule_url: Option<String>,
    cmd_tx: mpsc::Sender<PlayerCommand>,
) {
    let Some(client) = crate::http::http_client() else {
        tracing::error!("Failed to create HTTP client for metadata");
        return;
    };
    let schedule = if let Some(ref s_url) = schedule_url {
        let s = fetch_schedule(&client, s_url).await;
        if s.is_none() {
            tracing::warn!("Failed to fetch schedule; only the show name will be displayed");
        }
        s
    } else {
        None
    };

    loop {
        let history_body: Option<String> = if let Some(ref h_url) = history_url {
            match client.get(h_url).send().await {
                Ok(resp) if resp.status().is_success() => resp.text().await.ok(),
                Ok(resp) => {
                    tracing::warn!("History API HTTP {}", resp.status());
                    None
                }
                Err(e) => {
                    tracing::warn!("History API no disponible: {e}");
                    None
                }
            }
        } else {
            None
        };

        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.text().await {
                    Ok(body) => {
                        if let Some(cmd) =
                            parse_api_response(&body, history_body.as_deref(), schedule.as_ref())
                        {
                            if cmd_tx.send(cmd).await.is_err() {
                                break; // channel closed; audio thread stopped
                            }
                        }
                    }
                    Err(e) => tracing::warn!("Error reading metadata API body: {e}"),
                }
            }
            Ok(resp) => tracing::warn!("Metadata API HTTP {}: falling back to ICY", resp.status()),
            Err(e) => tracing::warn!("Metadata API unavailable ({e}): falling back to ICY"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}
