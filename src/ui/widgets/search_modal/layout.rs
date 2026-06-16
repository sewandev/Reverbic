use ratatui::layout::{Constraint, Layout, Rect};

use crate::app::{RadioSubTab, SearchMode, SpotifySubTab, YoutubeSubTab};
use crate::i18n::t;
use crate::ui::widgets::scroll_offset_for_selection;

pub(crate) const MODAL_MIN_WIDTH: u16 = 52;
pub(crate) const MODAL_MIN_HEIGHT: u16 = 14;

const MODAL_MAX_WIDTH: u16 = 120;
const MODAL_MAX_HEIGHT: u16 = 30;
const MODAL_TABS_ROWS: u16 = 1;
const SEARCH_TOP_GAP_ROWS: u16 = 1;
const SEARCH_INPUT_ROWS: u16 = 1;
const SEARCH_CAP_ROWS: u16 = 1;
const YOUTUBE_SEARCH_HINT_ROWS: u16 = 2;
const LIST_HEADER_ROWS: u16 = 1;
const RADIO_TOP_GAP_ROWS: u16 = 1;
const RADIO_SUBTAB_ROWS: u16 = 1;
const RADIO_FAVORITES_TOP_GAP_ROWS: u16 = 1;
const SETTINGS_TOP_GAP_ROWS: u16 = 1;
const SETTINGS_TOOLTIP_ROWS: u16 = 3;
const SPOTIFY_TOP_GAP_ROWS: u16 = 1;
const SPOTIFY_SUBTAB_ROWS: u16 = 1;
const SPOTIFY_BODY_GAP_ROWS: u16 = 1;
const SPOTIFY_FOOTER_ROWS: u16 = 1;
const SPOTIFY_TITLE_ROWS: u16 = 1;
const YOUTUBE_TOP_GAP_ROWS: u16 = 1;
const YOUTUBE_SUBTAB_ROWS: u16 = 1;
const YOUTUBE_BODY_GAP_ROWS: u16 = 1;
const SCROLLBAR_RESERVED_ROWS: u16 = 1;
const MODAL_CONTENT_HORIZONTAL_PADDING: u16 = 2;
const MODAL_SUBTAB_INDENT: u16 = 2;
const TAB_GAP_WIDTH: u16 = 2;

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
pub(crate) struct YoutubeLayout {
    pub subtab: Rect,
    pub body: Rect,
}

