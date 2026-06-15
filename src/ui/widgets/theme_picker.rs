use ratatui::layout::Rect;

const PANEL_MIN_WIDTH: u16 = 34;
const PANEL_MAX_WIDTH: u16 = 46;
const PANEL_MIN_HEIGHT: u16 = 5;
const PANEL_CHROME_ROWS: u16 = 4;
const MAX_VISIBLE_ROWS: usize = 12;

pub(crate) fn panel(area: Rect, theme_count: usize) -> Rect {
    if area.width == 0 || area.height == 0 {
        return Rect::new(area.x, area.y, 0, 0);
    }

    let width = area
        .width
        .min(PANEL_MAX_WIDTH)
        .max(area.width.min(PANEL_MIN_WIDTH));
    let visible_rows = visible_rows(area, theme_count) as u16;
    let target_height = visible_rows.saturating_add(PANEL_CHROME_ROWS);
    let min_height = area.height.min(PANEL_MIN_HEIGHT);
    let height = target_height.max(min_height).min(area.height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Rect::new(x, y, width, height)
}

pub(crate) fn visible_rows(area: Rect, theme_count: usize) -> usize {
    let viewport_rows = area.height.saturating_sub(PANEL_CHROME_ROWS) as usize;

    theme_count.min(viewport_rows).min(MAX_VISIBLE_ROWS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_rows_caps_long_theme_lists() {
        assert_eq!(visible_rows(Rect::new(0, 0, 80, 30), 20), 12);
    }

    #[test]
    fn visible_rows_shrinks_for_small_terminals() {
        assert_eq!(visible_rows(Rect::new(0, 0, 40, 8), 20), 4);
    }

    #[test]
    fn visible_rows_does_not_exceed_theme_count() {
        assert_eq!(visible_rows(Rect::new(0, 0, 80, 30), 1), 1);
    }

    #[test]
    fn visible_rows_handles_tiny_terminals() {
        assert_eq!(visible_rows(Rect::new(0, 0, 20, 3), 20), 0);
    }

    #[test]
    fn panel_fits_narrow_terminals() {
        let panel = panel(Rect::new(2, 3, 20, 10), 20);

        assert_eq!(panel.x, 2);
        assert_eq!(panel.y, 3);
        assert_eq!(panel.width, 20);
        assert_eq!(panel.height, 10);
    }
}
