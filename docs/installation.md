# Installation

Reverbic is a terminal player for Windows, macOS, and Linux. Prebuilt binaries are published for Windows (x86_64), macOS (Intel and Apple Silicon), and Linux (x86_64). You can also build from source on any platform with a Rust toolchain.

## Windows quick install (recommended)

Run the installer script in PowerShell:

```powershell
irm https://raw.githubusercontent.com/sewandev/Reverbic/main/install.ps1 | iex
```

The script:

1. Queries the latest GitHub release of `sewandev/Reverbic`.
2. Downloads the Windows executable for your architecture. On AMD64 it picks the `x86_64-windows` asset; on ARM64 it uses the same x86_64 binary via emulation.
3. Verifies the asset's SHA256 digest. If the release provides no digest, installation aborts unless you explicitly set `REVERBIC_SKIP_VERIFY=1`.
4. Removes the "downloaded from the internet" mark (`Unblock-File`) to avoid the SmartScreen prompt, then launches Reverbic in the same terminal.

On first run, Reverbic copies itself to `%LOCALAPPDATA%\Programs\reverbic\reverbic.exe` and adds that folder to your user `PATH`, so you can type `reverbic` from any terminal afterward.

To include pre-release versions, set `REVERBIC_PRERELEASE` to any non-empty value before running the script.

## macOS quick install (Homebrew)

```sh
brew install sewandev/reverbic/reverbic
```

This installs the prebuilt binary for your Mac (Intel or Apple Silicon) from the [Homebrew tap](https://github.com/sewandev/homebrew-reverbic).

## Scoop

```powershell
scoop bucket add reverbic https://github.com/sewandev/scoop-reverbic
scoop install reverbic
```

## Direct binary download

Download the Windows executable from the [releases page](https://github.com/sewandev/Reverbic/releases/latest). The asset is named:

```
reverbic-vX.Y.Z-x86_64-windows.exe
```

Place it in a folder on your `PATH` and run it from a terminal. When you launch the binary, Reverbic self-installs to `%LOCALAPPDATA%\Programs\reverbic\reverbic.exe` and registers itself on the user `PATH`.

## Build from source

Any operating system with a Rust toolchain can build Reverbic.

Install the latest published version with Cargo:

```sh
cargo install --git https://github.com/sewandev/Reverbic.git --locked
```

Or clone and build a release binary:

```sh
git clone https://github.com/sewandev/Reverbic.git
cd Reverbic
cargo build --release
```

The resulting binary is `target/release/reverbic` (`target/release/reverbic.exe` on Windows). Source builds are validated in CI on Windows, macOS, and Linux.

## Release assets

Each release publishes prebuilt binaries for all three platforms, built by `.github/workflows/release.yml`:

- `reverbic-vX.Y.Z-x86_64-windows.exe` — Windows x86_64
- `reverbic-vX.Y.Z-x86_64-macos.tar.gz` — macOS Intel
- `reverbic-vX.Y.Z-aarch64-macos.tar.gz` — macOS Apple Silicon
- `reverbic-vX.Y.Z-x86_64-linux.tar.gz` — Linux x86_64
- `checksums.txt` — SHA-256 digests for every artifact

The in-app auto-updater applies to Windows, macOS, and Linux.

## Windows notes

- **SmartScreen:** the binaries are unsigned, so Windows SmartScreen may warn the first time you run a manually downloaded executable. Choose "More info" then "Run anyway". The installer script clears this mark automatically.
- **Terminal:** for the best visual experience, run Reverbic in [Windows Terminal](https://apps.microsoft.com/detail/9n0dx20hk701) with [PowerShell 7+](https://apps.microsoft.com/detail/9mz1snwt0n5d).

---
[Back to documentation index](README.md)
