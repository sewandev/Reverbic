pub fn parse_icy_title(raw: &str) -> Option<String> {
    let key = "StreamTitle='";
    let start = raw.find(key)? + key.len();
    let rest = &raw[start..];
    let end = rest.find("';")?;
    let title = &rest[..end];
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_title_standard() {
        let raw = "StreamTitle='Armin van Buuren - Blah Blah';StreamUrl='';";
        assert_eq!(parse_icy_title(raw), Some("Armin van Buuren - Blah Blah".to_string()));
    }

    #[test]
    fn parse_title_empty() {
        let raw = "StreamTitle='';StreamUrl='';";
        assert_eq!(parse_icy_title(raw), None);
    }

    #[test]
    fn parse_title_missing() {
        assert_eq!(parse_icy_title(""), None);
    }
}
