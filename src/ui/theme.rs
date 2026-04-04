
use ratatui::style::{Color, Modifier, Style};

pub const ACCENT:          Color = Color::Cyan;
pub const MUTED:           Color = Color::DarkGray;
pub const HIGHLIGHT:       Color = Color::White;
pub const ERROR:           Color = Color::Red;
pub const PLAYING:         Color = Color::Green;
pub const FESTIVAL:        Color = Color::Magenta;
pub const FESTIVAL_ACCENT: Color = Color::Yellow;

pub const SELECTED_STYLE: Style = Style::new().fg(HIGHLIGHT).add_modifier(Modifier::BOLD);
pub const PLAYING_STYLE:  Style = Style::new().fg(PLAYING).add_modifier(Modifier::BOLD);
pub const BORDER_STYLE:   Style = Style::new().fg(MUTED);
