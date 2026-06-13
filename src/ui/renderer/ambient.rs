use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::audio::{PlayerState, PlayerStatus};
use crate::i18n::t;
use crate::station::StationDetails;
use crate::ui::strings;
use crate::ui::theme::{self, Palette};

pub(crate) const MIN_AMBIENT_WIDTH: u16 = 60;
pub(crate) const MIN_AMBIENT_HEIGHT: u16 = 15;

struct AmbientCtx<'a> {
    cx: u16,
    cw: u16,
    inner: Rect,
    bg: ratatui::style::Color,
    palette: &'a Palette,
    config: &'a crate::config::Config,
}

pub(crate) enum AmbientContent<'a> {
    Radio {
        state: &'a PlayerState,
        details: Option<&'a StationDetails>,
        is_favorite: bool,
        enriched_track: Option<&'a crate::metadata::EnrichedTrack>,
    },
    Spotify {
        playback: &'a crate::integrations::spotify::SpotifyPlaybackState,
        profile_name: Option<&'a str>,
        country: Option<&'a str>,
        followers: Option<u32>,
        is_premium: Option<bool>,
    },
}

impl<'a> AmbientContent<'a> {
    fn required_rows(&self, cw_est: u16, config: &crate::config::Config) -> u16 {
        match self {
            AmbientContent::Radio {
                state,
                details,
                enriched_track,
                ..
            } => {
                let has_recent = !state.recent_titles.is_empty();

                let (has_meta, has_tags, has_url) = details
                    .map(|d| {
                        (
                            !d.country.is_empty() || !d.language.is_empty() || !d.codec.is_empty(),
                            !d.tags.is_empty(),
                            !d.homepage.is_empty(),
                        )
                    })
                    .unwrap_or((false, false, false));

                let detail_rows: u16 =
                    u16::from(has_meta) + u16::from(has_tags) + u16::from(has_url);
                let title = state.title.as_deref().unwrap_or("—");
                let title_rows: u16 = if title.chars().count() > cw_est as usize {
                    2
                } else {
                    1
                };
                let title_block_h: u16 = if enriched_track.is_some() {
                    3
                } else {
                    title_rows
                };
                let has_playback_progress = state.playback_pos_secs.is_some();

                title_block_h
                    + if config.screensaver_progress_bar && has_playback_progress { 1 } else { 0 }
                    + 1 // gap
                    + if config.screensaver_visualizer { 1 } else { 0 }
                    + 1 // gap
                    + detail_rows
                    + if config.screensaver_recent_tracks && has_recent { 1 + 6 } else { 0 }
            }
            AmbientContent::Spotify {
                profile_name,
                country,
                followers,
                is_premium,
                ..
            } => {
                let has_name_row = profile_name.is_some() || country.is_some();
                let has_plan_row =
                    is_premium.is_some_and(|is_premium| is_premium) || followers.is_some();
                let profile_rows = u16::from(has_name_row) + u16::from(has_plan_row);
                let has_profile = profile_rows > 0;

                1 // device
                + 1 // gap
                + 3 // Artist, Track, Album
                + 1 // gap
                + if config.screensaver_progress_bar { 2 } else { 0 }
                + 1 // gap
                + if config.screensaver_visualizer { 2 } else { 0 } // gap inside visualization
                + 1 // gap
                + if has_profile { 1 + profile_rows } else { 0 }
            }
        }
    }
}

