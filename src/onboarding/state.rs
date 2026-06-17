use crate::config::{Config, OverlayMode, OverlayPosition, ThemeId};
use crate::i18n::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Appearance,
    Setup,
}

impl Step {
    pub const ALL: [Step; 2] = [Step::Appearance, Step::Setup];

    pub fn position(self) -> usize {
        Self::ALL
            .iter()
            .position(|step| *step == self)
            .expect("Step::ALL must list every Step variant")
    }

    pub fn option_count(self) -> usize {
        match self {
            Step::Appearance => 2,
            Step::Setup => {
                if cfg!(target_os = "windows") {
                    4
                } else {
                    2
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OnboardingState {
    pub step: Step,
    pub focused_option: usize,
    pub language: Language,
    pub theme: ThemeId,
    pub theme_picker_open: bool,
    pub theme_before_picker: ThemeId,
    pub overlay_mode: OverlayMode,
    pub overlay_position: OverlayPosition,
    pub autoplay_last: bool,
    pub auto_update: bool,
}

impl OnboardingState {
    pub fn from_config(config: &Config) -> Self {
        Self {
            step: Step::Appearance,
            focused_option: 0,
            language: config.language,
            theme: config.theme,
            theme_picker_open: false,
            theme_before_picker: config.theme,
            overlay_mode: config.overlay_mode,
            overlay_position: config.overlay_position,
            autoplay_last: config.autoplay_last,
            auto_update: config.auto_update,
        }
    }

    pub fn apply_to(&self, config: &mut Config) {
        config.language = self.language;
        config.theme = self.theme;
        config.overlay_mode = self.overlay_mode;
        config.overlay_position = self.overlay_position;
        config.autoplay_last = self.autoplay_last;
        config.auto_update = self.auto_update;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_position_matches_order_in_all() {
        assert_eq!(Step::Appearance.position(), 0);
        assert_eq!(Step::Setup.position(), Step::ALL.len() - 1);
    }

    #[test]
    fn from_config_seeds_selections_from_existing_config() {
        let config = Config {
            theme: ThemeId::Reverbic,
            overlay_mode: OverlayMode::Always,
            autoplay_last: true,
            ..Config::default()
        };

        let state = OnboardingState::from_config(&config);

        assert_eq!(state.theme, ThemeId::Reverbic);
        assert_eq!(state.overlay_mode, OverlayMode::Always);
        assert!(state.autoplay_last);
    }

    #[test]
    fn apply_to_writes_selections_back_into_config() {
        let mut state = OnboardingState::from_config(&Config::default());
        state.theme = ThemeId::Reverbic;
        state.overlay_mode = OverlayMode::Hidden;
        state.autoplay_last = true;

        let mut config = Config::default();
        state.apply_to(&mut config);

        assert_eq!(config.theme, ThemeId::Reverbic);
        assert_eq!(config.overlay_mode, OverlayMode::Hidden);
        assert!(config.autoplay_last);
    }
}
