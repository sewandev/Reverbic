pub mod enrichment;
pub mod on_demand;
pub mod radio_browser;
pub mod registry;

pub use enrichment::{enrich, find_enrichment};
pub use radio_browser::{is_uuid, search_stations, search_stations_by_tag, search_stations_by_country, fetch_station_details, fetch_station_details_by_name, DynamicStation, StationDetails, GENRES, COUNTRIES};
pub use registry::Station;

pub fn filter_items(
    list: &[(&'static str, &'static str)],
    filter: &str,
) -> Vec<(&'static str, &'static str)> {
    let f = filter.to_lowercase();
    list.iter()
        .copied()
        .filter(|(_, label)| f.is_empty() || label.to_lowercase().contains(&f))
        .collect()
}
