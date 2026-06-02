use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget},
};

use crate::i18n::t;
use crate::station::filter_items;
use crate::ui::theme;

use super::BG;

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

pub(super) fn render_filter_list_body(
    filter: &str,
    placeholder: &str,
    items: &[(&'static str, &'static str)],
    selected: usize,
    loading: bool,
    loading_text: &str,
    area: Rect,
    content_x: u16,
    content_w: u16,
    buf: &mut Buffer,
) -> (bool, Rect, usize) {
    let [_gap, input_row, cap_row, list_body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    buf[(content_x, input_row.y)]
        .set_symbol("┃").set_fg(theme::ACCENT).set_bg(BG);

    let text_x    = content_x + 2;
    let text_w    = content_w.saturating_sub(2);
    let text_area = Rect::new(text_x, input_row.y, text_w, 1);

    render_filter_input(filter, placeholder, text_area, buf);

    buf[(content_x, cap_row.y)]
        .set_symbol("╹").set_fg(theme::ACCENT).set_bg(BG);

    if loading {
        Paragraph::new(Span::styled(
            format!("{}  {}", spin_frame(), loading_text),
            Style::default().fg(theme::MUTED),
        ))
        .render(Rect::new(text_x, list_body.y, text_w, 1), buf);
        return (false, Rect::default(), 0);
    }

    let list_area = Rect::new(text_x, list_body.y, text_w, list_body.height);
    let visible_n = list_area.height.saturating_sub(1) as usize;
    let filtered  = filter_items(items, filter);

    if filtered.is_empty() {
        Paragraph::new(Span::styled(t("modal.empty.no_match"), Style::default().fg(theme::MUTED)))
            .render(list_area, buf);
        return (false, list_area, 0);
    }

    let offset = crate::ui::widgets::scroll_offset(selected, visible_n);

    let list_items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_n)
        .map(|(i, (_, label))| {
            let active = i == selected;
            let (prefix, style) = if active {
                ("▶  ", Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD))
            } else {
                ("   ", Style::default().fg(theme::HIGHLIGHT))
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(*label, style),
            ]))
        })
        .collect();

    List::new(list_items).render(list_area, buf);

    (filtered.len() > visible_n, list_area, filtered.len())
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