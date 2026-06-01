
use chrono::Local;

pub fn strip_version_info(title: &str) -> String {
    const VERSION_KEYWORDS: &[&str] = &[
        "remix", "edit", "mix", "version", "remaster", "live", "extended",
        "radio", "original", "club", "vip", "instrumental", "acoustic",
        "bootleg", "rework", "flip", "dub", "remi",
    ];
    let mut result = title.to_string();
    loop {
        let before = result.clone();
        for (open, close) in [('(', ')'), ('[', ']')] {
            if let Some(start) = result.find(open) {
                if let Some(rel_end) = result[start..].find(close) {
                    let end = start + rel_end;
                    let inner = result[start + 1..end].to_lowercase();
                    if VERSION_KEYWORDS.iter().any(|kw| inner.contains(kw)) {
                        let prefix = result[..start].trim_end();
                        let suffix = result[end + 1..].trim_start();
                        result = if suffix.is_empty() {
                            prefix.to_string()
                        } else {
                            format!("{prefix} {suffix}")
                        };
                    }
                }
            }
        }
        if result == before { break; }
    }
    result.trim().to_string()
}

fn log_deezer_not_found(raw: &str, query: &str) {
    use std::io::Write;
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let path = std::path::PathBuf::from(home)
        .join(".reverbic")
        .join("deezer_not_found.log");

    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let ts = Local::now().format("%Y-%m-%dT%H:%M:%S");
    let line = format!("{ts}  original: \"{raw}\"  query: {query}\n");

    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).create(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

pub async fn deezer_preview(raw: &str) -> Option<(String, String)> {
    let clean = raw.split_once("  ").map(|(_, r)| r).unwrap_or(raw).trim();
    let q = if let Some(sep) = clean.find(" - ") {
        let raw_artist = clean[..sep].trim();
        let raw_title  = clean[sep + 3..].trim();
        let primary_artist = raw_artist
            .split([',', '&'])
            .next()
            .unwrap_or(raw_artist)
            .trim();
        let clean_title = strip_version_info(raw_title);

        tracing::debug!("Deezer query: artist='{primary_artist}' track='{clean_title}' (original: '{clean}')");
        format!(r#"artist:"{primary_artist}" track:"{clean_title}""#)
    } else {
        strip_version_info(clean)
    };

    let client = crate::http::http_client()?;

    let search_resp = client
        .get("https://api.deezer.com/search")
        .query(&[("q", q.as_str()), ("limit", "1")])
        .send()
        .await
        .ok()?;

    if !search_resp.status().is_success() {
        tracing::warn!("Deezer search HTTP {}", search_resp.status());
        return None;
    }

    let body = search_resp.text().await.ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;

    let data = json["data"].as_array()?;
    if data.is_empty() {
        tracing::warn!("Deezer: sin resultado para query '{q}'");
        log_deezer_not_found(raw, &q);
        return None;
    }
    let track = data.first()?;
    let preview_url = track["preview"].as_str()?;
    if preview_url.is_empty() {
        tracing::warn!("Deezer: track encontrado pero sin preview URL");
        return None;
    }

    let artist        = track["artist"]["name"].as_str().unwrap_or("");
    let title         = track["title"].as_str().unwrap_or("");
    let display_title = if artist.is_empty() { title.to_string() } else { format!("{artist} - {title}") };

    tracing::info!("Deezer: preview encontrado para '{}' — {preview_url}", display_title);
    Some((preview_url.to_string(), display_title))
}

pub fn parse_seek_input(s: &str) -> Option<f32> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        [mins] => mins.parse::<f32>().ok().map(|m| m * 60.0),
        [mins, secs] => {
            let m = mins.parse::<f32>().ok()?;
            let sv = secs.parse::<f32>().ok()?;
            Some(m * 60.0 + sv)
        }
        [hrs, mins, secs] => {
            let h = hrs.parse::<f32>().ok()?;
            let m = mins.parse::<f32>().ok()?;
            let sv = secs.parse::<f32>().ok()?;
            Some(h * 3600.0 + m * 60.0 + sv)
        }
        _ => None,
    }
}
