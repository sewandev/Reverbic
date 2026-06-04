# Registro de cambios

Todos los cambios notables de Reverbic se documentan aquí.
Formato: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
Versionado: [Semantic Versioning](https://semver.org/)

> También disponible en [English](CHANGELOG.md)

---

## [Sin publicar]

---

## [1.2.0] — 2026-06-03

### Agregado
- Borde del modal animado que cicla entre los colores del logo (celeste → morado → carmesí)
- Borde del strip "en reproducción" animado mientras hay una estación sonando
- Barras del ecualizador animadas en el logo: versión SVG con CSS keyframes (README/browser) y versión TUI con caracteres de bloque Unicode

### Cambiado
- La pestaña principal de Spotify usa el verde de marca de Spotify (#1ED760)

### Corregido
- Hacer clic en cualquier parte del modal ya no interrumpe ni reinicia la radio en reproducción
- El espaciado entre las sub-pestañas de Spotify y el input de búsqueda ahora coincide con el layout de la pestaña de radio

---

## [1.1.0] — 2026-06-03

### Agregado
- Barra de progreso en el strip del overlay de Spotify con tiempo transcurrido y total
- Logo de la app visible en la vista principal cuando hay espacio suficiente en el terminal
- Atajos contextuales en el pie del modal adaptados al estado actual (resultados, modo, pestaña)
- Opción para mostrar u ocultar el reloj digital en el screensaver (activado por defecto)
- Detección de URLs de radio muertas (HTTP 404): sin reintentos, error inmediato
- Indicador visual `!` en favoritos para estaciones con URL no encontrada (404)
- Pestaña YouTube en el modal (próximamente)
- CI con `cargo check`, `cargo clippy` y `cargo fmt` en PRs hacia `main` y `develop`
- Rama `develop` como rama de integración (GitFlow)

### Cambiado
- El atajo para marcar favoritos cambió de `F` a `Alt+F`, consistente con el resto de atajos del modal
- Los separadores de sección cambian de color según el estado del reproductor (reproduciendo, pausado, etc.)

### Corregido
- La feature flag del keyring ahora apunta correctamente al store nativo de Windows (`windows-native`)

---

## [1.0.0] — 2026-06-03

### Agregado
- Reproducción de radio online vía RadioBrowser API (búsqueda por nombre, género y país)
- Integración Spotify completa: autenticación OAuth 2.0 PKCE, control de reproducción, historial y cola
- Almacenamiento seguro del refresh token de Spotify en Windows Credential Manager
- Overlay Win32 always-on-top con auto-ducking de audio via WASAPI
- Screensaver activado tras 10 s de inactividad: visualizador de barras, reloj, últimas canciones y detalles de la estación
- Integración Dota 2 GSI: detección de fase de partida e instalación automática del cfg
- Auto-instalación en PATH al primer arranque (sin permisos de administrador)
- Ícono en bandeja del sistema con restauración de ventana por doble clic
- Crossfade real entre estaciones con dos streams simultáneos
- Internacionalización español / inglés con detección automática del idioma de Windows
- Soporte de teclas multimedia (Play/Pause, Stop)
- Pestañas de favoritas, recientes, géneros y países
- Control de volumen con persistencia inmediata
- Screensaver configurable desde la pestaña Config
- Transparencia del overlay configurable
- Posición del overlay configurable
- Reconexión automática con backoff exponencial ante fallos de red
- Distribución via Scoop (`sewandev/scoop-reverbic`) y winget (`Sewandev.Reverbic`)
- GitHub Actions: build y publicación automática de releases con binario y SHA256
- Templates de issues (bug, feature, pregunta)
- Logo y assets embebidos en el ejecutable (sin dependencias externas)

[Sin publicar]: https://github.com/sewandev/Reverbic/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/sewandev/Reverbic/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/sewandev/Reverbic/releases/tag/v1.0.0
