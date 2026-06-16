use ratatui::layout::Rect;

const PANEL_MIN_WIDTH: u16 = 34;
const PANEL_MAX_WIDTH: u16 = 44;
const PANEL_MIN_HEIGHT: u16 = 5;
const PANEL_CHROME_ROWS: u16 = 4;
const MAX_VISIBLE_ROWS: usize = 10;

pub(crate) fn panel(area: Rect, item_count: usize) -> Rect {
    if area.width == 0 || area.height == 0 {
        return Rect::new(area.x, area.y, 0, 0);
    }

    let width = area
        .width
        .min(PANEL_MAX_WIDTH)
        .max(area.width.min(PANEL_MIN_WIDTH));
    let visible_rows = visible_rows(area, item_count) as u16;
    let target_height = visible_rows.saturating_add(PANEL_CHROME_ROWS);
    let min_height = area.height.min(PANEL_MIN_HEIGHT);
    let height = target_height.max(min_height).min(area.height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Rect::new(x, y, width, height)
}

pub(crate) fn visible_rows(area: Rect, item_count: usize) -> usize {
    let viewport_rows = area.height.saturating_sub(PANEL_CHROME_ROWS) as usize;

    item_count.min(viewport_rows).min(MAX_VISIBLE_ROWS)
}
