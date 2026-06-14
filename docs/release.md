# Release Process

This document describes how Reverbic releases are produced and what to verify before publishing one.

## Branch policy

- Day-to-day work (features, fixes, refactors) targets the `develop` branch. Community pull requests should be opened against `develop` and are merged with **squash and merge**.
- `main` is reserved for production releases. It only receives changes at release time, never direct daily work.
- Releases are promoted from `develop` to `main` with a **merge commit** — never squash or rebase. This is enforced by branch rulesets (`develop` allows squash only, `main` allows merge commits only) and keeps the two branches from diverging across releases.
- Continuous integration (`.github/workflows/ci.yml`) runs on pull requests to `main` and `develop`, and on pushes to `develop`. It checks `cargo fmt`, `cargo check`, `cargo test`, and `cargo clippy -D warnings` across Windows, macOS, and Linux.

## Release workflow

The release pipeline is defined in `.github/workflows/release.yml` and is triggered by pushing a Git tag that matches `v*`:

```yaml
on:
  push:
    tags:
      - "v*"
```

When triggered, the workflow builds in parallel across runners and then publishes:

1. **Windows** (`windows-latest`): `cargo build --release --target x86_64-pc-windows-msvc`, packaged as `reverbic-vX.Y.Z-x86_64-windows.exe`.
2. **macOS** (`macos-latest`): builds both `x86_64-apple-darwin` and `aarch64-apple-darwin`, packaged as `.tar.gz` archives.
3. **Linux** (`ubuntu-22.04`, pinned for broad glibc compatibility): `x86_64-unknown-linux-gnu`, packaged as a `.tar.gz`.
4. **Publish**: gathers every artifact, generates `checksums.txt` with the SHA-256 of each, and creates the GitHub Release with all artifacts attached.

A tag containing a hyphen (for example `v2.0.0-rc1`) is published as a prerelease automatically; all other tags are published as stable.

## Release artifact names

Each release publishes one artifact per platform plus a checksums file:

```
reverbic-vX.Y.Z-x86_64-windows.exe     # Windows x86_64
reverbic-vX.Y.Z-x86_64-macos.tar.gz    # macOS Intel
reverbic-vX.Y.Z-aarch64-macos.tar.gz   # macOS Apple Silicon
reverbic-vX.Y.Z-x86_64-linux.tar.gz    # Linux x86_64
checksums.txt                          # SHA-256 of every artifact
```

The version segment comes directly from the pushed tag (`github.ref_name`). These exact names are what the in-app updater looks for when selecting a compatible asset, so they must not change without also updating `src/update.rs`.

## SHA-256 digests

The publish job runs `sha256sum` over every artifact and uploads the result as `checksums.txt` alongside the binaries.

The in-app updater independently validates downloads against the digest GitHub stores for each asset. GitHub exposes that digest in the `sha256:<64 hex chars>` format, and the updater rejects any download whose digest is missing, malformed, or does not match the downloaded bytes. Publishing through the official workflow keeps these digests consistent with the updater's expectations.

## Package managers

Reverbic is distributed through three external package managers, each in its own repository. After every release their manifests must be updated to point at the new version and digests:

- **Scoop** (Windows) — `sewandev/scoop-reverbic`. Update `bucket/reverbic.json` with the new version, the `reverbic-vX.Y.Z-x86_64-windows.exe` URL, and its SHA-256.
- **Homebrew** (macOS) — `sewandev/homebrew-reverbic`. Update `Formula/reverbic.rb` with the new version and the URLs and SHA-256 digests for both the Intel (`x86_64-macos`) and Apple Silicon (`aarch64-macos`) archives.
- **Winget** (Windows) — `Sewandev.Reverbic`. Submitted with the official `wingetcreate` tool against `microsoft/winget-pkgs`; uses the SHA-256 of the Windows `.exe`.

The `checksums.txt` published with each release is the source for these digests.

## Changelog

Every significant change must be recorded before the release in both changelog files:

- `CHANGELOG.md` (English)
- `CHANGELOG.es.md` (Spanish)

Both follow the Keep a Changelog format. When cutting a release, move the `[Unreleased]` section to a dated `[x.y.z] - YYYY-MM-DD` heading and open a fresh empty `[Unreleased]` section. Valid categories are `Added`, `Changed`, `Fixed`, `Removed`, and `Security`.

---
[Back to documentation index](README.md)
