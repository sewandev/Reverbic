# Reverbic — Reglas del proyecto

## GitHub CLI
- `gh` está disponible y autenticado como `sewandev` — úsalo para releases, PRs y uploads.

## Investigación y analisis
- Siempre dispara multi-agentes para investigar y analizar, incluso si la tarea es pequeña.
- Siempre busca optimizar la quema de tokens y contexto, los multi-agentes ayudan mucho.
- Cualquier tipo de refactorización hacerla con multi-agentes
- Usa siempre la skills de rust-best-practices

## Lenguaje
- Responder siempre en español neutro (sin argentinismos: no "escribí", "buscá", "fijate", etc.)
- Variables, funciones, structs y módulos siempre en inglés.
- Prohibido estrictamente realizar commits como Claude Code y/o Co-author.
- Commits en Conventional Commits, descripción en inglés neutro y máximo 1 línea de descripcion.

## Código
- Clean Code, DRY, principios SOLID
- Sin comentarios triviales — solo el WHY no obvio
- Sin `.unwrap()` — usar `.expect("razón")` o `?`
- Sin `#[allow(dead_code)]` — si no se usa, se elimina
- Sin emojis en código ni en UI
- `render()` pura: solo lee estado, nunca muta
- Sin argentinismos en strings visibles al usuario
- Nunca agregar comentarios al código
- No crear tests nuevos proactivamente, pero mantener y reparar los tests existentes si se ven afectados por un cambio.
- Priorizar el hardening de seguridad: validar siempre rutas, variables de entorno y no confiar en tamaños de archivos en caché. Evitar manipular cookies o credenciales de terceros sin un diseño de seguridad previo.

## Rust
- Usar el skill `rust-best-practices` siempre que se escriba o refactorice código Rust
- Sin `blocking_send` en hilos que pueden interferir con el runtime de tokio — usar `try_send` para datos no críticos
- Estado compartido entre hilos: preferir `Arc<Mutex<T>>` o globales `OnceLock` sobre canales de comandos cuando el dato es de solo lectura para la UI
- WASAPI y cualquier llamada Win32 bloqueante deben correr en hilos `std::thread` dedicados, nunca en el loop Win32 del overlay ni en tareas tokio

## Assets y archivos binarios
- Videos, imágenes pesadas y cualquier binario grande se almacenan con **Git LFS** (`git lfs track "*.mp4"`)
- Nunca subir binarios grandes directamente al repo ni usar URLs de CDN externas en el README
- Para agregar o reemplazar un video: reemplazar el archivo local y hacer `git add / commit / push` normal — LFS lo maneja

