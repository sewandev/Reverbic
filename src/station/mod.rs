pub mod enrichment;
pub mod on_demand;
pub mod radio_browser;
pub mod registry;

pub use enrichment::{enrich, find_enrichment};
pub use radio_browser::{
    fetch_station_details, fetch_station_details_by_name, is_uuid, search_stations,
    search_stations_by_country, search_stations_by_tag, DynamicStation, StationDetails, COUNTRIES,
    GENRES,
};
pub use registry::Station;

use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};

pub fn filter_items(
    list: &[(&'static str, &'static str)],
    filter: &str,
) -> Vec<(&'static str, &'static str)> {
    if filter.is_empty() {
        return list.to_vec();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(filter, CaseMatching::Ignore, Normalization::Smart);

    let mut scored: Vec<(u32, (&'static str, &'static str))> = list
        .iter()
        .copied()
        .filter_map(|(tag, label)| {
            let mut buf = Vec::new();
            pattern
                .score(Utf32Str::new(label, &mut buf), &mut matcher)
                .map(|score| (score, (tag, label)))
        })
        .collect();

    scored.sort_unstable_by_key(|b| std::cmp::Reverse(b.0));
    scored.into_iter().map(|(_, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &[(&str, &str)] = &[
        ("jazz", "Jazz"),
        ("classical", "Classical"),
        ("dnb", "Drum and Bass"),
        ("electronic", "Electronic"),
        ("country", "Country"),
        ("latin", "Latin"),
    ];

    #[test]
    fn empty_filter_returns_full_list() {
        assert_eq!(filter_items(SAMPLE, ""), SAMPLE.to_vec());
    }

    #[test]
    fn exact_match_returns_item() {
        let results = filter_items(SAMPLE, "Jazz");
        assert!(results.contains(&("jazz", "Jazz")));
    }

    #[test]
    fn fuzzy_subsequence_matches_classical() {
        let results = filter_items(SAMPLE, "cls");
        assert!(results.contains(&("classical", "Classical")));
    }

    #[test]
    fn fuzzy_acronym_matches_drum_and_bass() {
        let results = filter_items(SAMPLE, "dnb");
        assert!(results.contains(&("dnb", "Drum and Bass")));
    }

    #[test]
    fn no_match_returns_empty() {
        let results = filter_items(SAMPLE, "zzzqqqxxx");
        assert!(results.is_empty());
    }

    #[test]
    fn best_match_is_first() {
        let results = filter_items(SAMPLE, "jazz");
        assert_eq!(results.first(), Some(&("jazz", "Jazz")));
    }
}
