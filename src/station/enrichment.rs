use super::registry::Station;

pub struct StationEnrichment {
    pub fallback_key:    &'static str,
    pub search_name:     &'static str,
    pub metadata_api_url: Option<&'static str>,
    pub history_api_url:  Option<&'static str>,
    pub schedule_url:     Option<&'static str>,
    pub show_countdown:  bool,
    pub bitrate_kbps:    Option<u16>,
}

const ENRICHMENTS: &[StationEnrichment] = &[
    StationEnrichment {
        fallback_key:    "tomorrowland_owr",
        search_name:     "Tomorrowland One World Radio",
        metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=main"),
        history_api_url:  None,
        schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/owr-schedule.json"),
        show_countdown:  true,
        bitrate_kbps:    Some(256),
    },
    StationEnrichment {
        fallback_key:    "tomorrowland_anthems",
        search_name:     "Tomorrowland Anthems",
        metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=anthems"),
        history_api_url:  None,
        schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/anthems-schedule.json"),
        show_countdown:  true,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "tomorrowland_daybreak",
        search_name:     "Tomorrowland Daybreak Sessions",
        metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=daybreak"),
        history_api_url:  None,
        schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/daybreak-schedule.json"),
        show_countdown:  true,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "onlyhit_onlyhits",
        search_name:     "OnlyHits",
        metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=onlyhit"),
        history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=onlyhit&limit=10"),
        schedule_url:     None,
        show_countdown:  false,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "onlyhit_tophits",
        search_name:     "OnlyHit Top Hits",
        metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=tophits"),
        history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=tophits&limit=10"),
        schedule_url:     None,
        show_countdown:  false,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "onlyhit_kpop",
        search_name:     "OnlyHit Kpop",
        metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=kpop"),
        history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=kpop&limit=10"),
        schedule_url:     None,
        show_countdown:  false,
        bitrate_kbps:    Some(128),
    },
    StationEnrichment {
        fallback_key:    "onlyhit_japan",
        search_name:     "OnlyHit Japan",
        metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=japan"),
        history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=japan&limit=10"),
        schedule_url:     None,
        show_countdown:  false,
        bitrate_kbps:    Some(128),
    },
];
fn word_set(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2)
        .map(|w| w.to_string())
        .collect()
}
pub fn find_enrichment(name: &str) -> Option<&'static StationEnrichment> {
    let name_words = word_set(name);

    ENRICHMENTS.iter().find(|e| {
        let search_words = word_set(e.search_name);
        let search_in_name = search_words.iter().all(|w| name_words.contains(w));
        let name_in_search = name_words.iter().all(|w| search_words.contains(w));

        search_in_name || name_in_search
    })
}
pub fn enrich(station: &mut Station, e: &'static StationEnrichment) {
    station.key             = e.fallback_key.to_string();
    station.metadata_api_url = e.metadata_api_url;
    station.history_api_url  = e.history_api_url;
    station.schedule_url     = e.schedule_url;
    station.show_countdown   = e.show_countdown;
    if station.bitrate_kbps.is_none() {
        station.bitrate_kbps = e.bitrate_kbps;
    }
}
