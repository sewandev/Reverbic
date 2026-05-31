use std::sync::{Mutex, OnceLock};

static GAME: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn store() -> &'static Mutex<Option<String>> {
    GAME.get_or_init(|| Mutex::new(None))
}

/// Llamado desde el hilo wasapi-monitor. Usa try_lock para nunca bloquear.
pub fn set(name: Option<String>) {
    if let Ok(mut g) = store().try_lock() {
        *g = name;
    }
}

/// Llamado desde el loop de render. Lock instantáneo — nadie lo retiene largo.
pub fn get() -> Option<String> {
    store().lock().ok()?.clone()
}
