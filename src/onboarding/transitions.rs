use super::state::{OnboardingState, Step};

pub fn next(state: &mut OnboardingState) -> bool {
    match Step::ALL.get(state.step.position() + 1) {
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

pub fn cycle_overlay_mode(state: &mut OnboardingState) {
    state.overlay_mode = state.overlay_mode.next();
}

pub fn cycle_theme(state: &mut OnboardingState) {
    state.theme = state.theme.next();
}

pub fn cycle_overlay_style(state: &mut OnboardingState) {
    state.overlay_style = state.overlay_style.next();
}

pub fn cycle_overlay_position(state: &mut OnboardingState) {
    state.overlay_position = state.overlay_position.next();
}

pub fn cycle_overlay_alpha(state: &mut OnboardingState) {
    state.overlay_alpha = match state.overlay_alpha {
        v if v < 30 => 30,
        v if v < 50 => 50,
        v if v < 70 => 70,
        v if v < 90 => 90,
        _ => 20,
    };
}

pub fn adjust_volume(state: &mut OnboardingState, delta: f32) {
    state.volume = (state.volume + delta).clamp(0.0, 1.0);
    state.volume_changed = true;
}

pub fn toggle_autoplay_last(state: &mut OnboardingState) {
    state.autoplay_last = !state.autoplay_last;
}

pub fn toggle_restore_volume(state: &mut OnboardingState) {
    state.restore_volume = !state.restore_volume;
}

pub fn toggle_auto_update(state: &mut OnboardingState) {
    state.auto_update = !state.auto_update;
}

pub fn cycle_crossfade(state: &mut OnboardingState) {
    state.crossfade_secs = match state.crossfade_secs {
        0 => 1,
        1 => 2,
        2 => 3,
        _ => 0,
    };
}

pub fn cycle_screensaver(state: &mut OnboardingState) {
    state.screensaver_secs = match state.screensaver_secs {
        0 => 10,
        10 => 20,
        20 => 30,
        30 => 60,
        60 => 120,
        120 => 300,
        _ => 0,
    };
}

pub fn toggle_muted(state: &mut OnboardingState) {
    state.muted = !state.muted;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn state() -> OnboardingState {
        OnboardingState::from_config(&Config::default())
    }

    #[test]
    fn next_advances_through_every_step_and_stops_at_summary() {
        let mut state = state();
        for _ in 1..Step::ALL.len() {
            assert!(next(&mut state));
        }
        assert_eq!(state.step, Step::Summary);
        assert!(!next(&mut state));
        assert_eq!(state.step, Step::Summary);
    }

    #[test]
    fn back_returns_to_welcome_and_stops_there() {
        let mut state = state();
        next(&mut state);
        assert!(back(&mut state));
        assert_eq!(state.step, Step::Welcome);
        assert!(!back(&mut state));
    }

    #[test]
    fn cycle_overlay_alpha_wraps_through_known_steps() {
        let mut state = state();
        state.overlay_alpha = 90;
        cycle_overlay_alpha(&mut state);
        assert_eq!(state.overlay_alpha, 20);
        cycle_overlay_alpha(&mut state);
        assert_eq!(state.overlay_alpha, 30);
    }

    #[test]
    fn cycle_theme_uses_theme_infrastructure() {
        let mut state = state();
        let before = state.theme;

        cycle_theme(&mut state);

        assert_eq!(state.theme, before.next());
    }

    #[test]
    fn cycle_overlay_style_uses_overlay_style_infrastructure() {
        let mut state = state();
        let before = state.overlay_style;

        cycle_overlay_style(&mut state);

        assert_eq!(state.overlay_style, before.next());
    }

    #[test]
    fn toggles_flip_their_respective_flags() {
        let mut state = state();
        let autoplay_before = state.autoplay_last;
        let restore_before = state.restore_volume;
        let auto_update_before = state.auto_update;
        let muted_before = state.muted;

        toggle_autoplay_last(&mut state);
        toggle_restore_volume(&mut state);
        toggle_auto_update(&mut state);
        toggle_muted(&mut state);

        assert_eq!(state.autoplay_last, !autoplay_before);
        assert_eq!(state.restore_volume, !restore_before);
        assert_eq!(state.auto_update, !auto_update_before);
        assert_eq!(state.muted, !muted_before);
    }

    #[test]
    fn adjust_volume_marks_volume_as_explicitly_changed() {
        let mut state = state();

        adjust_volume(&mut state, -0.1);

        assert!(state.volume_changed);
    }
}
