use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::i18n::t;
use crate::ui::renderer::{render_logo_above, LOGO_W};
use crate::ui::strings::truncate;
use crate::ui::theme::{self, Palette};

use super::state::{OnboardingState, Step};

const PANEL_WIDTH: u16 = 88;
const PANEL_HEIGHT: u16 = 18;

pub struct ViewCtx<'a> {
    pub palette: &'a Palette,
    pub border_tick: u32,
}

/// Renders the current onboarding step. Reads `OnboardingState` only — never mutates it.
pub fn render(frame: &mut Frame, area: Rect, state: &OnboardingState, ctx: &ViewCtx<'_>) {
    let palette = ctx.palette;
    let bg = palette.panel_bg;

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            frame.buffer_mut()[(x, y)].set_bg(palette.overlay_color);
        }
    }

    let panel = centered(area, PANEL_WIDTH, PANEL_HEIGHT);
    let title = format!(
        " {} · {}/{} ",
        t("onboarding.title"),
        state.step.position() + 1,
        Step::ALL.len()
    );
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
        Step::Welcome => render_welcome(frame, body, ctx),
        Step::OverlayPreferences => render_overlay_step(frame, body, state, palette),
        Step::PlaybackPreferences => render_playback_step(frame, body, state, palette),
        Step::Summary => render_summary(frame, body, state, palette),
    }

    render_footer(frame, footer, state, palette);
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w, h)
}

fn render_welcome(frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
    let palette = ctx.palette;
    let bg = palette.panel_bg;
    let bg_style = Style::default().bg(bg);

    let logo_y = area.y + 2;
    render_logo_above(
        frame,
        area.x,
        area.width.max(LOGO_W),
        logo_y,
        bg,
        ctx.border_tick,
        palette,
    );

    let text_y = logo_y + 3;
    let text_area = Rect::new(
        area.x + 2,
        text_y,
        area.width.saturating_sub(4),
        area.bottom().saturating_sub(text_y),
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                t("onboarding.welcome.heading"),
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                t("onboarding.welcome.body"),
                Style::default().fg(palette.highlight),
            )),
        ])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .style(bg_style),
        text_area,
    );
}

fn render_overlay_step(frame: &mut Frame, area: Rect, state: &OnboardingState, palette: &Palette) {
    let rows = [
        (t("onboarding.overlay.mode"), state.overlay_mode.display()),
        (
            t("onboarding.overlay.position"),
            state.overlay_position.display(),
        ),
        (
            t("onboarding.overlay.alpha"),
            format!("{}%", state.overlay_alpha),
        ),
    ];
    render_option_step(
        frame,
        area,
        &t("onboarding.overlay.heading"),
        &rows,
        state.focused_option,
        palette,
    );
}

fn render_playback_step(frame: &mut Frame, area: Rect, state: &OnboardingState, palette: &Palette) {
    let on = t("config.value.on");
    let off = t("config.value.off");
    let rows = [
        (
            t("onboarding.playback.autoplay"),
            if state.autoplay_last {
                on.clone()
            } else {
                off.clone()
            },
        ),
        (
            t("onboarding.playback.restore_volume"),
            if state.restore_volume {
                on.clone()
            } else {
                off.clone()
            },
        ),
        (
            t("onboarding.playback.auto_update"),
            if state.auto_update { on } else { off },
        ),
    ];
    render_option_step(
        frame,
        area,
        &t("onboarding.playback.heading"),
        &rows,
        state.focused_option,
        palette,
    );
}

fn render_option_step(
    frame: &mut Frame,
    area: Rect,
    heading: &str,
    rows: &[(String, String)],
    focused: usize,
    palette: &Palette,
) {
    let bg_style = Style::default().bg(palette.panel_bg);
    let list_x = area.x + 2;
    let list_w = area.width.saturating_sub(4);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            heading.to_uppercase(),
            Style::default()
                .fg(palette.muted)
                .add_modifier(Modifier::BOLD),
        )))
        .style(bg_style),
        Rect::new(list_x, area.y + 1, list_w, 1),
    );

    for (i, (label, value)) in rows.iter().enumerate() {
        let y = area.y + 3 + i as u16 * 2;
        if y >= area.bottom() {
            break;
        }
        render_radio_row(
            frame,
            Rect::new(list_x, y, list_w, 1),
            label,
            value,
            i == focused,
            palette,
        );
    }
}

