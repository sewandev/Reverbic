use ratatui::style::Color;

use super::Palette;

const OCEAN_BORDER: [(u8, u8, u8); 3] = [(56, 189, 248), (59, 130, 246), (129, 140, 248)];
const OCEAN_SPECTRUM: [Color; 8] = [
    Color::Rgb(56, 189, 248),
    Color::Rgb(14, 165, 233),
    Color::Rgb(59, 130, 246),
    Color::Rgb(99, 102, 241),
    Color::Rgb(129, 140, 248),
    Color::Rgb(45, 212, 191),
    Color::Rgb(34, 211, 238),
    Color::Rgb(125, 211, 252),
];

const FOREST_BORDER: [(u8, u8, u8); 3] = [(52, 211, 153), (45, 212, 191), (163, 230, 53)];
const FOREST_SPECTRUM: [Color; 8] = [
    Color::Rgb(52, 211, 153),
    Color::Rgb(16, 185, 129),
    Color::Rgb(45, 212, 191),
    Color::Rgb(34, 197, 94),
    Color::Rgb(132, 204, 22),
    Color::Rgb(163, 230, 53),
    Color::Rgb(187, 247, 208),
    Color::Rgb(110, 231, 183),
];

const OCEAN: Palette = Palette {
    accent: Color::Rgb(56, 189, 248),
    radio_accent: Color::Rgb(14, 165, 233),
    playing: Color::Rgb(56, 189, 248),
    muted: Color::Rgb(100, 116, 139),
    dim: Color::Rgb(148, 163, 184),
    highlight: Color::Rgb(226, 232, 240),
    danger: Color::Rgb(248, 113, 113),
    warning: Color::Rgb(251, 191, 36),
    buffering: Color::Rgb(51, 65, 85),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(52, 211, 153),
    caution: Color::Rgb(245, 158, 11),
    panel_bg: Color::Rgb(15, 23, 42),
    overlay_color: Color::Rgb(2, 6, 23),
    border_cycle: OCEAN_BORDER,
    spectrum: OCEAN_SPECTRUM,
    logo_letters: OCEAN_SPECTRUM,
};

const FOREST: Palette = Palette {
    accent: Color::Rgb(52, 211, 153),
    radio_accent: Color::Rgb(45, 212, 191),
    playing: Color::Rgb(52, 211, 153),
    muted: Color::Rgb(101, 128, 105),
    dim: Color::Rgb(167, 183, 165),
    highlight: Color::Rgb(236, 253, 245),
    danger: Color::Rgb(248, 113, 113),
    warning: Color::Rgb(250, 204, 21),
    buffering: Color::Rgb(45, 58, 49),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(74, 222, 128),
    caution: Color::Rgb(234, 179, 8),
    panel_bg: Color::Rgb(26, 26, 24),
    overlay_color: Color::Rgb(10, 15, 12),
    border_cycle: FOREST_BORDER,
    spectrum: FOREST_SPECTRUM,
    logo_letters: FOREST_SPECTRUM,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pilot_palettes_define_visual_motion_sets() {
        for palette in [&OCEAN, &FOREST] {
            assert_eq!(palette.border_cycle.len(), 3);
            assert_eq!(palette.spectrum.len(), 8);
            assert_eq!(palette.logo_letters.len(), 8);
        }
    }
}
