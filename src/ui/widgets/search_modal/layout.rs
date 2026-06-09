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

#[cfg(test)]
mod tests {
    use super::{modal_body_area, modal_layout, modal_rect, MODAL_MIN_HEIGHT, MODAL_MIN_WIDTH};
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
}