pub(crate) fn render_ambient_mode(
    frame: &mut Frame,
    area: Rect,
    content: AmbientContent<'_>,
    config: &crate::config::Config,
    border_tick: u32,
    palette: &Palette,
) {
    let overlay = palette.overlay_color;
    let bg = palette.panel_bg;

    frame.render_widget(Clear, area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            frame.buffer_mut()[(x, y)].set_bg(overlay);
        }
    }

    if area.width < MIN_AMBIENT_WIDTH || area.height < MIN_AMBIENT_HEIGHT {
        let msg = Paragraph::new(t("screensaver.min_size_required"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(palette.muted).bg(overlay))
            .wrap(Wrap { trim: true });

        let msg_area = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(1) / 2,
            area.width,
            1,
        );
        frame.render_widget(msg, msg_area);
        return;
    }

    let pw = ((u32::from(area.width) * 85 / 100) as u16)
        .clamp(60, 120)
        .min(area.width);
    let cw_est = pw.saturating_sub(6);

    let specific_rows = content.required_rows(cw_est, config);

    let clock_h: u16 = if config.screensaver_clock {
        crate::ui::widgets::clock::ClockWidget::HEIGHT
    } else {
        0
    };
    let ph = 2 // borders
        + 1 // top margin
        + clock_h
        + 1 // station/spotify name
        + specific_rows
        + 1 // bottom gap
        + 1; // bottom bar

    let ph_clamped = ph.min(area.height);

    let px = area.x + area.width.saturating_sub(pw) / 2;
    let py = area.y + area.height.saturating_sub(ph_clamped) / 2;
    let panel = Rect::new(px, py, pw, ph_clamped);

    if config.screensaver_logo {
        if py >= 2 {
            crate::ui::widgets::logo::LogoWidget::new(overlay, border_tick, palette).render_centered(
                frame,
                area.x,
                area.width,
                py - 2,
            );
        }
    }

    if let Some((ref name, ref genre)) = crate::game_detect::get() {
        if py >= 3 {
            let panel_h: u16 = 3;
            super::overlays::render_game_strip(
                frame,
                Rect::new(px, py.saturating_sub(panel_h), pw, panel_h),
                name,
                genre,
                border_tick,
                palette,
            );
        }
    }

    let border_color = match &content {
        AmbientContent::Radio { state, .. } => match &state.status {
            PlayerStatus::Playing => theme::border_color_for(palette, border_tick),
            PlayerStatus::Paused => palette.warning,
            PlayerStatus::Buffering(_) | PlayerStatus::Reconnecting(_) => palette.buffering,
            _ => palette.muted,
        },
        AmbientContent::Spotify { playback, .. } => {
            if playback.is_playing {
                theme::border_color_for(palette, border_tick)
            } else {
                palette.warning
            }
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let cx = inner.x + 2;
    let cw = inner.width.saturating_sub(4);
    let mut row = inner.y + 1;

    if config.screensaver_clock {
        let clock_w = crate::ui::widgets::clock::ClockWidget::WIDTH;
        let clock_h = crate::ui::widgets::clock::ClockWidget::HEIGHT;
        let clock_x = cx + cw.saturating_sub(clock_w) / 2;

        if row + clock_h <= inner.bottom() {
            let clock_widget = crate::ui::widgets::clock::ClockWidget::new(border_color, bg);
            frame.render_widget(
                clock_widget,
                Rect::new(clock_x, row, clock_w, clock_h).intersection(inner),
            );
            row += clock_h + 1;
        }
    }

    if row < inner.bottom() {
        match &content {
            AmbientContent::Radio {
                state, is_favorite, ..
            } => {
                let raw_name = state
                    .station
                    .as_ref()
                    .map(|s| s.name.as_str())
                    .unwrap_or("—");
                let prefix = if *is_favorite { "★  " } else { "" };
                let station_str =
                    strings::truncate(&format!("{prefix}{}", raw_name.to_uppercase()), cw as usize);
                frame.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        station_str,
                        Style::default()
                            .fg(border_color)
                            .add_modifier(Modifier::BOLD),
                    )))
                    .alignment(Alignment::Center)
                    .style(Style::default().bg(bg)),
                    Rect::new(cx, row, cw, 1).intersection(inner),
                );
            }
            AmbientContent::Spotify { .. } => {
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        "SPOTIFY",
                        Style::default()
                            .fg(palette.playing)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .alignment(Alignment::Center)
                    .style(Style::default().bg(bg)),
                    Rect::new(cx, row, cw, 1).intersection(inner),
                );
            }
        }
        row += 1;
    }

    let ctx = AmbientCtx {
        cx,
        cw,
        inner,
        bg,
        palette,
        config,
    };

    match &content {
        AmbientContent::Radio {
            state,
            details,
            enriched_track,
            ..
        } => {
            row = render_radio_info(
                frame,
                row,
                &ctx,
                state,
                *details,
                *enriched_track,
                border_color,
            );
        }
        AmbientContent::Spotify {
            playback,
            profile_name,
            country,
            followers,
            is_premium,
        } => {
            row = render_spotify_info(
                frame,
                row,
                &ctx,
                playback,
                *profile_name,
                *country,
                *followers,
                *is_premium,
                border_color,
            );
        }
    }

    row += 1;

    if row < inner.bottom() {
        let (vol_pct, vol_color, _is_playing, shortcuts) = match &content {
            AmbientContent::Radio { state, .. } => {
                let pct = (state.volume.clamp(0.0, 1.0) * 100.0).round() as u32;
                let color = if state.volume > 0.85 {
                    palette.warning
                } else {
                    palette.accent
                };
                (
                    pct,
                    color,
                    matches!(state.status, PlayerStatus::Playing),
                    "[Space] ⏸/▶  [+/-] Vol  [Alt+S] ■  [any] →",
                )
            }
            AmbientContent::Spotify { playback, .. } => {
                let color = if playback.volume_pct > 85 {
                    palette.warning
                } else {
                    palette.playing
                };
                (
                    playback.volume_pct as u32,
                    color,
                    playback.is_playing,
                    "[Space] ⏸/▶  [+/-] Vol  [any] →",
                )
            }
        };

        frame.render_widget(
            Paragraph::new(Span::styled(shortcuts, Style::default().fg(palette.dim)))
                .style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
        row += 1;

        let vol_w = cw.saturating_sub(12) as usize;
        let progress_bar = crate::ui::widgets::progress::ProgressBarWidget::new(
            vol_pct as f32 / 100.0,
            vol_color,
            palette.dim,
            bg,
        );

        let mut spans = vec![Span::styled("VOL  ", Style::default().fg(palette.muted))];
        spans.extend(progress_bar.into_spans(vol_w));
        spans.push(Span::styled(
            format!("  {vol_pct:3}%"),
            Style::default().fg(palette.muted),
        ));

        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
            Rect::new(cx, row, cw, 1).intersection(inner),
        );
    }
}

