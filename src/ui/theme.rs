use ratatui::style::{Color, Modifier, Style};

// Verde Reverbic — coherente con overlay Win32 (0x44CC33)
pub const ACCENT:          Color = Color::Rgb(68, 204, 51);
pub const PLAYING:         Color = Color::Rgb(68, 204, 51);
pub const MUTED:           Color = Color::DarkGray;
pub const DIM:             Color = Color::Gray;
pub const HIGHLIGHT:       Color = Color::White;
pub const DANGER:          Color = Color::Red;
pub const WARNING:         Color = Color::Yellow;
pub const FESTIVAL_ACCENT: Color = Color::Yellow;

pub const SELECTED_STYLE: Style = Style::new().fg(HIGHLIGHT).add_modifier(Modifier::BOLD);
pub const PLAYING_STYLE:  Style = Style::new().fg(PLAYING).add_modifier(Modifier::BOLD);
pub const BORDER_STYLE:   Style = Style::new().fg(MUTED);
