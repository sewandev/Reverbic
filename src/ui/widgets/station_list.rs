use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget},
};

use crate::audio::PlayerStatus;
use crate::favorites::FavoriteStation;
use crate::i18n::t;
use crate::station::{DynamicStation, Station};
use crate::ui::theme;

pub struct StationListWidget<'a> {
    pub stations:               &'a [Station],
    pub dynamic_stations:       &'a [DynamicStation],
    pub favorites:              &'a [FavoriteStation],
    pub selected:               usize,
    pub playing_index:          Option<usize>,
    pub playing_dynamic_index:  Option<usize>,
    pub playing_favorite_index: Option<usize>,
    pub player_status:          &'a PlayerStatus,
    pub search_query:           &'a str,
    pub search_loading:         bool,
    pub is_searching:           bool,
    pub flash_index:            Option<usize>,
}

impl<'a> StationListWidget<'a> {
    fn fav_len(&self) -> usize { self.favorites.len() }
    fn sta_len(&self) -> usize { self.stations.len() }

    fn is_dynamic_selected(&self) -> bool {
        self.selected >= self.fav_len() + self.sta_len()
    }

    fn dynamic_index(&self) -> Option<usize> {
        if self.is_dynamic_selected() {
            Some(self.selected - self.fav_len() - self.sta_len())
        } else {
            None
        }
    }
}

impl<'a> Widget for StationListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        const SEARCH_H: u16 = 2;
        let search_area = Rect::new(area.x, area.y, area.width, SEARCH_H.min(area.height));
        let list_area   = Rect::new(
            area.x,
            area.y + SEARCH_H,
            area.width,
            area.height.saturating_sub(SEARCH_H),
        );

        self.render_search(search_area, buf);

        let fav_len = self.fav_len();

        const FLASH_STYLE: Style = Style::new()
            .fg(Color::Black)
            .bg(theme::ACCENT)
            .add_modifier(Modifier::BOLD);

        let (header_area, list_area) = if !self.favorites.is_empty() {
            let h = Rect::new(list_area.x, list_area.y, list_area.width, 1);
            let l = Rect::new(
                list_area.x,
                list_area.y + 1,
                list_area.width,
                list_area.height.saturating_sub(1),
            );
            (Some(h), l)
        } else {
            (None, list_area)
        };

        if let Some(h) = header_area {
            Paragraph::new(Line::from(vec![
                Span::styled("★ ", Style::default().fg(theme::ACCENT)),
                Span::styled("Favoritas", Style::default().fg(theme::MUTED)),
                Span::styled(
                    format!(" {}", "─".repeat(h.width.saturating_sub(12) as usize)),
                    Style::default().fg(theme::DIM),
                ),
            ]))
            .style(Style::default().bg(theme::PANEL_BG))
            .render(h, buf);
        }

        let fav_items = self.favorites.iter().enumerate().map(|(i, fav)| {
            let is_sel     = i == self.selected;
            let is_playing = self.playing_favorite_index == Some(i);
            let is_flash   = self.flash_index == Some(i);
            let style      = if is_flash {
                FLASH_STYLE
            } else if is_sel {
                theme::SELECTED_STYLE
            } else if is_playing {
                theme::PLAYING_STYLE
            } else {
                Style::new().fg(theme::HIGHLIGHT)
            };
            let (star, star_style, name_style) = if is_sel || is_flash {
                ("▶", style, style)
            } else if is_playing {
                ("▶", theme::PLAYING_STYLE, theme::PLAYING_STYLE)
            } else {
                ("★", Style::new().fg(theme::ACCENT), Style::new().fg(theme::HIGHLIGHT))
            };
            let status_tag = if is_playing { status_span(self.player_status) } else { Span::raw("") };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{star} "), star_style),
                Span::styled(fav.name.clone(), name_style),
                status_tag,
            ]))
        });

        let station_items = self.stations.iter().enumerate().map(|(i, station)| {
            let abs_i      = fav_len + i;
            let is_sel     = !self.is_dynamic_selected() && abs_i == self.selected;
            let is_playing = self.playing_index == Some(i);
            let is_flash   = self.flash_index == Some(abs_i);
            let prefix     = if is_sel || is_flash { "▶ " } else { "  " };
            let style      = if is_flash {
                FLASH_STYLE
            } else if is_playing {
                theme::PLAYING_STYLE
            } else if is_sel {
                theme::SELECTED_STYLE
            } else {
                Style::default()
            };
            let status_tag = if is_playing { status_span(self.player_status) } else { Span::raw("") };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{prefix}{}", station.name), style),
                status_tag,
            ]))
        });

        let dynamic_items = self.dynamic_stations.iter().enumerate().map(|(i, s)| {
            let is_sel     = self.is_dynamic_selected() && Some(i) == self.dynamic_index();
            let is_playing = self.playing_dynamic_index == Some(i);
            let prefix     = if is_sel { "▶ " } else { "  " };
            let style      = if is_playing {
                theme::PLAYING_STYLE
            } else if is_sel {
                theme::SELECTED_STYLE
            } else {
                Style::default().fg(theme::MUTED)
            };
            let bitrate_tag = s.bitrate_kbps
                .map(|br| Span::styled(format!(" [{}k]", br), Style::default().fg(theme::MUTED)))
                .unwrap_or(Span::raw(""));
            let status_tag = if is_playing { status_span(self.player_status) } else { Span::raw("") };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{prefix}{}", s.name), style),
                bitrate_tag,
                status_tag,
            ]))
        });

        let items: Vec<ListItem> = fav_items.chain(station_items).chain(dynamic_items).collect();
        List::new(items).render(list_area, buf);
    }
}

