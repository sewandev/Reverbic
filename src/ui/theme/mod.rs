use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

mod reverbic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    #[default]
    Reverbic,
}

impl ThemeId {
    const ALL: &'static [Self] = &[Self::Reverbic];

    pub fn all() -> &'static [Self] {
        Self::ALL
    }

    pub fn display(self) -> String {
        use crate::i18n::t;
        match self {
            Self::Reverbic => t("theme.reverbic"),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Reverbic => Self::Reverbic,
        }
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

pub fn palette(theme_id: ThemeId) -> &'static Palette {
    match theme_id {
        ThemeId::Reverbic => &reverbic::PALETTE,
    }
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
}
