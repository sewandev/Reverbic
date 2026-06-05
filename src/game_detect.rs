#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::{Mutex, OnceLock};

#[cfg(target_os = "windows")]
use serde::Deserialize;

#[cfg(target_os = "windows")]
#[derive(Clone, Deserialize)]
pub struct GameInfo {
    pub name: String,
    pub genre: String,
}
#[cfg(target_os = "windows")]
static GAME: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
#[cfg(target_os = "windows")]
static DB: OnceLock<HashMap<String, GameInfo>> = OnceLock::new();

#[cfg(target_os = "windows")]
fn store() -> &'static Mutex<Option<(String, String)>> {
    GAME.get_or_init(|| Mutex::new(None))
}

#[cfg(target_os = "windows")]
pub fn init_game_db() {
    const EMBEDDED: &str = include_str!("../assets/games.json");
    let raw: HashMap<String, serde_json::Value> =
        serde_json::from_str(EMBEDDED).unwrap_or_default();
    let mut db: HashMap<String, GameInfo> = raw
        .into_iter()
        .filter_map(|(k, v)| serde_json::from_value::<GameInfo>(v).ok().map(|i| (k, i)))
        .collect();
    let user_path = crate::config::reverbic_dir().join("games.json");
    if let Ok(data) = std::fs::read_to_string(&user_path) {
        if let Ok(raw_user) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&data) {
            for (k, v) in raw_user {
                if let Ok(info) = serde_json::from_value::<GameInfo>(v) {
                    db.insert(k, info);
                }
            }
        }
    }

    let _ = DB.set(db);
}

#[cfg(not(target_os = "windows"))]
pub fn init_game_db() {}

#[cfg(target_os = "windows")]
pub fn set(raw: Option<String>) {
    let resolved = raw.as_deref().and_then(|n| {
        let key = n.to_lowercase();
        let info = DB.get()?.get(&key)?;
        Some((info.name.clone(), info.genre.clone()))
    });
    if let Ok(mut g) = store().try_lock() {
        *g = resolved;
    }
}

#[cfg(target_os = "windows")]
pub fn get() -> Option<(String, String)> {
    store().lock().ok()?.clone()
}

#[cfg(not(target_os = "windows"))]
pub fn get() -> Option<(String, String)> {
    None
}
