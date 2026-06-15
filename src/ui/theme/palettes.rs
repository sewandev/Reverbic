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

const GRUVBOX_BORDER: [(u8, u8, u8); 3] = [(142, 192, 124), (250, 189, 47), (211, 134, 155)];
const GRUVBOX_SPECTRUM: [Color; 8] = [
    Color::Rgb(142, 192, 124),
    Color::Rgb(184, 187, 38),
    Color::Rgb(250, 189, 47),
    Color::Rgb(254, 128, 25),
    Color::Rgb(251, 73, 52),
    Color::Rgb(211, 134, 155),
    Color::Rgb(131, 165, 152),
    Color::Rgb(235, 219, 178),
];

const AYU_BORDER: [(u8, u8, u8); 3] = [(230, 180, 80), (95, 180, 180), (255, 120, 120)];
const AYU_SPECTRUM: [Color; 8] = [
    Color::Rgb(230, 180, 80),
    Color::Rgb(255, 180, 84),
    Color::Rgb(255, 120, 120),
    Color::Rgb(214, 112, 214),
    Color::Rgb(95, 180, 180),
    Color::Rgb(89, 174, 255),
    Color::Rgb(180, 190, 110),
    Color::Rgb(230, 225, 207),
];

const NIGHT_OWL_BORDER: [(u8, u8, u8); 3] = [(130, 170, 255), (127, 219, 202), (199, 146, 234)];
const NIGHT_OWL_SPECTRUM: [Color; 8] = [
    Color::Rgb(130, 170, 255),
    Color::Rgb(127, 219, 202),
    Color::Rgb(173, 219, 103),
    Color::Rgb(255, 203, 107),
    Color::Rgb(247, 140, 108),
    Color::Rgb(199, 146, 234),
    Color::Rgb(137, 221, 255),
    Color::Rgb(214, 222, 235),
];

const VESPER_BORDER: [(u8, u8, u8); 3] = [(255, 199, 153), (153, 204, 204), (255, 128, 128)];
const VESPER_SPECTRUM: [Color; 8] = [
    Color::Rgb(255, 199, 153),
    Color::Rgb(255, 179, 128),
    Color::Rgb(255, 128, 128),
    Color::Rgb(204, 153, 204),
    Color::Rgb(153, 204, 204),
    Color::Rgb(128, 170, 255),
    Color::Rgb(204, 204, 153),
    Color::Rgb(238, 238, 238),
];