fn render_radio_info(
    frame: &mut Frame,
    mut row: u16,
    ctx: &AmbientCtx<'_>,
    state: &PlayerState,
    details: Option<&StationDetails>,
    enriched_track: Option<&crate::metadata::EnrichedTrack>,
    _border_color: ratatui::style::Color,
) -> u16 {
    if row >= ctx.inner.bottom() {
        return row;
    }

    let title = state.title.as_deref().unwrap_or("—");
    let title_rows: u16 = if title.chars().count() > ctx.cw as usize {
        2
    } else {
        1
    };

    if let Some(et) = enriched_track {
        if row < ctx.inner.bottom() {
            frame.render_widget(
                Paragraph::new(Span::styled(
                    strings::truncate(&et.artist, ctx.cw as usize),
                    Style::default().fg(ctx.palette.muted),
                ))
                .alignment(Alignment::Center)
                .style(Style::default().bg(ctx.bg)),
                Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
            );
            row += 1;
        }
        if row < ctx.inner.bottom() {
            frame.render_widget(
                Paragraph::new(Span::styled(
                    strings::truncate(&et.title, ctx.cw as usize),
                    Style::default()
                        .fg(ctx.palette.highlight)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Center)
                .style(Style::default().bg(ctx.bg)),
                Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
            );
            row += 1;
        }
        if !et.album.is_empty() {
            if row < ctx.inner.bottom() {
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        strings::truncate(&et.album, ctx.cw as usize),
                        Style::default().fg(ctx.palette.dim),
                    ))
                    .alignment(Alignment::Center)
                    .style(Style::default().bg(ctx.bg)),
                    Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
                );
                row += 1;
            }
        }
    } else {
        if row < ctx.inner.bottom() {
            frame.render_widget(
                Paragraph::new(Span::styled(
                    title.to_owned(),
                    Style::default().fg(ctx.palette.highlight),
                ))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .style(Style::default().bg(ctx.bg)),
                Rect::new(ctx.cx, row, ctx.cw, title_rows).intersection(ctx.inner),
            );
            row += title_rows;
        }
    }

    if ctx.config.screensaver_progress_bar
        && state.playback_pos_secs.is_some()
        && row < ctx.inner.bottom()
    {
        let pos = state.playback_pos_secs.unwrap_or(0.0);
        let duration = state.playback_duration_secs.unwrap_or(pos.max(1.0));
        let p_m = (pos / 60.0) as u32;
        let p_s = (pos % 60.0) as u32;
        let t_m = (duration / 60.0) as u32;
        let t_s = (duration % 60.0) as u32;

        let progress_bar = crate::ui::widgets::progress::ProgressBarWidget::new(
            pos / duration.max(1.0),
            ctx.palette.playing,
            ctx.palette.dim,
            ctx.bg,
        );

        let progress_w = ctx.cw.saturating_sub(15) as usize; // reserve space for text

        let mut spans = vec![Span::styled(
            format!("{p_m:02}:{p_s:02}  "),
            Style::default().fg(ctx.palette.muted),
        )];
        spans.extend(progress_bar.into_spans(progress_w));
        spans.push(Span::styled(
            format!("  {t_m:02}:{t_s:02}"),
            Style::default().fg(ctx.palette.muted),
        ));

        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;
    }

    row += 1;
    if ctx.config.screensaver_visualizer && row < ctx.inner.bottom() {
        let viz_source = crate::ui::widgets::visualizer::AudioSource::Live(state.level_db);
        let viz_widget =
            crate::ui::widgets::visualizer::VisualizerWidget::new(viz_source, ctx.bg, ctx.palette);
        frame.render_widget(
            viz_widget,
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;
    }
    row += 1;

    let has_recent = !state.recent_titles.is_empty();
    let (has_meta, has_tags, has_url) = details
        .map(|d| {
            (
                !d.country.is_empty() || !d.language.is_empty() || !d.codec.is_empty(),
                !d.tags.is_empty(),
                !d.homepage.is_empty(),
            )
        })
        .unwrap_or((false, false, false));

    if has_meta || has_tags || has_url {
        let sep = "─".repeat(ctx.cw as usize);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                sep,
                Style::default().fg(ctx.palette.dim),
            )))
            .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;

        if let Some(d) = details {
            if has_meta && row < ctx.inner.bottom() {
                let mut meta_spans = vec![];
                if !d.country.is_empty() {
                    meta_spans.push(Span::styled(
                        format!("[o]  {}  ", d.country),
                        Style::default().fg(ctx.palette.muted),
                    ));
                }
                if !d.language.is_empty() {
                    meta_spans.push(Span::styled(
                        format!("[o]  {}  ", d.language),
                        Style::default().fg(ctx.palette.muted),
                    ));
                }
                if !d.codec.is_empty() {
                    meta_spans.push(Span::styled(
                        format!("[o]  {}", d.codec),
                        Style::default().fg(ctx.palette.muted),
                    ));
                }
                frame.render_widget(
                    Paragraph::new(Line::from(meta_spans)).style(Style::default().bg(ctx.bg)),
                    Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
                );
                row += 1;
            }
            if has_tags && row < ctx.inner.bottom() {
                let tags_str =
                    strings::truncate(&d.tags.join(", "), ctx.cw.saturating_sub(5) as usize);
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled("[o]  ", Style::default().fg(ctx.palette.accent)),
                        Span::styled(tags_str, Style::default().fg(ctx.palette.muted)),
                    ]))
                    .style(Style::default().bg(ctx.bg)),
                    Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
                );
                row += 1;
            }

            if has_url && row < ctx.inner.bottom() {
                let url = strings::truncate(
                    d.homepage.trim_end_matches('/'),
                    ctx.cw.saturating_sub(5) as usize,
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled("[o]  ", Style::default().fg(ctx.palette.accent)),
                        Span::styled(
                            url,
                            Style::default()
                                .fg(ctx.palette.muted)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                    ]))
                    .style(Style::default().bg(ctx.bg)),
                    Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
                );
                row += 1;
            }
        }
    }

    if ctx.config.screensaver_recent_tracks && has_recent && row < ctx.inner.bottom() {
        let sep = "─".repeat(ctx.cw as usize);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                sep,
                Style::default().fg(ctx.palette.dim),
            )))
            .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;

        let recent_tracks = crate::ui::widgets::recent_tracks::RecentTracksWidget::new(
            &state.recent_titles,
            ctx.bg,
            ctx.palette,
        );
        frame.render_widget(
            recent_tracks,
            Rect::new(ctx.cx, row, ctx.cw, 6).intersection(ctx.inner),
        );
        row += 6;
    }

    row
}

