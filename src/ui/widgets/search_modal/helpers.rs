use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget},
};

use crate::i18n::t;
use crate::station::filter_items;
use crate::ui::theme as ui_palette;
use ui_palette::Palette;

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
    const SPIN: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    SPIN[(ms / 120) as usize % SPIN.len()]
}

pub(super) fn key(palette: &Palette, s: &'static str) -> Span<'static> {
    Span::styled(
        s,
        Style::default()
            .fg(palette.highlight)
            .add_modifier(Modifier::BOLD),
    )
}

pub(super) fn sep_s(palette: &Palette, s: String) -> Span<'static> {
    Span::styled(s, Style::default().fg(palette.muted))
}

pub(super) struct FilterListParams<'a> {
    pub filter: &'a str,
    pub placeholder: &'a str,
    pub items: &'a [(&'static str, &'static str)],
    pub selected: usize,
    pub loading: bool,
    pub loading_text: &'a str,
}

pub(super) fn render_filter_list_body(
    p: FilterListParams<'_>,
    palette: &Palette,
    area: Rect,
    content_x: u16,
    content_w: u16,
    buf: &mut Buffer,
) -> (bool, Rect, usize) {
    let FilterListParams {
        filter,
        placeholder,
        items,
        selected,
        loading,
        loading_text,
    } = p;
    let [_gap, input_row, cap_row, list_body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    buf[(content_x, input_row.y)]
        .set_symbol("┃")
        .set_fg(palette.accent)
        .set_bg(palette.panel_bg);

    let text_x = content_x + 2;
    let text_w = content_w.saturating_sub(2);
    let text_area = Rect::new(text_x, input_row.y, text_w, 1);

    render_filter_input(filter, placeholder, text_area, palette, buf, palette.accent);

    buf[(content_x, cap_row.y)]
        .set_symbol("╹")
        .set_fg(palette.accent)
        .set_bg(palette.panel_bg);

    if loading {
        Paragraph::new(Span::styled(
            format!("{}  {}", spin_frame(), loading_text),
            Style::default().fg(palette.muted),
        ))
        .render(Rect::new(text_x, list_body.y, text_w, 1), buf);
        return (false, Rect::default(), 0);
    }

    let list_area = Rect::new(text_x, list_body.y, text_w, list_body.height);
    let visible_n = list_area.height.saturating_sub(1) as usize;
    let filtered = filter_items(items, filter);

    if filtered.is_empty() {
        Paragraph::new(Span::styled(
            t("modal.empty.no_match"),
            Style::default().fg(palette.muted),
        ))
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
                (
                    "▶  ",
                    Style::default()
                        .fg(palette.playing)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("   ", Style::default().fg(palette.highlight))
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

pub(super) fn render_filter_input(
    filter: &str,
    placeholder: &str,
    text_area: Rect,
    palette: &Palette,
    buf: &mut Buffer,
    cursor_color: ratatui::style::Color,
) {
    if filter.is_empty() {
        Paragraph::new(Span::styled(
            placeholder,
            Style::default().fg(palette.muted),
        ))
        .render(text_area, buf);
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(filter, Style::default().fg(palette.highlight)),
            Span::styled(
                "_",
                Style::default()
                    .fg(cursor_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .render(text_area, buf);
    }
}