const ROSE_PINE_BORDER: [(u8, u8, u8); 3] = [(235, 188, 186), (196, 167, 231), (156, 207, 216)];
const ROSE_PINE_SPECTRUM: [Color; 8] = [
    Color::Rgb(235, 188, 186),
    Color::Rgb(235, 111, 146),
    Color::Rgb(246, 193, 119),
    Color::Rgb(156, 207, 216),
    Color::Rgb(49, 116, 143),
    Color::Rgb(196, 167, 231),
    Color::Rgb(144, 122, 169),
    Color::Rgb(224, 222, 244),
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

pub const GRUVBOX: Palette = Palette {
    accent: Color::Rgb(142, 192, 124),
    radio_accent: Color::Rgb(131, 165, 152),
    playing: Color::Rgb(142, 192, 124),
    muted: Color::Rgb(146, 131, 116),
    dim: Color::Rgb(168, 153, 132),
    highlight: Color::Rgb(235, 219, 178),
    danger: Color::Rgb(251, 73, 52),
    warning: Color::Rgb(250, 189, 47),
    buffering: Color::Rgb(60, 56, 54),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(184, 187, 38),
    caution: Color::Rgb(254, 128, 25),
    panel_bg: Color::Rgb(40, 40, 40),
    overlay_color: Color::Rgb(29, 32, 33),
    border_cycle: GRUVBOX_BORDER,
    spectrum: GRUVBOX_SPECTRUM,
    logo_letters: GRUVBOX_SPECTRUM,
};

pub const AYU: Palette = Palette {
    accent: Color::Rgb(230, 180, 80),
    radio_accent: Color::Rgb(95, 180, 180),
    playing: Color::Rgb(230, 180, 80),
    muted: Color::Rgb(92, 103, 121),
    dim: Color::Rgb(171, 183, 199),
    highlight: Color::Rgb(230, 225, 207),
    danger: Color::Rgb(255, 120, 120),
    warning: Color::Rgb(230, 180, 80),
    buffering: Color::Rgb(42, 51, 64),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(180, 190, 110),
    caution: Color::Rgb(255, 180, 84),
    panel_bg: Color::Rgb(16, 20, 28),
    overlay_color: Color::Rgb(9, 12, 18),
    border_cycle: AYU_BORDER,
    spectrum: AYU_SPECTRUM,
    logo_letters: AYU_SPECTRUM,
};

pub const NIGHT_OWL: Palette = Palette {
    accent: Color::Rgb(130, 170, 255),
    radio_accent: Color::Rgb(127, 219, 202),
    playing: Color::Rgb(130, 170, 255),
    muted: Color::Rgb(99, 119, 148),
    dim: Color::Rgb(150, 164, 190),
    highlight: Color::Rgb(214, 222, 235),
    danger: Color::Rgb(239, 83, 80),
    warning: Color::Rgb(255, 203, 107),
    buffering: Color::Rgb(13, 50, 79),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(173, 219, 103),
    caution: Color::Rgb(247, 140, 108),
    panel_bg: Color::Rgb(1, 22, 39),
    overlay_color: Color::Rgb(1, 13, 24),
    border_cycle: NIGHT_OWL_BORDER,
    spectrum: NIGHT_OWL_SPECTRUM,
    logo_letters: NIGHT_OWL_SPECTRUM,
};

pub const VESPER: Palette = Palette {
    accent: Color::Rgb(255, 199, 153),
    radio_accent: Color::Rgb(153, 204, 204),
    playing: Color::Rgb(255, 199, 153),
    muted: Color::Rgb(112, 112, 112),
    dim: Color::Rgb(176, 176, 176),
    highlight: Color::Rgb(238, 238, 238),
    danger: Color::Rgb(255, 128, 128),
    warning: Color::Rgb(255, 199, 153),
    buffering: Color::Rgb(42, 42, 42),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(153, 204, 153),
    caution: Color::Rgb(255, 179, 128),
    panel_bg: Color::Rgb(16, 16, 16),
    overlay_color: Color::Rgb(8, 8, 8),
    border_cycle: VESPER_BORDER,
    spectrum: VESPER_SPECTRUM,
    logo_letters: VESPER_SPECTRUM,
};

pub const ROSE_PINE: Palette = Palette {
    accent: Color::Rgb(235, 188, 186),
    radio_accent: Color::Rgb(156, 207, 216),
    playing: Color::Rgb(235, 188, 186),
    muted: Color::Rgb(110, 106, 134),
    dim: Color::Rgb(144, 140, 170),
    highlight: Color::Rgb(224, 222, 244),
    danger: Color::Rgb(235, 111, 146),
    warning: Color::Rgb(246, 193, 119),
    buffering: Color::Rgb(38, 35, 58),
    spotify: Color::Rgb(30, 215, 96),
    youtube: Color::Rgb(255, 0, 0),
    status_ok: Color::Rgb(156, 207, 216),
    caution: Color::Rgb(234, 154, 151),
    panel_bg: Color::Rgb(25, 23, 36),
    overlay_color: Color::Rgb(12, 10, 18),
    border_cycle: ROSE_PINE_BORDER,
    spectrum: ROSE_PINE_SPECTRUM,
    logo_letters: ROSE_PINE_SPECTRUM,
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
            &GRUVBOX,
            &AYU,
            &NIGHT_OWL,
            &VESPER,
            &ROSE_PINE,
        ] {
            assert_eq!(palette.border_cycle.len(), 3);
            assert_eq!(palette.spectrum.len(), 8);
            assert_eq!(palette.logo_letters.len(), 8);
        }
    }
}
