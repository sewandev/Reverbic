use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};

use super::AuthResult;

// Leído desde .env en tiempo de compilación. Ver .env.example.
const CLIENT_ID: &str = env!("SPOTIFY_CLIENT_ID", "Falta SPOTIFY_CLIENT_ID en .env");
const SCOPES: &str = "user-read-email user-read-private streaming";

pub async fn start_flow() -> AuthResult {
    match pkce_flow().await {
        Ok(username) => AuthResult::Success { username },
        Err(msg)     => AuthResult::Failure(msg),
    }
}

async fn pkce_flow() -> Result<String, String> {
    if CLIENT_ID.is_empty() {
        return Err("SPOTIFY_CLIENT_ID no definido en .env".to_string());
    }

    let code_verifier  = generate_verifier();
    let code_challenge = sha256_base64url(&code_verifier);

    const CALLBACK_PORT: u16 = 8888;
    let listener = TcpListener::bind(("127.0.0.1", CALLBACK_PORT)).await
        .map_err(|e| format!("Puerto {CALLBACK_PORT} ocupado, cierra otras aplicaciones: {e}"))?;
    let redirect_uri = format!("http://127.0.0.1:{CALLBACK_PORT}/callback");

    let auth_url = build_auth_url(&code_challenge, &redirect_uri);
    open_browser(&auth_url);

    let code = wait_for_callback(listener).await?;
    let access_token = exchange_code(&code, &code_verifier, &redirect_uri).await?;
    fetch_username(&access_token).await
}

fn generate_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn sha256_base64url(input: &str) -> String {
    let hash = Sha256::digest(input.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

fn build_auth_url(challenge: &str, redirect_uri: &str) -> String {
    format!(
        "https://accounts.spotify.com/authorize\
         ?client_id={CLIENT_ID}\
         &response_type=code\
         &redirect_uri={}\
         &code_challenge_method=S256\
         &code_challenge={challenge}\
         &scope={}",
        percent_encode(redirect_uri),
        percent_encode(SCOPES),
    )
}

fn percent_encode(s: &str) -> String {
    s.bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![b as char]
            }
            _ => format!("%{b:02X}").chars().collect::<Vec<_>>(),
        })
        .collect()
}

fn open_browser(url: &str) {
    crate::shell::open_url(url);
}

async fn wait_for_callback(listener: TcpListener) -> Result<String, String> {
    let timeout = std::time::Duration::from_secs(120);
    let (mut stream, _) = tokio::time::timeout(timeout, listener.accept())
        .await
        .map_err(|_| "Tiempo de espera agotado. El navegador no respondió.".to_string())?
        .map_err(|e| e.to_string())?;

    let mut buf = [0u8; 4096];
    let n = tokio::time::timeout(std::time::Duration::from_secs(10), stream.read(&mut buf))
        .await
        .map_err(|_| "Timeout leyendo la respuesta del navegador".to_string())?
        .map_err(|e| e.to_string())?;

    let request = String::from_utf8_lossy(&buf[..n]);

    // GET /callback?code=XXXX&state=... HTTP/1.1
    let code = request
        .lines()
        .next()
        .and_then(|line| line.split('?').nth(1))
        .and_then(|qs| qs.split(' ').next())
        .and_then(|qs| qs.split('&').find(|p| p.starts_with("code=")))
        .and_then(|p| p.strip_prefix("code="))
        .map(str::to_string)
        .ok_or_else(|| {
            let error_desc = request
                .lines()
                .next()
                .and_then(|l| l.split('?').nth(1))
                .and_then(|qs| qs.split('&').find(|p| p.starts_with("error=")))
                .and_then(|p| p.strip_prefix("error="))
                .unwrap_or("sin código de autorización");
            format!("Autorización rechazada: {error_desc}")
        })?;

    let body = "<html><body style='font-family:sans-serif;text-align:center;padding:60px'>\
        <h2 style='color:#1DB954'>Autorización exitosa</h2>\
        <p>Puedes cerrar esta ventana y volver a Reverbic.</p>\
        </body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;

    Ok(code)
}

async fn exchange_code(code: &str, verifier: &str, redirect_uri: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id",     CLIENT_ID),
        ("grant_type",    "authorization_code"),
        ("code",          code),
        ("redirect_uri",  redirect_uri),
        ("code_verifier", verifier),
    ];
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Error de red al obtener token: {e}"))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Respuesta inválida del servidor: {e}"))?;

    if let Some(token) = json["access_token"].as_str() {
        return Ok(token.to_string());
    }
    let desc = json["error_description"]
        .as_str()
        .or_else(|| json["error"].as_str())
        .unwrap_or("Error desconocido al obtener token");
    Err(desc.to_string())
}

async fn fetch_username(access_token: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.spotify.com/v1/me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Error al obtener perfil: {e}"))?;

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Perfil inválido: {e}"))?;

    json["display_name"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| json["id"].as_str())
        .map(str::to_string)
        .ok_or_else(|| "No se pudo obtener el nombre de usuario".to_string())
}