## Changelog
- Todo cambio significativo (feat, fix, refactor, chore visible al usuario) debe registrarse en `CHANGELOG.md`
- Seguir el formato [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
- Al crear un release: mover la sección `[Unreleased]` a `[x.y.z] — YYYY-MM-DD` y crear un nuevo `[Unreleased]` vacío
- Categorías válidas: `Added`, `Changed`, `Fixed`, `Removed`, `Security`
- Descripciones en español neutro, orientadas al usuario (no al desarrollador)

## Flujo de ramas

- La rama por defecto del repositorio debe ser `develop` para que los PRs comunitarios apunten ahí automáticamente desde su creación.
- El trabajo diario (features, fixes, refactors) va siempre a `develop`.
- `main` solo recibe cambios en el momento del release — nunca se trabaja directamente en `main`. Las actualizaciones a `main` deben pasar estrictamente por un Pull Request debido a la protección de rama.
- **Paciencia con el CI**: Nunca forzar un merge usando privilegios de administrador (`--admin`) si los checks de GitHub Actions están corriendo o fallando. Esperar siempre el check verde antes de mezclar para evitar romper `develop`.
- Al finalizar un release, `develop` se iguala a `main` para que ambas ramas queden al mismo punto.
- **PRs Comunitarios (PR Puentes)**: Para integrar PRs de terceros que presenten conflictos, usar siempre una rama puente (`ci/resolve-pr-X`) para resolver el choque localmente. **Obligatorio**: ejecutar `cargo check` y `cargo fmt` localmente para validar la resolución *antes* de hacer push y abrir el PR puente. Una vez aprobado por el CI, fusionar manteniendo el crédito del autor original.

## Release y winget

### Determinar la versión siguiente
Nunca asumir ni inventar la versión. Recuperarla desde GitHub antes de hacer cualquier bump:
```
gh release list --limit 1 --json tagName --jq '.[0].tagName'
```
Con eso se conoce la versión publicada actual (ej. `v1.3.1`). Incrementar según el tipo de cambio:
- **patch** (x.y.**Z**): fixes, reparaciones, y **pequeñas nuevas funcionalidades o mejoras visuales** (evitar inflar la versión Minor innecesariamente).
- **minor** (x.**Y**.0): **Reservado estrictamente para funcionalidades (feats) MUY potentes** o integraciones grandes (ej. integrar YouTube, un Asistente Inicial).
- **major** (**X**.0.0): cambios arquitectónicos que rompen compatibilidad radicalmente.

### Version bump
- Solo editar `version` en `Cargo.toml` y correr `cargo update --workspace` — **nunca** `cargo generate-lockfile` al hacer un release, actualiza dependencias transitivas y puede romper builds

### Antes de cualquier commit
- `cargo fmt` — obligatorio antes de todo commit, sin excepción. El CI lo verifica y falla si el formato no es exacto.
- `cargo clippy` — obligatorio antes de todo commit.

### Checklist antes de publicar
1. Recuperar versión actual desde GitHub: `gh release list --limit 1 --json tagName --jq '.[0].tagName'`
2. Determinar siguiente versión (patch / minor / major)
3. Asegurarse de estar en `develop` con todo commiteado: `git checkout develop`
4. Bump `version` en `Cargo.toml` + `cargo update --workspace`
5. Mover `[Unreleased]` → `[x.y.z] — YYYY-MM-DD` en `CHANGELOG.md` y `CHANGELOG.es.md`; crear nuevo `[Unreleased]` vacío
6. Actualizar badge de versión en `README.md` y `README.es.md`
7. `cargo fmt && cargo clippy` — ambos deben pasar limpios
8. Commit en `develop`: `feat: release vX.Y.Z ...`
9. Merge a `main` y publicar:
   - Crear un Pull Request desde `develop` hacia `main` (ej. `gh pr create --base main --head develop`).
   - Esperar que pasen los checks obligatorios de GitHub Actions.
   - Fusionar el PR y luego publicar el tag:
   ```
   git checkout main
   git pull origin main
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```
   → el action construye el binario y publica el release en GitHub
10. Igualar `develop` con `main` (por si el merge generó divergencia):
    ```
    git push origin main:develop
    ```

### PR de winget (después de que el release action termine)
- Obtener SHA256 del log del action: `gh run view <run_id> --log | grep -i sha256`
- El fork ya existe en `sewandev/winget-pkgs`; crear rama: `gh api -X POST repos/sewandev/winget-pkgs/git/refs --field ref="refs/heads/Sewandev.Reverbic-X.Y.Z" --field sha="<master_sha>"`
- Subir los 3 manifests vía `gh api -X PUT repos/sewandev/winget-pkgs/contents/manifests/s/Sewandev/Reverbic/X.Y.Z/Sewandev.Reverbic[.installer/.locale.en-US].yaml`
- PR: `gh pr create --repo microsoft/winget-pkgs --head "sewandev:Sewandev.Reverbic-X.Y.Z" --base master`
- **Nunca abrir más de un PR por versión para el mismo paquete** — cerrar los anteriores si no fueron mergeados antes de abrir uno nuevo

### Update de Scoop (después de que el release action termine)
- Clonar (o actualizar) el repositorio `sewandev/scoop-reverbic`.
- Obtener el SHA256 del ejecutable recién publicado en la página de Releases de Reverbic.
- Actualizar `version`, `url`, `bin` y `hash` en el archivo `bucket/reverbic.json` para que apunte a la nueva versión.
- Commit y push directo a la rama `main` de ese repositorio.

## Workaround: eliminar colaborador fantasma en GitHub

**Solo aplicar cuando el usuario lo indique explícitamente.**

Si GitHub muestra a `claude` como colaborador después de haber limpiado los commits con Co-Authored-By, el caché no es el problema — GitHub indexa por nombre de rama. La solución es recrear las ramas con un nombre nuevo que nunca haya existido en el repo:

1. Cambiar default branch a `develop` temporalmente: `gh api -X PATCH repos/sewandev/Reverbic --field default_branch=develop`
2. Renombrar `main` → `mainN` → `main` (donde N es el siguiente número no usado: main2, main3, etc.):
   ```
   git push origin origin/main:refs/heads/mainN
   git push origin --delete main
   git push origin origin/mainN:refs/heads/main
   git push origin --delete mainN
   ```
3. Restaurar default branch: `gh api -X PATCH repos/sewandev/Reverbic --field default_branch=main`
4. Repetir el mismo ciclo para `develop` (develop2, develop3, etc.)

El número N debe ser siempre uno mayor al último usado (main1 ya fue usado, se usó main2, el próximo sería main3).

## Estilo de respuesta
- Respuestas cortas y directas
- Sin resúmenes al final si el diff ya lo dice
- Sin emojis
