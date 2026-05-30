use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::ui::theme;

const DB_MIN:  f32 = -60.0;
const DB_WARN: f32 = -6.0;
const DB_CLIP: f32 = -1.0;

pub struct VuMeterWidget {
    pub level_db:        f32,
    pub volume:          f32,
    pub buffer_fill_pct: Option<f32>,
}

impl Widget for VuMeterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width < 10 {
            return;
        }
        let line = if let Some(pct) = self.buffer_fill_pct {
            build_buffer_line(pct, self.volume, area.width)
        } else {
            build_lvl_vol_line(self.level_db, self.volume, area.width)
        };
        Paragraph::new(line).render(area, buf);
    }
}

fn build_lvl_vol_line(level_db: f32, volume: f32, width: u16) -> Line<'static> {
    let w = width as usize;
    const FIXED: usize = 23;
    let bars = w.saturating_sub(FIXED);
    let bar_lvl_w = bars * 3 / 5;
    let bar_vol_w = bars.saturating_sub(bar_lvl_w);

    let db        = level_db.clamp(DB_MIN, 0.0);
    let ratio_lvl = (db - DB_MIN) / -DB_MIN;
    let filled_lvl = (ratio_lvl * bar_lvl_w as f32) as usize;
    let db_label   = format!(" {:>4.0}dB", db);

    let lvl_color = if db >= DB_CLIP {
        Color::Red
    } else if db >= DB_WARN {
        Color::Yellow
    } else {
        theme::PLAYING
    };

    let ratio_vol  = volume.clamp(0.0, 1.0);
    let filled_vol = (ratio_vol * bar_vol_w as f32) as usize;
    let vol_pct    = (ratio_vol * 100.0).round() as u32;
    let vol_label  = format!("  {:>3}%", vol_pct);
    let vol_color  = if volume > 0.85 { Color::Yellow } else { theme::ACCENT };

    Line::from(vec![
        Span::styled(" LVL ", Style::default().fg(theme::MUTED)),
        Span::styled("█".repeat(filled_lvl),                      Style::default().fg(lvl_color)),
        Span::styled("░".repeat(bar_lvl_w.saturating_sub(filled_lvl)), Style::default().fg(theme::MUTED)),
        Span::styled(db_label, Style::default().fg(theme::MUTED)),
        Span::styled("   VOL ", Style::default().fg(theme::MUTED)),
        Span::styled("█".repeat(filled_vol),                      Style::default().fg(vol_color)),
        Span::styled("░".repeat(bar_vol_w.saturating_sub(filled_vol)), Style::default().fg(theme::MUTED)),
        Span::styled(vol_label, Style::default().fg(theme::ACCENT)),
    ])
}

fn build_buffer_line(pct: f32, volume: f32, width: u16) -> Line<'static> {
    let spinner = super::spinner_frame();
    let w = width as usize;
    const FIXED: usize = 25;
    let bars      = w.saturating_sub(FIXED);
    let bar_buf_w = bars * 3 / 5;
    let bar_vol_w = bars.saturating_sub(bar_buf_w);

    let buf_fill   = pct.clamp(0.0, 1.0);
    let filled_buf = (buf_fill * bar_buf_w as f32) as usize;
    let pct_label  = format!("  {:.0}%", buf_fill * 100.0);

    let ratio_vol  = volume.clamp(0.0, 1.0);
    let filled_vol = (ratio_vol * bar_vol_w as f32) as usize;
    let vol_pct    = (ratio_vol * 100.0).round() as u32;
    let vol_label  = format!("  {:>3}%", vol_pct);

    Line::from(vec![
        Span::styled(format!(" BUF {spinner} "), Style::default().fg(theme::ACCENT)),
        Span::styled("█".repeat(filled_buf),                       Style::default().fg(theme::ACCENT)),
        Span::styled("░".repeat(bar_buf_w.saturating_sub(filled_buf)), Style::default().fg(theme::MUTED)),
        Span::styled(pct_label, Style::default().fg(theme::ACCENT)),
        Span::styled("   VOL ", Style::default().fg(theme::MUTED)),
        Span::styled("█".repeat(filled_vol),                       Style::default().fg(theme::ACCENT)),
        Span::styled("░".repeat(bar_vol_w.saturating_sub(filled_vol)), Style::default().fg(theme::MUTED)),
        Span::styled(vol_label, Style::default().fg(theme::ACCENT)),
    ])
}
