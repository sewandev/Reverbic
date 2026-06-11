# Guía de YouTube

> [English](youtube.md) | Notas legales: [LEGAL.md](../LEGAL.md) (en inglés)

Reverbic reproduce audio de YouTube a través de [yt-dlp](https://github.com/yt-dlp/yt-dlp), usando el runtime [Deno](https://deno.com/) para resolver los desafíos de firma JavaScript de YouTube. Ambos binarios se descargan automáticamente desde sus releases oficiales de GitHub, se verifican con checksum SHA256, y **yt-dlp se mantiene actualizado solo** (chequeo diario) — YouTube cambia sus medidas anti-bot constantemente, y un yt-dlp desactualizado es la causa más común de fallos de reproducción.

## Funcionalidades

| Funcionalidad | Cómo se usa |
| --- | --- |
| Buscar y reproducir | Escribe en la pestaña de YouTube, `↵` para reproducir |
| Me gusta y playlists | Sub-pestañas con `←→` (requiere cookies, ver abajo) |
| Reproducción continua | Al terminar una canción, la siguiente de la lista suena automáticamente |
| Mix (radio infinita) | `Ctrl+R` sobre cualquier video inicia una cola de canciones similares que se extiende sola |
| Capítulos | En videos largos el capítulo actual se muestra junto al título; `[` y `]` saltan entre capítulos |
| Crossfade | Ajustes → *Crossfade (YouTube)* funde el final de cada canción con la siguiente |
| SponsorBlock | Ajustes → *SponsorBlock (YouTube)* salta las secciones sin música. Desactivado por defecto; al activarlo, el ID del video se envía a la [API comunitaria de SponsorBlock](https://sponsor.ajay.app/) |
| Seek preciso | Las pistas se descargan a un archivo temporal a velocidad completa, así que adelantar/retroceder es instantáneo y la reproducción sobrevive cortes de red |

Las URLs de stream resueltas se guardan en caché por 4 horas, así que volver a reproducir un video reciente arranca casi al instante — incluso tras reiniciar Reverbic.

## Configuración de cookies (opcional)

Algunos videos requieren iniciar sesión (restricción de edad, de región, o contenido solo para miembros). Reverbic puede usar un archivo `cookies.txt` para acceder a ellos.

> [!WARNING]
> **Usa una cuenta secundaria ("burner")** — nunca tu cuenta principal de Google. El archivo de cookies da acceso a la sesión de YouTube de esa cuenta, y yt-dlp puede reescribir el archivo a medida que las cookies rotan.

1. Abre una **ventana privada/de incógnito** e inicia sesión en YouTube con tu cuenta secundaria.
2. Instala [Get cookies.txt LOCALLY](https://github.com/kairi003/Get-cookies.txt-LOCALLY), una extensión open-source que nunca envía tus cookies a ningún lado.
3. En youtube.com, exporta tus cookies en formato Netscape y guarda el archivo en un lugar privado.
4. En Reverbic, abre Ajustes y configura **Archivo de cookies de YouTube** con la ruta del archivo guardado.
5. Usa **Validar sesión de YouTube** en Ajustes cuando quieras comprobar que las cookies siguen funcionando.

Higiene del archivo: en Linux/macOS ejecuta `chmod 600 cookies.txt`; en Windows, evita carpetas sincronizadas a la nube (OneDrive, Dropbox, etc.).

Privacidad: Reverbic solo le pasa la ruta del archivo a yt-dlp — las cookies nunca se transmiten a otro lugar, y los datos de sesión nunca se escriben en los cachés ni logs de Reverbic. Los videos públicos siempre se resuelven **sin** cookies; la sesión solo se usa como respaldo para videos que la requieren.

## Limitaciones conocidas (fuera del control de Reverbic)

- **"Sign in to confirm you're not a bot"** — el sistema anti-bot de YouTube (PO Tokens) puede bloquear el acceso al stream sin importar las cookies. Suele resolverse solo en horas; las actualizaciones de yt-dlp también ayudan, y Reverbic las instala automáticamente.
- **Un video restringido suena un segundo y salta** — los videos que requieren cookies solo obtienen el formato combinado web de YouTube, cuya variante de audio HE-AAC el decodificador aún no puede reproducir. Reverbic detecta el fallo y pasa a la siguiente pista en vez de quedarse colgado.
- **HTTP 403 en un video reproducido hace poco** — las URLs de stream expiran tras ~6 horas y están atadas a tu IP. Reverbic descarta la URL muerta automáticamente; solo vuelve a reproducir el video.
- **Un video no está disponible en el Mix o la búsqueda** — los bloqueos por región, videos eliminados y restricciones "made for kids" vienen directamente de YouTube.

## Diagnóstico

Reverbic registra cada resolución y evento de reproducción:

```powershell
Get-Content "$env:USERPROFILE\.reverbic\logs\reverbic.log" | Select-String "yt-dlp|youtube:|track finished|ended early"
```

Una resolución sana registra `resolved YouTube audio format` con `format_id=140` (audio puro AAC). Si abres un issue, incluye las líneas relevantes del log.
