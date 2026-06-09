use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::i18n::t;
use crate::ui::renderer::{render_logo_above, LOGO_W};
use crate::ui::strings::{crossfade_display, screensaver_display, truncate};
use crate::ui::theme::{self, Palette};

use super::ascii_gif::AsciiGif;
use super::state::{OnboardingState, Step};

const PANEL_WIDTH: u16 = 88;
const PANEL_HEIGHT: u16 = 21;
const GIF_GAP: u16 = 4;
const RADIO_MARKER_WIDTH: usize = 4;

pub struct ViewCtx<'a> {
    pub palette: &'a Palette,
    pub border_tick: u32,
    pub ascii_gif: Option<&'a AsciiGif>,
}
pub fn render(frame: &mut Frame, area: Rect, state: &OnboardingState, ctx: &ViewCtx<'_>) {
    let palette = ctx.palette;
    let bg = palette.panel_bg;

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            frame.buffer_mut()[(x, y)].set_bg(palette.overlay_color);
        }
    }
    let gif = ctx
        .ascii_gif
        .filter(|gif| area.width >= PANEL_WIDTH + GIF_GAP + gif.cols);
    let content_area = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1),
    );
    let group_width = PANEL_WIDTH + gif.map_or(0, |gif| GIF_GAP + gif.cols);
    let group = centered(content_area, group_width, PANEL_HEIGHT);
    let panel = Rect::new(group.x, group.y, PANEL_WIDTH.min(group.width), group.height);

    render_maximize_hint(frame, panel, palette);

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
        Step::Welcome => render_welcome(frame, body, ctx),
        Step::Appearance => render_appearance_step(frame, body, state, palette),
        Step::OverlayPreferences => render_overlay_step(frame, body, state, palette),
        Step::PlaybackPreferences => render_playback_step(frame, body, state, palette),
        Step::SpotifyPreferences => render_spotify_step(frame, body, state, palette),
        Step::Summary => render_summary(frame, body, state, palette),
    }

    render_footer(frame, footer, state, palette);

    if let Some(gif) = gif {
        render_ascii_gif(frame, panel, gif);
    }
}

fn render_ascii_gif(frame: &mut Frame, panel: Rect, gif: &AsciiGif) {
    let x = panel.right() + GIF_GAP;
    let y = panel.y + panel.height.saturating_sub(gif.rows) / 2;
    let height = gif.rows.min(panel.bottom().saturating_sub(y));
    gif.render(frame.buffer_mut(), Rect::new(x, y, gif.cols, height));
}

fn render_maximize_hint(frame: &mut Frame, panel: Rect, palette: &Palette) {
    frame.render_widget(
        Paragraph::new(Span::styled(
            t("onboarding.maximize_hint"),
            Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Rect::new(panel.x, panel.y - 1, panel.width, 1),
    );
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
            Line::from(""),
            Line::from(Span::styled(
                t("onboarding.welcome.sources"),
                Style::default().fg(palette.dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                t("onboarding.welcome.performance"),
                Style::default().fg(palette.dim),
            )),
        ])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .style(bg_style),
        text_area,
    );
}

fn render_appearance_step(
    frame: &mut Frame,
    area: Rect,
    state: &OnboardingState,
    palette: &Palette,
) {
    let rows = [
        (
            t("onboarding.appearance.language"),
            state.language.display(),
            t("config.tooltip.language"),
        ),
        (
            t("onboarding.appearance.theme"),
            state.theme.display(),
            t("config.tooltip.theme"),
        ),
        (
            t("config.setting.overlay_style"),
            state.overlay_style.display(),
            t("config.tooltip.overlay_style"),
        ),
    ];
    render_option_step(
        frame,
        area,
        &t("onboarding.appearance.heading"),
        &rows,
        state.focused_option,
        palette,
    );
}

fn render_overlay_step(frame: &mut Frame, area: Rect, state: &OnboardingState, palette: &Palette) {
    let rows = [
        (
            t("onboarding.overlay.mode"),
            state.overlay_mode.display(),
            t("config.tooltip.overlay"),
        ),
        (
            t("onboarding.overlay.position"),
            state.overlay_position.display(),
            t("config.tooltip.overlay_position"),
        ),
        (
            t("onboarding.overlay.alpha"),
            format!("{}%", state.overlay_alpha),
            t("config.tooltip.overlay_alpha"),
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

    let note_y = area.y + 3 + rows.len() as u16 * 2;
    if note_y < area.bottom() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                t("onboarding.overlay.windows_only"),
                Style::default().fg(palette.dim),
            )))
            .alignment(Alignment::Center)
            .style(Style::default().bg(palette.panel_bg)),
            Rect::new(area.x + 2, note_y, area.width.saturating_sub(4), 1),
        );
    }
}

