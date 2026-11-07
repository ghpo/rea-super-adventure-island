//! The Controls tab's data: the keyboard binding of each SNES button and the
//! controller layout. It lives beside `launcher.cfg` as `controls.cfg` and is
//! written into the private RetroArch configuration as
//! `input_player1_*`/`input_player1_*_btn`; the player's own RetroArch input
//! configuration is never changed.
//!
//! The controller default is **Auto**: RetroArch's system autoconfig (the
//! player's DualSense on Linux, XInput on Windows). The Nintendo/Xbox layouts
//! set the A/B (and the other face/shoulder buttons) explicitly for the
//! standard pad. A fully custom pad button map stays in RetroArch's Input
//! settings.

use std::fs;
use std::path::Path;

use rea_launcher::fsx;

pub const FILE_NAME: &str = "controls.cfg";

/// The twelve SNES buttons, in the order the Controls tab draws them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnesButton {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    L,
    R,
    Start,
    Select,
}

impl SnesButton {
    pub const ALL: [SnesButton; 12] = [
        SnesButton::Up,
        SnesButton::Down,
        SnesButton::Left,
        SnesButton::Right,
        SnesButton::A,
        SnesButton::B,
        SnesButton::X,
        SnesButton::Y,
        SnesButton::L,
        SnesButton::R,
        SnesButton::Start,
        SnesButton::Select,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SnesButton::Up => "Up",
            SnesButton::Down => "Down",
            SnesButton::Left => "Left",
            SnesButton::Right => "Right",
            SnesButton::A => "A",
            SnesButton::B => "B",
            SnesButton::X => "X",
            SnesButton::Y => "Y",
            SnesButton::L => "L",
            SnesButton::R => "R",
            SnesButton::Start => "Start",
            SnesButton::Select => "Select",
        }
    }

    /// The `controls.cfg` key.
    pub fn key(self) -> &'static str {
        match self {
            SnesButton::Up => "key_up",
            SnesButton::Down => "key_down",
            SnesButton::Left => "key_left",
            SnesButton::Right => "key_right",
            SnesButton::A => "key_a",
            SnesButton::B => "key_b",
            SnesButton::X => "key_x",
            SnesButton::Y => "key_y",
            SnesButton::L => "key_l",
            SnesButton::R => "key_r",
            SnesButton::Start => "key_start",
            SnesButton::Select => "key_select",
        }
    }

    /// The RetroArch configuration key.
    pub fn input(self) -> &'static str {
        match self {
            SnesButton::Up => "input_player1_up",
            SnesButton::Down => "input_player1_down",
            SnesButton::Left => "input_player1_left",
            SnesButton::Right => "input_player1_right",
            SnesButton::A => "input_player1_a",
            SnesButton::B => "input_player1_b",
            SnesButton::X => "input_player1_x",
            SnesButton::Y => "input_player1_y",
            SnesButton::L => "input_player1_l",
            SnesButton::R => "input_player1_r",
            SnesButton::Start => "input_player1_start",
            SnesButton::Select => "input_player1_select",
        }
    }

    pub fn index(self) -> usize {
        match self {
            SnesButton::Up => 0,
            SnesButton::Down => 1,
            SnesButton::Left => 2,
            SnesButton::Right => 3,
            SnesButton::A => 4,
            SnesButton::B => 5,
            SnesButton::X => 6,
            SnesButton::Y => 7,
            SnesButton::L => 8,
            SnesButton::R => 9,
            SnesButton::Start => 10,
            SnesButton::Select => 11,
        }
    }

    pub fn from_key(key: &str) -> Option<SnesButton> {
        SnesButton::ALL
            .into_iter()
            .find(|button| button.key() == key)
    }
}

/// RetroArch's default keyboard keys for a SNES pad.
pub const DEFAULT_KEYS: [&str; 12] = [
    "up",
    "down",
    "left",
    "right",
    "x",
    "z",
    "s",
    "a",
    "q",
    "w",
    "enter",
    "backspace",
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PadLayout {
    /// RetroArch's system autoconfig; the recommended choice.
    #[default]
    Auto,
    /// The SNES positions: A is the right button, B the bottom one.
    Nintendo,
    /// The literal face labels: A is A, B is B.
    Xbox,
}

impl PadLayout {
    pub const ALL: [PadLayout; 3] = [PadLayout::Auto, PadLayout::Nintendo, PadLayout::Xbox];

    pub fn key(self) -> &'static str {
        match self {
            PadLayout::Auto => "auto",
            PadLayout::Nintendo => "nintendo",
            PadLayout::Xbox => "xbox",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PadLayout::Auto => "Auto",
            PadLayout::Nintendo => "Nintendo (SNES A/B)",
            PadLayout::Xbox => "Xbox (A = A, B = B)",
        }
    }

    pub fn from_key(key: &str) -> Option<PadLayout> {
        PadLayout::ALL
            .into_iter()
            .find(|layout| layout.key() == key)
    }
}

/// The Controls tab's values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Controls {
    /// One RetroArch key name per [`SnesButton`], in `SnesButton::ALL` order.
    pub keys: [String; 12],
    pub pad: PadLayout,
}

impl Default for Controls {
    fn default() -> Controls {
        Controls {
            keys: DEFAULT_KEYS.map(str::to_owned),
            pad: PadLayout::Auto,
        }
    }
}