fn status_span(status: &PlayerStatus) -> Span<'static> {
    match status {
        PlayerStatus::Playing            => Span::styled(" >>>", Style::default().fg(theme::PLAYING)),
        PlayerStatus::Paused             => Span::styled(" ⏸",   Style::default().fg(theme::ACCENT)),
        PlayerStatus::Connecting
        | PlayerStatus::Buffering(_)     => Span::styled(
            format!(" {}", super::spinner_frame()),
            Style::default().fg(theme::ACCENT),
        ),
        PlayerStatus::Error(_)           => Span::styled(" ✕",   Style::default().fg(theme::DANGER)),
        PlayerStatus::Reconnecting(n)    => Span::styled(
            format!(" ↻{n}"),
            Style::default().fg(theme::WARNING),
        ),
        PlayerStatus::Idle               => Span::raw(""),
    }
}

impl<'a> StationListWidget<'a> {
    fn render_search(&self, area: Rect, buf: &mut Buffer) {
        let spinner = super::spinner_frame();

        let input_line = if self.is_searching {
            let mut spans = vec![
                Span::styled(" / ", Style::default().fg(theme::ACCENT)),
                Span::styled(self.search_query, Style::default().fg(theme::HIGHLIGHT)),
            ];
            if self.search_loading {
                spans.push(Span::styled(
                    format!(" {spinner}"),
                    Style::default().fg(theme::ACCENT),
                ));
            } else {
                spans.push(Span::styled("█", Style::default().fg(theme::ACCENT)));
            }
            Line::from(spans)
        } else {
            Line::from(vec![
                Span::styled(" / ", Style::default().fg(theme::MUTED)),
                Span::styled(t("status.search_placeholder"), Style::default().fg(theme::MUTED)),
            ])
        };

        Paragraph::new(input_line).render(
            Rect::new(area.x, area.y, area.width, 1),
            buf,
        );

        if area.height >= 2 {
            let sep_color = if self.is_searching { theme::ACCENT } else { theme::MUTED };
            Paragraph::new(Line::from(Span::styled(
                "─".repeat(area.width as usize),
                Style::default().fg(sep_color),
            )))
            .render(Rect::new(area.x, area.y + 1, area.width, 1), buf);
        }
    }
}
