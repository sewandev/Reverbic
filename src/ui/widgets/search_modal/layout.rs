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
pub(crate) struct FilterListLayout {
    pub input: Rect,
    pub cap: Rect,
    pub list: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeaderListLayout {
    pub header: Rect,
    pub list: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RadioNameLayout {
    pub subtab: Rect,
    pub body: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SettingsLayout {
    pub items: Rect,
    pub tooltip: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpotifyLayout {
    pub subtab: Rect,
    pub body: Rect,
    pub footer: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpotifySearchLayout {
    pub input: Rect,
    pub cap: Rect,
    pub list: Rect,
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
    let body = modal_body_area(area)?;
    Some(filter_list_layout(radio_name_layout(body).body).list)
}

pub(crate) fn radio_favorites_list_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    Some(radio_favorites_list_layout(radio_name_layout(body).body))
}

pub(crate) fn radio_filter_list_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    Some(filter_list_layout(body).list)
}

pub(crate) fn radio_filtered_results_list_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    Some(header_list_layout(body).list)
}

pub(crate) fn settings_items_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    Some(settings_layout(body).items)
}

pub(crate) fn settings_visible_rows(area: Rect) -> usize {
    settings_items_area(area)
        .map(|area| area.height.saturating_sub(1) as usize)
        .unwrap_or(0)
}

pub(crate) fn spotify_body_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    Some(spotify_layout(body).body)
}

pub(crate) fn spotify_search_list_area(area: Rect) -> Option<Rect> {
    let body = spotify_body_area(area)?;
    Some(spotify_search_layout(body).list)
}

pub(crate) fn spotify_titled_track_list_area(area: Rect) -> Option<Rect> {
    spotify_body_area(area).map(spotify_titled_track_list_layout)
}

pub(crate) fn youtube_list_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    Some(filter_list_layout(body).list)
}

