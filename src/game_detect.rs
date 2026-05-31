use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct GameInfo {
    pub name:  String,
    pub genre: String,
}

// (display_name, genre) del juego activo; None si no hay juego detectado.
static GAME: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
static DB:   OnceLock<HashMap<String, GameInfo>>       = OnceLock::new();

fn store() -> &'static Mutex<Option<(String, String)>> {
    GAME.get_or_init(|| Mutex::new(None))
}

/// Carga el JSON embebido y, si existe, el override del usuario en ~/.reverbic/games.json.
/// Las entradas del usuario tienen prioridad.
pub fn init_game_db() {
    const EMBEDDED: &str = include_str!("../assets/games.json");

    // Parseo leniente: saltar entradas que no sean {name, genre} (ej: _note)
    let raw: HashMap<String, serde_json::Value> =
        serde_json::from_str(EMBEDDED).unwrap_or_default();
    let mut db: HashMap<String, GameInfo> = raw
        .into_iter()
        .filter_map(|(k, v)| serde_json::from_value::<GameInfo>(v).ok().map(|i| (k, i)))
        .collect();

    // Override del usuario
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

/// Actualiza el juego activo desde el exe stem (ej: "dota2").
/// Resuelve nombre y género via la base de datos; si no hay entrada usa el nombre crudo.
/// Usa try_lock para nunca bloquear el hilo que llama.
pub fn set(raw: Option<String>) {
    let resolved = raw.as_deref().map(|n| {
        let key = n.to_lowercase();
        if let Some(info) = DB.get().and_then(|db| db.get(&key)) {
            (info.name.clone(), info.genre.clone())
        } else {
            (n.to_owned(), String::new())
        }
    });
    if let Ok(mut g) = store().try_lock() {
        *g = resolved;
    }
}

/// Nombre y género del juego activo. None si no hay juego.
pub fn get() -> Option<(String, String)> {
    store().lock().ok()?.clone()
}

/// Solo el nombre para el overlay Win32.
pub fn get_name() -> Option<String> {
    get().map(|(name, _)| name)
}
