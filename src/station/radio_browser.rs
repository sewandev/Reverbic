use serde::Deserialize;

pub const COUNTRIES: &[(&str, &str)] = &[
    ("United States",    "United States"),
    ("Germany",          "Germany"),
    ("France",           "France"),
    ("Brazil",           "Brazil"),
    ("Mexico",           "Mexico"),
    ("Spain",            "Spain"),
    ("United Kingdom",   "United Kingdom"),
    ("Argentina",        "Argentina"),
    ("Italy",            "Italy"),
    ("Russia",           "Russia"),
    ("Poland",           "Poland"),
    ("Netherlands",      "Netherlands"),
    ("Belgium",          "Belgium"),
    ("Austria",          "Austria"),
    ("Switzerland",      "Switzerland"),
    ("Australia",        "Australia"),
    ("Canada",           "Canada"),
    ("Colombia",         "Colombia"),
    ("Chile",            "Chile"),
    ("Peru",             "Peru"),
    ("Venezuela",        "Venezuela"),
    ("Bolivia",          "Bolivia"),
    ("Uruguay",          "Uruguay"),
    ("Paraguay",         "Paraguay"),
    ("Ecuador",          "Ecuador"),
    ("Cuba",             "Cuba"),
    ("Costa Rica",       "Costa Rica"),
    ("Puerto Rico",      "Puerto Rico"),
    ("India",            "India"),
    ("Japan",            "Japan"),
    ("South Korea",      "South Korea"),
    ("China",            "China"),
    ("Thailand",         "Thailand"),
    ("Indonesia",        "Indonesia"),
    ("Philippines",      "Philippines"),
    ("Turkey",           "Turkey"),
    ("Israel",           "Israel"),
    ("Ukraine",          "Ukraine"),
    ("Czech Republic",   "Czech Republic"),
    ("Romania",          "Romania"),
    ("Hungary",          "Hungary"),
    ("Greece",           "Greece"),
    ("Portugal",         "Portugal"),
    ("Sweden",           "Sweden"),
    ("Norway",           "Norway"),
    ("Denmark",          "Denmark"),
    ("Finland",          "Finland"),
    ("Ireland",          "Ireland"),
    ("New Zealand",      "New Zealand"),
    ("South Africa",     "South Africa"),
    ("Nigeria",          "Nigeria"),
    ("Kenya",            "Kenya"),
    ("Egypt",            "Egypt"),
    ("Morocco",          "Morocco"),
    ("Bulgaria",         "Bulgaria"),
    ("Serbia",           "Serbia"),
    ("Croatia",          "Croatia"),
    ("Slovakia",         "Slovakia"),
    ("Lithuania",        "Lithuania"),
    ("Latvia",           "Latvia"),
    ("Estonia",          "Estonia"),
    ("Slovenia",         "Slovenia"),
];

