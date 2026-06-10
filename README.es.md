<p align="center">
  <img src="assets/logo.svg" alt="Reverbic" width="340">
</p>

<p align="center">Reproductor de radio en terminal y control remoto de Spotify para Windows, macOS y Linux.</p>

<p align="center">
  <img alt="Version" src="https://img.shields.io/badge/version-1.4.2-blueviolet?style=flat-square" />
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-0078d4?style=flat-square" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-CE422B?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

<p align="center">
  <a href="CHANGELOG.md">Changelog</a> |
  <a href="CHANGELOG.es.md">Changelog ES</a>
</p>

<p align="center">
  <img src="assets/Preview-Reverbic.gif" alt="Vista previa de Reverbic" width="100%">
</p>

---

## Funcionalidades

**Radio**
- Busca estaciones de radio por nombre, género o país via [radio-browser.info](https://www.radio-browser.info)
- Lista de estaciones curada con metadatos enriquecidos (codec, bitrate, etiquetas, web oficial)
- Favoritas con soporte de renombrado
- Historial de canciones recientes
- Crossfade entre estaciones (1–3 s)
- Guardar canciones en una lista local
- Catálogo de programas on-demand

**Spotify**
- Control remoto: buscar, reproducir, pausar, seek, volumen
- Transferencia de dispositivos (se requiere cuenta Premium para reproducir)
- Sub-pestañas: Búsqueda y Dispositivos
- Manejo de rate-limit con cuenta regresiva

**YouTube**
- Búsqueda y streaming de audio desde YouTube
- Nota: La reproducción no utiliza cookies ni credenciales de usuario. Los videos con restricción de edad, privados o que requieran inicio de sesión podrían fallar.

**Windows / Escritorio**
- Overlay flotante — siempre encima, posición configurable (4 esquinas) y transparencia ajustable
- Icono en la bandeja del sistema con notificaciones balloon
- Soporte de teclas de medios (Play/Pause, Stop)
- Audio ducking — reduce el volumen automáticamente cuando otra aplicación produce audio
- Detección de juegos — el overlay cambia a modo de información del juego
- Discord Rich Presence — muestra la estación y canción actual en tu perfil de Discord

**UI / UX**
- Protector de pantalla con reloj, información de la estación y metadatos de la canción
- Soporte completo de ratón (clic, scroll, doble clic)
- Búsqueda fuzzy en la lista de estaciones y el modal
- Navegación orientada al teclado
- i18n: inglés / español

---

## Capturas de pantalla

<table align="center">
  <tr>
    <td align="center">
      <img src="assets/Overlay.gif" alt="Overlay para juegos" width="280"><br>
      <sub>Overlay para juegos</sub>
    </td>
    <td align="center">
      <img src="assets/Discord-Rich-Presence.gif" alt="Discord Rich Presence" width="240"><br>
      <sub>Discord Rich Presence</sub>
    </td>
  </tr>
</table>

---

## Por que una app de terminal?

| | Reverbic | Navegador + radio web |
|---|---|---|
| Uso de RAM | ~25 MB | 300–600 MB |
| CPU en reposo | < 1 % | 3–8 % |
| Tiempo de inicio | < 1 s | 3–8 s |
| Espacio en disco | ~8 MB | 500 MB+ |
| Corre en segundo plano | La terminal debe quedar abierta | Necesita una ventana abierta |
| Teclas multimedia | Soporte nativo | Depende del sitio |
| Audio ducking | Integrado | No disponible |
| Publicidad / rastreo | Ninguno | Presente en la mayoria de sitios |
| Protector de pantalla / overlay | Si | No disponible |
| Configuracion local | JSON local | Cuenta / cookies |

---

## Instalación

### Requisitos

- Windows 10/11, macOS o Linux
- [Rust](https://rustup.rs/) (última versión estable)

> [!IMPORTANT]
> **Terminal recomendada:** [Windows Terminal](https://aka.ms/terminal) con PowerShell 7+. Reverbic funciona en CMD pero algunos iconos, animaciones y caracteres Unicode pueden no verse correctamente. Windows Terminal ofrece la mejor experiencia visual.

<table>
  <tr>
    <td align="center">
      <a href="https://apps.microsoft.com/detail/9n0dx20hk701?hl"><img src="assets/Terminal Windows.PNG" alt="Terminal Windows" width="180"></a><br>
      <sub>Terminal Windows</sub>
    </td>
    <td align="center">
      <a href="https://apps.microsoft.com/detail/9mz1snwt0n5d?hl"><img src="assets/Powershell.PNG" alt="Powershell 7" width="180"></a><br>
      <sub>Powershell 7</sub>
    </td>
  </tr>
</table>

<details>
  <summary>Classic CMD vs Modern Terminal + Powershell 7</summary>

  <table>
  <tr>
    <td align="center">
      <img src="assets/CMD.PNG" alt="Terminal Windows" width="580"><br>
      <sub>Classic CMD</sub>
    </td>
    <td align="center">
      <img src="assets/TerminalPWS7.PNG" alt="Powershell 7" width="500"><br>
      <sub>Modern Terminal + Powershell 7</sub>
    </td>
  </tr>
</table>
</details>

### Descarga

**Instalación rápida en 1 paso (PowerShell)**
```powershell
irm https://raw.githubusercontent.com/sewandev/Reverbic/main/install.ps1 | iex
```

**Vía Scoop**
```powershell
scoop bucket add reverbic https://github.com/sewandev/scoop-reverbic
scoop install reverbic
```

**Descarga directa**
Los binarios precompilados están disponibles en la [página de Releases](https://github.com/sewandev/Reverbic/releases/latest).

Descarga el archivo `.exe` desde la página de releases y ejecútalo una vez desde cualquier terminal. En el primer arranque, el binario se copia automáticamente a `%LOCALAPPDATA%\Programs\reverbic\` y agrega esa carpeta al PATH del usuario. Luego abre una nueva terminal y escribe `reverbic` desde cualquier lugar.

> **Windows SmartScreen** puede mostrar una advertencia para binarios sin firma. Haz clic en "Más información" → "Ejecutar de todas formas".

### Compilar desde el código fuente

```powershell
git clone https://github.com/sewandev/Reverbic.git
cd Reverbic
cargo build --release
.\target\release\reverbic.exe
```

### Cobertura de tests de Spotify

`cargo test` incluye fixtures locales de Spotify Web API en `src/integrations/spotify/fixtures/`.
Estos tests son offline y no requieren credenciales de Spotify, una cuenta allowlisted, Premium
ni acceso a red.

| Area | Validan las fixtures | Requisito live no cubierto por tests locales |
|------|----------------------|----------------------------------------------|
| Busqueda | Parsing de resultados de canciones, paginacion, campos opcionales ausentes | Client ID valido, scopes, disponibilidad del API |
| Biblioteca | Saved tracks, top tracks y wrappers de recently played | Contenido real de la biblioteca e historial de la cuenta |
| Albumes | Saved albums y paginacion de tracks de album | Contenido real de la biblioteca |
| Playlists | Totals actuales y legacy, wrappers de items, filtrado de no-tracks | Acceso a playlists privadas y scopes |
| Playback / Dispositivos | Estado de reproduccion de track, playback vacio o no-track, lista de dispositivos | Premium, dispositivos activos, transferencia de playback |
| Perfil | Perfiles completos y minimos, campos opcionales ausentes | Cuenta allowlisted y disponibilidad de `/v1/me` |

Cualquier check live de Spotify que se agregue en el futuro debe ser opt-in mediante variables
de entorno explicitas. No debe correr como parte de `cargo test` normal.

### Configuración de Spotify

La integración con Spotify requiere un client ID del [Spotify Developer Dashboard](https://developer.spotify.com/dashboard).

1. Crea una app en el dashboard
2. Agrega `http://127.0.0.1:8888/callback` como Redirect URI
3. Abre Reverbic, presiona `Alt+O` para abrir Ajustes, navega hasta **Spotify Client ID** y presiona `Espacio`
4. Pega tu Client ID y presiona `Enter` — no necesitas recompilar

> La reproducción de Spotify requiere una cuenta **Premium**. Las cuentas gratuitas pueden usar búsqueda y listado de dispositivos.

---

## Configuración

Todos los ajustes son accesibles desde la aplicación con `Alt+O`. No es necesario editar ningún archivo de configuración.

| Ajuste | Descripción |
|--------|-------------|
| Autoplay última estación | Reanuda la última estación al iniciar |
| Restaurar volumen | Recupera el nivel de volumen de la sesión anterior |
| Crossfade | Duración del fundido entre estaciones |
| Paso de volumen | Cambio de volumen por tecla |
| Pre-buffer | Segundos de buffer antes de reproducir on-demand |
| Modo overlay | Oculto / Al reproducir / Siempre / Solo en juegos |
| Transparencia del overlay | 0–100 % |
| Posición del overlay | Arriba-izquierda / Arriba-derecha / Abajo-izquierda / Abajo-derecha |
| Protector de pantalla | Tiempo de inactividad antes de activar el protector |
| Reloj del protector | Muestra el reloj digital grande en el protector de pantalla |
| Audio ducking | Reduce el volumen cuando otra app reproduce audio |
| Volumen duck | Nivel de volumen objetivo al activar el ducking |
| Teclas multimedia | Activar soporte de teclas multimedia |
| Bandeja del sistema | Mostrar icono de Reverbic en la bandeja del sistema |
| Notificaciones | Enviar notificaciones del sistema al cambiar de canción |
| Actualización automática | Descargar y aplicar actualizaciones automáticamente |
| Discord Rich Presence | Muestra la estación y canción actual en tu perfil de Discord |
| Idioma | Inglés / Español |
| Spotify: detener al cerrar | Detiene la reproducción de Spotify al cerrar Reverbic |
| Spotify: ir a Spotify al conectar | Cambia a la pestaña Spotify automáticamente al conectarse |
| Spotify: Client ID | Tu Client ID de la app de Spotify (presiona Espacio para editar) |

La configuración se guarda en `~/.reverbic/config.json`.

---

## Construido con

**Fuentes de datos**
| Fuente | Uso |
|--------|-----|
| [radio-browser.info](https://www.radio-browser.info) | Búsqueda de estaciones por nombre, género y país |
| [Spotify Web API](https://developer.spotify.com/documentation/web-api) | Búsqueda de canciones, control de reproducción, dispositivos |
| [Deezer API](https://developers.deezer.com) | Enriquecimiento de metadatos (artista, álbum, portada) |
| [iTunes Search API](https://developer.apple.com/library/archive/documentation/AudioVideo/Conceptual/iTuneSearchAPI) | Metadatos de canciones como respaldo |

**Librerías principales**
| Crate | Propósito |
|-------|-----------|
| [ratatui](https://github.com/ratatui-org/ratatui) | Framework de UI en terminal |
| [librespot](https://github.com/librespot-org/librespot) | Streaming de audio Spotify (Premium) |
| [rodio](https://github.com/RustAudio/rodio) | Motor de reproducción de audio |
| [tokio](https://tokio.rs) | Runtime asíncrono |
| [crossterm](https://github.com/crossterm-rs/crossterm) | Entrada/salida de terminal multiplataforma |
| [serde](https://serde.rs) | Serialización de configuración |

---

## Contribuidores

<a href="https://github.com/sewandev/Reverbic/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=sewandev/Reverbic" />
</a>
