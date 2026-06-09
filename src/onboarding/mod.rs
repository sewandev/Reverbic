mod ambience;
mod ascii_gif;
mod state;
mod transitions;
mod view;

use std::time::{Duration, Instant};

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::audio::AudioPlayer;
use crate::config::Config;
use crate::error::{AppError, Result};
use crate::terminal::Tui;
use crate::ui::theme;

use ambience::{AmbienceResolution, AmbienceTrack};
use ascii_gif::AsciiGif;
use state::{OnboardingState, Step};
use view::ViewCtx;

const WELCOME_VOLUME: f32 = 0.5;
const AMBIENCE_START_TIMEOUT: Duration = Duration::from_secs(12);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Continue,
    Finish,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AmbiencePlayback {
    Idle,
    Starting { since: Instant },
    Playing,
}

struct AmbienceRuntime<'a> {
    track: &'a mut AmbienceTrack,
    tx: &'a mpsc::Sender<AmbienceResolution>,
    resolve_task: &'a mut Option<JoinHandle<()>>,
    playback: &'a mut AmbiencePlayback,
}

pub async fn run(tui: &mut Tui, config: &mut Config, player: &AudioPlayer) -> Result<()> {
    let mut state = OnboardingState::from_config(config);
    let volume_step = config.volume_step as f32 / 100.0;
    let original_volume = player.state().volume;
    let (gif_tx, mut gif_rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = gif_tx.send(AsciiGif::load());
    });
    let mut gif_loaded = false;
    let mut ascii_gif: Option<AsciiGif> = None;

    state.volume = WELCOME_VOLUME;
    player
        .send(crate::audio::PlayerCommand::SetVolume(state.volume))
        .await;

    let mut events = EventStream::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    let mut border_tick: u32 = 0;
    let mut ambience_playback = AmbiencePlayback::Idle;
    let mut ambience_track = AmbienceTrack::Pending;
    let (ambience_tx, mut ambience_rx) = mpsc::channel(1);
    let mut ambience_resolve_task: Option<JoinHandle<()>> = None;
    draw(tui, &state, border_tick, ascii_gif.as_ref())?;

    if !state.muted {
        start_ambience_resolution(
            &mut ambience_track,
            &ambience_tx,
            &mut ambience_resolve_task,
        );
    }

    let outcome = loop {
        draw(tui, &state, border_tick, ascii_gif.as_ref())?;

        tokio::select! {
            _ = ticker.tick() => {
                border_tick = border_tick.wrapping_add(1);
                ambience_playback = advance_ambience_playback(
                    &state,
                    player,
                    &ambience_track,
                    ambience_playback,
                ).await;
            }
            resolution = ambience_rx.recv(), if ambience_resolve_task.is_some() => {
                ambience_resolve_task = None;
                ambience::finish_resolution(&mut ambience_track, resolution);
                if !state.muted {
                    ambience_playback = request_ambience_play(player, &ambience_track).await;
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
                let mut ambience_runtime = AmbienceRuntime {
                    track: &mut ambience_track,
                    tx: &ambience_tx,
                    resolve_task: &mut ambience_resolve_task,
                    playback: &mut ambience_playback,
                };
                match handle_key(
                    &mut state,
                    key.code,
                    player,
                    volume_step,
                    &mut ambience_runtime,
                ).await {
                    Outcome::Continue => {}
                    outcome @ (Outcome::Finish | Outcome::Skip) => break outcome,
                }
            }
        }
    };

    if let Some(task) = ambience_resolve_task {
        task.abort();
    }
    ambience::stop(player).await;

    if let Some(volume) = complete_outcome(outcome, &state, config, original_volume) {
        player
            .send(crate::audio::PlayerCommand::SetVolume(volume))
            .await;
    }
    config.save();
    Ok(())
}

fn start_ambience_resolution(
    ambience_track: &mut AmbienceTrack,
    ambience_tx: &mpsc::Sender<AmbienceResolution>,
    ambience_resolve_task: &mut Option<JoinHandle<()>>,
) {
    if ambience_resolve_task.is_some() {
        return;
    }

    if let Some(task) = ambience::start_resolution(ambience_track, ambience_tx.clone()) {
        *ambience_resolve_task = Some(task);
    }
}

fn complete_outcome(
    outcome: Outcome,
    state: &OnboardingState,
    config: &mut Config,
    original_volume: f32,
) -> Option<f32> {
    match outcome {
        Outcome::Finish => {
            state.apply_to(config);
            (!state.volume_changed).then_some(original_volume)
        }
        Outcome::Skip => Some(original_volume),
        Outcome::Continue => None,
    }
}

