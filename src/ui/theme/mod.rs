use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Deserializer, Serialize};

#[allow(dead_code)]
mod palettes;
mod reverbic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    #[default]
    Reverbic,
}

impl<'de> Deserialize<'de> for ThemeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "reverbic" => Self::Reverbic,
            _ => Self::Reverbic,
        })
    }
}

impl ThemeId {
    pub fn all() -> impl ExactSizeIterator<Item = Self> + DoubleEndedIterator {
        definitions().iter().map(|definition| definition.id)
    }

    pub fn display(self) -> String {
        use crate::i18n::t;

        t(definition(self).label_key)
    }

    pub fn next(self) -> Self {
        let themes = definitions();
        let position = themes
            .iter()
            .position(|definition| definition.id == self)
            .unwrap_or(0);
        themes[(position + 1) % themes.len()].id
    }
}

#[derive(Debug)]
pub struct Palette {
    pub accent: Color,
    pub radio_accent: Color,
    pub playing: Color,
    pub muted: Color,
    pub dim: Color,
    pub highlight: Color,
    pub danger: Color,
    pub warning: Color,
    pub buffering: Color,
    pub spotify: Color,
    pub youtube: Color,
    pub status_ok: Color,
    pub caution: Color,
    pub panel_bg: Color,
    pub overlay_color: Color,
    pub border_cycle: [(u8, u8, u8); 3],
    pub spectrum: [Color; 8],
    pub logo_letters: [Color; 8],
}

#[derive(Debug)]
pub struct ThemeDefinition {
    pub id: ThemeId,
    pub label_key: &'static str,
    pub palette: &'static Palette,
    pub preview: [Color; 3],
}

const THEME_DEFINITIONS: &[ThemeDefinition] = &[ThemeDefinition {
    id: ThemeId::Reverbic,
    label_key: "theme.reverbic",
    palette: &reverbic::PALETTE,
    preview: [
        Color::Rgb(0, 240, 255),
        Color::Rgb(112, 0, 255),
        Color::Rgb(255, 0, 85),
    ],
}];

pub fn definitions() -> &'static [ThemeDefinition] {
    THEME_DEFINITIONS
}

pub fn definition(theme_id: ThemeId) -> &'static ThemeDefinition {
    definitions()
        .iter()
        .find(|definition| definition.id == theme_id)
        .unwrap_or(&THEME_DEFINITIONS[0])
}

pub fn palette(theme_id: ThemeId) -> &'static Palette {
    definition(theme_id).palette
}

pub fn border_color_for(palette: &Palette, tick: u32) -> Color {
    let phase = tick % 180;
    let seg = (phase / 60) as usize;
    let t = (phase % 60) as f32 / 60.0;
    let (r1, g1, b1) = palette.border_cycle[seg];
    let (r2, g2, b2) = palette.border_cycle[(seg + 1) % palette.border_cycle.len()];
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
    Color::Rgb(lerp(r1, r2), lerp(g1, g2), lerp(b1, b2))
}

pub fn status_pulse(base: Color, tick: u32) -> Color {
    let Color::Rgb(r, g, b) = base else {
        return base;
    };
    let phase = (tick % 60) as f32 / 60.0;
    let level = 0.45 + 0.55 * (0.5 + 0.5 * (phase * std::f32::consts::TAU).sin());
    let scale = |c: u8| (c as f32 * level).round().min(255.0) as u8;
    Color::Rgb(scale(r), scale(g), scale(b))
}

pub fn playing_style(palette: &Palette) -> Style {
    Style::new()
        .fg(palette.playing)
        .add_modifier(Modifier::BOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverbic_palette_keeps_core_surface_colors() {
        let palette = palette(ThemeId::Reverbic);

        assert_eq!(palette.accent, Color::Rgb(0, 240, 255));
        assert_eq!(palette.panel_bg, Color::Rgb(13, 13, 13));
        assert_eq!(palette.overlay_color, Color::Rgb(5, 5, 5));
    }

    #[test]
    fn reverbic_palette_keeps_animated_color_sets() {
        let palette = palette(ThemeId::Reverbic);

        assert_eq!(
            palette.border_cycle,
            [(0, 240, 255), (112, 0, 255), (255, 0, 85)]
        );
        assert_eq!(
            palette.spectrum,
            [
                Color::Rgb(0, 240, 255),
                Color::Rgb(40, 160, 255),
                Color::Rgb(75, 80, 255),
                Color::Rgb(112, 0, 255),
                Color::Rgb(160, 0, 200),
                Color::Rgb(200, 0, 140),
                Color::Rgb(235, 0, 100),
                Color::Rgb(255, 0, 85),
            ]
        );
        assert_eq!(palette.logo_letters, palette.spectrum);
    }

    #[test]
    fn theme_registry_keeps_reverbic_as_default_first_theme() {
        let theme_definition = definition(ThemeId::Reverbic);

        assert_eq!(ThemeId::all().collect::<Vec<_>>(), vec![ThemeId::Reverbic]);
        assert_eq!(ThemeId::all().next(), Some(ThemeId::default()));
        assert_eq!(theme_definition.id, ThemeId::Reverbic);
        assert_eq!(theme_definition.label_key, "theme.reverbic");
        assert!(std::ptr::eq(
            theme_definition.palette,
            palette(ThemeId::Reverbic)
        ));
        assert_eq!(
            theme_definition.preview,
            [
                Color::Rgb(0, 240, 255),
                Color::Rgb(112, 0, 255),
                Color::Rgb(255, 0, 85),
            ]
        );
    }
}
