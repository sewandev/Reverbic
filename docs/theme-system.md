# Theme System

This guide explains how Reverbic themes are structured, how to add new dark
palettes, and how to use theme tokens when adding new UI surfaces.

## Core Rules

- `Reverbic` is the default theme and must stay visually unchanged.
- The current theme family is dark-only. Do not add a light/dark mode switch or
  a `ThemeMode` abstraction for these palettes.
- `src/ui/theme/reverbic.rs` is the canonical source for the default local
  theme. Do not edit it when adding new palettes.
- Every user-visible theme name must go through i18n with matching keys in
  `locales/es.json` and `locales/en.json`.
- The shared theme registry is the single source for picker order, display
  labels, palette lookup, and preview swatches.

## Main Files

- `src/ui/theme/mod.rs`: owns `ThemeId`, `Palette`, `ThemeDefinition`, the
  theme registry, lookup helpers, and tests.
- `src/ui/theme/reverbic.rs`: defines the default `Reverbic` palette.
- `src/ui/theme/palettes.rs`: defines additional dark palettes.
- `locales/es.json` and `locales/en.json`: define `theme.<id>` labels shown in
  the picker and settings UI.
- `src/ui/widgets/theme_picker.rs`: owns theme picker sizing and scroll math.

## Palette Tokens

Each `Palette` must define the same complete set of tokens:

- Base surface tokens: `panel_bg`, `overlay_color`, `highlight`, `dim`,
  `muted`.
- Identity tokens: `accent`, `radio_accent`, `playing`.
- State tokens: `danger`, `warning`, `buffering`, `status_ok`, `caution`.
- Source tokens: `spotify`, `youtube`.
- Motion tokens: `border_cycle`, `spectrum`, `logo_letters`.

Motion is part of the theme. A new palette is not complete until it provides:

- `border_cycle`: exactly 3 RGB tuples for the animated border.
- `spectrum`: exactly 8 colors for the visualizer.
- `logo_letters`: exactly 8 colors for the `REVERBIC` logo.
- `preview`: exactly 3 colors in the registry for the picker swatches.

## Adding A New Theme

1. Add constants in `src/ui/theme/palettes.rs`.

   Define a `*_BORDER`, a `*_SPECTRUM`, and a public `Palette` constant. Reuse
   the spectrum for `logo_letters` unless the theme needs a different logo
   rhythm.

2. Add a `ThemeId` variant in `src/ui/theme/mod.rs`.

   Keep `Reverbic` first and `#[default]`. New variants should use local,
   user-facing names that make sense inside Reverbic.

3. Update `ThemeId` deserialization.

   Add the `snake_case` config value to the manual `Deserialize` match. Unknown
   values must continue to fall back to `ThemeId::Reverbic`.

4. Register the theme in `THEME_DEFINITIONS`.

   Add a `ThemeDefinition` with:

   - `id`: the new `ThemeId`.
   - `label_key`: a `theme.<id>` key.
   - `palette`: a reference to the new palette constant.
   - `preview`: three swatch colors from the palette.

5. Add translations in both locale files.

   Add the same `theme.<id>` key to `locales/es.json` and `locales/en.json`.

6. Extend tests if needed.

   The registry tests in `src/ui/theme/mod.rs` should catch missing metadata,
   duplicate ids, and incomplete motion arrays. The palette test in
   `src/ui/theme/palettes.rs` should include the new palette.

## Using Themes In New UI Surfaces

New UI code should accept or derive a `&Palette` and use its tokens instead of
hardcoded colors. The goal is that changing `Config.theme` changes the entire
surface without special cases.

Use these defaults when choosing tokens:

- Panel or modal backgrounds: `palette.panel_bg`.
- Full-screen overlays or outer dark backgrounds: `palette.overlay_color`.
- Primary readable text: `palette.highlight`.
- Secondary text: `palette.dim`.
- Hints, inactive labels, and low-emphasis copy: `palette.muted`.
- Current selection, primary focus, and active affordances: `palette.accent` or
  `palette.playing`.
- Radio-specific accents: `palette.radio_accent`.
- Error states: `palette.danger`.
- Warnings and cautionary badges: `palette.warning` or `palette.caution`.
- Loading, buffering, disabled dividers, and quiet separators:
  `palette.buffering` or `palette.dim`.
- Success states: `palette.status_ok`.
- Spotify and YouTube brand markers: `palette.spotify` and `palette.youtube`.

Avoid `Color::Rgb(...)`, `Color::DarkGray`, `Color::Yellow`, and similar direct
color choices in UI widgets. Direct colors are acceptable inside theme
definitions and in content-derived rendering where the color is data rather
than UI chrome.

## Verification

Run these checks after adding or changing themes:

```sh
cargo fmt --check
cargo check
cargo test theme
jq empty locales/es.json locales/en.json
rg -n "Color::DarkGray|Color::Yellow|Color::Rgb\\(" src/ui src/onboarding
git diff -- src/ui/theme/reverbic.rs
```

The `rg` audit should show direct RGB colors only in theme definitions, theme
tests, theme helper functions, or justified content rendering. The
`reverbic.rs` diff should be empty unless the change explicitly targets the
default theme.

---
[Back to documentation index](README.md)
