use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct GameInfo {
    pub name:  String,
    pub genre: String,
}
static GAME: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
static DB:   OnceLock<HashMap<String, GameInfo>>       = OnceLock::new();

fn store() -> &'static Mutex<Option<(String, String)>> {
    GAME.get_or_init(|| Mutex::new(None))
}
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
pub fn set(raw: Option<String>) {
    // Solo activar si el proceso está en games.json; ignorar Steam, browsers, etc.
    let resolved = raw.as_deref().and_then(|n| {
        let key = n.to_lowercase();
        let info = DB.get()?.get(&key)?;
        Some((info.name.clone(), info.genre.clone()))
    });
    if let Ok(mut g) = store().try_lock() {
        *g = resolved;
    }
}
pub fn get() -> Option<(String, String)> {
    store().lock().ok()?.clone()
}
pub fn get_name() -> Option<String> {
    get().map(|(name, _)| name)
}
