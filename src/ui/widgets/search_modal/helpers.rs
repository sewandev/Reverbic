use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::ui::theme;

pub(super) const EXAMPLES: &[&str] = &[
    "The Jazz Radio",
    "BBC World Service",
    "Classic FM",
    "Radio Nacional",
    "Indie Rock",
    "Tomorrowland Radio",
    "Salsa y Bachata",
    "Lofi Hip Hop",
    "NPR News",
    "Deep House",
];

pub(super) fn placeholder_example() -> &'static str {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    EXAMPLES[(secs / 3) as usize % EXAMPLES.len()]
}

pub(super) fn spin_frame() -> &'static str {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    const SPIN: &[&str] = &["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
    SPIN[(ms / 120) as usize % SPIN.len()]
}

pub(super) fn screensaver_display(secs: u16) -> String {
    match secs {
        0          => "OFF".to_string(),
        s if s < 60 => format!("{}s", s),
        s           => format!("{}m", s / 60),
    }
}

pub(super) fn key(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::HIGHLIGHT).add_modifier(Modifier::BOLD))
}

pub(super) fn sep(s: &'static str) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::MUTED))
}

pub(super) fn sep_s(s: String) -> Span<'static> {
    Span::styled(s, Style::default().fg(theme::MUTED))
}

pub(super) fn render_filter_input(filter: &str, placeholder: &str, text_area: Rect, buf: &mut Buffer) {
    if filter.is_empty() {
        Paragraph::new(Span::styled(placeholder, Style::default().fg(theme::MUTED)))
            .render(text_area, buf);
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(filter, Style::default().fg(theme::HIGHLIGHT)),
            Span::styled("_", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        ]))
        .render(text_area, buf);
    }
}
