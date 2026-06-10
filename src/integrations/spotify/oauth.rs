use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use super::{AuthResult, SpotifyError};

const LIBRE_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";
const SCOPES: &str =
    "user-read-private user-read-playback-state user-modify-playback-state streaming \
     user-library-read playlist-read-private playlist-read-collaborative \
     user-top-read user-read-recently-played user-library-modify";

pub async fn start_flow(client_id: &str) -> AuthResult {
    let (search_token, refresh_token, userid) = match pkce_flow(client_id, 8888, "/callback").await
    {
        Ok(t) => t,
        Err(e) => return AuthResult::Failure(e),
    };
    let profile =
        match auth_profile_from_profile_result(fetch_user_profile(&search_token).await, &userid) {
            Ok(profile) => profile,
            Err(error) => return AuthResult::Failure(error),
        };
    let (username, is_premium, country, followers) = profile
        .map(|profile| {
            (
                Some(profile.display_name),
                profile.is_premium,
                profile.country,
                profile.followers,
            )
        })
        .unwrap_or((None, None, None, None));

    let (audio_token, native_error) = match pkce_flow(LIBRE_CLIENT_ID, 8898, "/login").await {
        Ok((token, _, _)) => (token, None),
        Err(error) => {
            tracing::warn!("spotify native auth failed: {error}");
            (String::new(), Some(format!("native_auth_failed: {error}")))
        }
    };

    AuthResult::Success {
        username,
        search_token,
        refresh_token,
        audio_token,
        native_error,
        is_premium,
        country,
        followers,
    }
}
pub async fn refresh_search_token(
    client_id: &str,
    refresh_token: &str,
) -> Result<(String, String), String> {
    let client = crate::http::http_client_timeout(8)
        .ok_or_else(|| "Failed to create HTTP client".to_string())?;
    let resp = client
        .post("https://accounts.spotify.com/api/token")
        .form(&[
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ])
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("{e}"))?;
    tracing::debug!("spotify refresh token — status={status}");

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Invalid JSON: {e}"))?;

    let access = json["access_token"]
        .as_str()
        .ok_or_else(|| format!("Missing access_token in refresh: {body}"))?
        .to_string();
    let refresh = json["refresh_token"]
        .as_str()
        .unwrap_or(refresh_token)
        .to_string();

    let granted_scopes = json["scope"].as_str().unwrap_or("");
    for required in SCOPES.split_whitespace() {
        if !granted_scopes.contains(required) {
            return Err(format!(
                "Missing required scope: {required}. Please reconnect your Spotify account."
            ));
        }
    }

    Ok((access, refresh))
}
async fn pkce_flow(
    client_id: &str,
    port: u16,
    path: &str,
) -> Result<(String, String, String), String> {
    let verifier = generate_verifier();
    let challenge = sha256_base64url(&verifier);
    let redirect = format!("http://127.0.0.1:{port}{path}");
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| format!("Puerto {port} ocupado: {e}"))?;

    crate::shell::open_url(&build_auth_url(client_id, &challenge, &redirect));

    let code = wait_for_callback(listener).await?;
    exchange_code(client_id, &code, &verifier, &redirect).await
}

fn generate_verifier() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    URL_SAFE_NO_PAD.encode(b)
}

fn sha256_base64url(s: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(s.as_bytes()))
}

fn build_auth_url(client_id: &str, challenge: &str, redirect_uri: &str) -> String {
    format!(
        "https://accounts.spotify.com/authorize\
         ?client_id={client_id}\
         &response_type=code\
         &redirect_uri={}\
         &code_challenge_method=S256\
         &code_challenge={challenge}\
         &scope={}",
        pct(redirect_uri),
        pct(SCOPES),
    )
}

fn pct(s: &str) -> String {
    s.bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => vec![b as char],
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}

