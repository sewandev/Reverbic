
#[derive(Debug, Clone)]
pub struct Station {
    pub key:              &'static str,
    pub name:             &'static str,
    pub url:              &'static str,
    pub metadata_api_url: Option<&'static str>,
    pub history_api_url:  Option<&'static str>,
    pub schedule_url:    Option<&'static str>,
    pub show_countdown:  bool,
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
        },
        Station {
            key:              "tomorrowland_anthems",
            name:             "Tomorrowland Anthems",
            url:              "http://playerservices.streamtheworld.com/api/livestream-redirect/OWR_DAB.mp3",
            metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=anthems"),
            history_api_url:  None,
            schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/anthems-schedule.json"),
            show_countdown:   true,
        },
        Station {
            key:              "tomorrowland_daybreak",
            name:             "Tomorrowland Daybreak Sessions",
            url:              "http://playerservices.streamtheworld.com/api/livestream-redirect/OWR_DAYBREAK.mp3",
            metadata_api_url: Some("https://playout-metadata.tomorrowland.com/metadata?tag=daybreak"),
            history_api_url:  None,
            schedule_url:     Some("https://owr-schedule-cdn.tomorrowland.com/daybreak-schedule.json"),
            show_countdown:   true,
        },
        Station {
            key:              "onlyhit_japan",
            name:             "OnlyHit Japan",
            url:              "https://delivery.onlyhitsradio.net/japan",
            metadata_api_url: Some("https://onlyhit.us/api/nowplaying/now?station=japan"),
            history_api_url:  Some("https://onlyhit.us/api/nowplaying/history?station=japan&limit=10"),
            schedule_url:     None,
            show_countdown:   false,
        },
    ];
    STATIONS
}
