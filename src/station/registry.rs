use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Station {
    pub key: String,
    pub name: String,
    pub url: String,
    pub metadata_api_url: Option<&'static str>,
    pub history_api_url: Option<&'static str>,
    pub schedule_url: Option<&'static str>,
    pub show_countdown: bool,
    pub bitrate_kbps: Option<u16>,
    pub custom_headers: Option<HashMap<String, String>>,
}
