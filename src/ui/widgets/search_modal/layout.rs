use ratatui::layout::{Constraint, Layout, Rect};

pub(crate) const MODAL_MIN_WIDTH: u16 = 52;
pub(crate) const MODAL_MIN_HEIGHT: u16 = 14;

const MODAL_MAX_WIDTH: u16 = 120;
const MODAL_MAX_HEIGHT: u16 = 30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ModalLayout {
    pub panel: Rect,
    pub inner: Rect,
    pub tabs: Rect,
    pub body: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ListItemHeight {
    OneLine,
    TwoLines,
}

impl ListItemHeight {
    fn rows(self) -> usize {
        match self {
            Self::OneLine => 1,
            Self::TwoLines => 2,
        }
    }
}

pub(crate) fn modal_rect(area: Rect) -> Rect {
    let w = (area.width * 78 / 100)
        .clamp(MODAL_MIN_WIDTH, MODAL_MAX_WIDTH)
        .min(area.width);
    let h = (area.height * 75 / 100)
        .clamp(MODAL_MIN_HEIGHT, MODAL_MAX_HEIGHT)
        .min(area.height);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    Rect::new(x, y, w, h)
}

pub(crate) fn modal_layout(area: Rect) -> Option<ModalLayout> {
    if area.width < MODAL_MIN_WIDTH || area.height < MODAL_MIN_HEIGHT {
        return None;
    }

    let panel = modal_rect(area);
    let inner = Rect::new(
        panel.x.saturating_add(1),
        panel.y.saturating_add(1),
        panel.width.saturating_sub(2),
        panel.height.saturating_sub(2),
    );
    let [tabs, body] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);

    Some(ModalLayout {
        panel,
        inner,
        tabs,
        body,
    })
}

pub(crate) fn modal_body_area(area: Rect) -> Option<Rect> {
    modal_layout(area).map(|layout| layout.body)
}

pub(crate) fn visible_items(area: Option<Rect>, item_height: ListItemHeight) -> usize {
    area.map(|area| area.height as usize / item_height.rows())
        .unwrap_or(0)
}

pub(crate) fn visible_rows(area: Option<Rect>, reserved_rows: u16) -> usize {
    area.map(|area| area.height.saturating_sub(reserved_rows) as usize)
        .unwrap_or(0)
}

pub(crate) fn radio_search_results_list_area(area: Rect) -> Option<Rect> {
    radio_name_body_area(area).map(|area| {
        let [_gap, _input_row, _cap_row, list_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);
        list_area
    })
}

pub(crate) fn radio_favorites_list_area(area: Rect) -> Option<Rect> {
    radio_name_body_area(area).map(|area| {
        let [_gap, list_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        list_area
    })
}

pub(crate) fn radio_filter_list_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    filter_list_area(body)
}

pub(crate) fn radio_filtered_results_list_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    let [_header_row, list_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(body);
    Some(list_area)
}

pub(crate) fn settings_items_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    let [_gap, items_area, _tooltip_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .areas(body);
    Some(items_area)
}

pub(crate) fn settings_visible_rows(area: Rect) -> usize {
    settings_items_area(area)
        .map(|area| area.height.saturating_sub(1) as usize)
        .unwrap_or(0)
}

pub(crate) fn spotify_body_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    let [_gap, _subtab_row, _body_gap, body, _footer_row] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(body);
    Some(body)
}

pub(crate) fn spotify_search_list_area(area: Rect) -> Option<Rect> {
    spotify_body_area(area).map(|area| {
        let [_input_row, _cap_row, list_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);
        list_area
    })
}

pub(crate) fn spotify_titled_track_list_area(area: Rect) -> Option<Rect> {
    spotify_body_area(area).map(|area| {
        Rect::new(
            area.x,
            area.y.saturating_add(1),
            area.width,
            area.height.saturating_sub(1),
        )
    })
}

pub(crate) fn youtube_list_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    filter_list_area(body)
}

fn radio_name_body_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    let [_gap, _subtab_row, body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(body);
    Some(body)
}

fn filter_list_area(area: Rect) -> Option<Rect> {
    let [_gap, _input_row, _cap_row, list_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);
    Some(list_area)
}

#[cfg(test)]
mod tests {
    use super::{
        modal_body_area, modal_layout, modal_rect, settings_visible_rows, spotify_search_list_area,
        visible_items, visible_rows, ListItemHeight, MODAL_MIN_HEIGHT, MODAL_MIN_WIDTH,
    };
    use ratatui::layout::Rect;

    #[test]
    fn modal_rect_does_not_exceed_small_terminal_area() {
        let area = Rect::new(0, 0, 40, MODAL_MIN_HEIGHT - 4);
        let modal = modal_rect(area);

        assert!(modal.right() <= area.right());
        assert!(modal.bottom() <= area.bottom());
        assert_eq!(modal.width, 40);
        assert_eq!(modal.height, MODAL_MIN_HEIGHT - 4);
    }

    #[test]
    fn modal_rect_uses_minimum_size_when_area_allows_it() {
        let area = Rect::new(0, 0, MODAL_MIN_WIDTH, MODAL_MIN_HEIGHT);
        let modal = modal_rect(area);

        assert_eq!(modal, area);
    }

    #[test]
    fn modal_body_area_matches_rendered_border_and_tab_chrome() {
        let area = Rect::new(0, 0, 100, 40);
        let panel = modal_rect(area);
        let body = modal_body_area(area).expect("modal should render");

        assert_eq!(body.x, panel.x + 1);
        assert_eq!(body.y, panel.y + 2);
        assert_eq!(body.width, panel.width.saturating_sub(2));
        assert_eq!(body.height, panel.height.saturating_sub(3));
    }

    #[test]
    fn modal_layout_is_empty_when_modal_cannot_render() {
        let area = Rect::new(0, 0, MODAL_MIN_WIDTH - 1, MODAL_MIN_HEIGHT);

        assert!(modal_layout(area).is_none());
        assert!(modal_body_area(area).is_none());
    }

    #[test]
    fn visible_items_makes_item_height_explicit() {
        let area = Some(Rect::new(0, 0, 10, 9));

        assert_eq!(visible_items(area, ListItemHeight::OneLine), 9);
        assert_eq!(visible_items(area, ListItemHeight::TwoLines), 4);
    }

    #[test]
    fn visible_rows_can_reserve_rendered_chrome_rows() {
        let area = Some(Rect::new(0, 0, 10, 9));

        assert_eq!(visible_rows(area, 1), 8);
    }

    #[test]
    fn spotify_search_list_area_matches_rendered_chrome() {
        let area = Rect::new(0, 0, 100, 40);
        let modal_body = modal_body_area(area).expect("modal should render");
        let list_area = spotify_search_list_area(area).expect("spotify list should render");

        assert_eq!(list_area.height, modal_body.height.saturating_sub(6));
    }

    #[test]
    fn settings_visible_rows_reserve_scrollbar_row() {
        let area = Rect::new(0, 0, 100, 40);
        let modal_body = modal_body_area(area).expect("modal should render");
        let rows = settings_visible_rows(area);

        assert_eq!(rows, modal_body.height.saturating_sub(5) as usize);
    }
}
