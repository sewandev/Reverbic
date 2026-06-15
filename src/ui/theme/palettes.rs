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

const ROSE_BORDER: [(u8, u8, u8); 3] = [(251, 113, 133), (244, 114, 182), (225, 29, 72)];
const ROSE_SPECTRUM: [Color; 8] = [
    Color::Rgb(251, 113, 133),
    Color::Rgb(244, 63, 94),
    Color::Rgb(244, 114, 182),
    Color::Rgb(236, 72, 153),
    Color::Rgb(225, 29, 72),
    Color::Rgb(251, 146, 60),
    Color::Rgb(252, 165, 165),
    Color::Rgb(253, 164, 175),
];

const AMBER_BORDER: [(u8, u8, u8); 3] = [(245, 158, 11), (251, 191, 36), (249, 115, 22)];
const AMBER_SPECTRUM: [Color; 8] = [
    Color::Rgb(245, 158, 11),
    Color::Rgb(251, 191, 36),
    Color::Rgb(234, 179, 8),
    Color::Rgb(249, 115, 22),
    Color::Rgb(251, 146, 60),
    Color::Rgb(253, 224, 71),
    Color::Rgb(252, 211, 77),
    Color::Rgb(217, 119, 6),
];

const LAVENDER_BORDER: [(u8, u8, u8); 3] = [(167, 139, 250), (192, 132, 252), (129, 140, 248)];
const LAVENDER_SPECTRUM: [Color; 8] = [
    Color::Rgb(167, 139, 250),
    Color::Rgb(139, 92, 246),
    Color::Rgb(192, 132, 252),
    Color::Rgb(216, 180, 254),
    Color::Rgb(129, 140, 248),
    Color::Rgb(96, 165, 250),
    Color::Rgb(244, 114, 182),
    Color::Rgb(196, 181, 253),
];

const NORD_BORDER: [(u8, u8, u8); 3] = [(136, 192, 208), (129, 161, 193), (180, 142, 173)];
const NORD_SPECTRUM: [Color; 8] = [
    Color::Rgb(136, 192, 208),
    Color::Rgb(143, 188, 187),
    Color::Rgb(129, 161, 193),
    Color::Rgb(94, 129, 172),
    Color::Rgb(180, 142, 173),
    Color::Rgb(163, 190, 140),
    Color::Rgb(235, 203, 139),
    Color::Rgb(216, 222, 233),
];

const SUNSET_BORDER: [(u8, u8, u8); 3] = [(251, 146, 60), (244, 114, 182), (248, 113, 113)];
const SUNSET_SPECTRUM: [Color; 8] = [
    Color::Rgb(251, 146, 60),
    Color::Rgb(249, 115, 22),
    Color::Rgb(248, 113, 113),
    Color::Rgb(244, 114, 182),
    Color::Rgb(217, 70, 239),
    Color::Rgb(251, 191, 36),
    Color::Rgb(253, 164, 175),
    Color::Rgb(253, 186, 116),
];

const CATPPUCCIN_BORDER: [(u8, u8, u8); 3] = [(203, 166, 247), (137, 180, 250), (245, 194, 231)];
const CATPPUCCIN_SPECTRUM: [Color; 8] = [
    Color::Rgb(203, 166, 247),
    Color::Rgb(137, 180, 250),
    Color::Rgb(116, 199, 236),
    Color::Rgb(148, 226, 213),
    Color::Rgb(166, 227, 161),
    Color::Rgb(249, 226, 175),
    Color::Rgb(245, 194, 231),
    Color::Rgb(243, 139, 168),
];

const SOLARIZED_BORDER: [(u8, u8, u8); 3] = [(42, 161, 152), (38, 139, 210), (181, 137, 0)];
const SOLARIZED_SPECTRUM: [Color; 8] = [
    Color::Rgb(42, 161, 152),
    Color::Rgb(38, 139, 210),
    Color::Rgb(108, 113, 196),
    Color::Rgb(211, 54, 130),
    Color::Rgb(220, 50, 47),
    Color::Rgb(203, 75, 22),
    Color::Rgb(181, 137, 0),
    Color::Rgb(133, 153, 0),
];

