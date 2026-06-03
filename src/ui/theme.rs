use ratatui::style::{Color, Modifier, Style};
pub const ACCENT:          Color = Color::Rgb(68, 204, 51);
pub const RADIO_ACCENT:    Color = Color::Rgb(64, 160, 255);
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
pub const CURSOR_STYLE:   Style = Style::new().fg(Color::Black).bg(ACCENT).add_modifier(Modifier::BOLD);
pub const NORMAL_STYLE:   Style = Style::new().fg(MUTED);

pub const CAUTION:       Color = Color::Rgb(180, 130, 30);
pub const PANEL_BG:      Color = Color::Rgb(13, 13, 13);
pub const OVERLAY_COLOR: Color = Color::Rgb(5,  5,  5);