async fn wait_for_callback(listener: TcpListener) -> Result<String, String> {
    let (mut stream, _) =
        tokio::time::timeout(std::time::Duration::from_secs(120), listener.accept())
            .await
            .map_err(|_| "Timed out waiting for authorization callback.".to_string())?
            .map_err(|e| e.to_string())?;

    let mut buf = [0u8; 4096];
    let n = tokio::time::timeout(std::time::Duration::from_secs(10), stream.read(&mut buf))
        .await
        .map_err(|_| "Timeout leyendo callback".to_string())?
        .map_err(|e| e.to_string())?;

    let req = String::from_utf8_lossy(&buf[..n]);
    let code = req
        .lines()
        .next()
        .and_then(|l| l.split('?').nth(1))
        .and_then(|qs| qs.split(' ').next())
        .and_then(|qs| qs.split('&').find(|p| p.starts_with("code=")))
        .and_then(|p| p.strip_prefix("code="))
        .map(str::to_string)
        .ok_or_else(|| {
            req.lines()
                .next()
                .and_then(|l| l.split('?').nth(1))
                .and_then(|qs| qs.split('&').find(|p| p.starts_with("error=")))
                .and_then(|p| p.strip_prefix("error="))
                .map(|e| format!("Authorization rejected: {e}"))
                .unwrap_or_else(|| "Missing authorization code".to_string())
        })?;

    let body = "<html><body style='font-family:sans-serif;text-align:center;padding:60px'>\
        <h2 style='color:#1DB954'>Authorization successful</h2>\
        <p>You can close this window and return to Reverbic.</p>\
        </body></html>";
    let _ = stream
        .write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .as_bytes(),
        )
        .await;
    Ok(code)
}
async fn exchange_code(
    client_id: &str,
    code: &str,
    verifier: &str,
    redirect: &str,
) -> Result<(String, String, String), String> {
    let client = crate::http::http_client_timeout(15)
        .ok_or_else(|| "Failed to create HTTP client".to_string())?;
    let resp = client
        .post("https://accounts.spotify.com/api/token")
        .form(&[
            ("client_id", client_id),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect),
            ("code_verifier", verifier),
        ])
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("{e}"))?;
    tracing::debug!(
        "token exchange ({}) status={status}",
        client_id_log_prefix(client_id)
    );

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("Invalid JSON: {e}"))?;

    if let Some(token) = json["access_token"].as_str() {
        let refresh = json["refresh_token"].as_str().unwrap_or("").to_string();
        let uid = json["username"].as_str().unwrap_or("").to_string();
        return Ok((token.to_string(), refresh, uid));
    }
    Err(format!(
        "Spotify ({status}): {}",
        json["error_description"]
            .as_str()
            .or_else(|| json["error"].as_str())
            .unwrap_or("Unknown error")
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpotifyUserProfile {
    pub(crate) display_name: String,
    pub(crate) is_premium: Option<bool>,
    pub(crate) country: Option<String>,
    pub(crate) followers: Option<u32>,
}

fn parse_user_profile_body(body: &str) -> Result<SpotifyUserProfile, String> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("Invalid JSON: {e}"))?;
    parse_user_profile_json(&json)
}

fn parse_user_profile_json(json: &serde_json::Value) -> Result<SpotifyUserProfile, String> {
    let display_name = json["display_name"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| json["id"].as_str().filter(|s| !s.trim().is_empty()))
        .map(str::to_string)
        .ok_or_else(|| "missing display_name".to_string())?;

    let is_premium = json
        .get("product")
        .and_then(|product| product.as_str())
        .filter(|product| !product.trim().is_empty())
        .map(|product| product.eq_ignore_ascii_case("premium"));

    let country = json["country"]
        .as_str()
        .map(str::trim)
        .filter(|country| !country.is_empty())
        .map(str::to_string);
    let followers = json["followers"]["total"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok());

    Ok(SpotifyUserProfile {
        display_name,
        is_premium,
        country,
        followers,
    })
}

fn auth_profile_from_profile_result(
    result: Result<SpotifyUserProfile, String>,
    token_userid: &str,
) -> Result<Option<SpotifyUserProfile>, String> {
    match result {
        Ok(profile) => Ok(Some(profile)),
        Err(error) if profile_error_blocks_auth(&error) => Err(error),
        Err(error) => {
            tracing::debug!("spotify profile unavailable during auth, continuing: {error}");
            Ok(token_userid_profile(token_userid))
        }
    }
}

fn token_userid_profile(token_userid: &str) -> Option<SpotifyUserProfile> {
    let display_name = token_userid.trim();
    if display_name.is_empty() {
        return None;
    }
    Some(SpotifyUserProfile {
        display_name: display_name.to_string(),
        is_premium: None,
        country: None,
        followers: None,
    })
}

fn profile_error_blocks_auth(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("premium")
        || lower.contains("developer dashboard")
        || lower.contains("development mode")
        || lower.contains("allowlist")
        || lower.contains("not registered")
        || lower.contains("blocked")
        || lower.contains("restricted")
        || lower.contains("spotify (403)")
}

fn client_id_log_prefix(client_id: &str) -> &str {
    client_id
        .char_indices()
        .nth(8)
        .map(|(idx, _)| &client_id[..idx])
        .unwrap_or(client_id)
}

pub async fn fetch_user_profile(token: &str) -> Result<SpotifyUserProfile, String> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| "Failed to create HTTP client".to_string())?;
    let resp = client
        .get("https://api.spotify.com/v1/me")
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("{e}"))?;
    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("{e}"))?;
    if !status.is_success() {
        return Err(SpotifyError::from_status(status, &body).to_string());
    }
    parse_user_profile_body(&body)
}

