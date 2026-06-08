use crate::config::{Config, OverlayMode, OverlayPosition, OverlayStyle, ThemeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Welcome,
    Appearance,
    OverlayPreferences,
    PlaybackPreferences,
    Summary,
}

impl Step {
    pub const ALL: [Step; 5] = [
        Step::Welcome,
        Step::Appearance,
        Step::OverlayPreferences,
        Step::PlaybackPreferences,
        Step::Summary,
    ];

    pub fn position(self) -> usize {
        Self::ALL
            .iter()
            .position(|step| *step == self)
            .expect("Step::ALL must list every Step variant")
    }
    pub fn option_count(self) -> usize {
        match self {
            Step::Welcome | Step::Summary => 0,
            Step::Appearance => 2,
            Step::OverlayPreferences => 3,
            Step::PlaybackPreferences => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OnboardingState {
    pub step: Step,
    pub focused_option: usize,
    pub theme: ThemeId,
    pub overlay_style: OverlayStyle,
    pub overlay_mode: OverlayMode,
    pub overlay_position: OverlayPosition,
    pub overlay_alpha: u8,
    pub volume: f32,
    pub autoplay_last: bool,
    pub restore_volume: bool,
    pub crossfade_secs: u8,
    pub screensaver_secs: u16,
    pub auto_update: bool,
    pub muted: bool,
}

impl OnboardingState {
    pub fn from_config(config: &Config) -> Self {
        Self {
            step: Step::Welcome,
            focused_option: 0,
            theme: config.theme,
            overlay_style: config.overlay_style,
            overlay_mode: config.overlay_mode,
            overlay_position: config.overlay_position,
            overlay_alpha: config.overlay_alpha,
            volume: config.volume,
            autoplay_last: config.autoplay_last,
            restore_volume: config.restore_volume,
            crossfade_secs: config.crossfade_secs,
            screensaver_secs: config.screensaver_secs,
            auto_update: config.auto_update,
            muted: false,
        }
    }

    pub fn apply_to(&self, config: &mut Config) {
        config.theme = self.theme;
        config.overlay_style = self.overlay_style;
        config.overlay_mode = self.overlay_mode;
        config.overlay_position = self.overlay_position;
        config.overlay_alpha = self.overlay_alpha;
        config.volume = self.volume;
        config.autoplay_last = self.autoplay_last;
        config.restore_volume = self.restore_volume;
        config.crossfade_secs = self.crossfade_secs;
        config.screensaver_secs = self.screensaver_secs;
        config.auto_update = self.auto_update;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_position_matches_order_in_all() {
        assert_eq!(Step::Welcome.position(), 0);
        assert_eq!(Step::Summary.position(), Step::ALL.len() - 1);
    }

    #[test]
    fn from_config_seeds_selections_from_existing_config() {
        let config = Config {
            theme: ThemeId::Reverbic,
            overlay_style: OverlayStyle::Compact,
            overlay_mode: OverlayMode::Always,
            autoplay_last: true,
            ..Config::default()
        };

        let state = OnboardingState::from_config(&config);

        assert_eq!(state.theme, ThemeId::Reverbic);
        assert_eq!(state.overlay_style, OverlayStyle::Compact);
        assert_eq!(state.overlay_mode, OverlayMode::Always);
        assert!(state.autoplay_last);
        assert!(!state.muted);
    }

    #[test]
    fn apply_to_writes_selections_back_into_config() {
        let mut state = OnboardingState::from_config(&Config::default());
        state.theme = ThemeId::Reverbic;
        state.overlay_style = OverlayStyle::Compact;
        state.overlay_mode = OverlayMode::Hidden;
        state.restore_volume = true;

        let mut config = Config::default();
        state.apply_to(&mut config);

        assert_eq!(config.theme, ThemeId::Reverbic);
        assert_eq!(config.overlay_style, OverlayStyle::Compact);
        assert_eq!(config.overlay_mode, OverlayMode::Hidden);
        assert!(config.restore_volume);
    }
}
