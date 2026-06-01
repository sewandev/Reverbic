
pub fn http_client() -> Option<reqwest::Client> {
    http_client_timeout(10)
}

pub fn http_client_timeout(secs: u64) -> Option<reqwest::Client> {
    match reqwest::Client::builder()
        .user_agent("reverbic/0.1")
        .timeout(std::time::Duration::from_secs(secs))
        .build()
    {
        Ok(client) => Some(client),
        Err(e) => {
            tracing::error!("No se pudo construir el cliente HTTP: {e}");
            None
        }
    }
}
