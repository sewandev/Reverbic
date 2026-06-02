use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::app::{SettingItem, settings_items};
use crate::i18n::{current_language, t, Language};
use crate::ui::theme;

use super::helpers::screensaver_display;
use super::SearchModalWidget;

impl<'a> SearchModalWidget<'a> {
    pub(super) fn item_value(&self, item: SettingItem) -> String {
        let on  = t("config.value.on");
        let off = t("config.value.off");
        match item {
            SettingItem::Autoplay        => if self.autoplay_last  { on } else { off },
            SettingItem::RestoreVolume   => if self.restore_volume { on } else { off },
            SettingItem::Crossfade       => self.crossfade.clone(),
            SettingItem::VolumeStep      => format!("{}%", self.volume_step),
            SettingItem::Prebuffer       => format!("{}s", self.prebuffer_secs),
            SettingItem::OverlayMode     => self.overlay_mode.clone(),
            SettingItem::OverlayAlpha    => format!("{}%", self.overlay_alpha),
            SettingItem::OverlayPosition => self.overlay_position.clone(),
            SettingItem::Screensaver     => screensaver_display(self.screensaver_secs),
            SettingItem::DuckEnabled     => if self.duck_enabled { on } else { off },
            SettingItem::DuckVolume      => format!("{}%", self.duck_volume),
            SettingItem::MediaKeys       => if self.media_keys    { on } else { off },
            SettingItem::TrayIcon        => if self.tray_icon     { on } else { off },
            SettingItem::Notifications   => if self.notifications { on } else { off },
            SettingItem::Language        => match current_language() {
                Language::Es => t("lang.display.es"),
                Language::En => t("lang.display.en"),
            },
        }
    }

    pub(super) fn render_settings_body(&self, area: Rect, content_x: u16, content_w: u16, buf: &mut Buffer) {
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
                if item_idx == self.settings_selected { selected_row = ri; }
                item_idx += 1;
            }
        }

        let list_x = content_x + 2;
        let list_w = content_w.saturating_sub(2);
        let [_gap, items_area, tooltip_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
        ]).areas(area);

        let visible_n = items_area.height.saturating_sub(1) as usize;
        let offset    = super::super::scroll_offset(selected_row, visible_n);

        item_idx = 0;
        for (ri, (label, val_opt)) in rows.iter().enumerate() {
            let display_y = ri as isize - offset as isize;
            if display_y < 0 || display_y >= visible_n as isize {
                if val_opt.is_some() { item_idx += 1; }
                continue;
            }
            let y = items_area.y + display_y as u16;

            if let Some(value) = val_opt {
                let active = item_idx == self.settings_selected;
                let (label_st, val_st) = if active {
                    (Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                     Style::default().fg(theme::PLAYING).add_modifier(Modifier::BOLD))
                } else {
                    (Style::default().fg(theme::HIGHLIGHT),
                     Style::default().fg(theme::MUTED))
                };
                let prefix = if active { "▶  " } else { "   " };
                Paragraph::new(Line::from(vec![
                    Span::styled(prefix,        label_st),
                    Span::styled(label.clone(), label_st),
                    Span::styled("  [",         Style::default().fg(theme::MUTED)),
                    Span::styled(value.clone(), val_st),
                    Span::styled("]",           Style::default().fg(theme::MUTED)),
                ])).render(Rect::new(list_x, y, list_w, 1), buf);
                item_idx += 1;
            } else {
                Paragraph::new(Span::styled(
                    format!("── {} ", label),
                    Style::default().fg(theme::MUTED),
                )).render(Rect::new(list_x, y, list_w, 1), buf);
            }
        }
        let total_rows = rows.len();
        if total_rows > visible_n {
            let scroll_area = Rect::new(list_x, items_area.y, list_w, items_area.height);
            self.render_scrollbar(scroll_area, total_rows, selected_row, buf);
        }

        let sep = "─".repeat(content_w as usize);
        Paragraph::new(Span::styled(sep, Style::default().fg(theme::DIM)))
            .render(Rect::new(content_x, tooltip_area.y, content_w, 1), buf);

        let tooltip = settings_items(self.duck_enabled)
            .get(self.settings_selected)
            .map(|item| t(item.tooltip_key()))
            .unwrap_or_default();
        Paragraph::new(tooltip)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(theme::MUTED))
            .render(Rect::new(list_x, tooltip_area.y + 1, list_w, 2), buf);
    }
}
