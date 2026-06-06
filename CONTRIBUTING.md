# Contributing to Reverbic

Thanks for your interest in contributing. This is an open-source project maintained in the author's free time — contributions are welcome but the maintainer has final say on everything that gets merged. Reviews and merges might take a few days, so please bear with us.

## Current Priorities (What We're Focusing On)

Right now, our main focus is on **security hardening** (specially around the auto-updater and credential management) and finishing full **native support for macOS**. Because of this, massive new features might take a backseat or take a bit longer to get reviewed.

## Branching Strategy & CI

- **`develop` is our default branch.** All new features, bug fixes, and refactors MUST be PR'd against `develop`.
- Please **do not open Pull Requests against `main`**. `main` is strictly reserved for production releases.
- **CI is Mandatory**: We do not bypass GitHub Actions. Your PR will not be merged until the `cargo fmt` and `cargo clippy` checks are perfectly green.
- **Merge Conflicts**: If your PR has merge conflicts, please resolve them locally, verify compilation with `cargo check` and `cargo fmt`, and push the fix. If you can't, a maintainer might create a "PR Bridge" to resolve it for you while keeping your author credits.

## Issues

Open an issue for:
- **Bug reports** — include OS version, terminal, steps to reproduce, and what you expected vs. what happened
- **Feature requests** — describe the use case, not just the feature. Dealing with third-party APIs, cookies, and tokens requires careful security decisions. Let's talk about it first so you don't waste your time building something we might not be able to merge safely.

## Pull Requests

PRs are welcome. Before opening one:

1. **Open an issue first** to discuss the change — this avoids wasted effort on PRs that won't be merged
2. **Keep the scope small** — one fix or one feature per PR, no unrelated cleanup
3. **Write a clear description** — explain what the change does and why it's needed; the diff alone is not enough

### Code conventions

- Rust — follow the existing style; run `cargo fmt` and `cargo clippy` before committing, both must pass cleanly
- Commits — use [Conventional Commits](https://www.conventionalcommits.org): `feat:`, `fix:`, `docs:`, `chore:`, etc.
- UI strings — English and Spanish (`locales/en.json` and `locales/es.json`) must both be updated when adding any visible text
- No commented-out code, no dead code left behind
- No `unwrap()` — use `expect("reason")` or `?`

### What is unlikely to be merged

- Large refactors without prior discussion
- New dependencies without a strong justification
- Features that conflict with the project's focused scope
- Anything that breaks the existing UX without a clear improvement

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
