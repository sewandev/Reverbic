use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Deserializer, Serialize};

mod palettes;
mod reverbic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    #[default]
    Reverbic,
    Ocean,
    Forest,
    Rose,
    Amber,
    Lavender,
    Nord,
    Sunset,
    Catppuccin,
    Solarized,
    TokyoNight,
    Gruvbox,
    Ayu,
    NightOwl,
    Vesper,
    RosePine,
    Kanagawa,
    Everforest,
    Synthwave84,
    Clay,
    TerminalGreen,
}

impl<'de> Deserialize<'de> for ThemeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "reverbic" => Self::Reverbic,
            "ocean" => Self::Ocean,
            "forest" => Self::Forest,
            "rose" => Self::Rose,
            "amber" => Self::Amber,
            "lavender" => Self::Lavender,
            "nord" => Self::Nord,
            "sunset" => Self::Sunset,
            "catppuccin" => Self::Catppuccin,
            "solarized" => Self::Solarized,
            "tokyo_night" => Self::TokyoNight,
            "gruvbox" => Self::Gruvbox,
            "ayu" => Self::Ayu,
            "night_owl" => Self::NightOwl,
            "vesper" => Self::Vesper,
            "rose_pine" => Self::RosePine,
            "kanagawa" => Self::Kanagawa,
            "everforest" => Self::Everforest,
            "synthwave84" => Self::Synthwave84,
            "clay" => Self::Clay,
            "terminal_green" => Self::TerminalGreen,
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

    pub fn prev(self) -> Self {
        let themes = definitions();
        let position = themes
            .iter()
            .position(|definition| definition.id == self)
            .unwrap_or(0);
        themes[(position + themes.len() - 1) % themes.len()].id
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

const THEME_DEFINITIONS: &[ThemeDefinition] = &[
    ThemeDefinition {
        id: ThemeId::Reverbic,
        label_key: "theme.reverbic",
        palette: &reverbic::PALETTE,
        preview: [
            Color::Rgb(0, 240, 255),
            Color::Rgb(112, 0, 255),
            Color::Rgb(255, 0, 85),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Ocean,
        label_key: "theme.ocean",
        palette: &palettes::OCEAN,
        preview: [
            Color::Rgb(56, 189, 248),
            Color::Rgb(59, 130, 246),
            Color::Rgb(129, 140, 248),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Forest,
        label_key: "theme.forest",
        palette: &palettes::FOREST,
        preview: [
            Color::Rgb(52, 211, 153),
            Color::Rgb(45, 212, 191),
            Color::Rgb(163, 230, 53),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Rose,
        label_key: "theme.rose",
        palette: &palettes::ROSE,
        preview: [
            Color::Rgb(251, 113, 133),
            Color::Rgb(244, 114, 182),
            Color::Rgb(225, 29, 72),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Amber,
        label_key: "theme.amber",
        palette: &palettes::AMBER,
        preview: [
            Color::Rgb(245, 158, 11),
            Color::Rgb(251, 191, 36),
            Color::Rgb(249, 115, 22),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Lavender,
        label_key: "theme.lavender",
        palette: &palettes::LAVENDER,
        preview: [
            Color::Rgb(167, 139, 250),
            Color::Rgb(192, 132, 252),
            Color::Rgb(129, 140, 248),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Nord,
        label_key: "theme.nord",
        palette: &palettes::NORD,
        preview: [
            Color::Rgb(136, 192, 208),
            Color::Rgb(129, 161, 193),
            Color::Rgb(180, 142, 173),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Sunset,
        label_key: "theme.sunset",
        palette: &palettes::SUNSET,
        preview: [
            Color::Rgb(251, 146, 60),
            Color::Rgb(244, 114, 182),
            Color::Rgb(248, 113, 113),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Catppuccin,
        label_key: "theme.catppuccin",
        palette: &palettes::CATPPUCCIN,
        preview: [
            Color::Rgb(203, 166, 247),
            Color::Rgb(137, 180, 250),
            Color::Rgb(245, 194, 231),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Solarized,
        label_key: "theme.solarized",
        palette: &palettes::SOLARIZED,
        preview: [
            Color::Rgb(42, 161, 152),
            Color::Rgb(38, 139, 210),
            Color::Rgb(181, 137, 0),
        ],
    },
    ThemeDefinition {
        id: ThemeId::TokyoNight,
        label_key: "theme.tokyo_night",
        palette: &palettes::TOKYO_NIGHT,
        preview: [
            Color::Rgb(122, 162, 247),
            Color::Rgb(187, 154, 247),
            Color::Rgb(125, 207, 255),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Gruvbox,
        label_key: "theme.gruvbox",
        palette: &palettes::GRUVBOX,
        preview: [
            Color::Rgb(142, 192, 124),
            Color::Rgb(250, 189, 47),
            Color::Rgb(211, 134, 155),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Ayu,
        label_key: "theme.ayu",
        palette: &palettes::AYU,
        preview: [
            Color::Rgb(230, 180, 80),
            Color::Rgb(95, 180, 180),
            Color::Rgb(255, 120, 120),
        ],
    },
    ThemeDefinition {
        id: ThemeId::NightOwl,
        label_key: "theme.night_owl",
        palette: &palettes::NIGHT_OWL,
        preview: [
            Color::Rgb(130, 170, 255),
            Color::Rgb(127, 219, 202),
            Color::Rgb(199, 146, 234),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Vesper,
        label_key: "theme.vesper",
        palette: &palettes::VESPER,
        preview: [
            Color::Rgb(255, 199, 153),
            Color::Rgb(153, 204, 204),
            Color::Rgb(255, 128, 128),
        ],
    },
    ThemeDefinition {
        id: ThemeId::RosePine,
        label_key: "theme.rose_pine",
        palette: &palettes::ROSE_PINE,
        preview: [
            Color::Rgb(235, 188, 186),
            Color::Rgb(196, 167, 231),
            Color::Rgb(156, 207, 216),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Kanagawa,
        label_key: "theme.kanagawa",
        palette: &palettes::KANAGAWA,
        preview: [
            Color::Rgb(126, 156, 216),
            Color::Rgb(152, 187, 108),
            Color::Rgb(255, 160, 102),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Everforest,
        label_key: "theme.everforest",
        palette: &palettes::EVERFOREST,
        preview: [
            Color::Rgb(167, 192, 128),
            Color::Rgb(131, 192, 146),
            Color::Rgb(127, 187, 179),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Synthwave84,
        label_key: "theme.synthwave84",
        palette: &palettes::SYNTHWAVE84,
        preview: [
            Color::Rgb(255, 126, 219),
            Color::Rgb(54, 209, 220),
            Color::Rgb(255, 231, 117),
        ],
    },
    ThemeDefinition {
        id: ThemeId::Clay,
        label_key: "theme.clay",
        palette: &palettes::CLAY,
        preview: [
            Color::Rgb(217, 119, 87),
            Color::Rgb(225, 164, 120),
            Color::Rgb(138, 154, 91),
        ],
    },
    ThemeDefinition {
        id: ThemeId::TerminalGreen,
        label_key: "theme.terminal_green",
        palette: &palettes::TERMINAL_GREEN,
        preview: [
            Color::Rgb(16, 163, 127),
            Color::Rgb(52, 211, 153),
            Color::Rgb(22, 199, 132),
        ],
    },
];

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
    use std::collections::HashSet;

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

        assert_eq!(ThemeId::all().next(), Some(ThemeId::default()));
        assert_eq!(
            definitions().first().map(|definition| definition.id),
            Some(ThemeId::Reverbic)
        );
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

    #[test]
    fn theme_registry_has_no_duplicate_theme_ids() {
        let mut seen = HashSet::new();

        for theme in ThemeId::all() {
            assert!(seen.insert(theme), "duplicate theme id: {theme:?}");
        }

        assert_eq!(seen.len(), definitions().len());
    }

    #[test]
    fn every_theme_id_has_a_definition_and_palette() {
        for theme in ThemeId::all() {
            let theme_definition = definition(theme);

            assert_eq!(theme_definition.id, theme);
            assert!(std::ptr::eq(theme_definition.palette, palette(theme)));
            assert!(theme_definition.label_key.starts_with("theme."));
        }
    }

    #[test]
    fn every_theme_definition_has_complete_preview_and_motion_sets() {
        for theme_definition in definitions() {
            assert_eq!(theme_definition.preview.len(), 3);
            assert_eq!(theme_definition.palette.border_cycle.len(), 3);
            assert_eq!(theme_definition.palette.spectrum.len(), 8);
            assert_eq!(theme_definition.palette.logo_letters.len(), 8);
        }
    }
}
