#![cfg(target_os = "windows")]

use crate::audio::{PlayerState, PlayerStatus};

pub struct DiscordActivity {
    pub details: String,
    pub state: Option<String>,
    pub include_timestamp: bool,
}

impl DiscordActivity {
    pub fn to_json(&self, start_timestamp: Option<u64>) -> String {
        let mut parts = vec![format!(
            r#""details":"{}""#,
            escape_json(&self.details)
        )];

        if let Some(ref s) = self.state {
            parts.push(format!(r#""state":"{}""#, escape_json(s)));
        }

        if self.include_timestamp {
            if let Some(ts) = start_timestamp {
                parts.push(format!(r#""timestamps":{{"start":{}}}"#, ts));
            }
        }

        parts.push(
            r#""assets":{"large_image":"reverbic","large_text":"Reverbic"}"#.to_string(),
        );

        format!("{{{}}}", parts.join(","))
    }
}

pub fn build(state: &PlayerState) -> Option<DiscordActivity> {
    match state.status {
        PlayerStatus::Playing
        | PlayerStatus::Buffering(_)
        | PlayerStatus::Reconnecting(_)
        | PlayerStatus::Connecting => {
            let station = state.station.as_ref()?;
            Some(DiscordActivity {
                details: station.name.clone(),
                state: state.title.clone(),
                include_timestamp: true,
            })
        }
        PlayerStatus::Paused => {
            let station = state.station.as_ref()?;
            Some(DiscordActivity {
                details: station.name.clone(),
                state: Some("Paused".to_string()),
                include_timestamp: false,
            })
        }
        PlayerStatus::Idle | PlayerStatus::Error(_) => None,
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', r"\\")
        .replace('"', r#"\""#)
        .replace('\n', r"\n")
        .replace('\r', r"\r")
}
