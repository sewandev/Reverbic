use chrono::{Datelike, Local};

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::ui::theme;

const MONTH_NAMES_ES: [&str; 12] = [
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
];

const DAY_NAMES_ES: [&str; 7] = ["Dom", "Lun", "Mar", "Mié", "Jue", "Vie", "Sáb"];

struct TimeComponents {
    time: String,
    weekday: &'static str,
    day: u32,
    month: &'static str,
}

fn get_time_components() -> TimeComponents {
    let now = Local::now();
    TimeComponents {
        time: now.format("%H:%M:%S").to_string(),
        weekday: DAY_NAMES_ES[now.weekday().num_days_from_sunday() as usize],
        day: now.day(),
        month: MONTH_NAMES_ES[(now.month() - 1) as usize],
    }
}

pub struct LocalTimeWidget;

impl LocalTimeWidget {
    pub fn new() -> Self {
        LocalTimeWidget
    }
}

impl Default for LocalTimeWidget {
    fn default() -> Self {
        LocalTimeWidget
    }
}

impl Widget for LocalTimeWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 {
            return;
        }

        let block = Block::default()
            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
            .border_style(Style::new().fg(theme::MUTED));

        block.render(area, buf);

        let tc = get_time_components();
        let date_str = format!("{} {:02} {}", tc.weekday, tc.day, tc.month);

        let line = Line::from(vec![
            Span::styled(
                tc.time,
                Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::new().fg(theme::MUTED)),
            Span::styled(date_str, Style::new().fg(theme::MUTED)),
        ]);

        Paragraph::new(line)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}
