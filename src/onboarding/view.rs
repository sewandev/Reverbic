use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::i18n::t;

use crate::ui::renderer::overlays::render_theme_picker_overlay;
use crate::ui::strings::truncate;
use crate::ui::theme::{self, Palette, ThemeId};
use crate::ui::widgets::{scroll_offset_for_selection, theme_picker};

use super::state::{OnboardingState, Step};

const PANEL_WIDTH: u16 = 88;
const PANEL_HEIGHT: u16 = 21;
const LOGO_RESERVED: u16 = 3;
const MARKER_WIDTH: usize = 2;
const SWATCH_WIDTH: usize = 2;
const SWATCH_GAP: usize = 1;
const PREVIEW_WIDTH: u16 = (SWATCH_WIDTH * 3 + SWATCH_GAP * 2) as u16;

pub struct ViewCtx<'a> {
    pub palette: &'a Palette,
    pub border_tick: u32,
}

pub fn render(frame: &mut Frame, area: Rect, state: &OnboardingState, ctx: &ViewCtx<'_>) {
    let palette = ctx.palette;
    let bg = palette.panel_bg;

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            frame.buffer_mut()[(x, y)].set_bg(palette.overlay_color);
        }
    }

    let content_area = Rect::new(
        area.x,
        area.y + LOGO_RESERVED,
        area.width,
        area.height.saturating_sub(LOGO_RESERVED),
    );
    let panel = centered(content_area, PANEL_WIDTH, PANEL_HEIGHT);

    render_logo_header(frame, panel, palette, ctx.border_tick);

    let title = format!(" {}/{} ", state.step.position() + 1, Step::ALL.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::border_color_for(palette, ctx.border_tick)))
        .style(Style::default().bg(bg))
        .title(Span::styled(
            title,
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);
    frame.render_widget(Paragraph::new("").style(Style::default().bg(bg)), inner);

    let [body, footer] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(inner);

    match state.step {
        Step::Appearance => render_appearance_step(frame, body, state, palette),
        Step::Setup => render_setup_step(frame, body, state, palette),
    }

    render_footer(frame, footer, state, palette);

    if state.theme_picker_open {
        render_theme_picker(frame, area, state, palette);
    }
}

fn render_theme_picker(frame: &mut Frame, area: Rect, state: &OnboardingState, palette: &Palette) {
    let count = ThemeId::all().len();
    let selected = ThemeId::all()
        .position(|theme| theme == state.theme)
        .unwrap_or(0);
    let visible = theme_picker::visible_rows(area, count);
    let scroll = scroll_offset_for_selection(selected, visible, 0);
    render_theme_picker_overlay(frame, state.theme_before_picker, selected, scroll, palette);
}

fn render_logo_header(frame: &mut Frame, panel: Rect, palette: &Palette, tick: u32) {
    crate::ui::widgets::logo::LogoWidget::new(palette.overlay_color, tick, palette)
        .render_centered(
            frame,
            panel.x,
            panel.width,
            panel.y.saturating_sub(LOGO_RESERVED),
        );
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w, h)
}

fn render_appearance_step(
    frame: &mut Frame,
    area: Rect,
    state: &OnboardingState,
    palette: &Palette,
) {
    let bg = palette.panel_bg;
    let bg_style = Style::default().bg(bg);
    let list_x = area.x + 2;
    let list_w = area.width.saturating_sub(4);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            t("onboarding.appearance.heading").to_uppercase(),
            Style::default()
                .fg(palette.muted)
                .add_modifier(Modifier::BOLD),
        )))
        .style(bg_style),
        Rect::new(list_x, area.y + 1, list_w, 1),
    );

    field_row(
        frame,
        Rect::new(list_x, area.y + 3, list_w, 1),
        &t("onboarding.appearance.language"),
        &state.language.display(),
        state.focused_option == 0,
        palette,
    );

    let theme_y = area.y + 5;
    field_row(
        frame,
        Rect::new(list_x, theme_y, list_w.saturating_sub(PREVIEW_WIDTH + 2), 1),
        &t("onboarding.appearance.theme"),
        &state.theme.display(),
        state.focused_option == 1,
        palette,
    );
    render_swatches(
        frame,
        Rect::new(
            list_x + list_w.saturating_sub(PREVIEW_WIDTH),
            theme_y,
            PREVIEW_WIDTH,
            1,
        ),
        theme::definition(state.theme).preview,
        bg,
    );

    let tooltip = if state.focused_option == 0 {
        t("config.tooltip.language")
    } else {
        t("config.tooltip.theme")
    };
    render_tooltip(frame, list_x, list_w, area.bottom(), &tooltip, palette);
}

fn render_swatches(frame: &mut Frame, area: Rect, preview: [Color; 3], bg: Color) {
    let mut spans = Vec::new();
    for (i, color) in preview.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                " ".repeat(SWATCH_GAP),
                Style::default().bg(bg),
            ));
        }
        spans.push(Span::styled(
            " ".repeat(SWATCH_WIDTH),
            Style::default().bg(color),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
        area,
    );
}

