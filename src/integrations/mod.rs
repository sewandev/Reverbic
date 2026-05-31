pub mod dota2;

/// Contrato que toda integración de juego debe cumplir.
///
/// Cada juego vive en su propio módulo bajo `integrations/<juego>/` e implementa
/// este trait en una struct unit (ej. `pub struct Dota2Integration`). Esto garantiza
/// en tiempo de compilación que el módulo expone la API esperada.
///
/// # Para agregar un nuevo juego
/// 1. Crear `src/integrations/<juego>/` con `mod.rs`, `server.rs`, `state.rs`
/// 2. Definir `pub struct <Juego>Integration`
/// 3. Implementar `GameIntegration for <Juego>Integration`
/// 4. Agregar `pub mod <juego>;` en este archivo
/// 5. Agregar el campo en `GameIntegrationsConfig` y los items en `SettingItem`
pub trait GameIntegration {
    /// Estado del juego expuesto al resto de la app. Debe ser barato de clonar.
    type State: Clone + Default + Send + 'static;

    /// Devuelve el estado actual si hay datos activos (partida en curso).
    /// Retorna `None` si el servidor no recibió datos o la partida terminó.
    fn get() -> Option<Self::State>;

    /// Levanta el servidor que recibe eventos del juego. Retorna el handle
    /// para poder abortar la tarea cuando el usuario desactiva la integración.
    fn spawn_server() -> tokio::task::JoinHandle<()>;

    /// Limpia el estado. Se llama al desactivar la integración o al cerrar la app.
    fn reset();
}
