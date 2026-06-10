# Registro de cambios

Todos los cambios notables de Reverbic se documentan aquí.
Formato: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
Versionado: [Semantic Versioning](https://semver.org/)

> También disponible en [English](CHANGELOG.md)

---

## [Sin publicar]

### Agregado
- Bienvenida inicial con animación del logo, música ambiental y opciones de configuración inicial (overlay, autoplay, restaurar volumen)
- Opción "Ver bienvenida de nuevo" en Ajustes para repetir la experiencia de primer inicio
- Reproducción continua en Spotify: al terminar una canción, la siguiente del contexto cargado avanza automáticamente (carga en lote via Spirc para reproducción sin pausas)
- Modo radio en Spotify: cuando se agota la cola, se reproducen automáticamente canciones similares del mismo artista; se puede desactivar en Ajustes
- Pestaña "Me gusta" en Spotify: explorar y reproducir canciones guardadas con paginación
- Pestaña Listas de reproducción en Spotify: explorar listas propias, abrirlas y reproducir canciones con continuación secuencial

---

## [1.4.2] — 2026-06-06

### Agregado
- Extracción del tema de UI a un sistema modular de paletas para permitir temas dinámicos
- Nuevo estilo de overlay compacto (`compact`)
- CI obligatorio y protección estricta de GitHub Actions en `develop`

### Corregido
- Corregida una aserción rota del test unitario para el ancho del layout del modal

---

## [1.4.1] — 2026-06-06

### Agregado
- Fortalecida la validación de payloads del updater contra riesgos de seguridad

---

## [1.4.2] — 2026-06-06

### Agregado
- Extracción del tema de UI a un sistema modular de paletas para permitir temas dinámicos
- Nuevo estilo de overlay compacto (`compact`)
- CI obligatorio y protección estricta de GitHub Actions en `develop`

### Corregido
- Corregida una aserción rota del test unitario para el ancho del layout del modal

---

## [1.4.1] — 2026-06-06

### Agregado
- Fortalecida la validación de payloads del updater contra riesgos de seguridad

---

## [1.4.0] — 2026-06-05

### Agregado
- Pestaña de YouTube nativa usando `yt-dlp` (búsqueda y reproducción)
- Instalación automática de `yt-dlp` en el primer uso
- Soporte para streaming bajo demanda en YouTube (reanuda la canción en lugar de reiniciarla tras un corte de red)

### Corregido
- Solucionada vulnerabilidad de inyección de rutas en PowerShell durante la instalación
- La persistencia del token de Spotify ahora usa correctamente el Credential Manager de Windows sin causar un panic si falta la configuración local
- Mejorada la lógica `on_demand` para clasificar correctamente los streams de YouTube
- Solucionadas rutas multiplataforma (macOS) para los binarios de `yt-dlp`

---

## [1.3.1] — 2026-06-04

### Cambiado
- Overlay: el título de la canción y las canciones recientes ahora son más brillantes y legibles
- Overlay: el reloj y el bitrate usan la fuente de marca (negrita, más grande) para mayor visibilidad
- Overlay: el nombre de la estación y el título en reproducción muestran más caracteres antes de truncar
- Overlay: indicador DUCK agregado para mostrar el estado del auto-duck de un vistazo
- La sub-pestaña Favoritas ahora muestra el total entre paréntesis junto al label

### Corregido
- Eliminado log de depuración sobrante de la auto-selección de dispositivo Spotify

---

## [1.3.0] — 2026-06-04

### Agregado
- Panel de modo gaming visible sobre el screensaver de radio cuando se detecta un juego en ejecución
- La sub-pestaña Favoritos ahora muestra país, etiquetas y URL de cada estación guardada
- Enriquecimiento automático al iniciar la app: completa los metadatos faltantes (país, etiquetas, URL) de los favoritos guardados

### Cambiado
- Overlay Win32 rediseñado: ventana más grande (380×145 px), 9 barras VU animadas con onda sinusoidal por barra, reloj en tiempo real, indicador de bitrate, barra de volumen y las últimas 2 canciones reproducidas en lugar del nombre del juego
- El label "Gaming Mode" en el panel de juego usa el color animado del borde (negrita)

### Corregido
- Corregido el panic "A Tokio 1.x context was found, but it is being shutdown" al pausar Spotify al cambiar a radio o al activar el screensaver

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

[Sin publicar]: https://github.com/sewandev/Reverbic/compare/v1.3.0...HEAD
[1.3.0]: https://github.com/sewandev/Reverbic/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/sewandev/Reverbic/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/sewandev/Reverbic/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/sewandev/Reverbic/releases/tag/v1.0.0
