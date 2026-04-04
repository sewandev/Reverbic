
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::station::Station;
use crate::audio::PlayerStatus;
use crate::ui::theme;

pub struct StationListWidget<'a> {
    pub stations:        &'a [Station],
    pub selected:        usize,
    pub playing_index:   Option<usize>,
    pub player_status:   &'a PlayerStatus,
}

impl<'a> Widget for StationListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .stations
            .iter()
            .enumerate()
            .map(|(i, station)| {
                let is_selected = i == self.selected;
                let is_playing  = self.playing_index == Some(i);

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
                        PlayerStatus::Playing    => Span::styled(" >>", Style::default().fg(theme::PLAYING)),
                        PlayerStatus::Paused     => Span::styled(" ⏸", Style::default().fg(theme::ACCENT)),
                        PlayerStatus::Connecting => Span::styled(" …", Style::default().fg(theme::ACCENT)),
                        PlayerStatus::Error(_)   => Span::styled(" !", Style::default().fg(theme::ERROR)),
                        PlayerStatus::Idle       => Span::raw(""),
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
            .collect();

        let block = Block::default()
            .title(" STATIONS ")
            .borders(Borders::ALL)
            .border_style(theme::BORDER_STYLE);

        List::new(items).block(block).render(area, buf);
    }
}