fn render_setup_step(frame: &mut Frame, area: Rect, state: &OnboardingState, palette: &Palette) {
    let mut rows = Vec::new();
    if cfg!(target_os = "windows") {
        rows.push((
            t("onboarding.overlay.mode"),
            state.overlay_mode.display(),
            t("config.tooltip.overlay"),
        ));
        rows.push((
            t("onboarding.overlay.position"),
            state.overlay_position.display(),
            t("config.tooltip.overlay_position"),
        ));
    }
    rows.push((
        t("onboarding.playback.autoplay"),
        on_off_label(state.autoplay_last),
        t("config.tooltip.autoplay"),
    ));
    rows.push((
        t("onboarding.playback.auto_update"),
        on_off_label(state.auto_update),
        t("config.tooltip.auto_update"),
    ));

    let bg_style = Style::default().bg(palette.panel_bg);
    let list_x = area.x + 2;
    let list_w = area.width.saturating_sub(4);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            t("onboarding.quicksetup.heading").to_uppercase(),
            Style::default()
                .fg(palette.muted)
                .add_modifier(Modifier::BOLD),
        )))
        .style(bg_style),
        Rect::new(list_x, area.y + 1, list_w, 1),
    );

    for (i, (label, value, _)) in rows.iter().enumerate() {
        let y = area.y + 3 + i as u16 * 2;
        if y >= area.bottom() {
            break;
        }
        field_row(
            frame,
            Rect::new(list_x, y, list_w, 1),
            label,
            value,
            i == state.focused_option,
            palette,
        );
    }

    let tooltip = rows
        .get(state.focused_option)
        .map(|(_, _, tooltip)| tooltip.as_str())
        .unwrap_or_default();
    render_tooltip(frame, list_x, list_w, area.bottom(), tooltip, palette);
}

fn render_tooltip(
    frame: &mut Frame,
    list_x: u16,
    list_w: u16,
    area_bottom: u16,
    tooltip: &str,
    palette: &Palette,
) {
    let bg_style = Style::default().bg(palette.panel_bg);
    let sep_y = area_bottom.saturating_sub(3);
    let sep = "─".repeat(list_w as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(sep, Style::default().fg(palette.dim))).style(bg_style),
        Rect::new(list_x, sep_y, list_w, 1),
    );
    frame.render_widget(
        Paragraph::new(tooltip.to_string())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(palette.dim).bg(palette.panel_bg)),
        Rect::new(list_x, sep_y + 1, list_w, 2),
    );
}

fn on_off_label(value: bool) -> String {
    if value {
        t("config.value.on")
    } else {
        t("config.value.off")
    }
}

fn field_row(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    palette: &Palette,
) {
    let bg = palette.panel_bg;
    let marker = if focused { "▸ " } else { "  " };
    let total = area.width as usize;

    let value_chars = value.chars().count();
    let value_slot = if focused {
        value_chars + 4
    } else {
        value_chars
    };
    let label_budget = total
        .saturating_sub(MARKER_WIDTH)
        .saturating_sub(value_slot + 1);
    let label_str = truncate(label, label_budget);
    let used = MARKER_WIDTH + label_str.chars().count() + value_slot;
    let pad = total.saturating_sub(used);

    let (marker_style, label_style) = if focused {
        (
            Style::default()
                .fg(palette.radio_accent)
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Style::default().fg(palette.dim),
            Style::default().fg(palette.dim),
        )
    };

    let mut spans = vec![
        Span::styled(marker, marker_style.bg(bg)),
        Span::styled(label_str, label_style.bg(bg)),
        Span::styled(" ".repeat(pad), Style::default().bg(bg)),
    ];
    if focused {
        let bracket = Style::default().fg(palette.accent).bg(bg);
        spans.push(Span::styled("‹ ", bracket));
        spans.push(Span::styled(
            value.to_string(),
            Style::default()
                .fg(palette.playing)
                .add_modifier(Modifier::BOLD)
                .bg(bg),
        ));
        spans.push(Span::styled(" ›", bracket));
    } else {
        spans.push(Span::styled(
            value.to_string(),
            Style::default().fg(palette.muted).bg(bg),
        ));
    }

    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
        area,
    );
}

fn hint(palette: &Palette, key: &str, label: String) -> [Span<'static>; 2] {
    [
        Span::styled(
            format!("[{key}] "),
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{label}  "), Style::default().fg(palette.dim)),
    ]
}

fn render_footer(frame: &mut Frame, area: Rect, state: &OnboardingState, palette: &Palette) {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let is_last = state.step.position() + 1 == Step::ALL.len();

    spans.extend(hint(palette, "↑↓", t("onboarding.hint.navigate")));
    spans.extend(hint(palette, "←→", t("onboarding.hint.change")));
    if state.step == Step::Appearance && state.focused_option == 1 {
        spans.extend(hint(palette, "↵", t("onboarding.hint.themes")));
    }
    if is_last {
        spans.extend(hint(palette, "↵", t("onboarding.hint.finish")));
    } else {
        spans.extend(hint(palette, "Tab", t("onboarding.hint.continue")));
    }
    if state.step.position() > 0 {
        spans.extend(hint(palette, "⇧Tab", t("hint.back")));
    }
    spans.extend(hint(palette, "Esc", t("onboarding.hint.skip")));

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(palette.panel_bg)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::ThemeId;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn field_row_renders_within_narrow_area() {
        let palette = theme::palette(ThemeId::Reverbic);
        let mut terminal =
            Terminal::new(TestBackend::new(20, 3)).expect("TestBackend creation is infallible");

        terminal
            .draw(|frame| {
                field_row(
                    frame,
                    Rect::new(0, 0, 20, 1),
                    "Resume last station",
                    "On",
                    true,
                    palette,
                );
            })
            .expect("TestBackend draw is infallible");
    }

    #[test]
    fn appearance_step_renders_without_panicking() {
        let state = OnboardingState::from_config(&crate::config::Config::default());
        let palette = theme::palette(state.theme);
        let mut terminal =
            Terminal::new(TestBackend::new(60, 18)).expect("TestBackend creation is infallible");

        terminal
            .draw(|frame| {
                render_appearance_step(frame, Rect::new(0, 0, 60, 17), &state, palette);
            })
            .expect("TestBackend draw is infallible");
    }
}
