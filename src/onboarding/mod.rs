mod state;
mod transitions;
mod view;

use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures_util::StreamExt;

use crate::config::Config;
use crate::error::{AppError, Result};
use crate::i18n;
use crate::terminal::Tui;
use crate::ui::theme;

use state::{OnboardingState, Step};
use view::ViewCtx;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Continue,
    Finish,
    Skip,
}

pub async fn run(tui: &mut Tui, config: &mut Config) -> Result<()> {
    let mut state = OnboardingState::from_config(config);

    let mut events = EventStream::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    let mut border_tick: u32 = 0;

    let outcome = loop {
        draw(tui, &state, border_tick)?;

        tokio::select! {
            _ = ticker.tick() => {
                border_tick = border_tick.wrapping_add(1);
            }
            maybe_event = events.next() => {
                let Some(Ok(Event::Key(key))) = maybe_event else { continue };
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match handle_key(&mut state, key.code) {
                    Outcome::Continue => {}
                    outcome @ (Outcome::Finish | Outcome::Skip) => break outcome,
                }
            }
        }
    };

    complete_outcome(outcome, &state, config);
    config.save();
    Ok(())
}

fn complete_outcome(outcome: Outcome, state: &OnboardingState, config: &mut Config) {
    match outcome {
        Outcome::Finish => state.apply_to(config),
        Outcome::Skip => i18n::set_language(config.language),
        Outcome::Continue => {}
    }
}

fn draw(tui: &mut Tui, state: &OnboardingState, border_tick: u32) -> Result<()> {
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
            },
        );
    })
    .map_err(|e| AppError::Terminal(e.to_string()))?;
    Ok(())
}

fn handle_key(state: &mut OnboardingState, code: KeyCode) -> Outcome {
    if state.theme_picker_open {
        handle_theme_picker_key(state, code);
        return Outcome::Continue;
    }

    match code {
        KeyCode::Esc => return Outcome::Skip,
        KeyCode::Tab => return advance(state),
        KeyCode::BackTab => {
            transitions::back(state);
        }
        KeyCode::Enter => {
            if focused_on_theme(state) {
                open_theme_picker(state);
            } else {
                return advance(state);
            }
        }
        KeyCode::Up => transitions::focus_prev_option(state),
        KeyCode::Down => transitions::focus_next_option(state),
        KeyCode::Left => change_focused_option(state, false),
        KeyCode::Right => change_focused_option(state, true),
        _ => {}
    }
    Outcome::Continue
}

fn focused_on_theme(state: &OnboardingState) -> bool {
    state.step == Step::Appearance && state.focused_option == 1
}

fn open_theme_picker(state: &mut OnboardingState) {
    state.theme_before_picker = state.theme;
    state.theme_picker_open = true;
}

fn handle_theme_picker_key(state: &mut OnboardingState, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            state.theme = state.theme_before_picker;
            state.theme_picker_open = false;
        }
        KeyCode::Up | KeyCode::Char('k') => transitions::prev_theme(state),
        KeyCode::Down | KeyCode::Char('j') => transitions::next_theme(state),
        KeyCode::Enter => state.theme_picker_open = false,
        _ => {}
    }
}

fn advance(state: &mut OnboardingState) -> Outcome {
    if !transitions::next(state) {
        Outcome::Finish
    } else {
        Outcome::Continue
    }
}

fn change_focused_option(state: &mut OnboardingState, forward: bool) {
    match state.step {
        Step::Appearance => match state.focused_option {
            0 => transitions::cycle_language(state),
            _ => {
                if forward {
                    transitions::next_theme(state);
                } else {
                    transitions::prev_theme(state);
                }
            }
        },
        Step::Setup => change_setup_option(state, forward),
    }
}

fn change_setup_option(state: &mut OnboardingState, forward: bool) {
    if cfg!(target_os = "windows") {
        match state.focused_option {
            0 => transitions::cycle_overlay_mode(state, forward),
            1 => transitions::cycle_overlay_position(state, forward),
            2 => transitions::toggle_autoplay_last(state),
            _ => transitions::toggle_auto_update(state),
        }
    } else {
        match state.focused_option {
            0 => transitions::toggle_autoplay_last(state),
            _ => transitions::toggle_auto_update(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OverlayMode;
    use crate::i18n::Language;

    struct LanguageReset(Language);

    impl Drop for LanguageReset {
        fn drop(&mut self) {
            i18n::set_language(self.0);
        }
    }

    #[test]
    fn finish_applies_onboarding_state() {
        let mut config = Config {
            overlay_mode: OverlayMode::WhenPlaying,
            ..Config::default()
        };
        let mut state = OnboardingState::from_config(&config);
        state.overlay_mode = OverlayMode::Always;

        complete_outcome(Outcome::Finish, &state, &mut config);

        assert_eq!(config.overlay_mode, OverlayMode::Always);
    }

    #[test]
    fn skip_does_not_apply_state() {
        let mut config = Config {
            overlay_mode: OverlayMode::WhenPlaying,
            ..Config::default()
        };
        let mut state = OnboardingState::from_config(&config);
        state.overlay_mode = OverlayMode::Always;

        complete_outcome(Outcome::Skip, &state, &mut config);

        assert_eq!(config.overlay_mode, OverlayMode::WhenPlaying);
    }

    #[test]
    fn skip_restores_previewed_language_without_applying_state() {
        let _reset_language = LanguageReset(i18n::current_language());
        let mut config = Config {
            language: Language::Es,
            ..Config::default()
        };
        let mut state = OnboardingState::from_config(&config);

        transitions::cycle_language(&mut state);
        assert_eq!(state.language, Language::En);
        assert_eq!(i18n::current_language(), Language::En);

        complete_outcome(Outcome::Skip, &state, &mut config);

        assert_eq!(config.language, Language::Es);
        assert_eq!(i18n::current_language(), Language::Es);
    }
}
