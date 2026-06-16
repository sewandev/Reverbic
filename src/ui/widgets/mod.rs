pub mod ambient_picker;
pub mod clock;
pub mod controls;
pub mod logo;
pub mod progress;
pub mod recent_tracks;
pub mod search_modal;
pub mod spotify_profile;
pub mod station_details;
pub mod theme_picker;
pub mod visualizer;

pub(crate) fn keep_selected_visible(scroll_offset: &mut usize, selected: usize, visible: usize) {
    if visible == 0 {
        *scroll_offset = 0;
    } else if selected < *scroll_offset {
        *scroll_offset = selected;
    } else if selected >= scroll_offset.saturating_add(visible) {
        *scroll_offset = selected + 1 - visible;
    }
}

pub(crate) fn scroll_offset_for_selection(
    selected: usize,
    visible: usize,
    scroll_offset: usize,
) -> usize {
    let mut scroll_offset = scroll_offset;
    keep_selected_visible(&mut scroll_offset, selected, visible);
    scroll_offset
}

#[cfg(test)]
mod tests {
    use super::{keep_selected_visible, scroll_offset_for_selection};

    #[test]
    fn keep_selected_visible_preserves_window_until_selection_leaves_it() {
        let mut offset = 14;

        keep_selected_visible(&mut offset, 19, 7);
        assert_eq!(offset, 14);
    }

    #[test]
    fn keep_selected_visible_moves_window_when_selection_crosses_top() {
        let mut offset = 14;

        keep_selected_visible(&mut offset, 13, 7);
        assert_eq!(offset, 13);
    }

    #[test]
    fn keep_selected_visible_moves_window_when_selection_crosses_bottom() {
        let mut offset = 14;

        keep_selected_visible(&mut offset, 21, 7);
        assert_eq!(offset, 15);
    }

    #[test]
    fn keep_selected_visible_resets_offset_when_nothing_is_visible() {
        let mut offset = 14;

        keep_selected_visible(&mut offset, 21, 0);
        assert_eq!(offset, 0);
    }

    #[test]
    fn keep_selected_visible_handles_wrapping_between_first_and_last_items() {
        let mut offset = 0;

        keep_selected_visible(&mut offset, 99, 7);
        assert_eq!(offset, 93);

        keep_selected_visible(&mut offset, 0, 7);
        assert_eq!(offset, 0);
    }

    #[test]
    fn scroll_offset_for_selection_recovers_from_stale_filtered_list_offset() {
        assert_eq!(scroll_offset_for_selection(0, 5, 20), 0);
        assert_eq!(scroll_offset_for_selection(3, 5, 20), 3);
    }
}
