# Reverbic — Reglas del proyecto

## Lenguaje
- Responder siempre en español neutro (sin argentinismos: no "escribí", "buscá", "fijate", etc.)
- Variables, funciones, structs y módulos siempre en inglés
- Commits en Conventional Commits, descripción en español neutro

## Código
- Clean Code, DRY, principios SOLID
- Sin comentarios triviales — solo el WHY no obvio
- Sin `.unwrap()` — usar `.expect("razón")` o `?`
- Sin `#[allow(dead_code)]` — si no se usa, se elimina
- Sin emojis en código ni en UI
- `render()` pura: solo lee estado, nunca muta
- Sin argentinismos en strings visibles al usuario

## Rust
- Usar el skill `rust-best-practices` siempre que se escriba o refactorice código Rust
- Sin `blocking_send` en hilos que pueden interferir con el runtime de tokio — usar `try_send` para datos no críticos
- Estado compartido entre hilos: preferir `Arc<Mutex<T>>` o globales `OnceLock` sobre canales de comandos cuando el dato es de solo lectura para la UI
- WASAPI y cualquier llamada Win32 bloqueante deben correr en hilos `std::thread` dedicados, nunca en el loop Win32 del overlay ni en tareas tokio

## Estilo de respuesta
- Respuestas cortas y directas
- Sin resúmenes al final si el diff ya lo dice
- Sin emojis
