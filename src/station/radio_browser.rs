use serde::Deserialize;

pub const GENRES: &[(&str, &str)] = &[
    ("pop",        "Pop"),
    ("rock",       "Rock"),
    ("jazz",       "Jazz"),
    ("classical",  "Classical"),
    ("electronic", "Electronic"),
    ("dance",      "Dance"),
    ("hip-hop",    "Hip Hop"),
    ("country",    "Country"),
    ("latin",      "Latin"),
    ("reggae",     "Reggae"),
    ("blues",      "Blues"),
    ("soul",       "Soul"),
    ("metal",      "Metal"),
    ("ambient",    "Ambient"),
    ("folk",       "Folk"),
    ("news",       "News"),
    ("talk",       "Talk"),
    ("oldies",     "Oldies"),
    ("hits",       "Hits"),
    ("lofi",       "Lo-Fi"),
];

const RADIO_BROWSER_SERVERS: &[&str] = &[
    "https://de1.api.radio-browser.info",
    "https://at1.api.radio-browser.info",
    "https://nl1.api.radio-browser.info",
];

#[derive(Debug, Clone, Deserialize)]
pub struct RadioBrowserStation {
    pub stationuuid: String,
    pub name: String,
    pub url_resolved: String,
    pub url: String,
    pub bitrate: u32,
}

#[derive(Debug, Clone)]
pub struct DynamicStation {
    pub key: String,
    pub name: String,
    pub url: String,
    pub bitrate_kbps: Option<u16>,
}

impl From<RadioBrowserStation> for DynamicStation {
    fn from(rb: RadioBrowserStation) -> Self {
        Self {
            key: rb.stationuuid,
            name: rb.name,
            url: if rb.url_resolved.is_empty() { rb.url } else { rb.url_resolved },
            bitrate_kbps: if rb.bitrate > 0 { Some(rb.bitrate as u16) } else { None },
        }
    }
}

pub async fn search_stations(name: &str, limit: u32) -> Option<Vec<DynamicStation>> {
    fetch("name", name, limit).await
}

pub async fn search_stations_by_tag(tag: &str, limit: u32) -> Option<Vec<DynamicStation>> {
    fetch("tag", tag, limit).await
}

async fn fetch(param: &str, value: &str, limit: u32) -> Option<Vec<DynamicStation>> {
    let value = value.trim();
    if value.is_empty() {
        return Some(Vec::new());
    }

    let client = reqwest::Client::builder()
        .user_agent("reverbic/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let limit_str = limit.to_string();
    let params = [
        (param, value),
        ("order", "votes"),
        ("reverse", "true"),
        ("limit", limit_str.as_str()),
        ("hidebroken", "true"),
    ];

    for server in RADIO_BROWSER_SERVERS {
        let url = format!("{server}/json/stations/search");
        let result = client.get(&url).query(&params).send().await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                match resp.text().await {
                    Ok(body) => match serde_json::from_str::<Vec<RadioBrowserStation>>(&body) {
                        Ok(stations) => {
                            let dynamic: Vec<DynamicStation> = stations
                                .into_iter()
                                .map(DynamicStation::from)
                                .filter(|s| !s.url.is_empty())
                                .collect();
                            tracing::info!("Radio Browser [{param}={value}]: {} stations", dynamic.len());
                            return Some(dynamic);
                        }
                        Err(e) => { tracing::warn!("Radio Browser parse error from {server}: {e}"); }
                    },
                    Err(e) => { tracing::warn!("Radio Browser body error from {server}: {e}"); }
                }
            }
            Ok(resp) => { tracing::warn!("Radio Browser HTTP {} from {server}", resp.status()); }
            Err(e)   => { tracing::warn!("Radio Browser request failed from {server}: {e}"); }
        }
    }

    None
}

pub fn is_duplicate(url: &str, existing_urls: &[&str]) -> bool {
    let normalized = url.to_lowercase();
    existing_urls.iter().any(|u| u.to_lowercase() == normalized)
}
