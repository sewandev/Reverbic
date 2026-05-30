use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::ui::theme;

const W1_START: u64 = 1_784_246_400;
const W1_END:   u64 = 1_784_505_600;
const W2_START: u64 = 1_784_851_200;
const W2_END:   u64 = 1_785_110_400;

enum FestivalStatus {
    Upcoming { days: u64, weekend: u8 },
    Live { weekend: u8 },
    Over,
}

fn festival_status() -> FestivalStatus {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if now < W1_START {
        FestivalStatus::Upcoming { days: (W1_START - now) / 86_400, weekend: 1 }
    } else if now < W1_END {
        FestivalStatus::Live { weekend: 1 }
    } else if now < W2_START {
        FestivalStatus::Upcoming { days: (W2_START - now) / 86_400, weekend: 2 }
    } else if now < W2_END {
        FestivalStatus::Live { weekend: 2 }
    } else {
        FestivalStatus::Over
    }
}

pub struct CountdownWidget;

impl Widget for CountdownWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        let (text, style) = match festival_status() {
            FestivalStatus::Upcoming { days, weekend } => (
                format!(" ✦ TOMORROWLAND 2026  ·  {days} días  ·  Fin de semana {weekend}"),
                Style::new().fg(theme::FESTIVAL_ACCENT).add_modifier(Modifier::BOLD),
            ),
            FestivalStatus::Live { weekend } => (
                format!(" ✦ ESTO ES TOMORROWLAND  —  FIN DE SEMANA {weekend}"),
                Style::new().fg(theme::PLAYING).add_modifier(Modifier::BOLD),
            ),
            FestivalStatus::Over => (
                " ✦ El próximo Tomorrowland está por anunciarse".to_string(),
                Style::new().fg(theme::MUTED),
            ),
        };

        Paragraph::new(Line::from(Span::styled(text, style)))
            .alignment(Alignment::Left)
            .render(area, buf);
    }
}
