use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::ui::theme;

const DB_MIN:  f32 = -60.0;
const DB_CLIP: f32 = -1.0;

const SPECTRUM: [Color; 8] = [
    Color::Rgb(0,   240, 255),
    Color::Rgb(40,  160, 255),
    Color::Rgb(75,  80,  255),
    Color::Rgb(112, 0,   255),
    Color::Rgb(160, 0,   200),
    Color::Rgb(200, 0,   140),
    Color::Rgb(235, 0,   100),
    Color::Rgb(255, 0,   85),
];

fn spectral_bar_spans(filled: usize, total: usize) -> Vec<Span<'static>> {
    if filled == 0 { return vec![]; }
    let n = total.max(1);
    let color_of = |i: usize| SPECTRUM[(i * 7 / n.saturating_sub(1).max(1)).min(7)];
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(8);
    let mut seg_start = 0usize;
    let mut prev = color_of(0);
    for i in 1..filled {
        let c = color_of(i);
        if c != prev {
            spans.push(Span::styled("█".repeat(i - seg_start), Style::default().fg(prev)));
            seg_start = i;
            prev = c;
        }
    }
    let tail = filled - seg_start;
    if tail > 0 {
        spans.push(Span::styled("█".repeat(tail), Style::default().fg(prev)));
    }
    spans
}

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
    if w <= FIXED {
        return Line::from(vec![Span::styled(
            format!("{:.0}%", volume * 100.0),
            Style::default().fg(theme::ACCENT),
        )]);
    }
    let bars = w.saturating_sub(FIXED);
    let bar_lvl_w = bars * 3 / 5;
    let bar_vol_w = bars.saturating_sub(bar_lvl_w);

    let db        = level_db.clamp(DB_MIN, 0.0);
    let ratio_lvl = (db - DB_MIN) / -DB_MIN;
    let filled_lvl = (ratio_lvl * bar_lvl_w as f32) as usize;
    let db_label   = format!(" {:>4.0}dB", db);

    let ratio_vol  = volume.clamp(0.0, 1.0);
    let filled_vol = (ratio_vol * bar_vol_w as f32) as usize;
    let vol_pct    = (ratio_vol * 100.0).round() as u32;
    let vol_label  = format!("  {:>3}%", vol_pct);
    let vol_color = if volume > 0.85 { Color::Yellow } else { theme::ACCENT };

    let mut spans: Vec<Span<'static>> = vec![
        Span::styled(" LVL ", Style::default().fg(theme::MUTED)),
    ];
    spans.extend(spectral_bar_spans(filled_lvl, bar_lvl_w));
    if db >= DB_CLIP && filled_lvl > 0 {
        if let Some(last) = spans.last_mut() {
            *last = Span::styled(last.content.clone(), Style::default().fg(Color::Red));
        }
    }
    spans.push(Span::styled("░".repeat(bar_lvl_w.saturating_sub(filled_lvl)), Style::default().fg(theme::MUTED)));
    spans.push(Span::styled(db_label, Style::default().fg(theme::MUTED)));
    spans.push(Span::styled("   VOL ", Style::default().fg(theme::MUTED)));
    spans.push(Span::styled("█".repeat(filled_vol),                           Style::default().fg(vol_color)));
    spans.push(Span::styled("░".repeat(bar_vol_w.saturating_sub(filled_vol)), Style::default().fg(theme::MUTED)));
    spans.push(Span::styled(vol_label, Style::default().fg(theme::ACCENT)));
    Line::from(spans)
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
