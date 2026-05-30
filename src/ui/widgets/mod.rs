pub mod countdown;
pub mod local_time;
pub mod now_playing;
pub mod now_playing_overlay;
pub mod on_demand_panel;
pub mod recent_tracks;
pub mod saved_tracks;
pub mod settings_panel;
pub mod station_list;
pub mod vu_meter;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const SPINNER_FRAME_MS: u128 = 120;

pub(crate) fn spinner_frame() -> &'static str {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    SPINNER_FRAMES[((ms / SPINNER_FRAME_MS) as usize) % SPINNER_FRAMES.len()]
}