fn draw(
    tui: &mut Tui,
    state: &OnboardingState,
    border_tick: u32,
    ascii_gif: Option<&AsciiGif>,
) -> Result<()> {
    let palette = theme::palette(state.theme);
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
    ambience: &mut AmbienceRuntime<'_>,
) -> Outcome {
    match code {
        KeyCode::Esc => return Outcome::Skip,
        KeyCode::Char('m' | 'M') => toggle_mute(state, player, ambience).await,
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
    ambience: &mut AmbienceRuntime<'_>,
) {
    transitions::toggle_muted(state);
    if state.muted {
        ambience::stop(player).await;
        *ambience.playback = AmbiencePlayback::Idle;
    } else {
        start_ambience_resolution(ambience.track, ambience.tx, ambience.resolve_task);
        *ambience.playback = request_ambience_play(player, ambience.track).await;
    }
}

async fn request_ambience_play(
    player: &AudioPlayer,
    ambience_track: &AmbienceTrack,
) -> AmbiencePlayback {
    if ambience::play(player, ambience_track).await {
        AmbiencePlayback::Starting {
            since: Instant::now(),
        }
    } else {
        AmbiencePlayback::Idle
    }
}

async fn advance_ambience_playback(
    state: &OnboardingState,
    player: &AudioPlayer,
    ambience_track: &AmbienceTrack,
    playback: AmbiencePlayback,
) -> AmbiencePlayback {
    if state.muted {
        return AmbiencePlayback::Idle;
    }

    let ambience_is_published = player.state().preview_title.as_deref() == Some(ambience::TITLE);
    match playback {
        AmbiencePlayback::Idle => AmbiencePlayback::Idle,
        AmbiencePlayback::Starting { since } => {
            if ambience_is_published {
                AmbiencePlayback::Playing
            } else if since.elapsed() >= AMBIENCE_START_TIMEOUT {
                tracing::debug!("onboarding ambience: start timed out before player state updated");
                AmbiencePlayback::Idle
            } else {
                playback
            }
        }
        AmbiencePlayback::Playing => {
            if ambience_is_published {
                AmbiencePlayback::Playing
            } else {
                request_ambience_play(player, ambience_track).await
            }
        }
    }
}

fn cycle_focused_option(state: &mut OnboardingState) {
    match state.step {
        Step::OverlayPreferences => match state.focused_option {
            0 => transitions::cycle_overlay_mode(state),
            1 => transitions::cycle_overlay_position(state),
            _ => transitions::cycle_overlay_alpha(state),
        },
        Step::Appearance => match state.focused_option {
            0 => transitions::cycle_language(state),
            1 => transitions::cycle_theme(state),
            _ => transitions::cycle_overlay_style(state),
        },
        Step::PlaybackPreferences => match state.focused_option {
            0 => transitions::toggle_autoplay_last(state),
            1 => transitions::toggle_restore_volume(state),
            2 => transitions::cycle_crossfade(state),
            3 => transitions::cycle_screensaver(state),
            _ => transitions::toggle_auto_update(state),
        },
        Step::SpotifyPreferences => match state.focused_option {
            0 => transitions::toggle_spotify_stop_on_quit(state),
            1 => transitions::toggle_spotify_start_on_spotify(state),
            2 => transitions::cycle_spotify_playback_mode(state),
            _ => transitions::toggle_spotify_radio_enabled(state),
        },
        Step::Welcome | Step::Summary => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OverlayMode;

    #[test]
    fn finish_applies_onboarding_state_without_restoring_volume() {
        let mut config = Config {
            overlay_mode: OverlayMode::WhenPlaying,
            volume: 0.8,
            ..Config::default()
        };
        let mut state = OnboardingState::from_config(&config);
        state.overlay_mode = OverlayMode::Always;
        state.volume = 0.5;
        state.volume_changed = true;

        let restore_volume = complete_outcome(Outcome::Finish, &state, &mut config, 0.8);

        assert_eq!(restore_volume, None);
        assert_eq!(config.overlay_mode, OverlayMode::Always);
        assert_eq!(config.volume, 0.5);
    }

    #[test]
    fn finish_without_volume_change_restores_runtime_volume_and_keeps_config_volume() {
        let mut config = Config {
            overlay_mode: OverlayMode::WhenPlaying,
            volume: 0.8,
            ..Config::default()
        };
        let mut state = OnboardingState::from_config(&config);
        state.overlay_mode = OverlayMode::Always;
        state.volume = WELCOME_VOLUME;

        let restore_volume = complete_outcome(Outcome::Finish, &state, &mut config, 0.8);

        assert_eq!(restore_volume, Some(0.8));
        assert_eq!(config.overlay_mode, OverlayMode::Always);
        assert_eq!(config.volume, 0.8);
    }

    #[test]
    fn skip_restores_original_volume_without_applying_state() {
        let mut config = Config {
            overlay_mode: OverlayMode::WhenPlaying,
            volume: 0.8,
            ..Config::default()
        };
        let mut state = OnboardingState::from_config(&config);
        state.overlay_mode = OverlayMode::Always;
        state.volume = 0.5;

        let restore_volume = complete_outcome(Outcome::Skip, &state, &mut config, 0.8);

        assert_eq!(restore_volume, Some(0.8));
        assert_eq!(config.overlay_mode, OverlayMode::WhenPlaying);
        assert_eq!(config.volume, 0.8);
    }
}