pub(crate) struct YoutubeSearchLayout {
    pub input: Rect,
    pub cap: Rect,
    pub list: Rect,
    pub hint: Rect,
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
    let [tabs, body] =
        Layout::vertical([Constraint::Length(MODAL_TABS_ROWS), Constraint::Fill(1)]).areas(inner);

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

pub(crate) fn modal_content_area(area: Rect) -> Option<Rect> {
    let layout = modal_layout(area)?;
    Some(Rect::new(
        layout
            .inner
            .x
            .saturating_add(MODAL_CONTENT_HORIZONTAL_PADDING),
        layout.inner.y,
        layout
            .inner
            .width
            .saturating_sub(MODAL_CONTENT_HORIZONTAL_PADDING * 2),
        layout.inner.height,
    ))
}

pub(crate) fn visible_items(area: Option<Rect>, item_height: ListItemHeight) -> usize {
    area.map(|area| area.height as usize / item_height.rows())
        .unwrap_or(0)
}

pub(crate) fn visible_rows(area: Option<Rect>, reserved_rows: u16) -> usize {
    area.map(|area| area.height.saturating_sub(reserved_rows) as usize)
        .unwrap_or(0)
}

pub(crate) fn visible_rows_excluding_scrollbar(area: Option<Rect>) -> usize {
    visible_rows(area, SCROLLBAR_RESERVED_ROWS)
}

pub(crate) fn contains(area: Rect, col: u16, row: u16) -> bool {
    col >= area.x && col < area.right() && row >= area.y && row < area.bottom()
}

pub(crate) fn one_line_list_index_at(
    area: Option<Rect>,
    col: u16,
    row: u16,
    selected: usize,
    visible: usize,
    scroll_offset: usize,
    item_count: usize,
) -> Option<usize> {
    list_index_at(ListHitTest {
        area,
        col,
        row,
        item_height: ListItemHeight::OneLine,
        selected,
        visible,
        scroll_offset,
        item_count,
    })
}

pub(crate) fn two_line_list_index_at(
    area: Option<Rect>,
    col: u16,
    row: u16,
    selected: usize,
    visible: usize,
    scroll_offset: usize,
    item_count: usize,
) -> Option<usize> {
    list_index_at(ListHitTest {
        area,
        col,
        row,
        item_height: ListItemHeight::TwoLines,
        selected,
        visible,
        scroll_offset,
        item_count,
    })
}

struct ListHitTest {
    area: Option<Rect>,
    col: u16,
    row: u16,
    item_height: ListItemHeight,
    selected: usize,
    visible: usize,
    scroll_offset: usize,
    item_count: usize,
}

fn list_index_at(hit: ListHitTest) -> Option<usize> {
    let area = hit.area?;
    if !contains(area, hit.col, hit.row) || hit.visible == 0 || hit.item_count == 0 {
        return None;
    }

    let clicked_slot = hit.row.saturating_sub(area.y) as usize / hit.item_height.rows();
    if clicked_slot >= hit.visible {
        return None;
    }

    let selected = hit.selected.min(hit.item_count.saturating_sub(1));
    let offset = scroll_offset_for_selection(selected, hit.visible, hit.scroll_offset);
    let index = offset.saturating_add(clicked_slot);
    (index < hit.item_count).then_some(index)
}

pub(crate) fn modal_tab_at(
    area: Rect,
    col: u16,
    row: u16,
    dots: crate::app::TabDots,
) -> Option<SearchMode> {
    let layout = modal_layout(area)?;
    let content = modal_content_area(area)?;
    let tab_area = Rect::new(content.x, layout.tabs.y, content.width, layout.tabs.height);
    let label = |has_dot: bool, text: String| {
        if has_dot {
            format!("\u{25CF} {text}")
        } else {
            text
        }
    };
    tab_value_at(
        tab_area,
        col,
        row,
        &[
            (
                label(dots.radio.is_some(), t("modal.tab.radio")),
                SearchMode::Name,
            ),
            (
                label(dots.spotify.is_some(), t("modal.tab.spotify")),
                SearchMode::Spotify,
            ),
            (
                label(dots.youtube.is_some(), t("modal.tab.youtube")),
                SearchMode::Youtube,
            ),
        ],
    )
}

pub(crate) fn radio_subtab_at(
    area: Rect,
    col: u16,
    row: u16,
    favorites_count: usize,
    playlists_count: usize,
) -> Option<RadioSubTab> {
    let body = modal_body_area(area)?;
    let content = modal_content_area(area)?;
    let subtab = radio_name_layout(body).subtab;
    let tab_area = Rect::new(
        content.x.saturating_add(MODAL_SUBTAB_INDENT),
        subtab.y,
        content.width.saturating_sub(MODAL_SUBTAB_INDENT),
        subtab.height,
    );

    tab_value_at(
        tab_area,
        col,
        row,
        &[
            (t("modal.radio.subtab.search"), RadioSubTab::Search),
            (
                format!(
                    "[ {} ({}) ]",
                    t("modal.radio.subtab.favorites.label"),
                    favorites_count
                ),
                RadioSubTab::Favorites,
            ),
            (
                format!(
                    "[ {} ({}) ]",
                    t("modal.radio.subtab.playlists.label"),
                    playlists_count
                ),
                RadioSubTab::Playlists,
            ),
        ],
    )
}

pub(crate) fn spotify_subtab_at(area: Rect, col: u16, row: u16) -> Option<SpotifySubTab> {
    let body = modal_body_area(area)?;
    let content = modal_content_area(area)?;
    let subtab = spotify_layout(body).subtab;
    let tab_area = Rect::new(
        content.x.saturating_add(MODAL_SUBTAB_INDENT),
        subtab.y,
        content.width.saturating_sub(MODAL_SUBTAB_INDENT),
        subtab.height,
    );

    tab_value_at(
        tab_area,
        col,
        row,
        &[
            (t("modal.spotify.subtab.search"), SpotifySubTab::Search),
            (t("modal.spotify.subtab.liked"), SpotifySubTab::Liked),
            (
                t("modal.spotify.subtab.playlists"),
                SpotifySubTab::Playlists,
            ),
            (t("Top Tracks"), SpotifySubTab::TopTracks),
            (t("Recent"), SpotifySubTab::Recent),
            (t("Albums"), SpotifySubTab::Albums),
        ],
    )
}

pub(crate) fn youtube_subtab_at(area: Rect, col: u16, row: u16) -> Option<YoutubeSubTab> {
    let body = modal_body_area(area)?;
    let content = modal_content_area(area)?;
    let subtab = youtube_layout(body).subtab;
    let tab_area = Rect::new(
        content.x.saturating_add(MODAL_SUBTAB_INDENT),
        subtab.y,
        content.width.saturating_sub(MODAL_SUBTAB_INDENT),
        subtab.height,
    );

    tab_value_at(
        tab_area,
        col,
        row,
        &[
            (t("modal.youtube.subtab.search"), YoutubeSubTab::Search),
            (
                t("modal.youtube.subtab.public_playlists"),
                YoutubeSubTab::PublicPlaylists,
            ),
            (
                t("modal.youtube.subtab.bookmarks"),
                YoutubeSubTab::Bookmarks,
            ),
            (t("modal.youtube.subtab.liked"), YoutubeSubTab::Liked),
            (
                t("modal.youtube.subtab.playlists"),
                YoutubeSubTab::Playlists,
            ),
        ],
    )
}

fn tab_value_at<T: Copy>(area: Rect, col: u16, row: u16, labels: &[(String, T)]) -> Option<T> {
    if !contains(area, col, row) {
        return None;
    }

    let mut x = area.x;
    for (idx, (label, value)) in labels.iter().enumerate() {
        if x >= area.right() {
            return None;
        }

        let end = x.saturating_add(text_cell_width(label)).min(area.right());
        if col >= x && col < end {
            return Some(*value);
        }

        x = x.saturating_add(text_cell_width(label));
        if idx + 1 < labels.len() {
            x = x.saturating_add(TAB_GAP_WIDTH);
        }
    }

    None
}

fn text_cell_width(text: &str) -> u16 {
    text.chars().count().min(u16::MAX as usize) as u16
}

pub(crate) fn radio_search_results_list_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    Some(filter_list_layout(radio_name_layout(body).body).list)
}