pub async fn fetch_username_from_token(token: &str) -> Result<String, String> {
    fetch_user_profile(token)
        .await
        .map(|profile| profile.display_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::spotify::test_fixtures;

    #[test]
    fn client_id_log_prefix_accepts_short_client_id() {
        assert_eq!(client_id_log_prefix("abc"), "abc");
    }

    #[test]
    fn client_id_log_prefix_truncates_long_client_id() {
        assert_eq!(client_id_log_prefix("abcdefghij"), "abcdefgh");
    }

    #[test]
    fn client_id_log_prefix_handles_utf8_boundary() {
        assert_eq!(client_id_log_prefix("åbcdefghij"), "åbcdefgh");
    }

    #[test]
    fn parse_user_profile_accepts_complete_legacy_profile() {
        let profile =
            parse_user_profile_body(test_fixtures::PROFILE_LEGACY_FULL).expect("profile parses");

        assert_eq!(profile.display_name, "Sewan");
        assert_eq!(profile.is_premium, Some(false));
        assert_eq!(profile.country.as_deref(), Some("CL"));
        assert_eq!(profile.followers, Some(42));
    }

    #[test]
    fn parse_user_profile_accepts_known_non_premium_product() {
        let profile = parse_user_profile_body(
            r#"{
                "display_name": "Listener",
                "id": "listener-id",
                "product": "free"
            }"#,
        )
        .expect("profile parses");

        assert_eq!(profile.is_premium, Some(false));
        assert_eq!(profile.country, None);
        assert_eq!(profile.followers, None);
    }

    #[test]
    fn parse_user_profile_keeps_premium_unknown_when_product_is_absent() {
        let profile = parse_user_profile_body(test_fixtures::PROFILE_CURRENT_MINIMAL)
            .expect("profile parses");

        assert_eq!(profile.display_name, "Listener");
        assert_eq!(profile.is_premium, None);
        assert_eq!(profile.country, None);
        assert_eq!(profile.followers, None);
    }

    #[test]
    fn parse_user_profile_falls_back_to_user_id_for_blank_name() {
        let profile = parse_user_profile_body(
            r#"{
                "display_name": "",
                "id": "listener-id",
                "product": null,
                "country": "",
                "followers": { "total": 4294967296 }
            }"#,
        )
        .expect("profile parses");

        assert_eq!(profile.display_name, "listener-id");
        assert_eq!(profile.is_premium, None);
        assert_eq!(profile.country, None);
        assert_eq!(profile.followers, None);
    }

    #[test]
    fn parse_user_profile_rejects_profile_without_name_or_id() {
        let error = parse_user_profile_body(r#"{"product":"premium"}"#)
            .expect_err("profile should be rejected");

        assert_eq!(error, "missing display_name");
    }

    #[test]
    fn profile_error_blocks_restricted_auth_failures() {
        assert!(profile_error_blocks_auth(
            "Spotify access is restricted by Development Mode or allowlist."
        ));
        assert!(profile_error_blocks_auth(
            "Spotify Premium is required for this feature."
        ));
        assert!(!profile_error_blocks_auth("Network error: timeout"));
    }

    #[test]
    fn auth_profile_keeps_successful_profile() {
        let profile = SpotifyUserProfile {
            display_name: "Listener".to_string(),
            is_premium: Some(true),
            country: Some("CL".to_string()),
            followers: Some(42),
        };

        let resolved = auth_profile_from_profile_result(Ok(profile.clone()), "")
            .expect("profile should resolve");

        assert_eq!(resolved, Some(profile));
    }

    #[test]
    fn auth_profile_allows_non_restrictive_profile_error_without_token_userid() {
        let resolved =
            auth_profile_from_profile_result(Err("Network error: timeout".to_string()), "")
                .expect("non-restrictive profile errors should not fail auth");

        assert_eq!(resolved, None);
    }

    #[test]
    fn auth_profile_falls_back_to_token_userid_for_non_restrictive_profile_error() {
        let resolved = auth_profile_from_profile_result(
            Err("Network error: timeout".to_string()),
            " listener ",
        )
        .expect("non-restrictive profile errors should not fail auth")
        .expect("token userid should become a partial profile");

        assert_eq!(resolved.display_name, "listener");
        assert_eq!(resolved.is_premium, None);
        assert_eq!(resolved.country, None);
        assert_eq!(resolved.followers, None);
    }

    #[test]
    fn auth_profile_rejects_restrictive_profile_error() {
        let error = auth_profile_from_profile_result(
            Err("Spotify access is restricted by Development Mode or allowlist.".to_string()),
            "listener",
        )
        .expect_err("restrictive profile errors should fail auth");

        assert_eq!(
            error,
            "Spotify access is restricted by Development Mode or allowlist."
        );
    }
}