pub const GENRES: &[(&str, &str)] = &[
    ("pop",                        "Pop"),
    ("music",                      "Music"),
    ("rock",                       "Rock"),
    ("news",                       "News"),
    ("radio",                      "Radio"),
    ("entretenimiento",            "Entretenimiento"),
    ("fm",                         "FM"),
    ("classical",                  "Classical"),
    ("dance",                      "Dance"),
    ("talk",                       "Talk"),
    ("hits",                       "Hits"),
    ("oldies",                     "Oldies"),
    ("pop music",                  "Pop Music"),
    ("80s",                        "80s"),
    ("top 40",                     "Top 40"),
    ("jazz",                       "Jazz"),
    ("public radio",               "Public Radio"),
    ("90s",                        "90s"),
    ("electronic",                 "Electronic"),
    ("christian",                  "Christian"),
    ("adult contemporary",         "Adult Contemporary"),
    ("classic hits",               "Classic Hits"),
    ("classic rock",               "Classic Rock"),
    ("pop rock",                   "Pop Rock"),
    ("community radio",            "Community Radio"),
    ("local news",                 "Local News"),
    ("house",                      "House"),
    ("country",                    "Country"),
    ("alternative",                "Alternative"),
    ("regional mexican",           "Regional Mexican"),
    ("folk",                       "Folk"),
    ("local radio",                "Local Radio"),
    ("noticias",                   "Noticias"),
    ("70s",                        "70s"),
    ("regional mexicana",          "Regional Mexicana"),
    ("regional",                   "Regional"),
    ("information",                "Information"),
    ("variety",                    "Variety"),
    ("news talk",                  "News Talk"),
    ("grupera",                    "Grupera"),
    ("metal",                      "Metal"),
    ("soul",                       "Soul"),
    ("indie",                      "Indie"),
    ("chillout",                   "Chillout"),
    ("easy listening",             "Easy Listening"),
    ("techno",                     "Techno"),
    ("retro",                      "Retro"),
    ("sports",                     "Sports"),
    ("banda",                      "Banda"),
    ("ambient",                    "Ambient"),
    ("top hits",                   "Top Hits"),
    ("religious",                  "Religious"),
    ("disco",                      "Disco"),
    ("hip-hop",                    "Hip-Hop"),
    ("reggae",                     "Reggae"),
    ("blues",                      "Blues"),
    ("latin",                      "Latin"),
    ("lofi",                       "Lo-Fi"),
    ("trance",                     "Trance"),
    ("rnb",                        "R&B"),
    ("gospel",                     "Gospel"),
    ("classical music",            "Classical Music"),
    ("soft rock",                  "Soft Rock"),
    ("hard rock",                  "Hard Rock"),
    ("rap",                        "Rap"),
    ("funk",                       "Funk"),
    ("jazz blues",                 "Jazz Blues"),
    ("world music",                "World Music"),
    ("salsa",                      "Salsa"),
    ("cumbia",                     "Cumbia"),
    ("tropical",                   "Tropical"),
    ("musica tropical",            "Tropical"),
    ("bachata",                    "Bachata"),
    ("mexicana",                   "Mexicana"),
    ("ranchera",                   "Ranchera"),
    ("norteña",                    "Norteña"),
    ("musica regional mexicana",   "Música Regional MX"),
    ("spanish",                    "Spanish"),
    ("romance",                    "Romance"),
    ("musica variada",             "Música Variada"),
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

#[derive(Debug, Clone)]
pub struct StationDetails {
    pub homepage:   String,
    pub country:    String,
    pub language:   String,
    pub tags:       Vec<String>,
    pub codec:      String,
    pub bitrate:    u32,
}

fn parse_details(s: &serde_json::Value) -> StationDetails {
    let tags = s["tags"].as_str().unwrap_or("")
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .take(4)
        .collect();
    StationDetails {
        homepage: s["homepage"].as_str().unwrap_or("").to_string(),
        country:  s["country"].as_str().unwrap_or("").to_string(),
        language: s["language"].as_str().unwrap_or("").to_string(),
        tags,
        codec:    s["codec"].as_str().unwrap_or("").to_string(),
        bitrate:  s["bitrate"].as_u64().unwrap_or(0) as u32,
    }
}

fn build_http_client() -> Option<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("reverbic/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()
}

async fn fetch_first(url_path: &str) -> Option<StationDetails> {
    let client = build_http_client()?;
    for server in RADIO_BROWSER_SERVERS {
        let url = format!("{server}{url_path}");
        let Ok(resp) = client.get(&url).send().await else { continue };
        if !resp.status().is_success() { continue }
        let Ok(body) = resp.text().await else { continue };
        let Ok(list) = serde_json::from_str::<Vec<serde_json::Value>>(&body) else { continue };
        if let Some(s) = list.into_iter().next() {
            return Some(parse_details(&s));
        }
    }
    None
}

pub fn is_uuid(s: &str) -> bool {
    s.len() == 36 && s.chars().filter(|&c| c == '-').count() == 4
}

pub async fn fetch_station_details(uuid: &str) -> Option<StationDetails> {
    fetch_first(&format!("/json/stations/byuuid/{uuid}")).await
}

pub async fn fetch_station_details_by_name(name: &str) -> Option<StationDetails> {
    let encoded = urlencoding_simple(name);
    fetch_first(&format!("/json/stations/search?name={encoded}&limit=1&hidebroken=true")).await
}

fn urlencoding_simple(s: &str) -> String {
    s.chars().map(|c| match c {
        ' '  => '+'.to_string(),
        c if c.is_alphanumeric() || "-_.~".contains(c) => c.to_string(),
        c    => format!("%{:02X}", c as u32),
    }).collect()
}


pub async fn search_stations(name: &str, limit: u32) -> Option<Vec<DynamicStation>> {
    fetch("name", name, limit).await
}

pub async fn search_stations_by_tag(tag: &str, limit: u32) -> Option<Vec<DynamicStation>> {
    fetch("tag", tag, limit).await
}

pub async fn search_stations_by_country(country: &str, limit: u32) -> Option<Vec<DynamicStation>> {
    fetch("country", country, limit).await
}

async fn fetch(param: &str, value: &str, limit: u32) -> Option<Vec<DynamicStation>> {
    let value = value.trim();
    if value.is_empty() {
        return Some(Vec::new());
    }

    let client = build_http_client()?;

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

