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
        recent_titles: &'a [String],
    },
    Spotify {
        playback: &'a crate::integrations::spotify::SpotifyPlaybackState,
        profile_name: Option<&'a str>,
        country: Option<&'a str>,
        followers: Option<u32>,
        is_premium: Option<bool>,
        recent_titles: &'a [String],
    },
}

impl<'a> AmbientContent<'a> {
    fn required_rows(&self, cw_est: u16, config: &crate::config::Config) -> u16 {
        match self {
            AmbientContent::Radio {
                state,
                details,
                enriched_track,
                recent_titles,
                ..
            } => {
                let has_recent = !recent_titles.is_empty();

                let detail_rows: u16 = if config.screensaver_station_details {
                    details
                        .map(|d| {
                            let c = crate::ui::widgets::station_details::StationDetailsWidget::content_rows(d);
                            if c > 0 {
                                c + 1
                            } else {
                                0
                            }
                        })
                        .unwrap_or(0)
                } else {
                    0
                };
                let title = state.title.as_deref().unwrap_or("—");
                let title_block_h: u16 = match enriched_track {
                    Some(et) => {
                        strings::wrapped_line_count(&et.artist, cw_est, 2)
                            + strings::wrapped_line_count(&et.title, cw_est, 2)
                            + if et.album.is_empty() {
                                0
                            } else {
                                strings::wrapped_line_count(&et.album, cw_est, 2)
                            }
                    }
                    None => strings::wrapped_line_count(title, cw_est, 2),
                };
                let chapter_rows: u16 = if is_youtube_state(state)
                    && state
                        .api_show
                        .as_deref()
                        .is_some_and(|show| show.contains(" · "))
                {
                    1
                } else {
                    0
                };
                let has_playback_progress = state.playback_pos_secs.is_some();
                let now_playing_rows = if config.screensaver_now_playing {
                    title_block_h + chapter_rows
                } else {
                    0
                };

                now_playing_rows
                    + if config.screensaver_progress_bar && has_playback_progress { 1 } else { 0 }
                    + 1 // gap
                    + if config.screensaver_visualizer { 1 } else { 0 }
                    + 1 // gap
                    + detail_rows
                    + if config.screensaver_recent_tracks && has_recent { 1 + 6 } else { 0 }
            }
            AmbientContent::Spotify {
                playback,
                profile_name,
                country,
                followers,
                is_premium,
                recent_titles,
            } => {
                let has_name_row = profile_name.is_some() || country.is_some();
                let has_plan_row =
                    is_premium.is_some_and(|is_premium| is_premium) || followers.is_some();
                let profile_rows = u16::from(has_name_row) + u16::from(has_plan_row);
                let has_profile = profile_rows > 0;
                let has_recent = !recent_titles.is_empty();

                let device_rows = strings::wrapped_line_count(&playback.device_name, cw_est, 2);
                let track_block = if config.screensaver_now_playing {
                    strings::wrapped_line_count(&playback.artist, cw_est, 2)
                        + strings::wrapped_line_count(&playback.track_name, cw_est, 2)
                        + strings::wrapped_line_count(&playback.album, cw_est, 2)
                } else {
                    0
                };

                device_rows
                + 1 // gap
                + track_block
                + 1 // gap
                + if config.screensaver_progress_bar { 2 } else { 0 }
                + 1 // gap
                + if config.screensaver_visualizer { 2 } else { 0 } // gap inside visualization
                + 1 // gap
                + if has_profile { 1 + profile_rows } else { 0 }
                + if config.screensaver_recent_tracks && has_recent { 1 + 6 } else { 0 }
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

    register_url_hit(None);

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
        crate::ui::widgets::clock::ClockWidget::HEIGHT + 1
    } else {
        0
    };
    let name_h: u16 = if config.screensaver_now_playing { 1 } else { 0 };
    let ph = 2 // borders
        + 1 // top margin
        + clock_h
        + name_h // station/spotify name
        + specific_rows
        + 1 // bottom gap
        + crate::ui::widgets::controls::ControlsWidget::HEIGHT; // shortcuts + volume

    let logo_band: u16 = if config.screensaver_logo { 3 } else { 0 };
    let avail_h = area.height.saturating_sub(logo_band);
    let ph_clamped = ph.min(avail_h);

    let px = area.x + area.width.saturating_sub(pw) / 2;
    let py = area.y + logo_band + avail_h.saturating_sub(ph_clamped) / 2;
    let panel = Rect::new(px, py, pw, ph_clamped);

    if config.screensaver_logo && py >= 2 {
        crate::ui::widgets::logo::LogoWidget::new(overlay, border_tick, palette).render_centered(
            frame,
            area.x,
            area.width,
            py - 2,
        );
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

    if config.screensaver_now_playing && row < inner.bottom() {
        match &content {
            AmbientContent::Radio {
                state, is_favorite, ..
            } => {
                if is_youtube_state(state) {
                    frame.render_widget(
                        Paragraph::new(Span::styled(
                            "YOUTUBE",
                            Style::default()
                                .fg(palette.youtube)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .alignment(Alignment::Center)
                        .style(Style::default().bg(bg)),
                        Rect::new(cx, row, cw, 1).intersection(inner),
                    );
                } else {
                    let raw_name = state
                        .station
                        .as_ref()
                        .map(|s| s.name.as_str())
                        .unwrap_or("—");
                    let prefix = if *is_favorite { "★  " } else { "" };
                    let station_str = strings::truncate(
                        &format!("{prefix}{}", raw_name.to_uppercase()),
                        cw as usize,
                    );
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
            recent_titles,
            ..
        } => {
            row = render_radio_info(
                frame,
                row,
                &ctx,
                state,
                *details,
                *enriched_track,
                recent_titles,
            );
        }
        AmbientContent::Spotify {
            playback,
            profile_name,
            country,
            followers,
            is_premium,
            recent_titles,
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
                recent_titles,
            );
        }
    }

    row += 1;

    if row < inner.bottom() {
        let pause = t("screensaver.action.pause");
        let volume = t("screensaver.action.volume");
        let exit = t("screensaver.action.exit");
        let any_key = t("screensaver.action.any_key");

        let (vol_pct, vol_color, shortcuts) = match &content {
            AmbientContent::Radio { state, .. } => {
                let pct = (state.volume.clamp(0.0, 1.0) * 100.0).round() as u32;
                let color = if state.volume > 0.85 {
                    palette.warning
                } else {
                    palette.accent
                };
                let shortcuts = vec![
                    ("Space".to_string(), pause),
                    ("+/-".to_string(), volume),
                    ("Alt+S".to_string(), t("screensaver.action.stop")),
                    (any_key, exit),
                ];
                (pct, color, shortcuts)
            }
            AmbientContent::Spotify { playback, .. } => {
                let color = if playback.volume_pct > 85 {
                    palette.warning
                } else {
                    palette.playing
                };
                let shortcuts = vec![
                    ("Space".to_string(), pause),
                    ("+/-".to_string(), volume),
                    (any_key, exit),
                ];
                (playback.volume_pct as u32, color, shortcuts)
            }
        };

        let controls = crate::ui::widgets::controls::ControlsWidget::new(
            shortcuts, vol_pct, vol_color, bg, palette,
        );
        frame.render_widget(
            controls,
            Rect::new(
                cx,
                row,
                cw,
                crate::ui::widgets::controls::ControlsWidget::HEIGHT,
            )
            .intersection(inner),
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
    recent_titles: &[String],
) -> u16 {
    if row >= ctx.inner.bottom() {
        return row;
    }

    let title = state.title.as_deref().unwrap_or("—");
    let title_rows = strings::wrapped_line_count(title, ctx.cw, 2);

    if ctx.config.screensaver_now_playing {
        if let Some(et) = enriched_track {
            if row < ctx.inner.bottom() {
                let artist_rows = strings::wrapped_line_count(&et.artist, ctx.cw, 2);
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        et.artist.clone(),
                        Style::default().fg(ctx.palette.muted),
                    ))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .style(Style::default().bg(ctx.bg)),
                    Rect::new(ctx.cx, row, ctx.cw, artist_rows).intersection(ctx.inner),
                );
                row += artist_rows;
            }
            if row < ctx.inner.bottom() {
                let title_rows = strings::wrapped_line_count(&et.title, ctx.cw, 2);
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        et.title.clone(),
                        Style::default()
                            .fg(ctx.palette.highlight)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .style(Style::default().bg(ctx.bg)),
                    Rect::new(ctx.cx, row, ctx.cw, title_rows).intersection(ctx.inner),
                );
                row += title_rows;
            }
            if !et.album.is_empty() && row < ctx.inner.bottom() {
                let album_rows = strings::wrapped_line_count(&et.album, ctx.cw, 2);
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        et.album.clone(),
                        Style::default().fg(ctx.palette.dim),
                    ))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .style(Style::default().bg(ctx.bg)),
                    Rect::new(ctx.cx, row, ctx.cw, album_rows).intersection(ctx.inner),
                );
                row += album_rows;
            }
        } else if row < ctx.inner.bottom() {
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

        if is_youtube_state(state) && row < ctx.inner.bottom() {
            if let Some(chapter) = state
                .api_show
                .as_deref()
                .and_then(|show| show.split_once(" · "))
                .map(|(_, chapter)| chapter)
            {
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        strings::truncate(chapter, ctx.cw as usize),
                        Style::default().fg(ctx.palette.accent),
                    ))
                    .alignment(Alignment::Center)
                    .style(Style::default().bg(ctx.bg)),
                    Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
                );
                row += 1;
            }
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

    let has_recent = !recent_titles.is_empty();
    let details = if ctx.config.screensaver_station_details {
        details
    } else {
        None
    };
    let detail_content = details
        .map(crate::ui::widgets::station_details::StationDetailsWidget::content_rows)
        .unwrap_or(0);

    if let Some(d) = details {
        if detail_content > 0 && row < ctx.inner.bottom() {
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

            let area = Rect::new(ctx.cx, row, ctx.cw, detail_content).intersection(ctx.inner);
            let widget = crate::ui::widgets::station_details::StationDetailsWidget::new(
                d,
                ctx.bg,
                ctx.palette,
            );
            register_url_hit(widget.url_rect(area));
            frame.render_widget(widget, area);
            row += detail_content;
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

        let badge = t("screensaver.now_playing");
        let recent_tracks = crate::ui::widgets::recent_tracks::RecentTracksWidget::new(
            recent_titles,
            &badge,
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

#[allow(clippy::too_many_arguments)]
fn render_spotify_info(
    frame: &mut Frame,
    mut row: u16,
    ctx: &AmbientCtx<'_>,
    playback: &crate::integrations::spotify::SpotifyPlaybackState,
    profile_name: Option<&str>,
    country: Option<&str>,
    followers: Option<u32>,
    is_premium: Option<bool>,
    recent_titles: &[String],
) -> u16 {
    if row >= ctx.inner.bottom() {
        return row;
    }
    let green = ctx.palette.playing;

    if row < ctx.inner.bottom() {
        let device_rows = strings::wrapped_line_count(&playback.device_name, ctx.cw, 2);
        frame.render_widget(
            Paragraph::new(Span::styled(
                playback.device_name.clone(),
                Style::default().fg(ctx.palette.dim),
            ))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, device_rows).intersection(ctx.inner),
        );
        row += device_rows;
    }

    row += 1; // gap

    if ctx.config.screensaver_now_playing {
        if row < ctx.inner.bottom() {
            let artist_rows = strings::wrapped_line_count(&playback.artist, ctx.cw, 2);
            frame.render_widget(
                Paragraph::new(Span::styled(
                    playback.artist.clone(),
                    Style::default().fg(ctx.palette.muted),
                ))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .style(Style::default().bg(ctx.bg)),
                Rect::new(ctx.cx, row, ctx.cw, artist_rows).intersection(ctx.inner),
            );
            row += artist_rows;
        }

        if row < ctx.inner.bottom() {
            let track_rows = strings::wrapped_line_count(&playback.track_name, ctx.cw, 2);
            frame.render_widget(
                Paragraph::new(Span::styled(
                    playback.track_name.clone(),
                    Style::default()
                        .fg(ctx.palette.highlight)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .style(Style::default().bg(ctx.bg)),
                Rect::new(ctx.cx, row, ctx.cw, track_rows).intersection(ctx.inner),
            );
            row += track_rows;
        }

        if row < ctx.inner.bottom() {
            let album_rows = strings::wrapped_line_count(&playback.album, ctx.cw, 2);
            frame.render_widget(
                Paragraph::new(Span::styled(
                    playback.album.clone(),
                    Style::default().fg(ctx.palette.dim),
                ))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .style(Style::default().bg(ctx.bg)),
                Rect::new(ctx.cx, row, ctx.cw, album_rows).intersection(ctx.inner),
            );
            row += album_rows;
        }
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

    let profile = crate::ui::widgets::spotify_profile::SpotifyProfileWidget::new(
        profile_name,
        country,
        followers,
        is_premium.unwrap_or(false),
        ctx.bg,
        ctx.palette,
    );

    if profile.has_content() && row < ctx.inner.bottom() {
        let sep = "─".repeat(ctx.cw as usize);
        frame.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(ctx.palette.dim)))
                .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;

        let profile_rows = profile.content_rows();
        frame.render_widget(
            profile,
            Rect::new(ctx.cx, row, ctx.cw, profile_rows).intersection(ctx.inner),
        );
        row += profile_rows;
    }

    if ctx.config.screensaver_recent_tracks && !recent_titles.is_empty() && row < ctx.inner.bottom()
    {
        let sep = "─".repeat(ctx.cw as usize);
        frame.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(ctx.palette.dim)))
                .style(Style::default().bg(ctx.bg)),
            Rect::new(ctx.cx, row, ctx.cw, 1).intersection(ctx.inner),
        );
        row += 1;

        let badge = t("screensaver.now_playing");
        let recent_tracks = crate::ui::widgets::recent_tracks::RecentTracksWidget::new(
            recent_titles,
            &badge,
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

fn is_youtube_state(state: &PlayerState) -> bool {
    state
        .station
        .as_ref()
        .is_some_and(|s| s.key.starts_with("youtube:"))
}

static URL_HIT_RECT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(crate) fn register_url_hit(rect: Option<Rect>) {
    let packed = match rect {
        Some(r) if r.width > 0 && r.height > 0 => {
            ((r.x as u64) << 48)
                | ((r.y as u64) << 32)
                | ((r.width as u64) << 16)
                | (r.height as u64)
        }
        _ => 0,
    };
    URL_HIT_RECT.store(packed, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn url_hit_at(col: u16, row: u16) -> bool {
    let packed = URL_HIT_RECT.load(std::sync::atomic::Ordering::Relaxed);
    if packed == 0 {
        return false;
    }
    let x = (packed >> 48) as u16;
    let y = (packed >> 32) as u16;
    let w = (packed >> 16) as u16;
    let h = packed as u16;
    col >= x && col < x.saturating_add(w) && row >= y && row < y.saturating_add(h)
}

fn fmt_ms(ms: u32) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}