fn render_playback_step(frame: &mut Frame, area: Rect, state: &OnboardingState, palette: &Palette) {
    let rows = [
        (
            t("onboarding.playback.autoplay"),
            on_off_label(state.autoplay_last),
            t("config.tooltip.autoplay"),
        ),
        (
            t("onboarding.playback.restore_volume"),
            on_off_label(state.restore_volume),
            t("config.tooltip.restore_volume"),
        ),
        (
            t("onboarding.playback.crossfade"),
            crossfade_display(state.crossfade_secs),
            t("config.tooltip.crossfade"),
        ),
        (
            t("onboarding.playback.screensaver"),
            screensaver_display(state.screensaver_secs),
            t("config.tooltip.screensaver"),
        ),
        (
            t("onboarding.playback.auto_update"),
            on_off_label(state.auto_update),
            t("config.tooltip.auto_update"),
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

fn render_spotify_step(frame: &mut Frame, area: Rect, state: &OnboardingState, palette: &Palette) {
    let rows = [
        (
            t("config.setting.spotify_stop_on_quit"),
            on_off_label(state.spotify_stop_on_quit),
            t("config.tooltip.spotify_stop_on_quit"),
        ),
        (
            t("config.setting.spotify_start_on_spotify"),
            on_off_label(state.spotify_start_on_spotify),
            t("config.tooltip.spotify_start_on_spotify"),
        ),
        (
            t("config.setting.spotify_playback_mode"),
            state.spotify_playback_mode.display(),
            t("config.tooltip.spotify_playback_mode"),
        ),
        (
            t("config.setting.spotify_radio_mode"),
            on_off_label(state.spotify_radio_enabled),
            t("config.tooltip.spotify_radio_mode"),
        ),
    ];
    render_option_step(
        frame,
        area,
        &t("config.group.spotify"),
        &rows,
        state.focused_option,
        palette,
    );
}

fn render_option_step(
    frame: &mut Frame,
    area: Rect,
    heading: &str,
    rows: &[(String, String, String)],
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

    for (i, (label, value, _)) in rows.iter().enumerate() {
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

    let sep_y = area.bottom().saturating_sub(3);
    let sep = "─".repeat(list_w as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(sep, Style::default().fg(palette.dim))).style(bg_style),
        Rect::new(list_x, sep_y, list_w, 1),
    );

    let tooltip = rows
        .get(focused)
        .map(|(_, _, tooltip)| tooltip.as_str())
        .unwrap_or_default();
    frame.render_widget(
        Paragraph::new(tooltip)
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

fn radio_label_layout(label: &str, label_col_w: usize) -> (String, usize) {
    let label_str = truncate(label, label_col_w.saturating_sub(RADIO_MARKER_WIDTH));
    let padding = label_col_w.saturating_sub(RADIO_MARKER_WIDTH + label_str.chars().count());
    (label_str, padding)
}

fn radio_marker(focused: bool) -> &'static str {
    if focused {
        "(•) "
    } else {
        "( ) "
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
    let marker = radio_marker(focused);
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
    let (label_str, padding) = radio_label_layout(label, label_col_w);

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
    let rows = summary_rows(state);

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
    let col_w = (list_w / 2).saturating_sub(2);
    let rows_count = rows.len() as u16;
    let rows_per_col = (rows_count + 1) / 2;

    for (i, (label, value)) in rows.iter().enumerate() {
        let col = i as u16 / rows_per_col;
        let row = i as u16 % rows_per_col;

        let y = area.y + 3 + row;
        if y >= area.bottom() {
            continue;
        }

        let x = if col == 0 { list_x } else { list_x + col_w + 4 };
        let w = col_w;
        let value_chars = value.chars().count();
        let max_label_w = w.saturating_sub(value_chars as u16 + 1) as usize;

        let label_str = truncate(label, max_label_w);
        let padding = w as usize - label_str.chars().count() - value_chars;
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(label_str, Style::default().fg(palette.dim)),
                Span::styled(" ".repeat(padding), Style::default()),
                Span::styled(value.clone(), Style::default().fg(palette.accent)),
            ]))
            .style(bg_style),
            Rect::new(x, y, w, 1),
        );
    }

    let body_y = area.y + 5 + rows_per_col;
    if body_y < area.bottom() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                t("onboarding.summary.body"),
                Style::default()
                    .fg(palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(bg_style),
            Rect::new(area.x + 2, body_y, area.width.saturating_sub(4), 2),
        );
    }

    let shortcuts_y = body_y + 3;
    if shortcuts_y < area.bottom() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                t("onboarding.summary.shortcuts"),
                Style::default().fg(palette.dim),
            ))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(bg_style),
            Rect::new(area.x + 2, shortcuts_y, area.width.saturating_sub(4), 1),
        );
    }
}

