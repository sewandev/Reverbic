use ratatui::layout::{Constraint, Layout, Rect};

pub(super) const HEIGHT_NORMAL:  u16 = 11;
pub(super) const HEIGHT_COMPACT: u16 = 5;

pub(super) struct AppLayout {
    pub logo_y:        Option<u16>,
    pub header:        Option<Rect>,
    pub sep_header:    Option<Rect>,
    pub stations:      Rect,
    pub on_demand:     Option<Rect>,
    pub saved_tracks:  Option<Rect>,
    pub recent_tracks: Option<Rect>,
    pub sep_body:      Option<Rect>,
    pub now_playing:   Option<Rect>,
    pub vu:            Option<Rect>,
    pub sep_footer:    Option<Rect>,
    pub countdown:     Option<Rect>,
    pub help:          Rect,
}

pub fn now_playing_rect(
    area:           Rect,
    has_recent:     bool,
    has_saved:      bool,
    show_countdown: bool,
    has_on_demand:  bool,
) -> Option<Rect> {
    compute_layout(area, has_recent, has_saved, show_countdown, has_on_demand).now_playing
}

pub(super) fn compute_layout(
    area:           Rect,
    has_recent:     bool,
    has_saved:      bool,
    show_countdown: bool,
    has_on_demand:  bool,
) -> AppLayout {
    let countdown_h: u16 = u16::from(show_countdown);

    if area.height >= HEIGHT_NORMAL + 2 + countdown_h {
        let layout_area = Rect::new(area.x, area.y + 2, area.width, area.height - 2);
        let rows = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(countdown_h),
            Constraint::Length(1),
        ])
        .split(layout_area);

        let countdown = if show_countdown { Some(rows[7]) } else { None };
        let (stations, on_demand, saved_tracks, recent_tracks) =
            build_columns(rows[2], has_on_demand, has_recent, has_saved);

        AppLayout {
            logo_y:       Some(area.y + 2),
            header:       Some(rows[0]),
            sep_header:   Some(rows[1]),
            stations,
            on_demand,
            saved_tracks,
            recent_tracks,
            sep_body:     Some(rows[3]),
            now_playing:  Some(rows[4]),
            vu:           Some(rows[5]),
            sep_footer:   Some(rows[6]),
            countdown,
            help:         rows[8],
        }
    } else if area.height >= HEIGHT_NORMAL + countdown_h {
        let rows = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(countdown_h),
            Constraint::Length(1),
        ])
        .split(area);

        let countdown = if show_countdown { Some(rows[7]) } else { None };
        let (stations, on_demand, saved_tracks, recent_tracks) =
            build_columns(rows[2], has_on_demand, has_recent, has_saved);

        AppLayout {
            logo_y:       None,
            header:       Some(rows[0]),
            sep_header:   Some(rows[1]),
            stations,
            on_demand,
            saved_tracks,
            recent_tracks,
            sep_body:     Some(rows[3]),
            now_playing:  Some(rows[4]),
            vu:           Some(rows[5]),
            sep_footer:   Some(rows[6]),
            countdown,
            help:         rows[8],
        }
    } else if area.height >= HEIGHT_COMPACT {
        let rows = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

        let (stations, on_demand, saved_tracks, recent_tracks) =
            build_columns(rows[0], has_on_demand, has_recent, has_saved);

        AppLayout {
            logo_y: None, header: None, sep_header: None,
            stations, on_demand, saved_tracks, recent_tracks,
            sep_body: None,
            now_playing:  Some(rows[1]),
            vu:           None,
            sep_footer:   None,
            countdown:    None,
            help:         rows[2],
        }
    } else {
        let rows = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);
        let (stations, on_demand, saved_tracks, recent_tracks) =
            build_columns(rows[0], false, false, false);

        AppLayout {
            logo_y: None, header: None, sep_header: None,
            stations, on_demand, saved_tracks, recent_tracks,
            sep_body: None, now_playing: None, vu: None, sep_footer: None, countdown: None,
            help: rows[1],
        }
    }
}

fn build_columns(
    top:           Rect,
    has_on_demand: bool,
    has_recent:    bool,
    has_saved:     bool,
) -> (Rect, Option<Rect>, Option<Rect>, Option<Rect>) {
    if has_on_demand {
        let show_right = top.width >= 90 && (has_recent || has_saved);
        if show_right {
            let cols = Layout::horizontal([
                Constraint::Max(38),
                Constraint::Max(35),
                Constraint::Fill(1),
            ])
            .split(top);
            let (saved, recent) = split_saved_recent(cols[2], has_recent, has_saved);
            (cols[0], Some(cols[1]), saved, recent)
        } else {
            let cols = Layout::horizontal([Constraint::Max(38), Constraint::Fill(1)]).split(top);
            (cols[0], Some(cols[1]), None, None)
        }
    } else if has_recent || has_saved {
        let cols = Layout::horizontal([Constraint::Max(40), Constraint::Fill(1)]).split(top);
        let (saved, recent) = split_saved_recent(cols[1], has_recent, has_saved);
        (cols[0], None, saved, recent)
    } else {
        (top, None, None, None)
    }
}

fn split_saved_recent(
    right:      Rect,
    has_recent: bool,
    has_saved:  bool,
) -> (Option<Rect>, Option<Rect>) {
    match (has_saved, has_recent) {
        (true, true) => {
            let rows = Layout::vertical([
                Constraint::Percentage(58),
                Constraint::Percentage(42),
            ])
            .split(right);
            (Some(rows[0]), Some(rows[1]))
        }
        (true, false) => (Some(right), None),
        (false, true) => (None, Some(right)),
        (false, false) => (None, None),
    }
}
