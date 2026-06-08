mod ambience;
mod ascii_gif;
mod state;
mod transitions;
mod view;

use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures_util::StreamExt;

use crate::audio::AudioPlayer;
use crate::config::Config;
use crate::error::{AppError, Result};
use crate::terminal::Tui;
use crate::ui::theme;

use ambience::AmbienceTrack;
use ascii_gif::AsciiGif;
use state::{OnboardingState, Step};
use view::ViewCtx;

enum Outcome {
    Continue,
    Finish,
    Skip,
}
pub async fn run(tui: &mut Tui, config: &mut Config, player: &AudioPlayer) -> Result<()> {
    let mut state = OnboardingState::from_config(config);
    let palette = theme::palette(config.theme);
    let volume_step = config.volume_step as f32 / 100.0;
    let (gif_tx, mut gif_rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = gif_tx.send(AsciiGif::load());
    });
    let mut gif_loaded = false;
    let mut ascii_gif: Option<AsciiGif> = None;

    state.volume = 0.5;
    player
        .send(crate::audio::PlayerCommand::SetVolume(state.volume))
        .await;

    let mut events = EventStream::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    let mut border_tick: u32 = 0;
    let mut ambience_was_playing = false;
    let mut ambience_track = AmbienceTrack::Pending;
    draw(tui, &state, palette, border_tick, ascii_gif.as_ref())?;

    if !state.muted {
        ambience::play(player, &mut ambience_track).await;
    }

    let outcome = loop {
        draw(tui, &state, palette, border_tick, ascii_gif.as_ref())?;

        tokio::select! {
            _ = ticker.tick() => {
                border_tick = border_tick.wrapping_add(1);
                if state.muted {
                    ambience_was_playing = false;
                } else {
                    let playing_now = player.state().preview_title.is_some();
                    if ambience_was_playing && !playing_now {
                        ambience::play(player, &mut ambience_track).await;
                    }
                    ambience_was_playing = playing_now;
                }
            }
            result = &mut gif_rx, if !gif_loaded => {
                gif_loaded = true;
                ascii_gif = result.ok().flatten();
            }
            maybe_event = events.next() => {
                let Some(Ok(Event::Key(key))) = maybe_event else { continue };
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match handle_key(&mut state, key.code, player, volume_step, &mut ambience_track).await {
                    Outcome::Continue => {}
                    outcome @ (Outcome::Finish | Outcome::Skip) => break outcome,
                }
            }
        }
    };

    ambience::stop(player).await;

    if let Outcome::Finish = outcome {
        state.apply_to(config);
    }
    config.save();
    Ok(())
}

fn draw(
    tui: &mut Tui,
    state: &OnboardingState,
    palette: &theme::Palette,
    border_tick: u32,
    ascii_gif: Option<&AsciiGif>,
) -> Result<()> {
    tui.draw(|frame| {
        let area = frame.area();
        view::render(
            frame,
            area,
            state,
            &ViewCtx {
                palette,
                border_tick,
                ascii_gif,
            },
        );
    })
    .map_err(|e| AppError::Terminal(e.to_string()))?;
    Ok(())
}

async fn handle_key(
    state: &mut OnboardingState,
    code: KeyCode,
    player: &AudioPlayer,
    volume_step: f32,
    ambience_track: &mut AmbienceTrack,
) -> Outcome {
    match code {
        KeyCode::Esc => return Outcome::Skip,
        KeyCode::Char('m' | 'M') => toggle_mute(state, player, ambience_track).await,
        KeyCode::Char('+' | '=') => adjust_volume(state, player, volume_step).await,
        KeyCode::Char('-') => adjust_volume(state, player, -volume_step).await,
        KeyCode::Up => transitions::focus_prev_option(state),
        KeyCode::Down => transitions::focus_next_option(state),
        KeyCode::Enter => {
            if state.step == Step::Summary {
                return Outcome::Finish;
            }
            cycle_focused_option(state);
        }
        KeyCode::Right => {
            if !transitions::next(state) {
                return Outcome::Finish;
            }
        }
        KeyCode::Left => {
            transitions::back(state);
        }
        _ => {}
    }
    Outcome::Continue
}
async fn adjust_volume(state: &mut OnboardingState, player: &AudioPlayer, delta: f32) {
    transitions::adjust_volume(state, delta);
    player
        .send(crate::audio::PlayerCommand::SetVolume(state.volume))
        .await;
}

async fn toggle_mute(
    state: &mut OnboardingState,
    player: &AudioPlayer,
    ambience_track: &mut AmbienceTrack,
) {
    transitions::toggle_muted(state);
    if state.muted {
        ambience::stop(player).await;
    } else {
        ambience::play(player, ambience_track).await;
    }
}
fn cycle_focused_option(state: &mut OnboardingState) {
    match state.step {
        Step::OverlayPreferences => match state.focused_option {
            0 => transitions::cycle_overlay_mode(state),
            1 => transitions::cycle_overlay_position(state),
            _ => transitions::cycle_overlay_alpha(state),
        },
        Step::PlaybackPreferences => match state.focused_option {
            0 => transitions::toggle_autoplay_last(state),
            1 => transitions::toggle_restore_volume(state),
            2 => transitions::cycle_crossfade(state),
            3 => transitions::cycle_screensaver(state),
            _ => transitions::toggle_auto_update(state),
        },
        Step::Welcome | Step::Summary => {}
    }
}
