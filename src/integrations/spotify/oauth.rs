use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use super::AuthResult;

const LIBRE_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";
const SCOPES: &str =
    "user-read-private user-read-playback-state user-modify-playback-state streaming";

pub async fn start_flow(client_id: &str) -> AuthResult {
    let (search_token, refresh_token, userid) = match pkce_flow(client_id, 8888, "/callback").await
    {
        Ok(t) => t,
        Err(e) => return AuthResult::Failure(e),
    };
    let (username, is_premium, country, followers) = fetch_user_profile(&search_token)
        .await
        .unwrap_or((userid, false, None, None));

    let audio_token = pkce_flow(LIBRE_CLIENT_ID, 8898, "/login")
        .await
        .map(|(t, _, _)| t)
        .unwrap_or_default();

    AuthResult::Success {
        username,
        search_token,
        refresh_token,
        audio_token,
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

fn client_id_log_prefix(client_id: &str) -> &str {
    client_id
        .char_indices()
        .nth(8)
        .map(|(idx, _)| &client_id[..idx])
        .unwrap_or(client_id)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

pub async fn fetch_user_profile(
    token: &str,
) -> Result<(String, bool, Option<String>, Option<u32>), String> {
    let client = crate::http::http_client_timeout(10)
        .ok_or_else(|| "Failed to create HTTP client".to_string())?;
    let resp = client
        .get("https://api.spotify.com/v1/me")
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| format!("{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("status {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| format!("{e}"))?;
    let name = json["display_name"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| json["id"].as_str())
        .map(str::to_string)
        .ok_or_else(|| "missing display_name".to_string())?;
    let is_premium = json["product"].as_str() == Some("premium");
    let country = json["country"].as_str().map(str::to_string);
    let followers = json["followers"]["total"].as_u64().map(|n| n as u32);
    Ok((name, is_premium, country, followers))
}

pub async fn fetch_username_from_token(token: &str) -> Result<String, String> {
    fetch_user_profile(token).await.map(|(name, _, _, _)| name)
}