fn render_spotify_info(
    frame: &mut Frame,
    mut row: u16,
    ctx: &AmbientCtx<'_>,
    playback: &crate::integrations::spotify::SpotifyPlaybackState,
    profile_name: Option<&str>,
    country: Option<&str>,
    followers: Option<u32>,
    is_premium: Option<bool>,
    _border_color: ratatui::style::Color,
) -> u16 {
    if row >= ctx.inner.bottom() {
        return row;
    }
    let green = ctx.palette.playing;

    if row < ctx.inner.bottom() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                strings::truncate(&playback.device_name, ctx.cw as usize),
                Style::default().fg(ctx.palette.dim),
            ))
            .alignment(Alignment::Center)
            .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;
    }

    row += 1; // gap

    if row < ctx.inner.bottom() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                strings::truncate(&playback.artist, ctx.cw as usize),
                Style::default().fg(ctx.palette.muted),
            ))
            .alignment(Alignment::Center)
            .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;
    }

    if row < ctx.inner.bottom() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                strings::truncate(&playback.track_name, ctx.cw as usize),
                Style::default()
                    .fg(ctx.palette.highlight)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center)
            .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;
    }

    if row < ctx.inner.bottom() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                strings::truncate(&playback.album, ctx.cw as usize),
                Style::default().fg(ctx.palette.dim),
            ))
            .alignment(Alignment::Center)
            .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;
    }

    row += 1; // gap

    if ctx.config.screensaver_progress_bar && playback.progress_ms > 0 && row < ctx.inner.bottom() {
        let progress_ratio = if playback.duration_ms > 0 {
            (playback.progress_ms as f32 / playback.duration_ms as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let time_cur = fmt_ms(playback.progress_ms);
        let time_tot = fmt_ms(playback.duration_ms);

        let time_prefix = format!("{} ", time_cur);
        let time_suffix = format!(" {}", time_tot);
        let bar_w = ctx
            .cw
            .saturating_sub(time_prefix.len() as u16 + time_suffix.len() as u16)
            as usize;
        let progress_bar = crate::ui::widgets::progress::ProgressBarWidget::new(
            progress_ratio,
            green,
            ctx.palette.dim,
            ctx.bg,
        );

        let mut spans = vec![Span::styled(
            time_prefix,
            Style::default().fg(ctx.palette.muted),
        )];
        spans.extend(progress_bar.into_spans(bar_w));
        spans.push(Span::styled(
            time_suffix,
            Style::default().fg(ctx.palette.muted),
        ));

        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;
    }

    if ctx.config.screensaver_visualizer && row < ctx.inner.bottom() {
        let viz_source = crate::ui::widgets::visualizer::AudioSource::Simulated;
        let viz_widget =
            crate::ui::widgets::visualizer::VisualizerWidget::new(viz_source, ctx.bg, ctx.palette);
        frame.render_widget(
            viz_widget,
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;
    }

    let has_name_row = profile_name.is_some() || country.is_some();
    let has_plan_row = is_premium.is_some_and(|is_premium| is_premium) || followers.is_some();
    let has_profile = has_name_row || has_plan_row;

    if has_profile && row < ctx.inner.bottom() {
        let sep = "─".repeat(ctx.cw as usize);
        frame.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(ctx.palette.dim)))
                .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;

        if has_name_row && row < ctx.inner.bottom() {
            let name_tc = profile_name.map(strings::title_case);
            let country_str = country.unwrap_or("");
            let mut spans: Vec<Span<'static>> = Vec::new();
            if let Some(name) = name_tc {
                spans.push(Span::styled(
                    name,
                    Style::default()
                        .fg(ctx.palette.highlight)
                        .add_modifier(Modifier::BOLD),
                ));
            }
            if !country_str.is_empty() {
                let used = spans
                    .iter()
                    .map(|s| s.content.chars().count() as u16)
                    .sum::<u16>();
                let pad = ctx
                    .cw
                    .saturating_sub(used + country_str.chars().count() as u16);
                spans.push(Span::styled(" ".repeat(pad as usize), Style::default()));
                spans.push(Span::styled(
                    country_str.to_string(),
                    Style::default().fg(ctx.palette.muted),
                ));
            }
            frame.render_widget(
                Paragraph::new(Line::from(spans)).style(Style::default().bg(ctx.bg)),
                Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
            );
            row += 1;
        }

        if has_plan_row && row < ctx.inner.bottom() {
            let plan_str = if is_premium.is_some_and(|is_premium| is_premium) {
                t("integrations.spotify.premium")
            } else {
                String::new()
            };
            let follow_str = followers
                .map(|f| format!("{f} {}", t("screensaver.followers")))
                .unwrap_or_default();
            let mut spans: Vec<Span<'static>> = Vec::new();
            if !plan_str.is_empty() {
                spans.push(Span::styled(
                    plan_str,
                    Style::default().fg(green).add_modifier(Modifier::BOLD),
                ));
            }
            if !follow_str.is_empty() {
                let used = spans
                    .iter()
                    .map(|s| s.content.chars().count() as u16)
                    .sum::<u16>();
                let pad = ctx
                    .cw
                    .saturating_sub(used + follow_str.chars().count() as u16);
                spans.push(Span::styled(" ".repeat(pad as usize), Style::default()));
                spans.push(Span::styled(
                    follow_str,
                    Style::default().fg(ctx.palette.muted),
                ));
            }
            if !spans.is_empty() {
                frame.render_widget(
                    Paragraph::new(Line::from(spans)).style(Style::default().bg(ctx.bg)),
                    Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
                );
                row += 1;
            }
        }
    }

    row
}

fn fmt_ms(ms: u32) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}
