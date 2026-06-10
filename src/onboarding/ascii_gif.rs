use std::io::Cursor;
use std::time::{Duration, Instant};

use image::{AnimationDecoder, Rgba, RgbaImage};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

const GIF_BYTES: &[u8] = include_bytes!("../../assets/pibble-rave.gif");
const COLS: u32 = 34;
const CHAR_ASPECT: f32 = 0.5;
const RAMP: &[u8] = b" .:-=+*#%@";
const ALPHA_THRESHOLD: u32 = 64;
const MIN_FRAME_DELAY: Duration = Duration::from_millis(20);

type Cell = Option<(char, Color)>;
type Grid = Vec<Vec<Cell>>;
pub struct AsciiGif {
    frames: Vec<Grid>,
    delays: Vec<Duration>,
    total: Duration,
    start: Instant,
    pub cols: u16,
    pub rows: u16,
}

impl AsciiGif {
    pub fn load() -> Option<Self> {
        let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(GIF_BYTES)).ok()?;
        let decoded = decoder.into_frames().collect_frames().ok()?;
        let (width, height) = decoded.first()?.buffer().dimensions();
        let rows = ((COLS as f32) * (height as f32 / width as f32) * CHAR_ASPECT)
            .round()
            .max(1.0) as u32;

        let mut frames = Vec::with_capacity(decoded.len());
        let mut delays = Vec::with_capacity(decoded.len());
        let mut total = Duration::ZERO;
        for frame in &decoded {
            let delay = Duration::from(frame.delay()).max(MIN_FRAME_DELAY);
            total += delay;
            delays.push(delay);
            frames.push(sample_grid(frame.buffer(), COLS, rows));
        }

        Some(Self {
            frames,
            delays,
            total,
            start: Instant::now(),
            cols: COLS as u16,
            rows: rows as u16,
        })
    }

    pub fn render(&self, buf: &mut Buffer, area: Rect) {
        for (row, cells) in self.current_frame().iter().enumerate() {
            let y = area.y + row as u16;
            if y >= area.bottom() {
                break;
            }
            for (col, cell) in cells.iter().enumerate() {
                let x = area.x + col as u16;
                if x >= area.right() {
                    break;
                }
                if let Some((ch, color)) = cell {
                    buf[(x, y)].set_char(*ch).set_fg(*color);
                }
            }
        }
    }

    fn current_frame(&self) -> &Grid {
        let total_nanos = self.total.as_nanos().max(1);
        let mut remaining =
            Duration::from_nanos((self.start.elapsed().as_nanos() % total_nanos) as u64);
        for (i, delay) in self.delays.iter().enumerate() {
            if remaining < *delay {
                return &self.frames[i];
            }
            remaining -= *delay;
        }
        &self.frames[self.frames.len() - 1]
    }
}

fn sample_grid(image: &RgbaImage, cols: u32, rows: u32) -> Grid {
    let (width, height) = image.dimensions();
    let block_w = (width / cols).max(1);
    let block_h = (height / rows).max(1);

    (0..rows)
        .map(|row| {
            (0..cols)
                .map(|col| sample_cell(image, col * block_w, row * block_h, block_w, block_h))
                .collect()
        })
        .collect()
}

fn sample_cell(image: &RgbaImage, x0: u32, y0: u32, w: u32, h: u32) -> Cell {
    let (img_w, img_h) = image.dimensions();
    let (mut r, mut g, mut b, mut a, mut count) = (0u32, 0u32, 0u32, 0u32, 0u32);
    for y in y0..(y0 + h).min(img_h) {
        for x in x0..(x0 + w).min(img_w) {
            let Rgba([pr, pg, pb, pa]) = *image.get_pixel(x, y);
            r += pr as u32;
            g += pg as u32;
            b += pb as u32;
            a += pa as u32;
            count += 1;
        }
    }
    if count == 0 || a / count < ALPHA_THRESHOLD {
        return None;
    }

    let (avg_r, avg_g, avg_b) = ((r / count) as u8, (g / count) as u8, (b / count) as u8);
    let luminance = 0.299 * avg_r as f32 + 0.587 * avg_g as f32 + 0.114 * avg_b as f32;
    let level = ((luminance / 255.0) * (RAMP.len() - 1) as f32).round() as usize;
    Some((RAMP[level] as char, Color::Rgb(avg_r, avg_g, avg_b)))
}
