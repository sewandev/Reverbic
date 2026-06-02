use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpotifyError {
    #[error("Error de red: {0}")]
    Network(String),

    #[error("Limite de requests alcanzado. Espera unos minutos.")]
    RateLimit,

    #[error("No autorizado. Reconecta tu cuenta de Spotify.")]
    Unauthorized,

    #[error("Se requiere Spotify Premium para esta funcion.")]
    PremiumRequired,

    #[error("Dispositivo no disponible o fuera de linea.")]
    DeviceUnavailable,

    #[error("Spotify ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("Error al procesar respuesta de Spotify: {0}")]
    Parse(String),
}

impl SpotifyError {
    pub fn from_status(status: reqwest::StatusCode, body: &str) -> Self {
        match status.as_u16() {
            429 => Self::RateLimit,
            401 => Self::Unauthorized,
            403 => {
                if body.contains("PREMIUM") || body.contains("premium") {
                    Self::PremiumRequired
                } else {
                    Self::Unauthorized
                }
            }
            404 => Self::DeviceUnavailable,
            _ => {
                let message = serde_json::from_str::<serde_json::Value>(body)
                    .ok()
                    .and_then(|j| j["error"]["message"].as_str().map(str::to_string))
                    .unwrap_or_else(|| body.chars().take(100).collect());
                Self::Api { status: status.as_u16(), message }
            }
        }
    }
}

impl From<SpotifyError> for String {
    fn from(e: SpotifyError) -> String {
        e.to_string()
    }
}
