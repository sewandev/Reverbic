# Contributing to Reverbic

Thanks for your interest in contributing. This is a personal project maintained by a single developer — contributions are welcome but the maintainer has final say on everything that gets merged.

## Issues

Open an issue for:
- **Bug reports** — include OS version, terminal, steps to reproduce, and what you expected vs. what happened
- **Feature requests** — describe the use case, not just the feature

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
