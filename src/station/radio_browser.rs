
use serde::Deserialize;

const RADIO_BROWSER_SERVERS: &[&str] = &[
    "https://de1.api.radio-browser.info",
    "https://at1.api.radio-browser.info",
    "https://nl1.api.radio-browser.info",
];

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct RadioBrowserStation {
    pub stationuuid: String,
    pub name: String,
    pub url_resolved: String,
    pub url: String,
    pub homepage: Option<String>,
    pub favicon: Option<String>,
    pub tags: Option<String>,
    pub country: Option<String>,
    pub countrycode: Option<String>,
    pub language: Option<String>,
    pub codec: Option<String>,
    pub bitrate: u32,
    pub votes: u32,
}

#[derive(Debug, Clone)]
pub struct DynamicStation {
    pub key: String,
    pub name: String,
    pub url: String,
    #[allow(dead_code)]
    pub source: String,
    pub bitrate_kbps: Option<u16>,
}

impl From<RadioBrowserStation> for DynamicStation {
    fn from(rb: RadioBrowserStation) -> Self {
        Self {
            key: rb.stationuuid,
            name: rb.name,
            url: if rb.url_resolved.is_empty() { rb.url } else { rb.url_resolved },
            source: "radio-browser".to_string(),
            bitrate_kbps: if rb.bitrate > 0 { Some(rb.bitrate as u16) } else { None },
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum RadioBrowserError {
    Network(reqwest::Error),
    Parse(serde_json::Error),
    NoServers,
}

impl std::fmt::Display for RadioBrowserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RadioBrowserError::Network(e) => write!(f, "Network error: {e}"),
            RadioBrowserError::Parse(e) => write!(f, "Parse error: {e}"),
            RadioBrowserError::NoServers => write!(f, "No Radio Browser servers available"),
        }
    }
}

impl std::error::Error for RadioBrowserError {}

pub async fn search_stations(query: &str, limit: u32) -> Result<Vec<DynamicStation>, RadioBrowserError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::builder()
        .user_agent("reverbic/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(RadioBrowserError::Network)?;

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
                                return Ok(dynamic);
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

    Err(RadioBrowserError::NoServers)
}

pub fn is_duplicate(url: &str, existing_urls: &[&str]) -> bool {
    let normalized = url.to_lowercase();
    existing_urls.iter().any(|u| u.to_lowercase() == normalized)
}
