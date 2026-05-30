
use serde::Deserialize;

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

pub async fn search_stations(query: &str, limit: u32) -> Option<Vec<DynamicStation>> {
    let query = query.trim();
    if query.is_empty() {
        return Some(Vec::new());
    }

    let client = reqwest::Client::builder()
        .user_agent("reverbic/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let limit_str = limit.to_string();

    for server in RADIO_BROWSER_SERVERS {
        let url = format!("{server}/json/stations/search");
        tracing::debug!("Searching Radio Browser: {url}?name={query}");

        let result = client
            .get(&url)
            .query(&[
                ("name", query),
                ("order", "votes"),
                ("reverse", "true"),
                ("limit", limit_str.as_str()),
                ("hidebroken", "true"),
            ])
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                match resp.text().await {
                    Ok(body) => {
                        match serde_json::from_str::<Vec<RadioBrowserStation>>(&body) {
                            Ok(stations) => {
                                let dynamic: Vec<DynamicStation> = stations
                                    .into_iter()
                                    .map(DynamicStation::from)
                                    .filter(|s| !s.url.is_empty())
                                    .collect();
                                tracing::info!(
                                    "Radio Browser: {} stations for '{}'",
                                    dynamic.len(),
                                    query
                                );
                                return Some(dynamic);
                            }
                            Err(e) => {
                                tracing::warn!("Radio Browser parse error from {server}: {e}");
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Radio Browser body error from {server}: {e}");
                        continue;
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!("Radio Browser HTTP {} from {server}", resp.status());
                continue;
            }
            Err(e) => {
                tracing::warn!("Radio Browser request failed from {server}: {e}");
                continue;
            }
        }
    }

    None
}

pub fn is_duplicate(url: &str, existing_urls: &[&str]) -> bool {
    let normalized = url.to_lowercase();
    existing_urls.iter().any(|u| u.to_lowercase() == normalized)
}
