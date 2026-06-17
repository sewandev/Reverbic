use super::state::{OnboardingState, Step};

pub fn next(state: &mut OnboardingState) -> bool {
    let index = state.step.position() + 1;
    match Step::ALL.get(index) {
        Some(step) => {
            state.step = *step;
            state.focused_option = 0;
            true
        }
        None => false,
    }
}

pub fn back(state: &mut OnboardingState) -> bool {
    let position = state.step.position();
    if position == 0 {
        return false;
    }
    state.step = Step::ALL[position - 1];
    state.focused_option = 0;
    true
}

pub fn focus_next_option(state: &mut OnboardingState) {
    let count = state.step.option_count();
    if count == 0 {
        return;
    }
    state.focused_option = (state.focused_option + 1) % count;
}

pub fn focus_prev_option(state: &mut OnboardingState) {
    let count = state.step.option_count();
    if count == 0 {
        return;
    }
    state.focused_option = (state.focused_option + count - 1) % count;
}

pub fn cycle_language(state: &mut OnboardingState) {
    state.language = state.language.next();
    crate::i18n::set_language(state.language);
}

pub fn next_theme(state: &mut OnboardingState) {
    state.theme = state.theme.next();
}

pub fn prev_theme(state: &mut OnboardingState) {
    state.theme = state.theme.prev();
}

pub fn cycle_overlay_mode(state: &mut OnboardingState, forward: bool) {
    state.overlay_mode = if forward {
        state.overlay_mode.next()
    } else {
        state.overlay_mode.prev()
    };
}

pub fn cycle_overlay_position(state: &mut OnboardingState, forward: bool) {
    state.overlay_position = if forward {
        state.overlay_position.next()
    } else {
        state.overlay_position.prev()
    };
}

pub fn toggle_autoplay_last(state: &mut OnboardingState) {
    state.autoplay_last = !state.autoplay_last;
}

pub fn toggle_auto_update(state: &mut OnboardingState) {
    state.auto_update = !state.auto_update;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn state() -> OnboardingState {
        OnboardingState::from_config(&Config::default())
    }

    #[test]
    fn next_advances_through_every_step_and_stops_at_last() {
        let mut state = state();
        for _ in 1..Step::ALL.len() {
            assert!(next(&mut state));
        }
        assert_eq!(state.step, Step::Setup);
        assert!(!next(&mut state));
        assert_eq!(state.step, Step::Setup);
    }

    #[test]
    fn back_returns_to_first_step_and_stops_there() {
        let mut state = state();
        next(&mut state);
        assert!(back(&mut state));
        assert_eq!(state.step, Step::Appearance);
        assert!(!back(&mut state));
    }

    #[test]
    fn theme_navigation_wraps_in_both_directions() {
        let mut state = state();
        let before = state.theme;

        next_theme(&mut state);
        assert_eq!(state.theme, before.next());

        prev_theme(&mut state);
        assert_eq!(state.theme, before);
    }

    #[test]
    fn toggles_flip_their_respective_flags() {
        let mut state = state();
        let autoplay_before = state.autoplay_last;
        let auto_update_before = state.auto_update;

        toggle_autoplay_last(&mut state);
        toggle_auto_update(&mut state);

        assert_eq!(state.autoplay_last, !autoplay_before);
        assert_eq!(state.auto_update, !auto_update_before);
    }
}
