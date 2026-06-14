use crate::config::{
    Config, OverlayMode, OverlayPosition, OverlayStyle, SpotifyPlaybackMode, ThemeId,
};
use crate::i18n::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Welcome,
    Appearance,
    OverlayPreferences,
    PlaybackPreferences,
    SpotifyPreferences,
    Summary,
}

impl Step {
    pub const ALL: [Step; 6] = [
        Step::Welcome,
        Step::Appearance,
        Step::OverlayPreferences,
        Step::PlaybackPreferences,
        Step::SpotifyPreferences,
        Step::Summary,
    ];

    pub fn position(self) -> usize {
        Self::ALL
            .iter()
            .position(|step| *step == self)
            .expect("Step::ALL must list every Step variant")
    }

    #[cfg(target_os = "windows")]
    pub fn is_enabled(self) -> bool {
        true
    }

    #[cfg(not(target_os = "windows"))]
    pub fn is_enabled(self) -> bool {
        !matches!(self, Step::OverlayPreferences)
    }

    pub fn enabled_count() -> usize {
        Self::ALL.iter().filter(|step| step.is_enabled()).count()
    }

    pub fn enabled_position(self) -> usize {
        Self::ALL
            .iter()
            .filter(|step| step.is_enabled())
            .position(|step| *step == self)
            .unwrap_or(0)
    }

    pub fn option_count(self) -> usize {
        match self {
            Step::Welcome | Step::Summary => 0,
            Step::Appearance => {
                if cfg!(target_os = "windows") {
                    3
                } else {
                    2
                }
            }
            Step::OverlayPreferences => 3,
            Step::PlaybackPreferences => 5,
            Step::SpotifyPreferences => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OnboardingState {
    pub step: Step,
    pub focused_option: usize,
    pub language: Language,
    pub theme: ThemeId,
    pub overlay_style: OverlayStyle,
    pub overlay_mode: OverlayMode,
    pub overlay_position: OverlayPosition,
    pub overlay_alpha: u8,
    pub volume: f32,
    pub volume_changed: bool,
    pub autoplay_last: bool,
    pub restore_volume: bool,
    pub crossfade_secs: u8,
    pub screensaver_secs: u16,
    pub auto_update: bool,
    pub muted: bool,
    pub spotify_stop_on_quit: bool,
    pub spotify_start_on_spotify: bool,
    pub spotify_playback_mode: SpotifyPlaybackMode,
    pub spotify_radio_enabled: bool,
}

impl OnboardingState {
    pub fn from_config(config: &Config) -> Self {
        Self {
            step: Step::Welcome,
            focused_option: 0,
            language: config.language,
            theme: config.theme,
            overlay_style: config.overlay_style,
            overlay_mode: config.overlay_mode,
            overlay_position: config.overlay_position,
            overlay_alpha: config.overlay_alpha,
            volume: config.volume,
            volume_changed: false,
            autoplay_last: config.autoplay_last,
            restore_volume: config.restore_volume,
            crossfade_secs: config.crossfade_secs,
            screensaver_secs: config.screensaver_secs,
            auto_update: config.auto_update,
            muted: false,
            spotify_stop_on_quit: config.spotify.stop_on_quit,
            spotify_start_on_spotify: config.spotify.start_on_spotify,
            spotify_playback_mode: config.spotify.playback_mode,
            spotify_radio_enabled: config.spotify.radio_enabled,
        }
    }

    pub fn apply_to(&self, config: &mut Config) {
        config.language = self.language;
        config.theme = self.theme;
        config.overlay_style = self.overlay_style;
        config.overlay_mode = self.overlay_mode;
        config.overlay_position = self.overlay_position;
        config.overlay_alpha = self.overlay_alpha;
        if self.volume_changed {
            config.volume = self.volume;
        }
        config.autoplay_last = self.autoplay_last;
        config.restore_volume = self.restore_volume;
        config.crossfade_secs = self.crossfade_secs;
        config.screensaver_secs = self.screensaver_secs;
        config.auto_update = self.auto_update;
        config.spotify.stop_on_quit = self.spotify_stop_on_quit;
        config.spotify.start_on_spotify = self.spotify_start_on_spotify;
        config.spotify.playback_mode = self.spotify_playback_mode;
        config.spotify.radio_enabled = self.spotify_radio_enabled;
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

    #[test]
    fn apply_to_only_writes_volume_after_explicit_change() {
        let mut state = OnboardingState::from_config(&Config::default());
        state.volume = 0.5;

        let mut config = Config {
            volume: 0.8,
            ..Config::default()
        };
        state.apply_to(&mut config);

        assert_eq!(config.volume, 0.8);

        state.volume_changed = true;
        state.apply_to(&mut config);

        assert_eq!(config.volume, 0.5);
    }
}
