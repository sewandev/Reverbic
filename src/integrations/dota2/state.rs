#[derive(Clone, Default, Debug)]
pub struct Dota2State {
    pub hero:           String,
    pub game_time_secs: i32,
    pub kills:          u32,
    pub deaths:         u32,
    pub assists:        u32,
    pub net_worth:      u32,
    pub in_game:        bool,
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
