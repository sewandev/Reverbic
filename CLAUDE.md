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
- Commits en Conventional Commits, descripción en español neutro y máximo 1 línea de descripcion.

## Código
- Clean Code, DRY, principios SOLID
- Sin comentarios triviales — solo el WHY no obvio
- Sin `.unwrap()` — usar `.expect("razón")` o `?`
- Sin `#[allow(dead_code)]` — si no se usa, se elimina
- Sin emojis en código ni en UI
- `render()` pura: solo lee estado, nunca muta
- Sin argentinismos en strings visibles al usuario
- Nunca agregar comentarios al código
- Prohibido crear test para este proyecto

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

## Release y winget

### Version bump
- Solo editar `version` en `Cargo.toml` y correr `cargo update --workspace` — **nunca** `cargo generate-lockfile` al hacer un release, actualiza dependencias transitivas y puede romper builds

### Checklist antes de publicar
1. Bumping `version` en `Cargo.toml` (ej. `1.2.0`)
2. Mover `[Unreleased]` → `[x.y.z] — YYYY-MM-DD` en `CHANGELOG.md` y `CHANGELOG.es.md`; crear nuevo `[Unreleased]` vacío
3. Actualizar badge de versión en `README.md` y `README.es.md`
4. Commit: `feat: release vX.Y.Z ...`
5. `git push origin main && git tag vX.Y.Z && git push origin vX.Y.Z` → el action construye el binario y publica el release en GitHub

### PR de winget (después de que el release action termine)
- Obtener SHA256 del log del action: `gh run view <run_id> --log | grep -i sha256`
- El fork ya existe en `sewandev/winget-pkgs`; crear rama: `gh api -X POST repos/sewandev/winget-pkgs/git/refs --field ref="refs/heads/Sewandev.Reverbic-X.Y.Z" --field sha="<master_sha>"`
- Subir los 3 manifests vía `gh api -X PUT repos/sewandev/winget-pkgs/contents/manifests/s/Sewandev/Reverbic/X.Y.Z/Sewandev.Reverbic[.installer/.locale.en-US].yaml`
- PR: `gh pr create --repo microsoft/winget-pkgs --head "sewandev:Sewandev.Reverbic-X.Y.Z" --base master`
- **Nunca abrir más de un PR por versión para el mismo paquete** — cerrar los anteriores si no fueron mergeados antes de abrir uno nuevo

## Estilo de respuesta
- Respuestas cortas y directas
- Sin resúmenes al final si el diff ya lo dice
- Sin emojis
