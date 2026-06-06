use ratatui::style::Color;

use super::Palette;

const BORDER_COLORS: [(u8, u8, u8); 3] = [(0, 240, 255), (112, 0, 255), (255, 0, 85)];
const SPECTRUM: [Color; 8] = [
    Color::Rgb(0, 240, 255),
    Color::Rgb(40, 160, 255),
    Color::Rgb(75, 80, 255),
    Color::Rgb(112, 0, 255),
    Color::Rgb(160, 0, 200),
    Color::Rgb(200, 0, 140),
    Color::Rgb(235, 0, 100),
    Color::Rgb(255, 0, 85),
];

pub const PALETTE: Palette = Palette {
    accent: Color::Rgb(0, 240, 255),
    radio_accent: Color::Rgb(64, 160, 255),
    playing: Color::Rgb(0, 240, 255),
    muted: Color::DarkGray,
    dim: Color::Gray,
    highlight: Color::White,
    danger: Color::Red,
    warning: Color::Yellow,
    buffering: Color::Rgb(80, 80, 80),
    spotify: Color::Rgb(30, 215, 96),
    caution: Color::Rgb(180, 130, 30),
    panel_bg: Color::Rgb(13, 13, 13),
    overlay_color: Color::Rgb(5, 5, 5),
    border_cycle: &BORDER_COLORS,
    spectrum: &SPECTRUM,
    logo_letters: &SPECTRUM,
};
