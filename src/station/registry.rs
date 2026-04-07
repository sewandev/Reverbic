/// Estación de radio. Los campos de texto son propios (String) para soportar
/// tanto estaciones hardcoded como las resueltas vía RadioBrowser API.
#[derive(Debug, Clone)]
pub struct Station {
    /// Identificador estable, usado para rutas de librería y config.
    pub key:              String,
    pub name:             String,
    pub url:              String,
    /// URLs de metadatos para estaciones conocidas (siempre 'static, del enrichment).
    pub metadata_api_url: Option<&'static str>,
    pub history_api_url:  Option<&'static str>,
    pub schedule_url:     Option<&'static str>,
    pub show_countdown:   bool,
    pub bitrate_kbps:     Option<u16>,
}
