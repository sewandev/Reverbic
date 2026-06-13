use chrono::{Local, Timelike};
use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

fn big_digit_rows(d: u8) -> [&'static str; 5] {
    match d {
        0 => ["███", "█ █", "█ █", "█ █", "███"],
        1 => [" █ ", "██ ", " █ ", " █ ", "███"],
        2 => ["███", "  █", "███", "█  ", "███"],
        3 => ["███", "  █", " ██", "  █", "███"],
        4 => ["█ █", "█ █", "███", "  █", "  █"],
        5 => ["███", "█  ", "███", "  █", "███"],
        6 => ["███", "█  ", "███", "█ █", "███"],
        7 => ["███", "  █", "  █", "  █", "  █"],
        8 => ["███", "█ █", "███", "█ █", "███"],
        9 => ["███", "█ █", "███", "  █", "███"],
        _ => ["   ", "   ", "   ", "   ", "   "],
    }
}

pub struct ClockWidget {
    color: ratatui::style::Color,
    bg: ratatui::style::Color,
}

impl ClockWidget {
    pub const HEIGHT: u16 = 5;
    pub const WIDTH: u16 = 19;

    pub fn new(color: ratatui::style::Color, bg: ratatui::style::Color) -> Self {
        Self { color, bg }
    }
}

impl Widget for ClockWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < Self::WIDTH || area.height < Self::HEIGHT {
            return;
        }

        let now_t = Local::now();
        let h1 = (now_t.hour() / 10) as u8;
        let h2 = (now_t.hour() % 10) as u8;
        let m1 = (now_t.minute() / 10) as u8;
        let m2 = (now_t.minute() % 10) as u8;
        let colon_on = now_t.second().rem_euclid(2) == 0;

        let style = Style::default().fg(self.color).bg(self.bg);

        let x_offset = area.x + (area.width.saturating_sub(Self::WIDTH)) / 2;

        for r in 0..5usize {
            let y = area.y + r as u16;
            if y >= area.bottom() {
                break;
            }

            let colon_ch = if colon_on {
                match r {
                    1 | 3 => "█",
                    _ => " ",
                }
            } else {
                " "
            };

            let row_str = format!(
                "{} {}  {}  {} {}",
                big_digit_rows(h1)[r],
                big_digit_rows(h2)[r],
                colon_ch,
                big_digit_rows(m1)[r],
                big_digit_rows(m2)[r],
            );

            buf.set_string(x_offset, y, row_str, style);
        }
    }
}
