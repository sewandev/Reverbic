pub mod oauth;

use librespot_core::{
    authentication::Credentials,
    config::SessionConfig,
    session::Session,
};

pub enum AuthResult {
    Success { username: String },
    Failure(String),
}

pub async fn authenticate(username: String, password: String) -> AuthResult {
    let config = SessionConfig::default();
    let credentials = Credentials::with_password(&username, &password);
    match Session::connect(config, credentials, None, false).await {
        Ok((session, _)) => AuthResult::Success { username: session.username().to_string() },
        Err(e)           => AuthResult::Failure(e.to_string()),
    }
}
