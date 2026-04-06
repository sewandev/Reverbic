
#[derive(Debug, Clone)]
pub struct Station {
    pub key:              &'static str,
    pub name:             &'static str,
    pub url:              &'static str,
    pub metadata_api_url: Option<&'static str>,
    pub history_api_url:  Option<&'static str>,
    pub schedule_url:    Option<&'static str>,
    pub show_countdown:  bool,
    pub bitrate_kbps:    Option<u16>,
}

pub fn all_stations() -> &'static [Station] {
    static STATIONS: &[Station] = &[
        Station {
            key:              "tomorrowland_owr",
            name:             "Tomorrowland One World Radio",
            url:              "http://playerservices.streamtheworld.com/api/livestream-redirect/OWR_INTERNATIONAL_ADP.aac",
            metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=main"),
            history_api_url:  None,
            schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/owr-schedule.json"),
            show_countdown:   true,
            bitrate_kbps:     Some(256),
        },
        Station {
            key:              "tomorrowland_anthems",
            name:             "Tomorrowland Anthems",
            url:              "http://playerservices.streamtheworld.com/api/livestream-redirect/OWR_DAB.mp3",
            metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=anthems"),
            history_api_url:  None,
            schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/anthems-schedule.json"),
            show_countdown:   true,
            bitrate_kbps:     Some(128),
        },
        Station {
            key:              "tomorrowland_daybreak",
            name:             "Tomorrowland Daybreak Sessions",
            url:              "http://playerservices.streamtheworld.com/api/livestream-redirect/OWR_DAYBREAK.mp3",
            metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=daybreak"),
            history_api_url:  None,
            schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/daybreak-schedule.json"),
            show_countdown:   true,
            bitrate_kbps:     Some(128),
        },
        Station {
            key:              "onlyhit_onlyhits",
            name:             "OnlyHits",
            url:              "https://delivery.onlyhitsradio.net/onlyhit",
            metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=onlyhit"),
            history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=onlyhit&limit=10"),
            schedule_url:     None,
            show_countdown:   false,
            bitrate_kbps:     Some(128),
        },
        Station {
            key:              "onlyhit_tophits",
            name:             "OnlyHit Top Hits",
            url:              "https://delivery.onlyhitsradio.net/tophits",
            metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=tophits"),
            history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=tophits&limit=10"),
            schedule_url:     None,
            show_countdown:   false,
            bitrate_kbps:     Some(128),
        },
        Station {
            key:              "onlyhit_kpop",
            name:             "OnlyHit Kpop",
            url:              "https://delivery.onlyhitsradio.net/kpop",
            metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=kpop"),
            history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=kpop&limit=10"),
            schedule_url:     None,
            show_countdown:   false,
            bitrate_kbps:     Some(128),
        },
        Station {
            key:              "onlyhit_japan",
            name:             "OnlyHit Japan",
            url:              "https://delivery.onlyhitsradio.net/japan",
            metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=japan"),
            history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=japan&limit=10"),
            schedule_url:     None,
            show_countdown:   false,
            bitrate_kbps:     Some(128),
        },
    ];
    STATIONS
}