impl Controls {
    pub fn key(&self, button: SnesButton) -> &str {
        &self.keys[button.index()]
    }

    pub fn set_key(&mut self, button: SnesButton, key: String) {
        self.keys[button.index()] = key;
    }

    pub fn reset(&mut self) {
        *self = Controls::default();
    }

    /// `controls.cfg` text with a short explanation of each part.
    pub fn render(&self) -> String {
        let mut out = String::from(
            "# Super Adventure Island controls. Keys are RetroArch key names\n\
             # (a-z, 0-9, up, down, left, right, enter, space, backspace, f1..).\n\
             # pad_layout: auto (system autoconfig), nintendo or xbox.\n\
             \n# Keyboard\n",
        );
        for button in SnesButton::ALL {
            out.push_str(button.key());
            out.push_str(" = ");
            out.push_str(self.key(button));
            out.push('\n');
        }
        out.push_str("\n# Controller\npad_layout = ");
        out.push_str(self.pad.key());
        out.push('\n');
        out
    }

    /// Reads `controls.cfg` (missing: the defaults). Unknown/invalid lines
    /// are warnings, not errors; the player can fix the file.
    pub fn parse(text: &str) -> (Controls, Vec<String>) {
        let mut controls = Controls::default();
        let mut warnings = Vec::new();
        for (number, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                warnings.push(format!("line {}: expected key = value", number + 1));
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            if key == "pad_layout" {
                match PadLayout::from_key(value) {
                    Some(pad) => controls.pad = pad,
                    None => warnings.push(format!(
                        "line {}: pad_layout needs auto, nintendo or xbox, not {value}",
                        number + 1
                    )),
                }
            } else if let Some(button) = SnesButton::from_key(key) {
                if value.is_empty() {
                    warnings.push(format!("line {}: {key} needs a key name", number + 1));
                } else {
                    controls.set_key(button, value.to_lowercase());
                }
            } else {
                warnings.push(format!("line {}: unknown control {key}", number + 1));
            }
        }
        (controls, warnings)
    }

    pub fn read(path: &Path) -> (Controls, Vec<String>) {
        match fs::read_to_string(path) {
            Ok(text) => Controls::parse(&text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (Controls::default(), Vec::new())
            }
            Err(error) => (
                Controls::default(),
                vec![format!("Cannot read {}: {error}", path.display())],
            ),
        }
    }

    pub fn write(path: &Path, controls: &Controls) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Cannot create {}: {error}", parent.display()))?;
        }
        fsx::write_atomic(path, controls.render())
            .map_err(|error| format!("Cannot write {}: {error}", path.display()))
    }

    /// The explicit controller keys for a preset. Auto writes none: the
    /// system autoconfig supplies the mapping.
    pub fn pad_keys(&self) -> Vec<(&'static str, &'static str)> {
        let (select, start) = if cfg!(windows) {
            ("6", "7")
        } else {
            ("8", "9")
        };
        let (a, b) = match self.pad {
            PadLayout::Auto => return Vec::new(),
            PadLayout::Nintendo => ("1", "0"),
            PadLayout::Xbox => ("0", "1"),
        };
        vec![
            ("input_player1_a_btn", a),
            ("input_player1_b_btn", b),
            ("input_player1_x_btn", "2"),
            ("input_player1_y_btn", "3"),
            ("input_player1_l_btn", "4"),
            ("input_player1_r_btn", "5"),
            ("input_player1_select_btn", select),
            ("input_player1_start_btn", start),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_retroarchs_keyboard_defaults() {
        let controls = Controls::default();
        assert_eq!(controls.key(SnesButton::A), "x");
        assert_eq!(controls.key(SnesButton::B), "z");
        assert_eq!(controls.key(SnesButton::Start), "enter");
        assert_eq!(controls.pad, PadLayout::Auto);
        assert!(controls.pad_keys().is_empty());
    }

    #[test]
    fn the_file_round_trips_and_keeps_the_defaults_for_missing_keys() {
        let (controls, warnings) =
            Controls::parse("# mine\nkey_a = k\npad_layout = xbox\nkey_up = w\n");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(controls.key(SnesButton::A), "k");
        assert_eq!(controls.key(SnesButton::B), "z");
        assert_eq!(controls.pad, PadLayout::Xbox);
        let again = Controls::parse(&controls.render()).0;
        assert_eq!(again, controls);
    }

    #[test]
    fn the_nintendo_and_xbox_layouts_swap_a_and_b() {
        let mut controls = Controls {
            pad: PadLayout::Nintendo,
            ..Controls::default()
        };
        let nintendo = controls.pad_keys();
        assert!(nintendo.contains(&("input_player1_a_btn", "1")));
        assert!(nintendo.contains(&("input_player1_b_btn", "0")));
        controls.pad = PadLayout::Xbox;
        let xbox = controls.pad_keys();
        assert!(xbox.contains(&("input_player1_a_btn", "0")));
        assert!(xbox.contains(&("input_player1_b_btn", "1")));
    }

    #[test]
    fn bad_lines_are_warnings_and_the_rest_survives() {
        let (controls, warnings) = Controls::parse("pad_layout = shiny\nkey_a =\nwat = 1\n");
        assert_eq!(controls.pad, PadLayout::Auto);
        assert_eq!(controls.key(SnesButton::A), "x");
        assert_eq!(warnings.len(), 3, "{warnings:?}");
    }
}
