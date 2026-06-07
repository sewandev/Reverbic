mod ambience;
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

use state::{OnboardingState, Step};
use view::ViewCtx;

enum Outcome {
    Continue,
    Finish,
    Skip,
}

/// Runs the first-setup stepper for new users: a brief welcome with a soft
/// background track and a few preference choices, applied onto `config` once
/// finished. Skipping leaves `config` untouched — both paths persist it so the
/// stepper never appears again after this run.
pub async fn run(tui: &mut Tui, config: &mut Config, player: &AudioPlayer) -> Result<()> {
    let mut state = OnboardingState::from_config(config);
    let palette = theme::palette(config.theme);
    let original_volume = player.state().volume;

    if !state.muted {
        ambience::play(player).await;
    }

    let mut events = EventStream::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    let mut border_tick: u32 = 0;
    let mut ambience_was_playing = false;

    let outcome = loop {
        tui.draw(|frame| {
            let area = frame.area();
            view::render(
                frame,
                area,
                &state,
                &ViewCtx {
                    palette,
                    border_tick,
                },
            );
        })
        .map_err(|e| AppError::Terminal(e.to_string()))?;

        tokio::select! {
            _ = ticker.tick() => {
                border_tick = border_tick.wrapping_add(1);
                // Loop the ambience: `preview_title` flips to `Some` once playback
                // starts and back to `None` once the source plays through to its
                // end (see `check_preview_ended`). A `Some -> None` transition while
                // unmuted means it ended on its own, so replay it. While muted we
                // reset the flag instead of polling, so unmuting can't be mistaken
                // for a natural end and trigger a spurious replay.
                if state.muted {
                    ambience_was_playing = false;
                } else {
                    let playing_now = player.state().preview_title.is_some();
                    if ambience_was_playing && !playing_now {
                        ambience::play(player).await;
                    }
                    ambience_was_playing = playing_now;
                }
            }
            maybe_event = events.next() => {
                let Some(Ok(Event::Key(key))) = maybe_event else { continue };
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match handle_key(&mut state, key.code, player).await {
                    Outcome::Continue => {}
                    outcome @ (Outcome::Finish | Outcome::Skip) => break outcome,
                }
            }
        }
    };

    ambience::stop(player, original_volume).await;

    if let Outcome::Finish = outcome {
        state.apply_to(config);
    }
    config.save();
    Ok(())
}

async fn handle_key(state: &mut OnboardingState, code: KeyCode, player: &AudioPlayer) -> Outcome {
    match code {
        KeyCode::Esc => return Outcome::Skip,
        KeyCode::Char('m' | 'M') => toggle_mute(state, player).await,
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

async fn toggle_mute(state: &mut OnboardingState, player: &AudioPlayer) {
    transitions::toggle_muted(state);
    if state.muted {
        player.send(crate::audio::PlayerCommand::StopPreview).await;
    } else {
        ambience::play(player).await;
    }
}

/// Maps the focused row to its transition. The row order here must match the
/// rows rendered by `view::render_overlay_step` / `view::render_playback_step`.
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
            _ => transitions::toggle_auto_update(state),
        },
        Step::Welcome | Step::Summary => {}
    }
}
