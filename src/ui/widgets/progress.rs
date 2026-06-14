use ratatui::{buffer::Buffer, layout::Rect, style::Style, text::Span, widgets::Widget};

pub struct ProgressBarWidget {
    ratio: f32,
    filled_color: ratatui::style::Color,
    empty_color: ratatui::style::Color,
    bg: ratatui::style::Color,
    filled_char: &'static str,
    empty_char: &'static str,
}

impl ProgressBarWidget {
    pub fn new(
        ratio: f32,
        filled_color: ratatui::style::Color,
        empty_color: ratatui::style::Color,
        bg: ratatui::style::Color,
    ) -> Self {
        Self {
            ratio: ratio.clamp(0.0, 1.0),
            filled_color,
            empty_color,
            bg,
            filled_char: "█",
            empty_char: "░",
        }
    }

    pub fn into_spans(self, width: usize) -> Vec<Span<'static>> {
        let filled_w = (self.ratio * width as f32).round() as usize;
        let filled_w = filled_w.min(width);
        let empty_w = width.saturating_sub(filled_w);

        let mut spans = Vec::new();
        if filled_w > 0 {
            spans.push(Span::styled(
                self.filled_char.repeat(filled_w),
                Style::default().fg(self.filled_color).bg(self.bg),
            ));
        }
        if empty_w > 0 {
            spans.push(Span::styled(
                self.empty_char.repeat(empty_w),
                Style::default().fg(self.empty_color).bg(self.bg),
            ));
        }
        spans
    }
}

impl Widget for ProgressBarWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let width = area.width as usize;
        let spans = self.into_spans(width);

        let mut x = area.left();
        let y = area.top();

        for span in spans {
            buf.set_string(x, y, &span.content, span.style);
            x += span.content.chars().count() as u16;
        }
    }
}
