//! Turns a user query into a concrete radio [`Station`].
//!
//! Resolution is offline-first and deterministic so the same command always
//! plays what the user expects:
//!   1. No query -> resume the last played station.
//!   2. Fuzzy match against the user's favorites (instant, no network).
//!   3. Fall back to an online Radio Browser search, taking the top result.

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config as MatcherConfig, Matcher, Utf32Str};

use crate::config::Config;
use crate::station::Station;

pub async fn resolve(query: Option<&str>, config: &Config) -> Result<Station, String> {
    match query.map(str::trim).filter(|q| !q.is_empty()) {
        None => config
            .last_station
            .as_ref()
            .map(|s| s.to_station())
            .ok_or_else(|| "No previous station to resume. Try: reverbic play <name>".to_string()),
        Some(query) => {
            if let Some(station) = best_favorite(query) {
                return Ok(station);
            }
            match crate::station::search_stations(query, 1).await {
                Some(results) => results
                    .first()
                    .map(|ds| ds.to_station())
                    .ok_or_else(|| format!("No station found for '{query}'.")),
                None => Err("Station search is unavailable. Check your connection.".to_string()),
            }
        }
    }
}

fn best_favorite(query: &str) -> Option<Station> {
    let favorites = crate::favorites::load();
    if favorites.is_empty() {
        return None;
    }

    let mut matcher = Matcher::new(MatcherConfig::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

    let mut best: Option<(u32, usize)> = None;
    for (index, favorite) in favorites.iter().enumerate() {
        let mut buf = Vec::new();
        if let Some(score) = pattern.score(Utf32Str::new(&favorite.name, &mut buf), &mut matcher) {
            if best.is_none_or(|(top, _)| score > top) {
                best = Some((score, index));
            }
        }
    }

    best.map(|(_, index)| favorites[index].to_station())
}
