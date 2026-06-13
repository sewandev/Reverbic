# Release Process

This document describes how Reverbic releases are produced and what to verify before publishing one.

## Branch policy

- Day-to-day work (features, fixes, refactors) targets the `develop` branch. Community pull requests should be opened against `develop`.
- `main` is reserved for production releases. It only receives changes at release time, never direct daily work.
- Continuous integration (`.github/workflows/ci.yml`) runs on pull requests to `main` and `develop`, and on pushes to `develop`. It checks `cargo fmt`, `cargo check`, `cargo test`, and `cargo clippy -D warnings` across Windows, macOS, and Linux.

## Release workflow

The release pipeline is defined in `.github/workflows/release.yml` and is triggered by pushing a Git tag that matches `v*`:

```yaml
on:
  push:
    tags:
      - "v*"
```

When triggered, the workflow runs on `windows-latest` and:

1. Builds the release binary with `cargo build --release --target x86_64-pc-windows-msvc`.
2. Renames the binary to the release artifact name.
3. Computes the SHA-256 digest of the artifact.
4. Creates a GitHub Release with the artifact attached.

A tag containing a hyphen (for example `v2.0.0-rc1`) is published as a prerelease automatically; all other tags are published as stable.

## Release artifact name

The Windows artifact is named:

```
reverbic-vX.Y.Z-x86_64-windows.exe
```

The version segment comes directly from the pushed tag (`github.ref_name`). This exact name is what the in-app updater looks for when selecting a compatible asset, so it must not change without also updating `src/update.rs`.

## SHA-256 digest

The workflow computes the lowercase SHA-256 hash of the artifact and embeds it in the release notes as `**SHA256:** <hash>`.

The in-app updater independently validates downloads against the digest GitHub stores for each asset. GitHub exposes that digest in the `sha256:<64 hex chars>` format, and the updater rejects any download whose digest is missing, malformed, or does not match the downloaded bytes. Publishing through the official workflow keeps these digests consistent with the updater's expectations.

## Scoop manifest

Reverbic is also distributed through a separate Scoop bucket repository (`sewandev/scoop-reverbic`). The release notes advertise:

```powershell
scoop bucket add reverbic https://github.com/sewandev/scoop-reverbic
scoop install reverbic
```

The Scoop manifest lives in that separate repository, so after each release its version, download URL, and hash must be updated there to point at the new `reverbic-vX.Y.Z-x86_64-windows.exe` artifact and its SHA-256.

## Changelog

Every significant change must be recorded before the release in both changelog files:

- `CHANGELOG.md` (English)
- `CHANGELOG.es.md` (Spanish)

Both follow the Keep a Changelog format. When cutting a release, move the `[Unreleased]` section to a dated `[x.y.z] - YYYY-MM-DD` heading and open a fresh empty `[Unreleased]` section. Valid categories are `Added`, `Changed`, `Fixed`, `Removed`, and `Security`.

---
[Back to documentation index](README.md)
