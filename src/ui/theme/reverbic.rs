use ratatui::style::{Color, Modifier, Style};

pub(super) const BORDER_COLORS: [(u8, u8, u8); 3] = [(0, 240, 255), (112, 0, 255), (255, 0, 85)];

pub const ACCENT: Color = Color::Rgb(0, 240, 255);
pub const RADIO_ACCENT: Color = Color::Rgb(64, 160, 255);
pub const PLAYING: Color = Color::Rgb(0, 240, 255);
pub const MUTED: Color = Color::DarkGray;
pub const DIM: Color = Color::Gray;
pub const HIGHLIGHT: Color = Color::White;
pub const DANGER: Color = Color::Red;
pub const WARNING: Color = Color::Yellow;
pub const PLAYING_STYLE: Style = Style::new().fg(PLAYING).add_modifier(Modifier::BOLD);

pub const SPOTIFY_GREEN: Color = Color::Rgb(30, 215, 96);

pub const CAUTION: Color = Color::Rgb(180, 130, 30);
pub const PANEL_BG: Color = Color::Rgb(13, 13, 13);
pub const OVERLAY_COLOR: Color = Color::Rgb(5, 5, 5);
