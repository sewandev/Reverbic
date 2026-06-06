use ratatui::style::Color;

mod reverbic;

pub use reverbic::{
    ACCENT, CAUTION, DANGER, DIM, HIGHLIGHT, MUTED, OVERLAY_COLOR, PANEL_BG, PLAYING,
    PLAYING_STYLE, RADIO_ACCENT, SPOTIFY_GREEN, WARNING,
};

pub fn border_color(tick: u32) -> Color {
    let phase = tick % 180;
    let seg = (phase / 60) as usize;
    let t = (phase % 60) as f32 / 60.0;
    let (r1, g1, b1) = reverbic::BORDER_COLORS[seg];
    let (r2, g2, b2) = reverbic::BORDER_COLORS[(seg + 1) % 3];
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
    Color::Rgb(lerp(r1, r2), lerp(g1, g2), lerp(b1, b2))
}