fn render_radio_row(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    palette: &Palette,
) {
    let marker = if focused { "(•) " } else { "( ) " };
    let label_style = if focused {
        Style::default()
            .fg(palette.radio_accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.highlight)
    };
    let value_style = if focused {
        Style::default()
            .fg(palette.playing)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(palette.accent)
    };

    let label_col_w = (area.width / 2) as usize;
    let label_str = truncate(label, label_col_w.saturating_sub(marker.len()));
    let padding = label_col_w.saturating_sub(marker.len() + label_str.chars().count());

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(marker, label_style),
            Span::styled(label_str, label_style),
            Span::styled(" ".repeat(padding), Style::default()),
            Span::styled("◀ ", Style::default().fg(palette.dim)),
            Span::styled(value.to_string(), value_style),
            Span::styled(" ▶", Style::default().fg(palette.dim)),
        ]))
        .style(Style::default().bg(palette.panel_bg)),
        area,
    );
}

fn render_summary(frame: &mut Frame, area: Rect, state: &OnboardingState, palette: &Palette) {
    let bg_style = Style::default().bg(palette.panel_bg);
    let on = t("config.value.on");
    let off = t("config.value.off");
    let rows = [
        (t("onboarding.overlay.mode"), state.overlay_mode.display()),
        (
            t("onboarding.overlay.position"),
            state.overlay_position.display(),
        ),
        (
            t("onboarding.overlay.alpha"),
            format!("{}%", state.overlay_alpha),
        ),
        (
            t("onboarding.playback.autoplay"),
            if state.autoplay_last {
                on.clone()
            } else {
                off.clone()
            },
        ),
        (
            t("onboarding.playback.restore_volume"),
            if state.restore_volume {
                on.clone()
            } else {
                off.clone()
            },
        ),
        (
            t("onboarding.playback.auto_update"),
            if state.auto_update { on } else { off },
        ),
    ];

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            t("onboarding.summary.heading"),
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .style(bg_style),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );

    let list_x = area.x + 4;
    let list_w = area.width.saturating_sub(8);
    let label_col_w = (list_w / 2) as usize;
    for (i, (label, value)) in rows.iter().enumerate() {
        let y = area.y + 3 + i as u16;
        if y >= area.bottom() {
            break;
        }
        let label_str = truncate(label, label_col_w);
        let padding = label_col_w.saturating_sub(label_str.chars().count());
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(label_str, Style::default().fg(palette.dim)),
                Span::styled(" ".repeat(padding), Style::default()),
                Span::styled(value.clone(), Style::default().fg(palette.accent)),
            ]))
            .style(bg_style),
            Rect::new(list_x, y, list_w, 1),
        );
    }

    let body_y = area.y + 3 + rows.len() as u16 + 1;
    if body_y < area.bottom() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                t("onboarding.summary.body"),
                Style::default().fg(palette.dim),
            ))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(bg_style),
            Rect::new(area.x + 2, body_y, area.width.saturating_sub(4), 2),
        );
    }
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

    if state.step.option_count() > 0 {
        spans.extend(hint(palette, "↑↓", t("onboarding.hint.navigate")));
        spans.extend(hint(palette, "↵", t("onboarding.hint.change")));
    }
    if state.step.position() > 0 {
        spans.extend(hint(palette, "←", t("hint.back")));
    }
    match state.step {
        Step::Summary => spans.extend(hint(palette, "↵", t("onboarding.hint.finish"))),
        _ => spans.extend(hint(palette, "→", t("onboarding.hint.continue"))),
    }
    let mute_label = if state.muted {
        t("onboarding.hint.unmute")
    } else {
        t("onboarding.hint.mute")
    };
    spans.extend(hint(palette, "M", mute_label));
    spans.extend(hint(palette, "Esc", t("onboarding.hint.skip")));

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(palette.panel_bg)),
        area,
    );
}