fn summary_rows(state: &OnboardingState) -> Vec<(String, String)> {
    vec![
        (
            t("onboarding.appearance.language"),
            state.language.display(),
        ),
        (t("onboarding.appearance.theme"), state.theme.display()),
        (
            t("config.setting.overlay_style"),
            state.overlay_style.display(),
        ),
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
            on_off_label(state.autoplay_last),
        ),
        (
            t("onboarding.playback.restore_volume"),
            on_off_label(state.restore_volume),
        ),
        (
            t("onboarding.playback.crossfade"),
            crossfade_display(state.crossfade_secs),
        ),
        (
            t("onboarding.playback.screensaver"),
            screensaver_display(state.screensaver_secs),
        ),
        (
            t("onboarding.playback.auto_update"),
            on_off_label(state.auto_update),
        ),
        (
            t("config.setting.spotify_stop_on_quit"),
            on_off_label(state.spotify_stop_on_quit),
        ),
        (
            t("config.setting.spotify_start_on_spotify"),
            on_off_label(state.spotify_start_on_spotify),
        ),
        (
            t("config.setting.spotify_playback_mode"),
            state.spotify_playback_mode.display(),
        ),
        (
            t("config.setting.spotify_radio_mode"),
            on_off_label(state.spotify_radio_enabled),
        ),
    ]
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

fn volume_indicator(volume: f32) -> String {
    let pct = (volume.clamp(0.0, 1.0) * 100.0).round() as u32;
    format!("♪ {pct}%")
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

    let bg_style = Style::default().bg(palette.panel_bg);

    let hints_area = if state.step == Step::Welcome {
        let indicator = volume_indicator(state.volume);
        let indicator_w = indicator.chars().count() as u16;
        frame.render_widget(
            Paragraph::new(Span::styled(
                indicator,
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(bg_style),
            Rect::new(
                area.right().saturating_sub(indicator_w),
                area.y,
                indicator_w,
                1,
            ),
        );
        Rect::new(
            area.x,
            area.y,
            area.width.saturating_sub(indicator_w + 2),
            area.height,
        )
    } else {
        area
    };

    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(bg_style),
        hints_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_markers_keep_the_same_visible_width() {
        assert_eq!(radio_marker(true).chars().count(), RADIO_MARKER_WIDTH);
        assert_eq!(radio_marker(false).chars().count(), RADIO_MARKER_WIDTH);
    }

    #[test]
    fn radio_label_layout_reserves_marker_width_for_padding() {
        let label_col_w = 24;
        let (label, padding) = radio_label_layout("Resume last station", label_col_w);

        assert_eq!(
            RADIO_MARKER_WIDTH + label.chars().count() + padding,
            label_col_w
        );
    }

    #[test]
    fn summary_rows_include_every_onboarding_setting() {
        let state = OnboardingState::from_config(&crate::config::Config::default());
        let rows = summary_rows(&state);
        let labels: Vec<_> = rows.iter().map(|(label, _)| label.as_str()).collect();

        assert!(labels.contains(&t("onboarding.appearance.theme").as_str()));
        assert!(labels.contains(&t("config.setting.overlay_style").as_str()));
        assert!(labels.contains(&t("onboarding.overlay.mode").as_str()));
        assert!(labels.contains(&t("onboarding.overlay.position").as_str()));
        assert!(labels.contains(&t("onboarding.overlay.alpha").as_str()));
        assert!(labels.contains(&t("onboarding.playback.autoplay").as_str()));
        assert!(labels.contains(&t("onboarding.playback.restore_volume").as_str()));
        assert!(labels.contains(&t("onboarding.playback.crossfade").as_str()));
        assert!(labels.contains(&t("onboarding.playback.screensaver").as_str()));
        assert!(labels.contains(&t("onboarding.playback.auto_update").as_str()));
    }
}
