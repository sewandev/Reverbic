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

pub fn wrapped_line_count(text: &str, width: u16, max: u16) -> u16 {
    if width == 0 || max == 0 {
        return 0;
    }
    let width = width as usize;
    let mut rows: u16 = 0;
    let mut current: usize = 0;
    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        if current == 0 {
            current = word_len;
        } else if current + 1 + word_len <= width {
            current += 1 + word_len;
        } else {
            rows = rows.saturating_add(1);
            current = word_len;
        }
        while current > width {
            rows = rows.saturating_add(1);
            current -= width;
        }
    }
    if current > 0 {
        rows = rows.saturating_add(1);
    }
    rows.clamp(1, max)
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

pub fn group_thousands(n: u32) -> String {
    let digits = n.to_string();
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let len = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(*b as char);
    }
    out
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
