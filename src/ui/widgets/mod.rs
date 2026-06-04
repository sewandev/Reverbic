pub mod search_modal;

pub(crate) fn scroll_offset(selected: usize, visible: usize) -> usize {
    if selected >= visible { selected + 1 - visible } else { 0 }
}
