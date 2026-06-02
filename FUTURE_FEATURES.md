# Features Futuras

## 1. Infraestructura cloud: API, auth y usuarios

Servidor liviano (Rust/Axum) desplegable en AWS u otro proveedor.
Cuentas de usuario con autenticación, perfil básico y lista de amigos.
No relay de audio — solo coordinación de mensajes en tiempo real via WebSocket.

## 2. Party Mode: DJ + sincronización

Sesiones compartidas donde un usuario asume el rol de DJ/líder.
El DJ elige la estación o canción por todos y emite `{ station, start_unix_ms }`.
Cada cliente usa **Cristian's Algorithm** para calcular su offset de reloj con el servidor
y estima la posición en el stream para sincronizarse con el DJ.

- **Radio**: sincronización aproximada (±2-3s) — ICY no soporta seek real,
  se descarta audio hasta llegar al offset calculado por bitrate × tiempo.
- **Spotify remoto**: sincronización precisa — `PUT /me/player/play?position_ms=X`
  para todos los miembros simultáneamente.
- **Volumen**: siempre individual por usuario — el DJ no tiene control sobre el volumen
  de los demás, cada persona gestiona el suyo localmente.
- **Restricciones por país (Spotify)**: si un miembro de la party está en un país
  distinto al del DJ, puede unirse igual, pero debe recibir una advertencia visible
  de que algunas canciones podrían no estar disponibles en su región.
  El país de cada miembro viene del campo `country` del perfil de Spotify (`/v1/me`)
  y se compara con el del DJ al momento de unirse a la sesión.

## 3. Ecualizador del DJ (solo Party Mode)

Disponible únicamente cuando el usuario tiene el rol de DJ en una sesión party.
EQ de 3 bandas (bajos, medios, agudos) en el pipeline de audio local del DJ.
Visible solo para el DJ — el resto de la party no ve los controles.
Los parámetros (3 floats) se propagan por WebSocket y cada cliente aplica
los mismos filtros biquad localmente, dando el mismo resultado de audio sin revelar la UI.

## 4. Tablero de sonidos (Soundboard) — Party Mode

Cada miembro puede cargar hasta 5 efectos de sonido personales activables durante la party.
La recepción de efectos es opt-in por persona — cada usuario decide si los quiere escuchar.
Los efectos se transmiten P2P entre los miembros de la sesión (no pasan por el servidor).
Restricciones por efecto: formato limitado (ej. MP3/OGG), tamaño máximo bajo (ej. 200 KB),
máximo 5 efectos por usuario, para mantener la transmisión liviana y evitar abuso.
