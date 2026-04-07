use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

use crate::audio::PlayerStatus;
use crate::station::{DynamicStation, Station};
use crate::ui::theme;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const FRAME_MS: u128 = 120;

fn spinner_frame() -> &'static str {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let idx = ((ms / FRAME_MS) as usize) % SPINNER_FRAMES.len();
    SPINNER_FRAMES[idx]
}

pub struct StationListWidget<'a> {
    pub stations: &'a [Station],
    pub dynamic_stations: &'a [DynamicStation],
    pub selected: usize,
    pub playing_index: Option<usize>,
    pub playing_dynamic_index: Option<usize>,
    pub player_status: &'a PlayerStatus,
    pub search_query: &'a str,
    pub search_loading: bool,
    pub is_searching: bool,
}

impl<'a> StationListWidget<'a> {
    fn is_dynamic_selected(&self) -> bool {
        self.selected >= self.stations.len()
    }

    fn dynamic_index(&self) -> Option<usize> {
        if self.is_dynamic_selected() {
            Some(self.selected - self.stations.len())
        } else {
            None
        }
    }
}

impl<'a> Widget for StationListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let search_height = 3u16;

        let border_style = if self.is_searching {
            Style::new().fg(theme::ACCENT)
        } else {
            theme::BORDER_STYLE
        };

        let block = Block::default()
            .title(" STATIONS ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        let search_area = Rect::new(inner.x, inner.y, inner.width, search_height);
        let list_area = Rect::new(
            inner.x,
            inner.y + search_height,
            inner.width,
            inner.height.saturating_sub(search_height),
        );

        self.render_search_input(search_area, buf);

        let items: Vec<ListItem> = self
            .stations
            .iter()
            .enumerate()
            .map(|(i, station)| {
                let is_selected = !self.is_dynamic_selected() && i == self.selected;
                let is_playing = self.playing_index == Some(i);

                let prefix = if is_selected { "▶ " } else { "  " };

                let style = if is_playing {
                    theme::PLAYING_STYLE
                } else if is_selected {
                    theme::SELECTED_STYLE
                } else {
                    Style::default()
                };

                let status_tag = if is_playing {
                    match self.player_status {
                        PlayerStatus::Playing => {
                            Span::styled(" >>", Style::default().fg(theme::PLAYING))
                        }
                        PlayerStatus::Paused => {
                            Span::styled(" ⏸", Style::default().fg(theme::ACCENT))
                        }
                        PlayerStatus::Connecting => {
                            Span::styled(" …", Style::default().fg(theme::ACCENT))
                        }
                        PlayerStatus::Error(_) => {
                            Span::styled(" !", Style::default().fg(theme::ERROR))
                        }
                        PlayerStatus::Reconnecting(n) => {
                            Span::styled(format!(" {} ", n), Style::default().fg(theme::ACCENT))
                        }
                        PlayerStatus::Idle => Span::raw(""),
                    }
                } else {
                    Span::raw("")
                };

                let line = Line::from(vec![
                    Span::styled(format!("{}{}", prefix, station.name), style),
                    status_tag,
                ]);
                ListItem::new(line)
            })
            .chain(
                self.dynamic_stations
                    .iter()
                    .enumerate()
                    .map(|(i, station)| {
                        let is_selected =
                            self.is_dynamic_selected() && Some(i) == self.dynamic_index();
                        let is_playing = self.playing_dynamic_index == Some(i);

                        let prefix = if is_selected { "▶ " } else { "  " };

                        let style = if is_playing {
                            theme::PLAYING_STYLE
                        } else if is_selected {
                            theme::SELECTED_STYLE
                        } else {
                            Style::default().fg(theme::MUTED)
                        };

                        let bitrate_tag = station
                            .bitrate_kbps
                            .map(|br| {
                                Span::styled(
                                    format!(" [{}k]", br),
                                    Style::default().fg(theme::MUTED),
                                )
                            })
                            .unwrap_or(Span::raw(""));

                        let status_tag = if is_playing {
                            match self.player_status {
                                PlayerStatus::Playing => {
                                    Span::styled(" >>", Style::default().fg(theme::PLAYING))
                                }
                                PlayerStatus::Paused => {
                                    Span::styled(" ⏸", Style::default().fg(theme::ACCENT))
                                }
                                PlayerStatus::Connecting => {
                                    Span::styled(" …", Style::default().fg(theme::ACCENT))
                                }
                                PlayerStatus::Error(_) => {
                                    Span::styled(" !", Style::default().fg(theme::ERROR))
                                }
                                PlayerStatus::Reconnecting(n) => Span::styled(
                                    format!(" {} ", n),
                                    Style::default().fg(theme::ACCENT),
                                ),
                                PlayerStatus::Idle => Span::raw(""),
                            }
                        } else {
                            Span::raw("")
                        };

                        let line = Line::from(vec![
                            Span::styled(format!("{}{}", prefix, station.name), style),
                            bitrate_tag,
                            status_tag,
                        ]);
                        ListItem::new(line)
                    }),
            )
            .collect();

        List::new(items).render(list_area, buf);
    }
}

impl<'a> StationListWidget<'a> {
    fn render_search_input(&self, area: Rect, buf: &mut Buffer) {
        let spinner = spinner_frame();

        let line = if self.is_searching {
            // Modo activo: muestra la query con cursor
            let mut spans = vec![
                Span::styled(" / ", Style::default().fg(theme::ACCENT)),
                Span::styled(
                    self.search_query,
                    Style::default().fg(theme::HIGHLIGHT),
                ),
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
            // Modo inactivo: muestra el hint
            Line::from(vec![
                Span::styled(" / ", Style::default().fg(theme::MUTED)),
                Span::styled("buscar estaciones", Style::default().fg(theme::MUTED)),
            ])
        };

        Paragraph::new(line)
            .alignment(Alignment::Left)
            .render(area, buf);

        // Separador
        let sep_color = if self.is_searching { theme::ACCENT } else { theme::MUTED };
        let sep = Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(sep_color),
        ));
        Paragraph::new(sep)
            .alignment(Alignment::Left)
            .render(Rect::new(area.x, area.y + 1, area.width, 1), buf);
    }
}
