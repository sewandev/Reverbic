use ratatui::style::Color;

mod reverbic;

pub use reverbic::{
    ACCENT, CAUTION, DANGER, DIM, HIGHLIGHT, MUTED, OVERLAY_COLOR, PANEL_BG, PLAYING,
    PLAYING_STYLE, RADIO_ACCENT, SPOTIFY_GREEN, WARNING,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeId {
    #[default]
    Reverbic,
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
    pub spotify: Color,
    pub caution: Color,
    pub panel_bg: Color,
    pub overlay_color: Color,
    pub border_cycle: &'static [(u8, u8, u8)],
}

pub fn palette(theme_id: ThemeId) -> &'static Palette {
    match theme_id {
        ThemeId::Reverbic => &reverbic::PALETTE,
    }
}

pub fn border_color(tick: u32) -> Color {
    border_color_for(palette(ThemeId::Reverbic), tick)
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
