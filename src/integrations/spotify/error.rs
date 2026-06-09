use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpotifyError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Rate limit reached. Retry in {0}s.")]
    RateLimit(u64),

    #[error("Unauthorized. Reconnect your Spotify account.")]
    Unauthorized,

    #[error("Spotify Premium is required for this feature.")]
    PremiumRequired,

    #[error("Spotify access is restricted by Development Mode or allowlist.")]
    DevelopmentModeRestricted,

    #[error("Device unavailable or offline.")]
    DeviceUnavailable,

    #[error("Spotify ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("Error processing Spotify response: {0}")]
    Parse(String),
}

impl SpotifyError {
    pub fn from_status(status: reqwest::StatusCode, body: &str) -> Self {
        let message = spotify_error_message(body);
        match status.as_u16() {
            429 => Self::RateLimit(60),
            401 => Self::Unauthorized,
            403 => {
                let lower = message.to_ascii_lowercase();
                if lower.contains("premium") || lower.contains("subscription") {
                    Self::PremiumRequired
                } else if lower.contains("developer dashboard")
                    || lower.contains("development mode")
                    || lower.contains("allowlist")
                    || lower.contains("not registered")
                    || lower.contains("blocked")
                    || lower.contains("restricted")
                {
                    Self::DevelopmentModeRestricted
                } else {
                    Self::Api {
                        status: status.as_u16(),
                        message,
                    }
                }
            }
            404 => Self::DeviceUnavailable,
            _ => Self::Api {
                status: status.as_u16(),
                message,
            },
        }
    }
}

fn spotify_error_message(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|json| {
            json["error"]["message"]
                .as_str()
                .or_else(|| json["error_description"].as_str())
                .or_else(|| json["error"].as_str())
                .map(str::to_string)
        })
        .filter(|message| !message.trim().is_empty())
        .unwrap_or_else(|| {
            let fallback = body.trim();
            if fallback.is_empty() {
                "Unknown error".to_string()
            } else {
                fallback.chars().take(100).collect()
            }
        })
}

impl From<SpotifyError> for String {
    fn from(e: SpotifyError) -> String {
        e.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_status_maps_premium_403() {
        let error = SpotifyError::from_status(
            reqwest::StatusCode::FORBIDDEN,
            r#"{"error":{"message":"Premium account required"}}"#,
        );

        assert!(matches!(error, SpotifyError::PremiumRequired));
    }

    #[test]
    fn from_status_maps_development_mode_403() {
        let error = SpotifyError::from_status(
            reqwest::StatusCode::FORBIDDEN,
            r#"{"error":{"message":"User not registered in the Developer Dashboard"}}"#,
        );

        assert!(matches!(error, SpotifyError::DevelopmentModeRestricted));
    }

    #[test]
    fn from_status_preserves_unknown_403_message() {
        let error = SpotifyError::from_status(
            reqwest::StatusCode::FORBIDDEN,
            r#"{"error":{"message":"Forbidden for this resource"}}"#,
        );

        assert!(matches!(
            error,
            SpotifyError::Api {
                status: 403,
                ref message
            } if message == "Forbidden for this resource"
        ));
    }

    #[test]
    fn from_status_uses_plain_body_when_json_is_unavailable() {
        let error =
            SpotifyError::from_status(reqwest::StatusCode::BAD_REQUEST, "plain spotify error");

        assert!(matches!(
            error,
            SpotifyError::Api {
                status: 400,
                ref message
            } if message == "plain spotify error"
        ));
    }
}
