use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

const UNICODE_VISUALIZER_GLYPHS: [char; 8] = [
    '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}',
];
const ASCII_VISUALIZER_GLYPHS: [char; 8] = ['.', ':', '-', '=', '+', '*', '#', '@'];

fn visualizer_glyphs() -> &'static [char; 8] {
    visualizer_glyphs_for_legacy_console(uses_legacy_windows_console())
}

fn visualizer_glyphs_for_legacy_console(legacy_console: bool) -> &'static [char; 8] {
    if legacy_console {
        &ASCII_VISUALIZER_GLYPHS
    } else {
        &UNICODE_VISUALIZER_GLYPHS
    }
}

fn uses_legacy_windows_console() -> bool {
    cfg!(windows)
        && std::env::var_os("WT_SESSION").is_none()
        && std::env::var_os("TERM_PROGRAM").is_none()
        && std::env::var_os("TERM").is_none()
        && std::env::var_os("ConEmuANSI").is_none()
        && std::env::var_os("ANSICON").is_none()
}

use crate::ui::theme::Palette;

#[derive(Clone, Copy, PartialEq)]
pub enum AudioSource {
    Live(f32),
    Simulated,
}

pub struct VisualizerWidget<'a> {
    source: AudioSource,
    bg: ratatui::style::Color,
    palette: &'a Palette,
}

impl<'a> VisualizerWidget<'a> {
    pub fn new(source: AudioSource, bg: ratatui::style::Color, palette: &'a Palette) -> Self {
        Self {
            source,
            bg,
            palette,
        }
    }

    pub fn into_spans(self, width: usize) -> Vec<Span<'static>> {
        visualizer_spans(self.source, width, self.bg, self.palette)
    }

    pub fn into_line(self, width: usize) -> Line<'static> {
        Line::from(self.into_spans(width))
    }
}

impl<'a> Widget for VisualizerWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let line = self.into_line(area.width as usize);
        let p = ratatui::widgets::Paragraph::new(line);
        p.render(area, buf);
    }
}

pub(crate) fn visualizer_spans(
    source: AudioSource,
    width: usize,
    bg: ratatui::style::Color,
    palette: &Palette,
) -> Vec<Span<'static>> {
    let glyphs = visualizer_glyphs();
    let level_db = match source {
        AudioSource::Live(db) => db,
        AudioSource::Simulated => -60.0,
    };

    let base = ((level_db + 60.0) / 60.0).clamp(0.0, 1.0) as f64;
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0) as f64;

    if width == 0 {
        return vec![];
    }
    let n_bars = (width / 2).max(1);
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(n_bars * 2);
    for i in 0..n_bars {
        let freq = 0.0025 + (i as f64) * 0.00025;
        let phase = i as f64 * 1.1;
        let wave = (ms * freq + phase).sin() * 0.35 + 0.35;
        let h = (base * 0.65 + wave * 0.35).clamp(0.0, 1.0);
        let idx = ((h * 7.0) as usize).min(7);
        let pos_idx = (i * 7 / n_bars.saturating_sub(1).max(1)).min(7);
        let color = if h < 0.05 {
            palette.muted
        } else {
            palette.spectrum[pos_idx]
        };
        spans.push(Span::styled(
            glyphs[idx].to_string(),
            Style::default().fg(color).bg(bg),
        ));
        if i + 1 < n_bars {
            spans.push(Span::styled(" ", Style::default().bg(bg)));
        }
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::visualizer_glyphs_for_legacy_console;

    #[test]
    fn legacy_console_visualizer_uses_ascii_fallback() {
        assert_eq!(
            visualizer_glyphs_for_legacy_console(true),
            &['.', ':', '-', '=', '+', '*', '#', '@']
        );
    }

    #[test]
    fn modern_terminal_visualizer_uses_unicode_blocks() {
        assert_eq!(
            visualizer_glyphs_for_legacy_console(false),
            &[
                '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
                '\u{2588}'
            ]
        );
    }
}
