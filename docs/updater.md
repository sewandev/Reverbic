# Updater Behavior

Reverbic ships with a self-updater implemented in `src/update.rs`. This document describes how it discovers, validates, and applies updates.

## Discovering the latest release

The updater queries the GitHub Releases API for the repository `sewandev/Reverbic`:

```
GET https://api.github.com/repos/sewandev/Reverbic/releases/latest
```

The request uses a `reverbic/<version>` user agent, a 10-second connect timeout, and a 15-second overall timeout. Any non-success HTTP status, network error, or malformed response simply yields "no update available" rather than an error to the user.

## Version comparison

The release tag (for example `v1.6.0`) has its leading `v` stripped and is compared against the running version (`CARGO_PKG_VERSION`). Both are parsed as `major.minor.patch` semantic versions and compared numerically; an update is offered only when the latest version is strictly greater. If either string cannot be parsed as a three-part semver, the updater falls back to a plain string inequality check.

## Supported update target

The only update target currently supported is **Windows x86_64**. The target is derived from `std::env::consts::OS` and `std::env::consts::ARCH`:

- A non-Windows OS resolves to `UnsupportedPlatform` and no update is offered.
- A non-`x86_64` architecture resolves to `NoCompatibleAsset`.

## Asset selection

For a resolved target, the updater builds the expected asset file name and selects the GitHub release asset whose `name` matches it exactly:

```
reverbic-v<version>-x86_64-windows.exe
```

If no asset with that exact name exists in the release, the update is skipped (`NoCompatibleAsset`).

## Digest format

Each candidate asset carries the digest GitHub stores for it. The updater requires this digest in the form:

```
sha256:<64 hex chars>
```

It strips the `sha256:` prefix and verifies that the remaining string is exactly 64 ASCII hexadecimal characters. A missing digest (`MissingDigest`), a non-`sha256:` prefix, or a wrong-length / non-hex value (`UnsupportedDigest`) causes the download to be rejected.

## Download, validation, and apply flow

1. **Temporary path**: the asset is downloaded into the system temp directory as `reverbic-update-<asset-name>`.
2. **`.part` staging**: bytes are streamed into a unique sibling file ending in `.part`. The unique suffix combines the process id, a nanosecond timestamp, and an atomic counter, so concurrent or repeated downloads never collide and a stale temp file is never reused.
3. **Validation**: once fully downloaded, the `.part` file is validated against the asset metadata:
   - Its on-disk size must equal the asset's reported size (`SizeMismatch` otherwise).
   - Its computed SHA-256 must match the expected digest, compared case-insensitively (`HashMismatch` otherwise).
   A failure deletes the `.part` file and aborts the update.
4. **Commit**: on success, any previous file at the final path is removed and the `.part` file is atomically renamed into place. If the rename fails, the `.part` file is cleaned up.
5. **Apply (Windows)**: applying the update spawns a detached batch helper (via `cmd /C`, created with `CREATE_NO_WINDOW`) written to the temp directory. The helper retries moving the running executable to `<name>.old`, copies the new binary into place, then deletes the backup, the downloaded payload, and finally itself. If copying fails, it restores the original from the backup.
6. **Stale cleanup**: on startup the updater removes any leftover `<exe>.old` file next to the current executable.

The download client uses a 10-second connect timeout and a 120-second overall timeout to accommodate the binary size.

---
[Back to documentation index](README.md)
