pub mod state;
mod server;

pub use state::Dota2State;

use std::sync::{Arc, Mutex};

/// Implementación de [`crate::integrations::GameIntegration`] para Dota 2 via GSI.
pub struct Dota2Integration;

impl crate::integrations::GameIntegration for Dota2Integration {
    type State = Dota2State;
    fn get() -> Option<Self::State>               { get() }
    fn spawn_server() -> tokio::task::JoinHandle<()> { spawn_server() }
    fn reset()                                    { reset() }
}

const GSI_PORT: u16 = 7836;

const GSI_CONFIG: &str = concat!(
    "\"Reverbic\"\n{\n",
    "    \"uri\"       \"http://127.0.0.1:7836\"\n",
    "    \"timeout\"   \"5.0\"\n",
    "    \"buffer\"    \"0.1\"\n",
    "    \"throttle\"  \"0.1\"\n",
    "    \"heartbeat\" \"30.0\"\n",
    "    \"data\"\n    {\n",
    "        \"map\"    \"1\"\n",
    "        \"player\" \"1\"\n",
    "        \"hero\"   \"1\"\n",
    "    }\n}\n"
);

static STATE: std::sync::OnceLock<Arc<Mutex<Dota2State>>> = std::sync::OnceLock::new();

fn shared_state() -> Arc<Mutex<Dota2State>> {
    Arc::clone(STATE.get_or_init(|| Arc::new(Mutex::new(Dota2State::default()))))
}

pub fn get() -> Option<Dota2State> {
    STATE.get()
        .and_then(|arc| arc.try_lock().ok())
        .map(|s| s.clone())
        .filter(|s| s.phase.is_active())
}

pub fn spawn_server() -> tokio::task::JoinHandle<()> {
    tokio::spawn(server::run(GSI_PORT, shared_state()))
}

pub fn reset() {
    if let Some(arc) = STATE.get() {
        if let Ok(mut s) = arc.lock() {
            *s = Dota2State::default();
        }
    }
}

pub enum InstallResult {
    Installed { needs_restart: bool },
    AlreadyInstalled,
    SteamNotFound,
    Dota2NotFound,
    WriteError(String),
}

pub fn install_gsi_config() -> InstallResult {
    let steam_path = match find_steam_path() {
        Some(p) => p,
        None    => return InstallResult::SteamNotFound,
    };

    let cfg_dir = steam_path
        .join("steamapps").join("common")
        .join("dota 2 beta").join("game")
        .join("dota").join("cfg");

    if !cfg_dir.exists() {
        return InstallResult::Dota2NotFound;
    }

    let cfg_path = cfg_dir.join("gamestate_integration_reverbic.cfg");
    if cfg_path.exists() {
        return InstallResult::AlreadyInstalled;
    }

    match std::fs::write(&cfg_path, GSI_CONFIG) {
        Ok(_)  => {
            let needs_restart = is_dota_running();
            tracing::info!("Dota2 GSI: config instalada (reinicio requerido: {needs_restart})");
            InstallResult::Installed { needs_restart }
        }
        Err(e) => InstallResult::WriteError(e.to_string()),
    }
}

fn is_dota_running() -> bool {
    std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq dota2.exe", "/NH"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).contains("dota2.exe"))
        .unwrap_or(false)
}

fn find_steam_path() -> Option<std::path::PathBuf> {
    if let Ok(out) = std::process::Command::new("reg")
        .args(["query", r"HKCU\SOFTWARE\Valve\Steam", "/v", "SteamPath"])
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.contains("SteamPath") {
                    if let Some(raw) = line.split("REG_SZ").nth(1) {
                        let p = std::path::PathBuf::from(raw.trim());
                        if p.exists() { return Some(p); }
                    }
                }
            }
        }
    }
    for path in [r"C:\Program Files (x86)\Steam", r"C:\Program Files\Steam"] {
        let p = std::path::PathBuf::from(path);
        if p.exists() { return Some(p); }
    }
    None
}
