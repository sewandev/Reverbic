use chrono::{Datelike, Local};

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::ui::theme;

const MONTH_NAMES_ES: [&str; 12] = [
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
];

const DAY_NAMES_ES: [&str; 7] = ["Dom", "Lun", "Mar", "Mié", "Jue", "Vie", "Sáb"];

fn format_local_time() -> String {
    let now = Local::now();
    let time = now.format("%H:%M").to_string();
    let weekday = DAY_NAMES_ES[now.weekday().num_days_from_sunday() as usize];
    let day = now.day();
    let month = MONTH_NAMES_ES[(now.month() - 1) as usize];
    format!("{time} · {weekday} {day} {month}")
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
        if area.width < 16 {
            return;
        }

        let block = Block::default()
            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
            .border_style(Style::new().fg(theme::MUTED));

        block.render(area, buf);

        let text = format_local_time();
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::new().fg(theme::HIGHLIGHT),
        )))
        .alignment(Alignment::Center)
        .render(area, buf);
    }
}
