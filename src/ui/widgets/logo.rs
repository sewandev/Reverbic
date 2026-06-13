use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::ui::theme::Palette;

pub struct LogoWidget<'a> {
    bg: ratatui::style::Color,
    tick: u32,
    palette: &'a Palette,
}

impl<'a> LogoWidget<'a> {
    pub const WIDTH: u16 = 39;
    pub const HEIGHT: u16 = 2;

    pub fn new(bg: ratatui::style::Color, tick: u32, palette: &'a Palette) -> Self {
        Self { bg, tick, palette }
    }

    pub fn render_centered(self, frame: &mut ratatui::Frame, area_x: u16, area_width: u16, y: u16) {
        let logo_x = area_x + area_width.saturating_sub(Self::WIDTH) / 2;
        frame.render_widget(self, Rect::new(logo_x, y, Self::WIDTH, Self::HEIGHT));
    }
}

fn eq_bar_level(tick: u32, period: u32, phase: u32, min_l: usize, max_l: usize) -> usize {
    let t = tick.wrapping_add(phase) % period;
    let half = period / 2;
    let range = max_l - min_l;
    let level = if t < half {
        (t as usize * range) / (half as usize).max(1) + min_l
    } else {
        ((period - t) as usize * range) / (half as usize).max(1) + min_l
    };
    level.clamp(min_l, max_l)
}

fn eq_bar_char(level: usize) -> &'static str {
    match level {
        1 => "▁▁",
        2 => "▂▂",
        3 => "▃▃",
        4 => "▄▄",
        5 => "▅▅",
        6 => "▆▆",
        7 => "▇▇",
        _ => "██",
    }
}

fn logo_lines(bg: ratatui::style::Color, tick: u32, palette: &Palette) -> [Line<'static>; 2] {
    const LETTERS: &[char] = &['R', 'E', 'V', 'E', 'R', 'B', 'I', 'C'];
    let cyan = palette.logo_letters[0];
    let violet = palette.logo_letters[3];
    let pink = palette.logo_letters[7];

    let b1 = eq_bar_level(tick, 36, 0, 1, 8);
    let b2 = eq_bar_level(tick, 42, 12, 2, 8);
    let b3 = eq_bar_level(tick, 30, 22, 1, 7);

    let top1 = if b1 == 8 { "▄▄" } else { "  " };
    let top2 = match b2 {
        8 => "▄▄",
        7 => "▁▁",
        _ => "  ",
    };
    let top3 = if b3 == 7 { "▁▁" } else { "  " };

    let s = Style::default().bg(bg);

    let top = Line::from(vec![
        Span::styled("    ", s),
        Span::styled(top1, s.fg(cyan)),
        Span::styled("  ", s),
        Span::styled(top2, s.fg(violet)),
        Span::styled("  ", s),
        Span::styled(top3, s.fg(pink)),
    ]);

    let mut spans: Vec<Span<'static>> = vec![
        Span::styled("▶", s.fg(cyan)),
        Span::styled("   ", s),
        Span::styled(eq_bar_char(b1), s.fg(cyan)),
        Span::styled("  ", s),
        Span::styled(eq_bar_char(b2), s.fg(violet)),
        Span::styled("  ", s),
        Span::styled(eq_bar_char(b3), s.fg(pink)),
        Span::styled("   ", s),
    ];
    for (i, &ch) in LETTERS.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", s));
        }
        spans.push(Span::styled(
            ch.to_string(),
            s.fg(palette.logo_letters[i]).add_modifier(Modifier::BOLD),
        ));
    }
    [top, Line::from(spans)]
}

impl<'a> Widget for LogoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < Self::WIDTH || area.height < Self::HEIGHT {
            return;
        }

        let [l1, l2] = logo_lines(self.bg, self.tick, self.palette);
        let st = Style::default().bg(self.bg);

        let p1 = Paragraph::new(l1).style(st);
        let p2 = Paragraph::new(l2).style(st);

        let row1 = Rect::new(area.x, area.y, Self::WIDTH, 1);
        let row2 = Rect::new(area.x, area.y + 1, Self::WIDTH, 1);

        p1.render(row1, buf);
        p2.render(row2, buf);
    }
}