pub(crate) fn radio_favorites_list_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    Some(radio_favorites_list_layout(radio_name_layout(body).body))
}

pub(crate) fn radio_playlists_list_area(area: Rect) -> Option<Rect> {
    radio_favorites_list_area(area)
}

pub(crate) fn radio_playlist_stations_list_area(area: Rect) -> Option<Rect> {
    radio_playlists_list_area(area).map(|list| header_list_layout(list).list)
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
    visible_rows_excluding_scrollbar(settings_items_area(area))
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

pub(crate) fn youtube_body_area(area: Rect) -> Option<Rect> {
    let body = modal_body_area(area)?;
    Some(youtube_layout(body).body)
}

pub(crate) fn youtube_search_list_area(area: Rect) -> Option<Rect> {
    let body = youtube_body_area(area)?;
    Some(youtube_search_layout(body).list)
}

pub(crate) fn youtube_public_list_area(area: Rect) -> Option<Rect> {
    let body = youtube_body_area(area)?;
    Some(youtube_search_layout(body).list)
}

pub(crate) fn youtube_liked_list_area(area: Rect) -> Option<Rect> {
    youtube_body_area(area)
}

pub(crate) fn youtube_playlists_list_area(area: Rect) -> Option<Rect> {
    youtube_body_area(area)
}

pub(crate) fn youtube_playlist_videos_list_area(area: Rect) -> Option<Rect> {
    youtube_body_area(area).map(spotify_titled_track_list_layout)
}

pub(crate) fn radio_name_layout(area: Rect) -> RadioNameLayout {
    let [_gap, subtab, body] = Layout::vertical([
        Constraint::Length(RADIO_TOP_GAP_ROWS),
        Constraint::Length(RADIO_SUBTAB_ROWS),
        Constraint::Fill(1),
    ])
    .areas(area);

    RadioNameLayout { subtab, body }
}

pub(crate) fn radio_favorites_list_layout(area: Rect) -> Rect {
    let [_gap, list] = Layout::vertical([
        Constraint::Length(RADIO_FAVORITES_TOP_GAP_ROWS),
        Constraint::Fill(1),
    ])
    .areas(area);
    list
}

pub(crate) fn header_list_layout(area: Rect) -> HeaderListLayout {
    let [header, list] =
        Layout::vertical([Constraint::Length(LIST_HEADER_ROWS), Constraint::Fill(1)]).areas(area);
    HeaderListLayout { header, list }
}

pub(crate) fn settings_layout(area: Rect) -> SettingsLayout {
    let [_gap, items, tooltip] = Layout::vertical([
        Constraint::Length(SETTINGS_TOP_GAP_ROWS),
        Constraint::Fill(1),
        Constraint::Length(SETTINGS_TOOLTIP_ROWS),
    ])
    .areas(area);

    SettingsLayout { items, tooltip }
}

pub(crate) fn spotify_layout(area: Rect) -> SpotifyLayout {
    let [_gap, subtab, _body_gap, body, footer] = Layout::vertical([
        Constraint::Length(SPOTIFY_TOP_GAP_ROWS),
        Constraint::Length(SPOTIFY_SUBTAB_ROWS),
        Constraint::Length(SPOTIFY_BODY_GAP_ROWS),
        Constraint::Fill(1),
        Constraint::Length(SPOTIFY_FOOTER_ROWS),
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
        Constraint::Length(SEARCH_INPUT_ROWS),
        Constraint::Length(SEARCH_CAP_ROWS),
        Constraint::Fill(1),
    ])
    .areas(area);

    SpotifySearchLayout { input, cap, list }
}

pub(crate) fn youtube_layout(area: Rect) -> YoutubeLayout {
    let [_gap, subtab, _body_gap, body] = Layout::vertical([
        Constraint::Length(YOUTUBE_TOP_GAP_ROWS),
        Constraint::Length(YOUTUBE_SUBTAB_ROWS),
        Constraint::Length(YOUTUBE_BODY_GAP_ROWS),
        Constraint::Fill(1),
    ])
    .areas(area);

    YoutubeLayout { subtab, body }
}

pub(crate) fn auth_notice_box(body: Rect) -> Option<Rect> {
    let box_w = body.width.saturating_sub(4).min(70);
    let box_h = 14.min(body.height);
    if box_w < 20 || box_h < 6 {
        return None;
    }
    Some(Rect::new(
        body.x + (body.width - box_w) / 2,
        body.y + body.height.saturating_sub(box_h) / 2,
        box_w,
        box_h,
    ))
}

pub(crate) fn youtube_auth_notice_at(area: Rect, col: u16, row: u16) -> bool {
    let Some(body) = modal_body_area(area) else {
        return false;
    };
    auth_notice_box(youtube_layout(body).body).is_some_and(|notice| contains(notice, col, row))
}

pub(crate) fn spotify_auth_notice_at(area: Rect, col: u16, row: u16) -> bool {
    let Some(body) = modal_body_area(area) else {
        return false;
    };
    auth_notice_box(body).is_some_and(|notice| contains(notice, col, row))
}

pub(crate) fn spotify_no_device_notice_at(area: Rect, col: u16, row: u16) -> bool {
    let Some(body) = modal_body_area(area) else {
        return false;
    };
    auth_notice_box(spotify_layout(body).body).is_some_and(|notice| contains(notice, col, row))
}

pub(crate) fn youtube_search_layout(area: Rect) -> YoutubeSearchLayout {
    let [input, cap, list, hint] = Layout::vertical([
        Constraint::Length(SEARCH_INPUT_ROWS),
        Constraint::Length(SEARCH_CAP_ROWS),
        Constraint::Fill(1),
        Constraint::Length(YOUTUBE_SEARCH_HINT_ROWS),
    ])
    .areas(area);

    YoutubeSearchLayout {
        input,
        cap,
        list,
        hint,
    }
}

pub(crate) fn spotify_titled_track_list_layout(area: Rect) -> Rect {
    Rect::new(
        area.x,
        area.y.saturating_add(SPOTIFY_TITLE_ROWS),
        area.width,
        area.height.saturating_sub(SPOTIFY_TITLE_ROWS),
    )
}

pub(crate) fn filter_list_layout(area: Rect) -> FilterListLayout {
    let [_gap, input, cap, list] = Layout::vertical([
        Constraint::Length(SEARCH_TOP_GAP_ROWS),
        Constraint::Length(SEARCH_INPUT_ROWS),
        Constraint::Length(SEARCH_CAP_ROWS),
        Constraint::Fill(1),
    ])
    .areas(area);

    FilterListLayout { input, cap, list }
}

#[cfg(test)]
mod tests {
    use super::{
        contains, filter_list_layout, header_list_layout, modal_body_area, modal_content_area,
        modal_layout, modal_rect, modal_tab_at, one_line_list_index_at, radio_favorites_list_area,
        radio_favorites_list_layout, radio_filter_list_area, radio_filtered_results_list_area,
        radio_name_layout, radio_search_results_list_area, radio_subtab_at, settings_items_area,
        settings_layout, settings_visible_rows, spotify_body_area, spotify_layout,
        spotify_search_layout, spotify_search_list_area, spotify_subtab_at,
        spotify_titled_track_list_area, spotify_titled_track_list_layout, text_cell_width,
        two_line_list_index_at, visible_items, visible_rows, visible_rows_excluding_scrollbar,
        youtube_layout, youtube_search_layout, youtube_search_list_area, ListItemHeight,
        MODAL_MIN_HEIGHT, MODAL_MIN_WIDTH, MODAL_SUBTAB_INDENT, TAB_GAP_WIDTH,
    };
    use crate::app::{RadioSubTab, SearchMode, SpotifySubTab};
    use crate::i18n::t;
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
        assert_eq!(
            visible_rows_excluding_scrollbar(radio_search_results_list_area(area)),
            0
        );
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
        assert_eq!(
            visible_rows_excluding_scrollbar(radio_search_results_list_area(area)),
            5
        );
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
    fn modal_tab_at_matches_rendered_top_tabs() {
        let area = normal_terminal();
        let layout = modal_layout(area).expect("modal should render");
        let content = modal_content_area(area).expect("content should render");
        let dots = crate::app::TabDots {
            radio: Some(crate::app::TabDot::Playing),
            ..Default::default()
        };
        let radio_w = text_cell_width(&t("modal.tab.radio")) + 2;
        let spotify_x = content.x + radio_w + TAB_GAP_WIDTH;

        assert!(matches!(
            modal_tab_at(area, content.x, layout.tabs.y, dots),
            Some(SearchMode::Name)
        ));
        assert!(matches!(
            modal_tab_at(area, spotify_x, layout.tabs.y, dots),
            Some(SearchMode::Spotify)
        ));
        assert!(modal_tab_at(area, content.x + radio_w, layout.tabs.y, dots).is_none());
    }

    #[test]
    fn radio_subtab_at_matches_rendered_subtabs() {
        let area = normal_terminal();
        let body = modal_body_area(area).expect("modal should render");
        let subtab = radio_name_layout(body).subtab;
        let content = modal_content_area(area).expect("content should render");
        let text_x = content.x + MODAL_SUBTAB_INDENT;
        let search_w = text_cell_width(&t("modal.radio.subtab.search"));
        let favorites_x = text_x + search_w + TAB_GAP_WIDTH;

        assert!(matches!(
            radio_subtab_at(area, text_x, subtab.y, 12, 3),
            Some(RadioSubTab::Search)
        ));
        assert!(matches!(
            radio_subtab_at(area, favorites_x, subtab.y, 12, 3),
            Some(RadioSubTab::Favorites)
        ));
    }

    #[test]
    fn spotify_subtab_at_matches_rendered_subtabs() {
        let area = normal_terminal();
        let body = modal_body_area(area).expect("modal should render");
        let subtab = spotify_layout(body).subtab;
        let content = modal_content_area(area).expect("content should render");
        let text_x = content.x + MODAL_SUBTAB_INDENT;
        let liked_x = text_x + text_cell_width(&t("modal.spotify.subtab.search")) + TAB_GAP_WIDTH;

        assert!(matches!(
            spotify_subtab_at(area, text_x, subtab.y),
            Some(SpotifySubTab::Search)
        ));
        assert!(matches!(
            spotify_subtab_at(area, liked_x, subtab.y),
            Some(SpotifySubTab::Liked)
        ));
    }

    #[test]
    fn contains_uses_terminal_cell_bounds() {
        let area = Rect::new(10, 4, 20, 6);

        assert!(contains(area, 10, 4));
        assert!(contains(area, 29, 9));
        assert!(!contains(area, 9, 4));
        assert!(!contains(area, 30, 9));
        assert!(!contains(area, 10, 10));
    }

    #[test]
    fn one_line_list_index_at_returns_clicked_visible_item() {
        let area = Some(Rect::new(10, 4, 20, 6));

        assert_eq!(one_line_list_index_at(area, 12, 6, 0, 6, 0, 10), Some(2));
    }

    #[test]
    fn one_line_list_index_at_ignores_clicks_outside_area() {
        let area = Some(Rect::new(10, 4, 20, 6));

        assert_eq!(one_line_list_index_at(area, 9, 6, 0, 6, 0, 10), None);
        assert_eq!(one_line_list_index_at(area, 12, 10, 0, 6, 0, 10), None);
        assert_eq!(one_line_list_index_at(None, 12, 6, 0, 6, 0, 10), None);
    }

    #[test]
    fn one_line_list_index_at_uses_rendered_visible_count() {
        let area = Some(Rect::new(10, 4, 20, 6));

        assert_eq!(one_line_list_index_at(area, 12, 7, 0, 4, 0, 10), Some(3));
        assert_eq!(one_line_list_index_at(area, 12, 8, 0, 4, 0, 10), None);
    }

    #[test]
    fn one_line_list_index_at_uses_scroll_offset_for_visible_window() {
        let area = Some(Rect::new(10, 4, 20, 6));

        assert_eq!(one_line_list_index_at(area, 12, 5, 8, 6, 3, 20), Some(4));
        assert_eq!(one_line_list_index_at(area, 12, 4, 0, 6, 20, 20), Some(0));
        assert_eq!(one_line_list_index_at(area, 12, 6, 3, 6, 20, 20), Some(5));
    }

    #[test]
    fn list_index_at_returns_none_when_clicked_slot_has_no_item() {
        let area = Some(Rect::new(10, 4, 20, 6));

        assert_eq!(one_line_list_index_at(area, 12, 7, 0, 6, 0, 3), None);
        assert_eq!(one_line_list_index_at(area, 12, 4, 0, 0, 0, 3), None);
        assert_eq!(one_line_list_index_at(area, 12, 4, 0, 6, 0, 0), None);
    }

    #[test]
    fn two_line_list_index_at_maps_both_rows_to_same_item() {
        let area = Some(Rect::new(10, 4, 20, 6));

        assert_eq!(two_line_list_index_at(area, 12, 4, 0, 3, 0, 10), Some(0));
        assert_eq!(two_line_list_index_at(area, 12, 5, 0, 3, 0, 10), Some(0));
        assert_eq!(two_line_list_index_at(area, 12, 6, 0, 3, 0, 10), Some(1));
    }

    #[test]
    fn two_line_list_index_at_ignores_partial_item_rows() {
        let area = Some(Rect::new(10, 4, 20, 5));

        assert_eq!(two_line_list_index_at(area, 12, 7, 0, 2, 0, 10), Some(1));
        assert_eq!(two_line_list_index_at(area, 12, 8, 0, 2, 0, 10), None);
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
        let rendered_youtube = youtube_layout(normal_modal_body());
        let rendered_search = youtube_search_layout(rendered_youtube.body);
        let input_list = youtube_search_list_area(terminal);

        assert_eq!(input_list, Some(rendered_search.list));
        assert_eq!(visible_items(input_list, ListItemHeight::TwoLines), 10);
    }

    #[test]
    fn modal_viewport_radio_search_matches_subtab_filter_layout() {
        let terminal = normal_terminal();
        let rendered_radio = radio_name_layout(normal_modal_body());
        let rendered_filter = filter_list_layout(rendered_radio.body);
        let input_list = radio_search_results_list_area(terminal);

        assert_eq!(input_list, Some(rendered_filter.list));
        assert_eq!(visible_rows_excluding_scrollbar(input_list), 21);
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
        assert_eq!(visible_rows_excluding_scrollbar(input_list), 23);
    }

    #[test]
    fn modal_viewport_genre_and_country_results_match_header_list_layout() {
        let terminal = normal_terminal();
        let rendered = header_list_layout(normal_modal_body());
        let input_list = radio_filtered_results_list_area(terminal);

        assert_eq!(input_list, Some(rendered.list));
        assert_eq!(visible_rows_excluding_scrollbar(input_list), 25);
    }

    #[test]
    fn modal_viewport_settings_matches_items_area_and_visual_rows() {
        let terminal = normal_terminal();
        let rendered = settings_layout(normal_modal_body());
        let input_items = settings_items_area(terminal);

        assert_eq!(input_items, Some(rendered.items));
        assert_eq!(
            settings_visible_rows(terminal),
            visible_rows_excluding_scrollbar(input_items)
        );
        assert_eq!(settings_visible_rows(terminal), 22);
    }
}
