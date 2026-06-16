# Registro de cambios

Todos los cambios notables de Reverbic se documentan aquí.
Formato: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
Versionado: [Semantic Versioning](https://semver.org/)

> También disponible en [English](CHANGELOG.md)

---

## [Sin publicar]

### Added
- Controles de pista siguiente/anterior (Ctrl+Derecha / Ctrl+Izquierda) para la reproducción de YouTube, que recorren el contexto actual (resultados de búsqueda, playlist, me gusta, favoritos o mix). El anterior se detiene en el primer elemento; el siguiente extiende el mix o continúa con YouTube Radio al final de una lista, igual que el avance automático.
- Siguiente/anterior (Ctrl+Derecha / Ctrl+Izquierda) también funcionan en la radio: recorren la lista desde la que se reprodujo la estación (Favoritos o resultados de Búsqueda), o avanzan dentro de la playlist activa. Aparece un aviso breve al inicio/fin de la lista, o cuando no hay nada en reproducción para navegar.
- Siguiente/anterior (Ctrl+Derecha / Ctrl+Izquierda) también funcionan en Spotify: en modo nativo recorren la cola local y un historial de sesión para el anterior; en modo remoto saltan de pista en el dispositivo activo mediante la Web API de Spotify. El atajo Ctrl+Izquierda/Derecha ahora aparece en la barra de controles del Modo Ambiente y en el panel de ayuda.
- El título de la ventana de la terminal ahora refleja lo que se está reproduciendo (por ejemplo, "Reverbic v1.5.5 Radio", "Reverbic v1.5.5 YouTube" o "Reverbic v1.5.5 Spotify"), y vuelve a mostrar solo la versión cuando no hay nada sonando.
- Nueva subpestaña "Playlists públicas" en YouTube, justo al lado de "Buscar", que busca listas de reproducción públicas por nombre (por ejemplo, escribir "nier automata" lista las playlists que coinciden). Al igual que "Buscar", funciona sin iniciar sesión; abre una playlist para explorar y reproducir sus videos.

### Changed
- La configuración del Modo Ambiente ahora se abre en una ventana emergente propia (como el selector de temas) en lugar de expandirse en la lista de ajustes. Al elegir "Modo Ambiente" se abre un modal pequeño donde se ajusta el tiempo de activación y se activan o desactivan todos los widgets (reloj, logo, visualizador, pistas recientes, barra de progreso, detalles de la estación, reproducción actual).
- La configuración del Overlay también se abre ahora en una ventana emergente propia: al elegir "Overlay" se abre un modal pequeño con el modo de visualización, el estilo, la transparencia y la posición. El Modo Ambiente y el Overlay ahora son secciones separadas en Configuración en lugar de estar agrupados juntos.

### Fixed
- La reproducción de audio ya no corre el riesgo de un fallo en cascada si un bloqueo interno del stream queda en estado inconsistente; el reproductor ahora se recupera en lugar de cerrarse abruptamente.
- YouTube ahora se recupera automáticamente dentro de la misma sesión si el runtime de Deno incluido desaparece o se corrompe: tras un resolve fallido vuelve a verificar el runtime (como máximo una vez cada pocos minutos) y lo reinstala si hace falta, en lugar de fallar hasta reiniciar la app.
- YouTube ya no deja de funcionar en instalaciones antiguas: el runtime de Deno incluido ahora se mantiene actualizado automáticamente (igual que yt-dlp), de modo que la reproducción sigue funcionando tras una actualización de yt-dlp que exige un Deno más nuevo.
- Se mejoró la fiabilidad del audio de YouTube para que más videos se resuelvan a una pista de solo audio limpia en lugar de caer a un formato combinado de menor calidad.

### Security
- Los identificadores de playlists de YouTube ahora se validan antes de usarse para construir la URL de la solicitud, igual que la verificación ya existente para los identificadores de video (defensa en profundidad).
- Las descargas de actualizaciones ahora usan un directorio privado del usuario en lugar de la carpeta temporal compartida del sistema, cerrando un vector teórico de secuestro por symlink local durante la auto-actualización en sistemas multiusuario.

## [1.5.5] - 2026-06-14

### Added
- Nuevo modo headless por línea de comandos para la radio en Windows: `reverbic play <estación>` inicia la reproducción en segundo plano y devuelve la terminal, y `reverbic stop`, `reverbic status`, `reverbic volume <0-100>` y `reverbic toggle` controlan el reproductor en curso; la reproducción continúa aunque se cierre la terminal. La estación se busca primero entre tus favoritos y luego mediante una búsqueda en línea; `reverbic play` sin nombre reanuda la última estación. Ejecutar `reverbic` sin argumentos sigue abriendo la interfaz completa.

### Changed
- Reverbic ahora guarda sus archivos siguiendo las ubicaciones estándar de cada sistema operativo (configuración, datos y caché quedan separados) en lugar de una única carpeta `~/.reverbic`. Las instalaciones existentes se migran automáticamente en el primer arranque, así que no se pierde ninguna configuración.

## [1.5.4] - 2026-06-13

### Agregado
- Se agregó el ajuste "Crossfade (Spotify)" (hasta 12 segundos) que funde el final de cada canción con el inicio de la siguiente al usar el modo de reproducción Nativo; el ajuste aparece deshabilitado hasta seleccionar el modo Nativo.
- La pestaña de búsqueda de YouTube ahora muestra una pista de que puedes presionar Ctrl+R sobre un video para iniciar una radio infinita.
- Reverbic ahora se actualiza solo en Linux (x86_64), igual que en Windows y macOS: se publica un binario de Linux por release y la app descarga, verifica e instala la nueva versión automáticamente.
- Reverbic ahora publica compilaciones para macOS (Intel y Apple Silicon) y se actualiza solo en macOS, igual que el auto-actualizador de Windows.

### Cambiado
- El tooltip de la opción "YouTube Radio" ahora indica que requiere un cookies.txt configurado, ya que YouTube bloquea los mixes en solicitudes sin autenticar.
- En macOS y Linux, las opciones que solo funcionan en Windows (overlay, ducking, teclas multimedia, ícono en la bandeja, notificaciones y Discord Rich Presence) ya no aparecen en Configuración ni en el asistente de primera ejecución.
- Se rediseñó el bloque de perfil de Spotify en el Modo Ambiente como un widget propio con texto centrado: el nombre destaca y una sola línea muestra Premium, país y cantidad de seguidores (ahora con separador de miles).
- Se reorganizó el menú de Configuración en categorías más claras: Radio, Spotify y YouTube tienen su propia sección, separadas de Overlay, Ducking, Sistema y Apariencia.
- Las secciones de configuración de Overlay y Ducking ahora indican "(solo Windows)" para dejar claro que no aplican en otras plataformas.
- El panel de atajos ([?]) ahora agrupa las teclas por ámbito con encabezados de sección (Radio, Spotify, YouTube, Global) en lugar de mostrar una sola lista plana.

### Corregido
- Iniciar una radio de YouTube (Ctrl+R o al terminar una lista con YouTube Radio activo) sin un cookies.txt configurado ahora muestra un mensaje claro en lugar de anunciar el mix y fallar en silencio.
- El panel de atajos mostraba una acción incorrecta para [Tab] e ignoraba YouTube; ahora indica "Cambiar fuente" de forma coherente en todas las pestañas.
- Se unificó la etiqueta del atajo "Abrir Configuración", que era inconsistente entre las vistas de Radio, Spotify y YouTube.
- Cuando el archivo de cookies de YouTube configurado dejaba de ser válido (eliminado, movido o ilegible), las pestañas Favoritos y Playlists quedaban vacías sin avisar; ahora muestran un error claro y la misma guía de recuperación que el estado no autenticado.
- En macOS, pegar texto (por ejemplo en el buscador o en la ruta del archivo de cookies) ahora funciona correctamente a través del portapapeles del sistema.
- El logo de Reverbic podía superponerse con la franja de juego; ahora el diseño reserva espacio para que ambos queden visibles.

### Seguridad
- Deshabilitar o eliminar el archivo de cookies de YouTube ahora detiene de inmediato la reproducción de videos restringidos respaldados por cookies dentro de la misma sesión; la caché en memoria de URLs resueltas ya no entrega un resultado obtenido con cookies una vez que las credenciales desaparecen.
- Se eliminó la versión vulnerable `rustls-webpki 0.102.8` del árbol de dependencias (alertas de Dependabot RUSTSEC-2026-0049, 0098, 0099 y 0104). Solo la arrastraba `hyper-proxy2` a través de la integración de Spotify; ahora se compila contra la cadena ya parcheada (rustls 0.23 / hyper-rustls 0.27) que el resto de la app utiliza, conservando el backend de cripto `ring`.

## [1.5.3] - 2026-06-13

### Agregado
- Se agregaron configuraciones individuales en Ajustes > Modo Ambiente para alternar el visualizador, pistas recientes, barra de progreso y logo dentro del Modo Ambiente.
- El Modo Ambiente ahora identifica la fuente de YouTube y muestra el capítulo actual del video en reproducción debajo del título.
- La lista de pistas recientes del Modo Ambiente ahora funciona en todas las fuentes (radio, YouTube y Spotify), conservando las últimas 5 pistas reproducidas durante la sesión.
- Los detalles de la estación en el Modo Ambiente ahora son un widget dedicado y resaltado con color, que muestra país, región, idioma, códec y bitrate, etiquetas, popularidad (votos y reproducciones) y un sitio web clicable.
- Se agregó un ajuste en Ajustes > Modo Ambiente para mostrar u ocultar los detalles de la estación.
- Se agregó un ajuste en Ajustes > Modo Ambiente para mostrar u ocultar el bloque de reproducción actual (nombre de la fuente más artista, título y álbum).
- El título de la ventana y la interfaz ahora muestran dinámicamente el estado de la actualización en curso (ej. "Downloading vX.Y.Z..." y "Update vX.Y.Z Ready") al detectar una nueva versión.

### Cambiado
- Se refactorizó el renderizado del Modo Ambiente para usar widgets modulares para el reloj, visualizador, barra de progreso y logo.
- Se rediseñó la barra de atajos del Modo Ambiente: sin emojis, centrada y con las teclas resaltadas (ej. Space Pausa · +/- Volumen · Alt+S Detener · Tecla Salir).
- Los títulos, artistas y nombres de álbum largos en el Modo Ambiente ahora se ajustan a una segunda línea en lugar de cortarse con puntos suspensivos.
- El Modo Ambiente ya no se activa cuando todos sus widgets (reloj, logo, visualizador, barra de progreso, pistas recientes) están desactivados.

### Corregido
- El indicador de origen de reproducción (Radio / YouTube / Spotify) en el overlay de Windows era casi invisible en gris; ahora usa un color distinto por origen para que resalte.
- El logo de Reverbic en el Modo Ambiente podía quedar fuera de pantalla cuando el panel crecía; ahora se reserva espacio para que siempre quede visible sobre el panel.
- El script de instalación ahora sobrescribe y actualiza correctamente el binario persistido cuando se vuelve a ejecutar el instalador.

## [1.5.2] - 2026-06-12

### Agregado
- Se agregó un ajuste para configurar un archivo cookies.txt de YouTube, que permite acceder a videos con restricción de edad, de región o solo para miembros
- Se agregaron las sub-pestañas [Me gusta] y [Playlists] a la pestaña de YouTube, igual que en Spotify, para explorar y reproducir tus videos con Me gusta y tus playlists personales (requiere un cookies.txt configurado)
- Reverbic ahora descarga y verifica automáticamente el runtime Deno, utilizado por yt-dlp para resolver los desafíos de firma de YouTube con tiempos de resolución casi instantáneos (el binario queda en disco y no se carga en memoria al iniciar)
- Reproducción continua en YouTube: al terminar un video, se reproduce automáticamente el siguiente de la lista activa (resultados de búsqueda, Me gusta o playlist), precargando el siguiente video por anticipado
- Se agregó el ajuste "Crossfade (YouTube)" para fundir el final de cada video con el inicio del siguiente al reproducir listas de YouTube
- Se agregó la acción "Validar sesión de YouTube" en Ajustes para comprobar al instante si el cookies.txt configurado sigue vigente
- Las URLs de audio de YouTube ya resueltas se reutilizan durante 4 horas, haciendo casi instantáneo volver a reproducir un video reciente; el cache ahora sobrevive reinicios (las resoluciones hechas con cookies nunca se guardan en disco)
- YouTube Mix: con Ctrl+R sobre cualquier video se inicia una "radio infinita" de canciones similares que se extiende sola al acercarse al final de la cola
- yt-dlp ahora se actualiza automáticamente (chequeo diario contra GitHub con verificación SHA256), evitando que los cambios de YouTube rompan la integración con el tiempo
- El video resaltado se pre-resuelve en segundo plano, haciendo que reproducirlo con Enter sea casi instantáneo
- Las pistas de YouTube ahora se descargan a un archivo temporal a velocidad completa, habilitando adelantar/retroceder con precisión exacta y reproducción inmune a cortes de red
- Capítulos de YouTube: en videos largos el capítulo actual se muestra junto al título, y las teclas [ y ] saltan entre capítulos
- Nuevo ajuste opcional "SponsorBlock (YouTube)" que salta automáticamente las secciones sin música usando la base de datos comunitaria (desactivado por defecto)
- Nuevo ajuste "Radio (YouTube)", activado por defecto: cuando la lista en reproducción se termina, continúa automáticamente con un mix de canciones similares
- El overlay de juego ahora muestra una cuenta regresiva con el tiempo restante de la pista actual (Spotify y YouTube), tanto en estilo Completo como Compacto; la Radio no tiene duración, así que ahí no cambia nada
- En modo Remoto de Spotify, cuando no se detecta ningún dispositivo Connect la pestaña de Spotify se bloquea con un aviso claro que explica abrir Spotify en un dispositivo (teléfono, computador o reproductor web); reescanea automáticamente cada pocos segundos y se desbloquea sola en cuanto aparece un dispositivo (Ctrl+D fuerza un escaneo inmediato)
- Playlists de radio: nueva sub-pestaña [ Playlists ] en la pestaña de Radio para agrupar tus estaciones en colecciones con nombre; con Alt+P sobre cualquier estación (en Buscar, Género, País o Favoritas) se agrega a una playlist existente o se crea una nueva, y las playlists se guardan en disco entre sesiones
- Dentro de la sub-pestaña [ Playlists ]: N crea una playlist vacía con nombre, R renombra la seleccionada, Shift+↑/↓ reordena las estaciones dentro de una playlist y Alt+F quita la estación o elimina la playlist según el nivel
- Con Ctrl+Shift+→/← se salta a la siguiente o anterior estación de la playlist activa sin abrir ninguna lista, ideal para cambiar de ambiente sin soltar lo que estás haciendo
- Favoritos locales de YouTube: nueva sub-pestaña [ Favoritos ] en la pestaña de YouTube; con Alt+F sobre cualquier video (resultados, Me gusta o una playlist) se guarda localmente para escucharlo después — sin cuenta de Google, cookies ni autenticación
- Punto de reproducción animado en las pestañas principales: un punto verde intenso pulsa junto a la pestaña cuya fuente está sonando en este momento (se mantiene ahí aunque navegues otras pestañas, y deja de pulsar en pausa); la pestaña activa además muestra un punto ámbar en [Spotify] en modo Remoto sin dispositivo o uno rojo en [YouTube] con la sesión de cookies expirada
- La sesión de YouTube ahora se valida automáticamente en segundo plano al iniciar y al cambiar el archivo de cookies; el resultado se muestra junto al ajuste "Validar sesión de YouTube" ("Sesión válida" / "Cookies expiradas") y alimenta el punto de estado de la pestaña [YouTube]
- Selector de dispositivos de Spotify: Ctrl+D ahora abre una lista con todos los dispositivos Connect (nombre, tipo, activo/disponible) y Enter transfiere la reproducción al elegido, en vez de saltar a ciegas al siguiente
- Las transmisiones en vivo ahora muestran una etiqueta roja EN VIVO en los resultados de búsqueda de YouTube, y al intentar reproducir una se explica de inmediato que aún no se soportan, antes de iniciar cualquier resolución
- Los paneles de aviso ahora incluyen el atajo [O] que salta directo al ajuste relevante (archivo de cookies de YouTube, Client ID de Spotify o el modo de reproducción de Spotify), con el ítem preseleccionado
- Nueva acción "Abrir carpeta de logs" en Ajustes para llegar a los registros de Reverbic sin usar la terminal; cada sesión ahora registra la versión de la app y el modo de reproducción de Spotify al inicio

### Cambiado
- La pestaña de YouTube ahora usa el rojo de YouTube de forma consistente en todos sus elementos (video seleccionado, campo de búsqueda, cursor de escritura, barra de scroll), replicando el patrón verde de la pestaña de Spotify para que siempre quede claro en qué pestaña estás
- Las sub-pestañas [Me gusta] y [Playlists] de YouTube ahora muestran un panel de aviso claro cuando no hay cookies.txt configurado: explica que se necesita autenticación, recomienda usar una cuenta secundaria y enlaza a la guía paso a paso con los riesgos; las etiquetas de las sub-pestañas también se ven deshabilitadas (el mensaje anterior se desbordaba del panel y pasaba desapercibido)
- La pestaña de Spotify ahora muestra el mismo estilo de panel de aviso cuando la cuenta no está conectada: explica que iniciar sesión es obligatorio, que se necesita una cuenta Premium y una app en el Spotify Developer Dashboard, y enlaza a la guía paso a paso (clickeable, incluye las notas legales); Enter sigue iniciando el flujo de conexión
- Los avisos inferiores ahora se colorean por severidad (errores en rojo, advertencias en ámbar, información en el color de la fuente) y se encolan en vez de pisarse, de modo que un error ya no puede quedar oculto por un mensaje rutinario
- Todos los paneles de aviso (conexión de Spotify, sin dispositivo de Spotify, autenticación de YouTube) ahora comparten un mismo componente consistente; el panel de "sin dispositivo" ganó el link clickeable a la guía que los otros ya tenían
- Conectar una cuenta de Spotify sin Premium ahora muestra una advertencia clara de que la reproducción no funcionará, en vez de fallar después con errores confusos
- El panel de autenticación de YouTube ahora menciona la alternativa local [ Favoritos ] para guardar videos sin cuenta

### Corregido
- Los videos de transmisiones en vivo recién finalizadas ya no se quedan en un ciclo infinito de reintentos; Reverbic ahora explica que YouTube todavía está procesando la grabación y que se intente más tarde
- Intentar reproducir un stream de YouTube que está en vivo en este momento ya no muestra el error genérico de "formato no compatible"; Reverbic ahora explica que es una transmisión en curso y que podrá reproducirse cuando termine
- El pie de la pestaña de Spotify ya no afirma "Modo: Remoto Escuchando en Desconocido [activo]" al usar el modo Auto sin dispositivos; ahora muestra el modo real (Auto o Remoto) y "ningún dispositivo de Spotify detectado" cuando no hay ninguno
- El pie de la pestaña de Spotify ahora distingue entre un dispositivo realmente reproduciendo ([activo]) y uno que Spotify solo lista como disponible ([disponible])
- Cuando un dispositivo de Spotify no responde al reproducir (ej. un teléfono cuya app se cerró pero Spotify aún lo lista), Reverbic ahora lo descarta, explica lo que pasó y reescanea en vez de mantenerlo como destino
- La ayuda [?] ahora tiene una sección propia para YouTube (antes mostraba atajos genéricos) y cada atajo listado fue auditado contra el comportamiento real
- La tecla Espacio ahora pausa/reanuda en todas las listas sin campo de texto (Favoritas y Playlists de radio, resultados de Género/País, y las sub-pestañas de biblioteca de Spotify y YouTube); antes solo funcionaba en Favoritas de radio aunque la ayuda decía lo contrario
- Alt+F y Alt+R ya no actúan sobre resultados de radio residuales mientras navegas las pestañas de Spotify o YouTube

### Seguridad
- Se actualizaron dependencias (OpenSSL, ratatui, crossterm y otras) para corregir vulnerabilidades conocidas reportadas por Dependabot
- El instalador de Windows ahora verifica el hash SHA256 del binario descargado antes de ejecutarlo, y solo retira la marca de "descargado de internet" tras una verificación exitosa
- El instalador de Windows ahora aborta si el asset del release no incluye un hash SHA256 contra el cual verificar, en vez de ejecutar un binario sin verificar; esto se puede omitir bajo el propio riesgo del usuario mediante la variable de entorno `REVERBIC_SKIP_VERIFY`

### Cambiado
- El ajuste de Crossfade ahora ofrece pasos de 1, 3, 5 y 7 segundos (antes 1, 2 y 3)
- El instalador de Windows ya no sobrescribe el PATH de la sesión actual; solo agrega la carpeta de instalación de Reverbic si falta
- El instalador de Windows ahora muestra un mensaje más claro antes de abrir Reverbic, ya que la terminal queda ocupada hasta cerrar la aplicación con `q`

### Corregido
- El instalador de Windows ahora maneja fallos de red y límites de la API de GitHub con mensajes claros en vez de errores crudos, elimina el archivo temporal al finalizar, y soporta ARM64 (vía emulación x86_64) y versiones pre-release (mediante la variable de entorno `REVERBIC_PRERELEASE`)
- Se corrigió el error "Requested format is not available" al buscar, resolver o explorar videos y playlists de YouTube, causado porque yt-dlp ahora requiere un runtime de JavaScript para resolver los desafíos de firma de YouTube
- Las reproducciones on-demand (YouTube y replays) ahora se reconectan y reanudan automáticamente desde el byte exacto si la conexión se corta a mitad de una canción
- Se corrigió que las canciones de YouTube se cortaran a la mitad o quedaran en silencio: YouTube solo entregaba un formato combinado de video cuyo audio HE-AAC el decodificador no soporta; ahora se usa el cliente android_vr de yt-dlp, que entrega audio puro AAC-LC de mayor calidad

---

## [1.5.1] — 2026-06-10

### Corregido
- Se corrigió un desajuste de versión donde la aplicación reportaba v1.4.2 en lugar de v1.5.0, lo que provocaba que el actualizador automático sugiriera repetidamente actualizar a la versión ya instalada

---

## [1.5.0] — 2026-06-09

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

[Sin publicar]: https://github.com/sewandev/Reverbic/compare/v1.5.3...HEAD
[1.5.3]: https://github.com/sewandev/Reverbic/compare/v1.5.2...v1.5.3
[1.5.2]: https://github.com/sewandev/Reverbic/compare/v1.5.1...v1.5.2
[1.5.1]: https://github.com/sewandev/Reverbic/compare/v1.5.0...v1.5.1
[1.5.0]: https://github.com/sewandev/Reverbic/compare/v1.4.2...v1.5.0
[1.4.2]: https://github.com/sewandev/Reverbic/compare/v1.4.1...v1.4.2
[1.4.1]: https://github.com/sewandev/Reverbic/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/sewandev/Reverbic/compare/v1.3.1...v1.4.0
[1.3.1]: https://github.com/sewandev/Reverbic/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/sewandev/Reverbic/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/sewandev/Reverbic/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/sewandev/Reverbic/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/sewandev/Reverbic/releases/tag/v1.0.0
