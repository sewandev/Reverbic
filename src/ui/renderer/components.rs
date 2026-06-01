use chrono::{Datelike, Local};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppFocus;
use crate::audio::PlayerStatus;
use crate::i18n::t;
use crate::ui::theme;

pub(super) fn render_header(frame: &mut Frame, area: Rect) {
    let now      = Local::now();
    let time_str = format!("{}  {:02} {}", now.format("%H:%M"), now.day(), month_es(now.month()));
    let brand    = " REVERBIC";
    let brand_w  = brand.chars().count();
    let time_w   = time_str.chars().count();
    let pad      = (area.width as usize).saturating_sub(brand_w + time_w + 1);

    let line = Line::from(vec![
        Span::styled(brand, Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(pad)),
        Span::styled(time_str, Style::new().fg(theme::MUTED)),
        Span::raw(" "),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

pub(super) fn render_sep(frame: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(Span::styled(line, Style::default().fg(theme::MUTED))),
        area,
    );
}

pub(super) fn render_help(
    frame:              &mut Frame,
    area:               Rect,
    status:             &PlayerStatus,
    focus:              &AppFocus,
    save_notice:        Option<&str>,
    save_notice_is_dup: bool,
    preview_title:      Option<&str>,
    preview_searching:  bool,
    seek_input:         &str,
) {
    let (text, color) = if let Some(title) = preview_title {
        (format!(" PREVIEW: {title}  {}", t("help.stop_preview")), theme::PLAYING)
    } else if preview_searching {
        (format!(" {}", t("help.searching_deezer")), theme::ACCENT)
    } else if let Some(msg) = save_notice {
        let color = if save_notice_is_dup { theme::ACCENT } else { theme::PLAYING };
        (format!(" {msg}"), color)
    } else {
        let hint = match focus {
            AppFocus::RecentTracks  => t("help.recent"),
            AppFocus::StationSearch => t("help.station_search"),
            AppFocus::OnDemandList  => {
                if !seek_input.is_empty() {
                    format!(" {}  {seek_input}_  {}", t("help.seek_prefix"), t("help.seek_suffix"))
                } else {
                    t("help.demand.hint")
                }
            }
            AppFocus::Stations => {
                let active = matches!(status, PlayerStatus::Playing | PlayerStatus::Paused);
                if active {
                    if matches!(status, PlayerStatus::Paused) {
                        t("help.stations_paused")
                    } else {
                        t("help.stations_playing")
                    }
                } else {
                    t("help.stations_idle")
                }
            }
        };
        (format!(" {hint}"), theme::MUTED)
    };

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text, Style::default().fg(color)))),
        area,
    );
}

pub(super) fn month_es(m: u32) -> &'static str {
    ["ene","feb","mar","abr","may","jun","jul","ago","sep","oct","nov","dic"]
        [(m.saturating_sub(1) as usize).min(11)]
}
