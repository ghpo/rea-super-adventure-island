//! The settings window's colours and sizes: a tropical sunset over deep
//! water, in the SDK's roles (`rea_launcher_ui::theme`).
//!
//! The tests keep the palette within WCAG AA and the text-size rules.

use rea_launcher_ui::egui::Color32;
use rea_launcher_ui::theme::{Metrics, Palette, tint};

pub const BASE: Color32 = Color32::from_rgb(0x08, 0x18, 0x26);
pub const PANEL: Color32 = Color32::from_rgb(0x10, 0x2f, 0x45);
pub const LINE: Color32 = Color32::from_rgb(0x6b, 0x9a, 0xb2);
pub const INK: Color32 = Color32::from_rgb(0xf3, 0xea, 0xd8);
pub const MUTED: Color32 = Color32::from_rgb(0xa8, 0xc1, 0xcc);
pub const ACCENT: Color32 = Color32::from_rgb(0xff, 0xb3, 0x47);
pub const ERROR: Color32 = Color32::from_rgb(0xff, 0x8f, 0x80);

pub const PALETTE: Palette = Palette {
    base: BASE,
    panel: PANEL,
    line: LINE,
    ink: INK,
    muted: MUTED,
    action: ACCENT,
    action_hover: Color32::from_rgb(0xff, 0xc9, 0x6b),
    on_action: BASE,
    select: ACCENT,
    on_select: BASE,
    error: ERROR,
    disabled_ink: Color32::from_rgb(0x5b, 0x72, 0x80),
    disabled_line: Color32::from_rgb(0x1b, 0x3a, 0x4d),
    focus: INK,
    selection: tint(ACCENT, 128),
};

pub const METRICS: Metrics = Metrics::DEFAULT;

#[cfg(test)]
mod tests {
    use super::*;
    use rea_launcher_ui::theme;

    #[test]
    fn the_palette_meets_wcag_aa() {
        assert_eq!(
            theme::contrast_problems(&PALETTE, &METRICS),
            Vec::<String>::new()
        );
    }

    #[test]
    fn no_text_is_too_small() {
        assert_eq!(theme::text_size_problems(&METRICS), Vec::<String>::new());
    }
}