pub(crate) fn radio_name_layout(area: Rect) -> RadioNameLayout {
    let [_gap, subtab, body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    RadioNameLayout { subtab, body }
}

pub(crate) fn radio_favorites_list_layout(area: Rect) -> Rect {
    let [_gap, list] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
    list
}

pub(crate) fn header_list_layout(area: Rect) -> HeaderListLayout {
    let [header, list] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
    HeaderListLayout { header, list }
}

pub(crate) fn settings_layout(area: Rect) -> SettingsLayout {
    let [_gap, items, tooltip] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .areas(area);

    SettingsLayout { items, tooltip }
}

pub(crate) fn spotify_layout(area: Rect) -> SpotifyLayout {
    let [_gap, subtab, _body_gap, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    SpotifyLayout {
        subtab,
        body,
        footer,
    }
}

pub(crate) fn spotify_search_layout(area: Rect) -> SpotifySearchLayout {
    let [input, cap, list] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    SpotifySearchLayout { input, cap, list }
}

pub(crate) fn spotify_titled_track_list_layout(area: Rect) -> Rect {
    Rect::new(
        area.x,
        area.y.saturating_add(1),
        area.width,
        area.height.saturating_sub(1),
    )
}

pub(crate) fn filter_list_layout(area: Rect) -> FilterListLayout {
    let [_gap, input, cap, list] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    FilterListLayout { input, cap, list }
}

#[cfg(test)]
mod tests {
    use super::{
        filter_list_layout, header_list_layout, modal_body_area, modal_layout, modal_rect,
        radio_favorites_list_area, radio_favorites_list_layout, radio_filter_list_area,
        radio_filtered_results_list_area, radio_name_layout, radio_search_results_list_area,
        settings_items_area, settings_layout, settings_visible_rows, spotify_body_area,
        spotify_layout, spotify_search_layout, spotify_search_list_area,
        spotify_titled_track_list_area, spotify_titled_track_list_layout, visible_items,
        visible_rows, youtube_list_area, ListItemHeight, MODAL_MIN_HEIGHT, MODAL_MIN_WIDTH,
    };
    use ratatui::layout::Rect;

    fn normal_terminal() -> Rect {
        Rect::new(0, 0, 100, 40)
    }

    fn normal_modal_body() -> Rect {
        modal_body_area(normal_terminal()).expect("modal should render")
    }

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
    fn modal_viewport_is_empty_below_minimum_height() {
        let area = Rect::new(0, 0, MODAL_MIN_WIDTH, MODAL_MIN_HEIGHT - 1);

        assert!(modal_layout(area).is_none());
        assert_eq!(
            visible_items(spotify_search_list_area(area), ListItemHeight::TwoLines),
            0
        );
        assert_eq!(visible_rows(radio_search_results_list_area(area), 1), 0);
        assert_eq!(settings_visible_rows(area), 0);
    }

    #[test]
    fn modal_viewport_renders_at_minimum_size_with_saturating_viewports() {
        let area = Rect::new(0, 0, MODAL_MIN_WIDTH, MODAL_MIN_HEIGHT);

        assert!(modal_layout(area).is_some());
        assert_eq!(
            visible_items(spotify_search_list_area(area), ListItemHeight::TwoLines),
            2
        );
        assert_eq!(visible_rows(radio_search_results_list_area(area), 1), 5);
        assert_eq!(settings_visible_rows(area), 6);
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
        let area = normal_terminal();
        let modal_body = modal_body_area(area).expect("modal should render");
        let list_area = spotify_search_list_area(area).expect("spotify list should render");

        assert_eq!(list_area.height, modal_body.height.saturating_sub(6));
    }

    #[test]
    fn settings_visible_rows_reserve_scrollbar_row() {
        let area = normal_terminal();
        let modal_body = modal_body_area(area).expect("modal should render");
        let rows = settings_visible_rows(area);

        assert_eq!(rows, modal_body.height.saturating_sub(5) as usize);
    }

    #[test]
    fn modal_viewport_spotify_search_matches_input_and_render_layouts() {
        let terminal = normal_terminal();
        let body = normal_modal_body();
        let rendered_spotify = spotify_layout(body);
        let rendered_search = spotify_search_layout(rendered_spotify.body);
        let input_list = spotify_search_list_area(terminal);

        assert_eq!(input_list, Some(rendered_search.list));
        assert_eq!(
            visible_items(input_list, ListItemHeight::TwoLines),
            rendered_search.list.height as usize / 2
        );
        assert_eq!(visible_items(input_list, ListItemHeight::TwoLines), 10);
    }

    #[test]
    fn modal_viewport_spotify_top_tracks_matches_common_spotify_body() {
        let terminal = normal_terminal();
        let rendered_spotify = spotify_layout(normal_modal_body());
        let input_body = spotify_body_area(terminal);

        assert_eq!(input_body, Some(rendered_spotify.body));
        assert_eq!(visible_items(input_body, ListItemHeight::TwoLines), 11);
    }

    #[test]
    fn modal_viewport_spotify_title_row_matches_playlist_and_album_tracks() {
        let terminal = normal_terminal();
        let rendered_spotify = spotify_layout(normal_modal_body());
        let rendered_titled_list = spotify_titled_track_list_layout(rendered_spotify.body);
        let input_list = spotify_titled_track_list_area(terminal);

        assert_eq!(input_list, Some(rendered_titled_list));
        assert_eq!(visible_items(input_list, ListItemHeight::TwoLines), 11);
    }

    #[test]
    fn modal_viewport_youtube_results_match_filter_layout() {
        let terminal = normal_terminal();
        let rendered = filter_list_layout(normal_modal_body());
        let input_list = youtube_list_area(terminal);

        assert_eq!(input_list, Some(rendered.list));
        assert_eq!(visible_items(input_list, ListItemHeight::TwoLines), 12);
    }

    #[test]
    fn modal_viewport_radio_search_matches_subtab_filter_layout() {
        let terminal = normal_terminal();
        let rendered_radio = radio_name_layout(normal_modal_body());
        let rendered_filter = filter_list_layout(rendered_radio.body);
        let input_list = radio_search_results_list_area(terminal);

        assert_eq!(input_list, Some(rendered_filter.list));
        assert_eq!(visible_rows(input_list, 1), 21);
    }

    #[test]
    fn modal_viewport_radio_favorites_matches_subtab_list_layout() {
        let terminal = normal_terminal();
        let rendered_radio = radio_name_layout(normal_modal_body());
        let rendered_list = radio_favorites_list_layout(rendered_radio.body);
        let input_list = radio_favorites_list_area(terminal);

        assert_eq!(input_list, Some(rendered_list));
        assert_eq!(visible_items(input_list, ListItemHeight::OneLine), 24);
    }

    #[test]
    fn modal_viewport_genre_and_country_filters_share_filter_layout() {
        let terminal = normal_terminal();
        let rendered = filter_list_layout(normal_modal_body());
        let input_list = radio_filter_list_area(terminal);

        assert_eq!(input_list, Some(rendered.list));
        assert_eq!(visible_rows(input_list, 1), 23);
    }

    #[test]
    fn modal_viewport_genre_and_country_results_match_header_list_layout() {
        let terminal = normal_terminal();
        let rendered = header_list_layout(normal_modal_body());
        let input_list = radio_filtered_results_list_area(terminal);

        assert_eq!(input_list, Some(rendered.list));
        assert_eq!(visible_rows(input_list, 1), 25);
    }

    #[test]
    fn modal_viewport_settings_matches_items_area_and_visual_rows() {
        let terminal = normal_terminal();
        let rendered = settings_layout(normal_modal_body());
        let input_items = settings_items_area(terminal);

        assert_eq!(input_items, Some(rendered.items));
        assert_eq!(
            settings_visible_rows(terminal),
            visible_rows(input_items, 1)
        );
        assert_eq!(settings_visible_rows(terminal), 22);
    }
}
