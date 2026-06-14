# Development Setup

This guide covers building, testing, and linting Reverbic locally, and the
expectations enforced by continuous integration.

## Prerequisites

- A stable Rust toolchain (`rustup` recommended), including the `clippy` and
  `rustfmt` components.
- Git, with [Git LFS](https://git-lfs.com/) installed for large binary assets
  (GIFs, images).

### Platform-specific dependencies

- **Linux**: the audio backend links against ALSA, so the build needs the
  development headers. On Debian/Ubuntu:

  ```sh
  sudo apt-get update
  sudo apt-get install -y pkg-config libasound2-dev
  ```

  These are the exact packages installed by CI for the Linux runner.
- **Windows** and **macOS**: no extra system packages are required to build.
  Several platform features (overlay, audio ducking, Discord Rich Presence,
  game detection) are currently Windows-only.

## Build, test, and lint

The project builds with Cargo:

```sh
cargo build              # debug build
cargo build --release    # optimized build
cargo run                # build and launch the TUI
```

CI runs the following commands, in this order, and all must pass before a pull
request can merge:

```sh
cargo fmt --check
cargo check --all-features
cargo test --all-features
cargo clippy --all-features -- -D warnings
```

Run these locally before opening a pull request. `cargo clippy` is run with
`-D warnings`, so any warning fails the build. Formatting must match `rustfmt`
exactly (`cargo fmt --check` does not modify files; run `cargo fmt` to apply).

CI executes the full matrix on `windows-latest`, `macos-latest`, and
`ubuntu-latest`, so changes must compile and pass on all three platforms.

## Internationalization rule

Any user-visible string is looked up through the i18n layer, which loads
`locales/en.json` and `locales/es.json`. When you add or change a visible
string you must update **both** files with the same key. A missing key in one
locale falls back to the other and ultimately to the raw key, which is not an
acceptable substitute for a real translation. See
[Internationalization (i18n)](i18n.md) for details.

## Logging

The application writes a log file to the cache directory
(`~/.cache/reverbic/logs/reverbic.log`, or `%LOCALAPPDATA%\Reverbic\cache\logs\reverbic.log`
on Windows) via the `tracing` ecosystem. Adding `tracing::info!` / `tracing::warn!` calls is the
preferred way to diagnose hard-to-reproduce bugs.

---
[Back to documentation index](README.md)
