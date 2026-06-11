<p align="center">
  <img src="assets/logo.svg" alt="Reverbic" width="265">
</p>

<p align="center">Reproductor de terminal todo en uno — Radio, Spotify y YouTube, para Windows, macOS y Linux.</p>

<p align="center">
  <a href="https://github.com/sewandev/Reverbic/actions/workflows/ci.yml"><img alt="Build" src="https://github.com/sewandev/Reverbic/actions/workflows/ci.yml/badge.svg" /></a>
  <a href="https://github.com/sewandev/Reverbic/actions/workflows/codeql.yml"><img alt="CodeQL" src="https://github.com/sewandev/Reverbic/actions/workflows/codeql.yml/badge.svg" /></a>
  <img alt="Version" src="https://img.shields.io/github/v/release/sewandev/Reverbic?style=flat-square&label=version&color=blueviolet" />
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-0078d4?style=flat-square" />
  <img alt="Built with Rust" src="https://img.shields.io/badge/built_with-Rust-CE422B?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green?style=flat-square" />
</p>

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.es.md">Español</a>
</p>

<p align="center">
  <img src="assets/Preview-Reverbic.gif" alt="Vista previa de Reverbic" width="100%">
</p>

---

## Instalación

```powershell
# Instalación rápida (Windows)
irm https://raw.githubusercontent.com/sewandev/Reverbic/main/install.ps1 | iex

# Gestores de paquetes
scoop bucket add reverbic https://github.com/sewandev/scoop-reverbic; scoop install reverbic   # Windows (Scoop)
cargo install --git https://github.com/sewandev/Reverbic.git --locked                          # Cualquier OS (Rust)

# Compilar desde el código fuente
git clone https://github.com/sewandev/Reverbic.git
cd Reverbic
cargo build --release
.\target\release\reverbic.exe
```

> [!TIP]
> Recomendado: ejecuta Reverbic en [Windows Terminal](https://apps.microsoft.com/detail/9n0dx20hk701?hl) con [PowerShell 7+](https://apps.microsoft.com/detail/9mz1snwt0n5d?hl) para la mejor experiencia visual.

> [!WARNING]
> **Windows SmartScreen** puede mostrar una advertencia para binarios sin firma. Haz clic en "Más información" → "Ejecutar de todas formas".

---

## Funcionalidades

- **Radio** — Busca y reproduce miles de estaciones de radio por nombre, género o país
- **Spotify** — Control remoto: buscar, reproducir, pausar, seek, volumen y transferencia de dispositivos (Premium requerido)
- **YouTube** — Busca y reproduce audio directamente desde YouTube
- **Liviano** — ~25 MB de RAM y < 1% de CPU en reposo, inicia en menos de un segundo
- **Overlay flotante** — siempre encima, con detección automática de juegos
- **Discord Rich Presence** — muestra tu estación y canción actual en tu perfil
- **Favoritas y crossfade** — guarda tus estaciones favoritas con crossfade suave entre ellas
- **Protector de pantalla** — reloj, información de la estación y metadatos de la canción cuando está inactivo

> [!NOTE]
> Los cambios de política de Spotify en 2026 podrían restringir la reproducción nativa (librespot) en cualquier momento. El modo de Control Remoto (búsqueda y control de reproducción vía la API oficial de Spotify) no depende de librespot y es un respaldo razonable para ese riesgo, aunque tiene sus propios requisitos (tu propia cuenta Premium de Spotify y app de Developer). Ver [LEGAL.md](LEGAL.md) para más detalles (en inglés).

---

## Autenticación de YouTube (opcional)

Algunos videos de YouTube requieren iniciar sesión (restricción de edad, de región, o contenido solo para miembros). Reverbic puede usar un archivo `cookies.txt` para acceder a ellos.

> [!WARNING]
> **Usa una cuenta secundaria ("burner")** para esto — nunca tu cuenta principal de Google. El archivo de cookies le da a Reverbic acceso a la sesión de YouTube de esa cuenta, y yt-dlp puede reescribir el archivo a medida que las cookies rotan.

Para configurarlo:

1. Abre una **ventana privada/de incógnito** e inicia sesión en YouTube con tu cuenta secundaria.
2. Instala [Get cookies.txt LOCALLY](https://github.com/kairi003/Get-cookies.txt-LOCALLY), una extensión de código abierto que nunca envía tus cookies a ningún lado.
3. En youtube.com, exporta tus cookies en formato Netscape y guarda el archivo en un lugar privado.
4. En Reverbic, abre Ajustes y configura **Archivo de cookies de YouTube** con la ruta del archivo guardado.

Sobre los permisos del archivo: en Linux/macOS, restringe el acceso con `chmod 600 cookies.txt`; en Windows, evita guardarlo en una carpeta sincronizada con la nube (OneDrive, Dropbox, etc.).

> [!NOTE]
> Las cookies ayudan con los videos que requieren inicio de sesión, pero no garantizan resolver todos los errores de "Sign in to confirm you're not a bot" — las verificaciones anti-bot de YouTube (PO Tokens) aún pueden bloquear la reproducción en algunos casos.

Reverbic solo lee la ruta que indiques y se la pasa a yt-dlp; nunca transmite ni guarda en caché el contenido del archivo de cookies. Ver [LEGAL.md](LEGAL.md) para las notas legales de la integración con YouTube (en inglés).

---

## Capturas de pantalla

<table align="center">
  <tr>
    <td align="center">
      <img src="assets/spotify.PNG" alt="Control remoto de Spotify" width="380"><br>
      <sub>Control remoto de Spotify</sub>
    </td>
    <td align="center">
      <img src="assets/youtube.PNG" alt="Búsqueda en YouTube" width="380"><br>
      <sub>Búsqueda en YouTube</sub>
    </td>
    <td align="center">
      <img src="assets/Overlay.gif" alt="Overlay para juegos" width="380"><br>
      <sub>Overlay para juegos</sub>
    </td>
  </tr>
  <tr>
    <td align="center">
      <img src="assets/screensaver.PNG" alt="Modo protector de pantalla" width="380"><br>
      <sub>Modo protector de pantalla</sub>
    </td>
    <td align="center">
      <img src="assets/configs.PNG" alt="Configuración" width="380"><br>
      <sub>Configuración</sub>
    </td>
    <td align="center">
      <img src="assets/Discord-Rich-Presence.gif" alt="Discord Rich Presence" width="380"><br>
      <sub>Discord Rich Presence</sub>
    </td>
  </tr>
</table>

---

## Changelog

Consulta [CHANGELOG.es.md](CHANGELOG.es.md) para conocer las novedades de cada versión. ([English](CHANGELOG.md))

---

## Contribuidores

<a href="https://github.com/sewandev/Reverbic/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=sewandev/Reverbic" />
</a>
