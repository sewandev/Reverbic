use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::ui::theme;

const DB_MIN: f32 = -60.0;
const DB_MAX: f32 = 0.0;
const DB_WARN: f32 = -6.0;
const DB_CLIP: f32 = -1.0;

pub struct VuMeterWidget {
    pub level_db: f32,
    pub volume: f32,
    /// Some(pct) durante buffering (0.0-1.0); reemplaza la barra LVL
    pub buffer_fill_pct: Option<f32>,
}

impl Widget for VuMeterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = if self.buffer_fill_pct.is_some() {
            " BUFFER  aguarda unos segundos... "
        } else {
            " AUDIO "
        };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(theme::BORDER_STYLE);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 8 || inner.height == 0 {
            return;
        }

        let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

        if let Some(pct) = self.buffer_fill_pct {
            render_buffer_bar(pct, rows[0], buf);
        } else {
            render_level_bar(self.level_db, rows[0], buf);
        }

        if inner.height >= 2 {
            render_volume_bar(self.volume, rows[1], buf);
        }
    }
}

fn render_buffer_bar(pct: f32, area: Rect, buf: &mut Buffer) {
    let spinner = super::spinner_frame();
    let label = format!(" {:>3.0}%", (pct * 100.0).min(100.0));
    // "BUF ⠹ " — spinner es 1 col, total prefijo: 6 cols
    let prefix = format!("BUF {} ", spinner);
    let prefix_display_len = 6; // "BUF " (4) + spinner (1) + " " (1)
    let bar_width = (area.width as usize)
        .saturating_sub(label.len() + prefix_display_len)
        .max(1);

    let filled = ((pct * bar_width as f32).round() as usize).min(bar_width);
    let empty = bar_width - filled;

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(theme::ACCENT)),
        Span::styled("█".repeat(filled), Style::default().fg(theme::ACCENT)),
        Span::styled("░".repeat(empty), Style::default().fg(theme::MUTED)),
        Span::styled(label, Style::default().fg(theme::ACCENT)),
    ]);
    Paragraph::new(line).render(area, buf);
}

fn render_level_bar(level_db: f32, area: Rect, buf: &mut Buffer) {
    let label = format!(" {:>4.0} dB", level_db.clamp(DB_MIN, DB_MAX));
    let prefix = "LVL ";
    let bar_width = (area.width as usize)
        .saturating_sub(label.len())
        .saturating_sub(prefix.len())
        .max(1);

    let db = level_db.clamp(DB_MIN, DB_MAX);
    let ratio = (db - DB_MIN) / (DB_MAX - DB_MIN);
    let filled = (ratio * bar_width as f32).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    let bar_color = if db >= DB_CLIP {
        Color::Red
    } else if db >= DB_WARN {
        Color::Yellow
    } else {
        theme::PLAYING
    };

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(theme::MUTED)),
        Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
        Span::styled("░".repeat(empty), Style::default().fg(theme::MUTED)),
        Span::styled(label, Style::default().fg(theme::ACCENT)),
    ]);
    Paragraph::new(line).render(area, buf);
}

fn render_volume_bar(volume: f32, area: Rect, buf: &mut Buffer) {
    let pct = (volume * 100.0).round() as u32;
    let label = format!(" {:>3}%", pct);
    let prefix = "VOL ";
    let bar_width = (area.width as usize)
        .saturating_sub(label.len())
        .saturating_sub(prefix.len())
        .max(1);

    let filled = (volume.clamp(0.0, 1.0) * bar_width as f32).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    let vol_color = if volume > 0.85 {
        Color::Yellow
    } else {
        theme::ACCENT
    };

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(theme::MUTED)),
        Span::styled("█".repeat(filled), Style::default().fg(vol_color)),
        Span::styled("░".repeat(empty), Style::default().fg(theme::MUTED)),
        Span::styled(label, Style::default().fg(theme::ACCENT)),
    ]);
    Paragraph::new(line).render(area, buf);
}