const TOKYO_NIGHT_BORDER: [(u8, u8, u8); 3] = [(122, 162, 247), (187, 154, 247), (125, 207, 255)];
const TOKYO_NIGHT_SPECTRUM: [Color; 8] = [
    Color::Rgb(122, 162, 247),
    Color::Rgb(125, 207, 255),
    Color::Rgb(115, 218, 202),
    Color::Rgb(158, 206, 106),
    Color::Rgb(224, 175, 104),
    Color::Rgb(247, 118, 142),
    Color::Rgb(187, 154, 247),
    Color::Rgb(169, 177, 214),
];

pub const OCEAN: Palette = Palette {
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

pub const FOREST: Palette = Palette {
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

pub const ROSE: Palette = Palette {
    accent: Color::Rgb(251, 113, 133),
    radio_accent: Color::Rgb(244, 114, 182),
    playing: Color::Rgb(251, 113, 133),
    muted: Color::Rgb(139, 92, 105),
    dim: Color::Rgb(190, 142, 153),
    highlight: Color::Rgb(255, 241, 242),
    danger: Color::Rgb(248, 113, 113),
    warning: Color::Rgb(251, 191, 36),
    buffering: Color::Rgb(62, 40, 48),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(52, 211, 153),
    caution: Color::Rgb(245, 158, 11),
    panel_bg: Color::Rgb(26, 18, 21),
    overlay_color: Color::Rgb(13, 8, 10),
    border_cycle: ROSE_BORDER,
    spectrum: ROSE_SPECTRUM,
    logo_letters: ROSE_SPECTRUM,
};

pub const AMBER: Palette = Palette {
    accent: Color::Rgb(245, 158, 11),
    radio_accent: Color::Rgb(251, 191, 36),
    playing: Color::Rgb(245, 158, 11),
    muted: Color::Rgb(128, 106, 72),
    dim: Color::Rgb(180, 154, 112),
    highlight: Color::Rgb(255, 251, 235),
    danger: Color::Rgb(248, 113, 113),
    warning: Color::Rgb(251, 191, 36),
    buffering: Color::Rgb(59, 48, 27),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(52, 211, 153),
    caution: Color::Rgb(249, 115, 22),
    panel_bg: Color::Rgb(26, 23, 16),
    overlay_color: Color::Rgb(12, 9, 4),
    border_cycle: AMBER_BORDER,
    spectrum: AMBER_SPECTRUM,
    logo_letters: AMBER_SPECTRUM,
};

pub const LAVENDER: Palette = Palette {
    accent: Color::Rgb(167, 139, 250),
    radio_accent: Color::Rgb(192, 132, 252),
    playing: Color::Rgb(167, 139, 250),
    muted: Color::Rgb(129, 111, 150),
    dim: Color::Rgb(185, 170, 205),
    highlight: Color::Rgb(245, 243, 255),
    danger: Color::Rgb(248, 113, 113),
    warning: Color::Rgb(251, 191, 36),
    buffering: Color::Rgb(49, 39, 65),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(52, 211, 153),
    caution: Color::Rgb(245, 158, 11),
    panel_bg: Color::Rgb(24, 20, 31),
    overlay_color: Color::Rgb(11, 8, 15),
    border_cycle: LAVENDER_BORDER,
    spectrum: LAVENDER_SPECTRUM,
    logo_letters: LAVENDER_SPECTRUM,
};

pub const NORD: Palette = Palette {
    accent: Color::Rgb(136, 192, 208),
    radio_accent: Color::Rgb(129, 161, 193),
    playing: Color::Rgb(136, 192, 208),
    muted: Color::Rgb(94, 110, 130),
    dim: Color::Rgb(163, 177, 197),
    highlight: Color::Rgb(236, 239, 244),
    danger: Color::Rgb(191, 97, 106),
    warning: Color::Rgb(235, 203, 139),
    buffering: Color::Rgb(67, 76, 94),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(163, 190, 140),
    caution: Color::Rgb(208, 135, 112),
    panel_bg: Color::Rgb(46, 52, 64),
    overlay_color: Color::Rgb(36, 41, 51),
    border_cycle: NORD_BORDER,
    spectrum: NORD_SPECTRUM,
    logo_letters: NORD_SPECTRUM,
};

pub const SUNSET: Palette = Palette {
    accent: Color::Rgb(251, 146, 60),
    radio_accent: Color::Rgb(244, 114, 182),
    playing: Color::Rgb(251, 146, 60),
    muted: Color::Rgb(139, 106, 86),
    dim: Color::Rgb(198, 155, 126),
    highlight: Color::Rgb(255, 247, 237),
    danger: Color::Rgb(248, 113, 113),
    warning: Color::Rgb(251, 191, 36),
    buffering: Color::Rgb(60, 43, 31),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(52, 211, 153),
    caution: Color::Rgb(249, 115, 22),
    panel_bg: Color::Rgb(26, 20, 16),
    overlay_color: Color::Rgb(12, 8, 5),
    border_cycle: SUNSET_BORDER,
    spectrum: SUNSET_SPECTRUM,
    logo_letters: SUNSET_SPECTRUM,
};

pub const CATPPUCCIN: Palette = Palette {
    accent: Color::Rgb(203, 166, 247),
    radio_accent: Color::Rgb(137, 180, 250),
    playing: Color::Rgb(203, 166, 247),
    muted: Color::Rgb(108, 112, 134),
    dim: Color::Rgb(166, 173, 200),
    highlight: Color::Rgb(205, 214, 244),
    danger: Color::Rgb(243, 139, 168),
    warning: Color::Rgb(249, 226, 175),
    buffering: Color::Rgb(49, 50, 68),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(166, 227, 161),
    caution: Color::Rgb(250, 179, 135),
    panel_bg: Color::Rgb(30, 30, 46),
    overlay_color: Color::Rgb(17, 17, 27),
    border_cycle: CATPPUCCIN_BORDER,
    spectrum: CATPPUCCIN_SPECTRUM,
    logo_letters: CATPPUCCIN_SPECTRUM,
};

pub const SOLARIZED: Palette = Palette {
    accent: Color::Rgb(42, 161, 152),
    radio_accent: Color::Rgb(38, 139, 210),
    playing: Color::Rgb(42, 161, 152),
    muted: Color::Rgb(88, 110, 117),
    dim: Color::Rgb(131, 148, 150),
    highlight: Color::Rgb(238, 232, 213),
    danger: Color::Rgb(220, 50, 47),
    warning: Color::Rgb(181, 137, 0),
    buffering: Color::Rgb(7, 54, 66),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(133, 153, 0),
    caution: Color::Rgb(203, 75, 22),
    panel_bg: Color::Rgb(0, 43, 54),
    overlay_color: Color::Rgb(0, 28, 35),
    border_cycle: SOLARIZED_BORDER,
    spectrum: SOLARIZED_SPECTRUM,
    logo_letters: SOLARIZED_SPECTRUM,
};

pub const TOKYO_NIGHT: Palette = Palette {
    accent: Color::Rgb(122, 162, 247),
    radio_accent: Color::Rgb(125, 207, 255),
    playing: Color::Rgb(122, 162, 247),
    muted: Color::Rgb(86, 95, 137),
    dim: Color::Rgb(169, 177, 214),
    highlight: Color::Rgb(192, 202, 245),
    danger: Color::Rgb(247, 118, 142),
    warning: Color::Rgb(224, 175, 104),
    buffering: Color::Rgb(65, 72, 104),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(158, 206, 106),
    caution: Color::Rgb(255, 158, 100),
    panel_bg: Color::Rgb(26, 27, 38),
    overlay_color: Color::Rgb(15, 15, 24),
    border_cycle: TOKYO_NIGHT_BORDER,
    spectrum: TOKYO_NIGHT_SPECTRUM,
    logo_letters: TOKYO_NIGHT_SPECTRUM,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pilot_palettes_define_visual_motion_sets() {
        for palette in [
            &OCEAN,
            &FOREST,
            &ROSE,
            &AMBER,
            &LAVENDER,
            &NORD,
            &SUNSET,
            &CATPPUCCIN,
            &SOLARIZED,
            &TOKYO_NIGHT,
        ] {
            assert_eq!(palette.border_cycle.len(), 3);
            assert_eq!(palette.spectrum.len(), 8);
            assert_eq!(palette.logo_letters.len(), 8);
        }
    }
}
