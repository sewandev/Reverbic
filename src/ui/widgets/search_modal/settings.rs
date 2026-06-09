use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::app::{settings_items, SettingItem};
use crate::i18n::{current_language, t, Language};
use crate::ui::strings::screensaver_display;
use crate::ui::widgets::scroll_offset_for_selection;

use super::settings_layout;
use super::SearchModalWidget;

impl<'a> SearchModalWidget<'a> {
    pub(super) fn item_value(&self, item: SettingItem) -> String {
        let on = t("config.value.on");
        let off = t("config.value.off");
        match item {
            SettingItem::Autoplay => {
                if self.autoplay_last {
                    on
                } else {
                    off
                }
            }
            SettingItem::RestoreVolume => {
                if self.restore_volume {
                    on
                } else {
                    off
                }
            }
            SettingItem::Crossfade => self.crossfade.clone(),
            SettingItem::VolumeStep => format!("{}%", self.volume_step),
            SettingItem::Prebuffer => format!("{}s", self.prebuffer_secs),
            SettingItem::OverlayMode => self.overlay_mode.clone(),
            SettingItem::OverlayStyle => self.overlay_style.clone(),
            SettingItem::OverlayAlpha => format!("{}%", self.overlay_alpha),
            SettingItem::OverlayPosition => self.overlay_position.clone(),
            SettingItem::Screensaver => screensaver_display(self.screensaver_secs),
            SettingItem::ScreensaverClock => {
                if self.screensaver_clock {
                    on
                } else {
                    off
                }
            }
            SettingItem::DuckEnabled => {
                if self.duck_enabled {
                    on
                } else {
                    off
                }
            }
            SettingItem::DuckVolume => format!("{}%", self.duck_volume),
            SettingItem::MediaKeys => {
                if self.media_keys {
                    on
                } else {
                    off
                }
            }
            SettingItem::TrayIcon => {
                if self.tray_icon {
                    on
                } else {
                    off
                }
            }
            SettingItem::Notifications => {
                if self.notifications {
                    on
                } else {
                    off
                }
            }
            SettingItem::Language => match current_language() {
                Language::Es => t("lang.display.es"),
                Language::En => t("lang.display.en"),
            },
            SettingItem::Theme => self.theme.display(),
            SettingItem::SpotifyStopOnQuit => {
                if self.spotify_stop_on_quit {
                    on
                } else {
                    off
                }
            }
            SettingItem::SpotifyStartOnSpotify => {
                if self.spotify_start_on_spotify {
                    on
                } else {
                    off
                }
            }
            SettingItem::SpotifyClientId => {
                if self.spotify_client_id.is_empty() {
                    t("config.spotify.not_set")
                } else {
                    let preview: String = self.spotify_client_id.chars().take(8).collect();
                    format!("{}...", preview)
                }
            }
            SettingItem::SpotifyPlaybackMode => self.spotify_playback_mode.clone(),
            SettingItem::SpotifyRadioMode => {
                if self.spotify_radio_mode {
                    on
                } else {
                    off
                }
            }
            SettingItem::AutoUpdate => {
                if self.auto_update {
                    on
                } else {
                    off
                }
            }
            SettingItem::DiscordRpc => {
                if self.discord_rpc {
                    on
                } else {
                    off
                }
            }
            SettingItem::ReplayOnboarding => t("hint.open"),
        }
    }

    fn is_on_value(value: &str) -> bool {
        value == t("config.value.on").as_str()
    }

    pub(super) fn render_settings_body(
        &self,
        area: Rect,
        content_x: u16,
        content_w: u16,
        buf: &mut Buffer,
    ) {
        let mut rows: Vec<(String, Option<String>)> = Vec::new();
        let mut last_group = "";
        for item in settings_items(self.duck_enabled) {
            let gk = item.group_key();
            if gk != last_group {
                rows.push((t(gk), None));
                last_group = gk;
            }
            rows.push((item.label(), Some(self.item_value(item))));
        }

        let mut item_idx = 0usize;
        let mut selected_row = 0usize;
        for (ri, (_, val)) in rows.iter().enumerate() {
            if val.is_some() {
                if item_idx == self.settings_selected {
                    selected_row = ri;
                }
                item_idx += 1;
            }
        }

        let list_x = content_x + 2;
        let list_w = content_w.saturating_sub(2);

        let layout = settings_layout(area);
        let items_area = layout.items;
        let tooltip_area = layout.tooltip;

        let visible_n = items_area.height.saturating_sub(1) as usize;
        let offset =
            scroll_offset_for_selection(selected_row, visible_n, self.settings_scroll_offset);
        let val_col_w: u16 = 16;
        let lbl_col_w: u16 = list_w.saturating_sub(3 + val_col_w);

        item_idx = 0;
        for (ri, (label, val_opt)) in rows.iter().enumerate() {
            let display_y = ri as isize - offset as isize;
            if display_y < 0 || display_y >= visible_n as isize {
                if val_opt.is_some() {
                    item_idx += 1;
                }
                continue;
            }
            let y = items_area.y + display_y as u16;

            if let Some(value) = val_opt {
                let active = item_idx == self.settings_selected;

                let prefix = if active { "▶  " } else { "   " };

                let label_str = crate::ui::strings::truncate(label, lbl_col_w as usize);
                let label_chars = label_str.chars().count() as u16;
                let padding = lbl_col_w.saturating_sub(label_chars) as usize;
                let label_st = if active {
                    Style::default()
                        .fg(self.palette.radio_accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.palette.highlight)
                };
                let is_on = Self::is_on_value(value);
                let is_off = value == t("config.value.off").as_str();
                let val_st = if active {
                    Style::default()
                        .fg(self.palette.playing)
                        .add_modifier(Modifier::BOLD)
                } else if is_on {
                    Style::default().fg(self.palette.playing)
                } else if is_off {
                    Style::default().fg(self.palette.muted)
                } else {
                    Style::default().fg(self.palette.accent)
                };

                Paragraph::new(Line::from(vec![
                    Span::styled(prefix, label_st),
                    Span::styled(label_str, label_st),
                    Span::styled(" ".repeat(padding), Style::default()),
                    Span::styled(value.clone(), val_st),
                ]))
                .render(Rect::new(list_x, y, list_w, 1), buf);
                item_idx += 1;
            } else {
                let label_upper = label.to_uppercase();
                let label_chars = label_upper.chars().count() as u16;
                let right_dashes = list_w.saturating_sub(3 + label_chars + 1) as usize;
                Paragraph::new(Line::from(vec![
                    Span::styled("── ", Style::default().fg(self.palette.dim)),
                    Span::styled(
                        label_upper,
                        Style::default()
                            .fg(self.palette.muted)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {}", "─".repeat(right_dashes)),
                        Style::default().fg(self.palette.dim),
                    ),
                ]))
                .render(Rect::new(list_x, y, list_w, 1), buf);
            }
        }

        if rows.len() > visible_n {
            let scroll_area = Rect::new(list_x, items_area.y, list_w, items_area.height);
            self.render_scrollbar(scroll_area, rows.len(), selected_row, buf);
        }
        let sep = "─".repeat(content_w as usize);
        Paragraph::new(Span::styled(sep, Style::default().fg(self.palette.dim)))
            .render(Rect::new(content_x, tooltip_area.y, content_w, 1), buf);

        let tooltip = settings_items(self.duck_enabled)
            .get(self.settings_selected)
            .map(|item| t(item.tooltip_key()))
            .unwrap_or_default();
        Paragraph::new(tooltip)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(self.palette.dim))
            .render(Rect::new(list_x, tooltip_area.y + 1, list_w, 2), buf);
    }
}
