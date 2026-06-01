use crate::i18n::t;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum DotaPhase {
    #[default]
    None,
    Loading,
    HeroSelection,
    StrategyTime,
    PreGame,
    InGame,
    PostGame,
}

impl DotaPhase {
    pub(super) fn from_str(s: &str) -> Self {
        match s {
            "DOTA_GAMERULES_STATE_WAIT_FOR_PLAYERS_TO_LOAD" => Self::Loading,
            "DOTA_GAMERULES_STATE_HERO_SELECTION"           => Self::HeroSelection,
            "DOTA_GAMERULES_STATE_STRATEGY_TIME"            => Self::StrategyTime,
            "DOTA_GAMERULES_STATE_PRE_GAME"                 => Self::PreGame,
            "DOTA_GAMERULES_STATE_GAME_IN_PROGRESS"         => Self::InGame,
            "DOTA_GAMERULES_STATE_POST_GAME"                => Self::PostGame,
            _                                               => Self::None,
        }
    }

    pub fn label(&self) -> Option<String> {
        match self {
            Self::None | Self::InGame => None,
            Self::Loading       => Some(t("dota.phase.loading")),
            Self::HeroSelection => Some(t("dota.phase.hero_selection")),
            Self::StrategyTime  => Some(t("dota.phase.strategy")),
            Self::PreGame       => Some(t("dota.phase.pregame")),
            Self::PostGame      => Some(t("dota.phase.finished")),
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

}

#[derive(Clone, Default, Debug)]
pub struct Dota2State {
    pub phase:          DotaPhase,
    pub hero:           String,
    pub team:           String,
    pub game_time_secs: i32,
    pub kills:          u32,
    pub deaths:         u32,
    pub assists:        u32,
    pub net_worth:      u32,
}

impl Dota2State {
    pub fn time_display(&self) -> String {
        let abs  = self.game_time_secs.unsigned_abs();
        let sign = if self.game_time_secs < 0 { "-" } else { "" };
        format!("{sign}{}:{:02}", abs / 60, abs % 60)
    }

    pub fn kda(&self) -> String {
        format!("{}/{}/{}", self.kills, self.deaths, self.assists)
    }

    pub fn gold(&self) -> String {
        if self.net_worth >= 1000 {
            format!("{:.1}k", self.net_worth as f32 / 1000.0)
        } else {
            self.net_worth.to_string()
        }
    }

    pub fn team_display(&self) -> &str {
        match self.team.as_str() {
            "radiant" => "Radiant",
            "dire"    => "Dire",
            _         => "",
        }
    }
    pub fn display_parts(&self) -> Vec<String> {
        match self.phase {
            DotaPhase::None => vec![],

            DotaPhase::InGame => {
                let mut v = vec![];
                if !self.hero.is_empty() { v.push(self.hero.clone()); }
                v.push(self.kda());
                v.push(self.time_display());
                v.push(self.gold());
                v
            }

            DotaPhase::PreGame => {
                let mut v = vec![];
                if !self.hero.is_empty() { v.push(self.hero.clone()); }
                v.push(t("dota.phase.pregame"));
                v.push(self.time_display());
                if !self.team_display().is_empty() { v.push(self.team_display().to_string()); }
                v
            }

            DotaPhase::PostGame => {
                let mut v = vec![];
                if !self.hero.is_empty() { v.push(self.hero.clone()); }
                v.push(t("dota.phase.finished"));
                v.push(self.kda());
                v
            }

            _ => {
                let mut v = vec![];
                if let Some(label) = self.phase.label() { v.push(label); }
                if !self.hero.is_empty() { v.push(self.hero.clone()); }
                if !self.team_display().is_empty() { v.push(self.team_display().to_string()); }
                v
            }
        }
    }
}

pub(super) fn hero_display(raw: &str) -> String {
    raw.strip_prefix("npc_dota_hero_")
        .unwrap_or(raw)
        .split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None    => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
