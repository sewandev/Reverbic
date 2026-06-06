use ratatui::style::{Color, Modifier, Style};

use super::Palette;

const BORDER_COLORS: [(u8, u8, u8); 3] = [(0, 240, 255), (112, 0, 255), (255, 0, 85)];

pub const PALETTE: Palette = Palette {
    accent: Color::Rgb(0, 240, 255),
    radio_accent: Color::Rgb(64, 160, 255),
    playing: Color::Rgb(0, 240, 255),
    muted: Color::DarkGray,
    dim: Color::Gray,
    highlight: Color::White,
    danger: Color::Red,
    warning: Color::Yellow,
    spotify: Color::Rgb(30, 215, 96),
    caution: Color::Rgb(180, 130, 30),
    panel_bg: Color::Rgb(13, 13, 13),
    overlay_color: Color::Rgb(5, 5, 5),
    border_cycle: &BORDER_COLORS,
};

pub const ACCENT: Color = PALETTE.accent;
pub const RADIO_ACCENT: Color = PALETTE.radio_accent;
pub const PLAYING: Color = PALETTE.playing;
pub const MUTED: Color = PALETTE.muted;
pub const DIM: Color = PALETTE.dim;
pub const HIGHLIGHT: Color = PALETTE.highlight;
pub const DANGER: Color = PALETTE.danger;
pub const WARNING: Color = PALETTE.warning;
pub const PLAYING_STYLE: Style = Style::new().fg(PLAYING).add_modifier(Modifier::BOLD);

pub const SPOTIFY_GREEN: Color = PALETTE.spotify;

pub const CAUTION: Color = PALETTE.caution;
pub const PANEL_BG: Color = PALETTE.panel_bg;
pub const OVERLAY_COLOR: Color = PALETTE.overlay_color;
