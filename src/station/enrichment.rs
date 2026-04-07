/// Metadatos especiales para estaciones conocidas.
/// El stream URL se resuelve vía RadioBrowser; si falla se usa `fallback_url`.
/// Los campos de metadatos (API, historial, schedule) son siempre 'static.
use serde::Deserialize;

use super::registry::Station;

const SERVERS: &[&str] = &[
    "https://de1.api.radio-browser.info",
    "https://at1.api.radio-browser.info",
    "https://nl1.api.radio-browser.info",
];

pub struct StationEnrichment {
    /// Clave estable usada para librería y config (no cambia entre runs).
    pub fallback_key:    &'static str,
    pub display_name:    &'static str,
    /// Término de búsqueda exacto en RadioBrowser.
    pub search_name:     &'static str,
    /// URL de stream de respaldo si RadioBrowser no devuelve nada.
    pub fallback_url:    &'static str,
    pub metadata_api_url: Option<&'static str>,
    pub history_api_url:  Option<&'static str>,
    pub schedule_url:     Option<&'static str>,
    pub show_countdown:  bool,
    pub bitrate_kbps:    Option<u16>,
}

const ENRICHMENTS: &[StationEnrichment] = &[
    StationEnrichment {
        fallback_key:    "tomorrowland_owr",
        display_name:    "Tomorrowland One World Radio",
        search_name:     "Tomorrowland One World Radio",
        fallback_url:    "http://playerservices.streamtheworld.com/api/livestream-redirect/OWR_INTERNATIONAL_ADP.aac",
        metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=main"),
        history_api_url:  None,
        schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/owr-schedule.json"),
        show_countdown:  true,
        bitrate_kbps:    Some(256),
    },
    StationEnrichment {
        fallback_key:    "tomorrowland_anthems",
        display_name:    "Tomorrowland Anthems",
        search_name:     "Tomorrowland Anthems",
        fallback_url:    "http://playerservices.streamtheworld.com/api/livestream-redirect/OWR_DAB.mp3",
        metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=anthems"),
        history_api_url:  None,
        schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/anthems-schedule.json"),
        show_countdown:  true,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "tomorrowland_daybreak",
        display_name:    "Tomorrowland Daybreak Sessions",
        search_name:     "Tomorrowland Daybreak Sessions",
        fallback_url:    "http://playerservices.streamtheworld.com/api/livestream-redirect/OWR_DAYBREAK.mp3",
        metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=daybreak"),
        history_api_url:  None,
        schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/daybreak-schedule.json"),
        show_countdown:  true,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "onlyhit_onlyhits",
        display_name:    "OnlyHits",
        search_name:     "OnlyHits",
        fallback_url:    "https://delivery.onlyhitsradio.net/onlyhit",
        metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=onlyhit"),
        history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=onlyhit&limit=10"),
        schedule_url:     None,
        show_countdown:  false,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "onlyhit_tophits",
        display_name:    "OnlyHit Top Hits",
        search_name:     "OnlyHit Top Hits",
        fallback_url:    "https://delivery.onlyhitsradio.net/tophits",
        metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=tophits"),
        history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=tophits&limit=10"),
        schedule_url:     None,
        show_countdown:  false,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "onlyhit_kpop",
        display_name:    "OnlyHit Kpop",
        search_name:     "OnlyHit Kpop",
        fallback_url:    "https://delivery.onlyhitsradio.net/kpop",
        metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=kpop"),
        history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=kpop&limit=10"),
        schedule_url:     None,
        show_countdown:  false,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "onlyhit_japan",
        display_name:    "OnlyHit Japan",
        search_name:     "OnlyHit Japan",
        fallback_url:    "https://delivery.onlyhitsradio.net/japan",
        metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=japan"),
        history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=japan&limit=10"),
        schedule_url:     None,
        show_countdown:  false,
        bitrate_kbps:    Some(128),
    },
];

/// Respuesta mínima de RadioBrowser que nos interesa.
#[derive(Deserialize)]
struct RbStation {
    url_resolved: String,
    url: String,
}

/// Intenta resolver la URL de stream de una estación vía RadioBrowser.
/// Devuelve la URL si encuentra resultado; `None` si no.
async fn fetch_url(client: &reqwest::Client, search_name: &str) -> Option<String> {
    for server in SERVERS {
        let endpoint = format!("{server}/json/stations/search");
        let result = client
            .get(&endpoint)
            .query(&[
                ("name", search_name),
                ("nameExact", "true"),
                ("order", "votes"),
                ("reverse", "true"),
                ("limit", "3"),
                ("hidebroken", "true"),
            ])
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.text().await {
                    if let Ok(stations) = serde_json::from_str::<Vec<RbStation>>(&body) {
                        if let Some(s) = stations.into_iter().next() {
                            let url = if s.url_resolved.is_empty() {
                                s.url
                            } else {
                                s.url_resolved
                            };
                            if !url.is_empty() {
                                return Some(url);
                            }
                        }
                    }
                }
            }
            _ => continue,
        }
    }
    None
}

/// Resuelve una estación: intenta RadioBrowser, cae en fallback si no encuentra nada.
async fn resolve_one(client: &reqwest::Client, e: &'static StationEnrichment) -> Station {
    match fetch_url(client, e.search_name).await {
        Some(url) => {
            tracing::info!("RadioBrowser: '{}' → {}", e.display_name, url);
            Station {
                key:             e.fallback_key.to_string(),
                name:            e.display_name.to_string(),
                url,
                metadata_api_url: e.metadata_api_url,
                history_api_url:  e.history_api_url,
                schedule_url:     e.schedule_url,
                show_countdown:  e.show_countdown,
                bitrate_kbps:    e.bitrate_kbps,
            }
        }
        None => {
            tracing::warn!(
                "RadioBrowser: '{}' no encontrado, usando URL de fallback",
                e.display_name
            );
            make_fallback(e)
        }
    }
}

fn make_fallback(e: &StationEnrichment) -> Station {
    Station {
        key:             e.fallback_key.to_string(),
        name:            e.display_name.to_string(),
        url:             e.fallback_url.to_string(),
        metadata_api_url: e.metadata_api_url,
        history_api_url:  e.history_api_url,
        schedule_url:     e.schedule_url,
        show_countdown:  e.show_countdown,
        bitrate_kbps:    e.bitrate_kbps,
    }
}

/// Resuelve todas las estaciones conocidas en paralelo.
/// Si RadioBrowser no está disponible, usa las URLs de fallback inmediatamente.
pub async fn resolve_stations() -> Vec<Station> {
    let client = match reqwest::Client::builder()
        .user_agent("reverbic/0.1")
        .timeout(std::time::Duration::from_secs(6))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("No se pudo crear cliente HTTP para resolve: {e}");
            return ENRICHMENTS.iter().map(make_fallback).collect();
        }
    };

    // Lanzar todas las resoluciones en paralelo
    let handles: Vec<_> = ENRICHMENTS
        .iter()
        .map(|e| {
            let c = client.clone();
            tokio::spawn(async move { resolve_one(&c, e).await })
        })
        .collect();

    let mut stations = Vec::with_capacity(handles.len());
    for handle in handles {
        match handle.await {
            Ok(station) => stations.push(station),
            Err(err) => tracing::error!("Error resolviendo estación: {err}"),
        }
    }
    stations
}
