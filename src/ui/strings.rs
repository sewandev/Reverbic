pub fn truncate(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if s.chars().count() <= max_chars {
        s.to_owned()
    } else {
        format!(
            "{}…",
            s.chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        )
    }
}

pub fn screensaver_display(secs: u16) -> String {
    match secs {
        0 => "OFF".to_string(),
        s if s < 60 => format!("{}s", s),
        s => format!("{}m", s / 60),
    }
}

pub fn crossfade_display(secs: u8) -> String {
    use crate::i18n::t;
    match secs {
        0 => t("crossfade.off"),
        1 => t("crossfade.1s"),
        3 => t("crossfade.3s"),
        5 => t("crossfade.5s"),
        _ => t("crossfade.7s"),
    }
}

pub fn spotify_crossfade_display(secs: u8) -> String {
    use crate::i18n::t;
    match secs {
        0 => t("crossfade.off"),
        1 => t("crossfade.1s"),
        3 => t("crossfade.3s"),
        5 => t("crossfade.5s"),
        7 => t("crossfade.7s"),
        10 => t("crossfade.10s"),
        _ => t("crossfade.12s"),
    }
}

pub fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
