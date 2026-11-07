//! `launcher.cfg`, the command line and `--help`, from one option table
//! (`rea_launcher::cfg::spec`, docs/STANDARDS.md "Settings window and CLI").
//!
//! [`OPTIONS`] names every setting once: its key, its flag (`--vsync` for
//! `vsync`), the values it takes, and the label and description the window
//! and `--help` show. The parser, `--help`, the first-run template and the
//! window rows all come from it, and the window's parity test fails when a
//! row has no control or a control has no row.
//!
//! Every option here was tried in the real game (docs/RUNTIME_NOTES.md):
//! the Enhanced look runs the first level at 60 fps on this laptop.

use std::path::PathBuf;

use rea_launcher::cfg::{self, ArgError, KeyStyle, Kind, OptionSpec, Rejected, Value, spec};
use rea_launcher::display;

/// The settings file, next to the launcher (portable) or in the launcher's
/// config folder.
pub const FILE_NAME: &str = "launcher.cfg";

/// Flags spell keys with hyphens: `--integer-scale` is `integer_scale`.
pub const STYLE: KeyStyle = KeyStyle::Underscore;

/// The launcher's own switches: they take no value and never reach
/// `launcher.cfg`.
pub const SWITCHES: &[&str] = &["--settings", "--no-window", "--check", "--linux"];

/// Looks, as `(value, label)` for the option table and the window cards.
pub const LOOKS: &[(&str, &str)] = &[
    ("original", "Original"),
    ("enhanced", "Enhanced"),
    ("crt", "CRT"),
    ("widescreen", "Widescreen"),
    ("steam_deck", "Steam Deck"),
];

pub const ASPECTS: &[(&str, &str)] = &[
    ("pixel", "Pixel-perfect (8:7)"),
    ("4:3", "4:3"),
    ("widescreen", "Widescreen (16:9)"),
];

pub const SHADERS: &[(&str, &str)] = &[
    ("off", "Off"),
    ("sharp", "Sharp"),
    ("crt", "CRT"),
    ("smooth", "Smooth"),
];

pub const OVERCLOCKS: &[(&str, &str)] = &[
    ("off", "Off"),
    ("light", "Light"),
    ("compatible", "Compatible"),
    ("max", "Max"),
];

pub const MODE7: &[(&str, &str)] = &[("off", "Off"), ("2x", "2×"), ("4x", "4×")];

pub const RUNAHEAD: &[(&str, &str)] = &[("0", "Off"), ("1", "1 frame"), ("2", "2 frames")];

pub const CORES: &[(&str, &str)] = &[
    ("auto", "Auto"),
    ("snes9x", "Snes9x"),
    ("bsnes_hd", "bsnes-hd"),
];

/// Every option: `launcher.cfg` key, flag and window control.
pub const OPTIONS: &[OptionSpec] = &[
    OptionSpec::new(
        "game",
        Kind::Path,
        "ROM folder",
        "The folder with Super Adventure Island.sfc, or the ROM file itself. It is \
         only read, never changed.",
    )
    .with_placeholder("<folder or .sfc file>"),
    OptionSpec::new(
        "look",
        Kind::Choice(LOOKS),
        "Look",
        "Sets the picture options at once: the game as shipped, the recommended \
         crisp look, a CRT, or the experimental Mode 7 widescreen. Options set after \
         it win.",
    ),
    OptionSpec::new(
        "aspect",
        Kind::Choice(ASPECTS),
        "Aspect ratio",
        "Pixel-perfect keeps the SNES 8:7 pixels square; 4:3 is the television \
         shape; widescreen is only for the bsnes-hd Mode 7 scenes.",
    )
    .unset_label("As the core"),
    OptionSpec::new(
        "integer_scale",
        Kind::Toggle,
        "Integer scaling",
        "Only whole-number enlargements, so every pixel stays the same size. It \
         leaves black borders on screens that do not fit another multiple.",
    ),
    OptionSpec::new(
        "shader",
        Kind::Choice(SHADERS),
        "Shader",
        "How the picture is enlarged: off (nearest), sharp, a CRT monitor look, or \
         a smooth xBR edge blend.",
    ),
    OptionSpec::new(
        "overclock",
        Kind::Choice(OVERCLOCKS),
        "Reduce slowdown",
        "Raises the emulated SNES CPU to remove in-game slowdown. Compatible is the \
         safe choice for most games; Max can break timing.",
    ),
    OptionSpec::new(
        "mode7_hd",
        Kind::Choice(MODE7),
        "HD Mode 7",
        "Draws Mode 7 backgrounds at 2× or 4× the original resolution. It only \
         affects Mode 7 scenes.",
    ),
    OptionSpec::new(
        "hires_blend",
        Kind::Toggle,
        "Hi-res blending",
        "Merges the SNES high-resolution layers instead of showing them as hard \
         alternating lines.",
    ),
    OptionSpec::new(
        "reduce_flicker",
        Kind::Toggle,
        "Reduce flicker",
        "Lets more sprites stay on screen at once, so fewer of them blink out.",
    ),
    OptionSpec::new(
        "widescreen",
        Kind::Toggle,
        "Widescreen (Mode 7)",
        "The bsnes-hd experimental widescreen. It only widens Mode 7 scenes; the \
         ordinary side-scrolling levels stay 4:3.",
    ),
    OptionSpec::new(
        "vsync",
        Kind::Toggle,
        "VSync",
        "Waits for the screen's refresh: no tearing, slightly more input delay.",
    ),
    OptionSpec::new(
        "run_ahead",
        Kind::Choice(RUNAHEAD),
        "Run-ahead",
        "Runs frames ahead of the screen to cut input lag. Two frames is a good \
         default; turn it off if audio crackles.",
    ),
    OptionSpec::new(
        "fullscreen",
        Kind::Toggle,
        "Fullscreen",
        "The game fills the screen instead of opening in a window.",
    ),
    OptionSpec::new(
        "core",
        Kind::Choice(CORES),
        "Core",
        "Which SNES core runs the game. Auto uses Snes9x, or bsnes-hd when \
         widescreen is on.",
    ),
    OptionSpec::new(
        "show_fps",
        Kind::Toggle,
        "Show FPS",
        "The emulator's frame-rate counter in the corner.",
    ),
    OptionSpec::new(
        "saves",
        Kind::Path,
        "Saves folder",
        "Where the launcher keeps your save files and save states.",
    )
    .launcher_only(),
    OptionSpec::new(
        "runtime",
        Kind::Path,
        "Runtime folder",
        "Where the launcher keeps its generated emulator configuration and logs.",
    )
    .launcher_only(),
    OptionSpec::new(
        "retroarch",
        Kind::Path,
        "RetroArch",
        "The RetroArch program. By default the launcher finds it on PATH, or in \
         the Windows package.",
    )
    .with_placeholder("<program>")
    .launcher_only(),
    OptionSpec::new(
        "core_dir",
        Kind::Path,
        "Core folder",
        "The folder with snes9x_libretro and bsnes_hd_beta_libretro. By default the \
         launcher finds it in the system's libretro folder or the Windows package.",
    )
    .launcher_only(),
    OptionSpec::new(
        "shader_dir",
        Kind::Path,
        "Shader folder",
        "The root of the slang shaders (the folder that contains crt/ and \
         pixel-art-scaling/).",
    )
    .launcher_only(),
];

/// Comment lines at the top of a new `launcher.cfg`.
const HEADING: &str = "Settings for the Super Adventure Island launcher.\n\
    Every key is also a command-line flag: integer_scale = off is --integer-scale off.\n\
    A key left commented out keeps the emulator core's own value.";

/// A look: a named set of picture values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Look {
    Original,
    Enhanced,
    Crt,
    Widescreen,
    SteamDeck,
}

impl Look {
    pub const ALL: [Look; 5] = [
        Look::Original,
        Look::Enhanced,
        Look::Crt,
        Look::Widescreen,
        Look::SteamDeck,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Look::Original => "original",
            Look::Enhanced => "enhanced",
            Look::Crt => "crt",
            Look::Widescreen => "widescreen",
            Look::SteamDeck => "steam_deck",
        }
    }

    pub fn from_key(key: &str) -> Option<Look> {
        Look::ALL.into_iter().find(|look| look.key() == key)
    }

    pub fn label(self) -> &'static str {
        LOOKS
            .iter()
            .find(|(key, _)| *key == self.key())
            .map_or("", |(_, label)| label)
    }

    /// The look recommended on `screen`: Steam Deck on a Deck.
    pub fn recommended(screen: Option<(u32, u32)>) -> Look {
        if display::is_steam_deck(screen) {
            Look::SteamDeck
        } else {
            Look::Enhanced
        }
    }

    /// The look's values. The screen does not change them; the emulator picks
    /// the largest integer multiple that fits.
    pub fn picture(self, _screen: Option<(u32, u32)>) -> Picture {
        match self {
            Look::Original => Picture {
                aspect: Some("4:3"),
                integer_scale: Some(false),
                shader: Some("off"),
                overclock: Some("off"),
                mode7_hd: Some("off"),
                hires_blend: Some(false),
                reduce_flicker: Some(false),
                widescreen: Some(false),
                vsync: Some(true),
                run_ahead: Some("2"),
                fullscreen: Some(true),
                core: Some("snes9x"),
                show_fps: Some(false),
            },
            Look::Enhanced | Look::SteamDeck => Picture {
                aspect: Some("pixel"),
                integer_scale: Some(true),
                shader: Some("off"),
                overclock: Some("compatible"),
                mode7_hd: Some("2x"),
                hires_blend: Some(true),
                reduce_flicker: Some(true),
                widescreen: Some(false),
                vsync: Some(true),
                run_ahead: Some("2"),
                fullscreen: Some(true),
                core: Some("snes9x"),
                show_fps: Some(false),
            },
            Look::Crt => Picture {
                shader: Some("crt"),
                ..Look::Enhanced.picture(None)
            },
            Look::Widescreen => Picture {
                aspect: Some("widescreen"),
                integer_scale: Some(false),
                shader: Some("off"),
                overclock: Some("compatible"),
                mode7_hd: Some("2x"),
                hires_blend: Some(true),
                reduce_flicker: Some(true),
                widescreen: Some(true),
                vsync: Some(true),
                run_ahead: Some("2"),
                fullscreen: Some(true),
                core: Some("bsnes_hd"),
                show_fps: Some(false),
            },
        }
    }
}

/// The picture options for one run. `None` keeps the core's own value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Picture {
    pub aspect: Option<&'static str>,
    pub integer_scale: Option<bool>,
    pub shader: Option<&'static str>,
    pub overclock: Option<&'static str>,
    pub mode7_hd: Option<&'static str>,
    pub hires_blend: Option<bool>,
    pub reduce_flicker: Option<bool>,
    pub widescreen: Option<bool>,
    pub vsync: Option<bool>,
    pub run_ahead: Option<&'static str>,
    pub fullscreen: Option<bool>,
    pub core: Option<&'static str>,
    pub show_fps: Option<bool>,
}

impl Picture {
    /// The values as `(key, launcher.cfg text)`, for the window and the log.
    pub fn values(&self) -> [(&'static str, Option<String>); 13] {
        let text = |value: Option<&'static str>| value.map(str::to_owned);
        let flag = |value: Option<bool>| value.map(|on| cfg::on_off(on).to_owned());
        [
            ("aspect", text(self.aspect)),
            ("integer_scale", flag(self.integer_scale)),
            ("shader", text(self.shader)),
            ("overclock", text(self.overclock)),
            ("mode7_hd", text(self.mode7_hd)),
            ("hires_blend", flag(self.hires_blend)),
            ("reduce_flicker", flag(self.reduce_flicker)),
            ("widescreen", flag(self.widescreen)),
            ("vsync", flag(self.vsync)),
            ("run_ahead", text(self.run_ahead)),
            ("fullscreen", flag(self.fullscreen)),
            ("core", text(self.core)),
            ("show_fps", flag(self.show_fps)),
        ]
    }

    /// One line for `launcher.log`.
    pub fn describe(&self) -> String {
        self.values()
            .iter()
            .map(|(key, value)| format!("{key} {}", value.as_deref().unwrap_or("(core's own)")))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// `flags`' set values over `self`'s.
    fn overlay(&mut self, flags: Picture) {
        if flags.aspect.is_some() {
            self.aspect = flags.aspect;
        }
        if flags.integer_scale.is_some() {
            self.integer_scale = flags.integer_scale;
        }
        if flags.shader.is_some() {
            self.shader = flags.shader;
        }
        if flags.overclock.is_some() {
            self.overclock = flags.overclock;
        }
        if flags.mode7_hd.is_some() {
            self.mode7_hd = flags.mode7_hd;
        }
        if flags.hires_blend.is_some() {
            self.hires_blend = flags.hires_blend;
        }
        if flags.reduce_flicker.is_some() {
            self.reduce_flicker = flags.reduce_flicker;
        }
        if flags.widescreen.is_some() {
            self.widescreen = flags.widescreen;
        }
        if flags.vsync.is_some() {
            self.vsync = flags.vsync;
        }
        if flags.run_ahead.is_some() {
            self.run_ahead = flags.run_ahead;
        }
        if flags.fullscreen.is_some() {
            self.fullscreen = flags.fullscreen;
        }
        if flags.core.is_some() {
            self.core = flags.core;
        }
        if flags.show_fps.is_some() {
            self.show_fps = flags.show_fps;
        }
    }
}

/// The settings from `launcher.cfg` and the command line. `None` is unset.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Settings {
    pub game: Option<PathBuf>,
    pub look: Option<Look>,
    pub picture: Picture,
    pub saves: Option<PathBuf>,
    pub runtime: Option<PathBuf>,
    pub retroarch: Option<PathBuf>,
    pub core_dir: Option<PathBuf>,
    pub shader_dir: Option<PathBuf>,
}

impl Settings {
    /// One checked value from the file or a flag.
    pub fn apply(&mut self, key: &str, value: Value) -> Result<(), Rejected> {
        let path = |text: &str| (!text.is_empty()).then(|| PathBuf::from(text));
        match (key, value) {
            ("game", Value::Text(text)) => self.game = path(text),
            ("saves", Value::Text(text)) => self.saves = path(text),
            ("runtime", Value::Text(text)) => self.runtime = path(text),
            ("retroarch", Value::Text(text)) => self.retroarch = path(text),
            ("core_dir", Value::Text(text)) => self.core_dir = path(text),
            ("shader_dir", Value::Text(text)) => self.shader_dir = path(text),
            ("look", Value::Choice(choice)) => {
                let look = Look::from_key(choice).ok_or(Rejected::Unknown)?;
                // A look sets all of its values; options after it win.
                self.look = Some(look);
                self.picture = look.picture(None);
            }
            ("aspect", Value::Choice(choice)) => self.picture.aspect = Some(choice),
            ("integer_scale", Value::Toggle(on)) => self.picture.integer_scale = Some(on),
            ("shader", Value::Choice(choice)) => self.picture.shader = Some(choice),
            ("overclock", Value::Choice(choice)) => self.picture.overclock = Some(choice),
            ("mode7_hd", Value::Choice(choice)) => self.picture.mode7_hd = Some(choice),
            ("hires_blend", Value::Toggle(on)) => self.picture.hires_blend = Some(on),
            ("reduce_flicker", Value::Toggle(on)) => self.picture.reduce_flicker = Some(on),
            ("widescreen", Value::Toggle(on)) => self.picture.widescreen = Some(on),
            ("vsync", Value::Toggle(on)) => self.picture.vsync = Some(on),
            ("run_ahead", Value::Choice(choice)) => self.picture.run_ahead = Some(choice),
            ("fullscreen", Value::Toggle(on)) => self.picture.fullscreen = Some(on),
            ("core", Value::Choice(choice)) => self.picture.core = Some(choice),
            ("show_fps", Value::Toggle(on)) => self.picture.show_fps = Some(on),
            _ => return Err(Rejected::Unknown),
        }
        Ok(())
    }

    /// These settings with the command line's on top: a flag wins over the
    /// file, and `--look` replaces the file's picture options.
    pub fn merged(mut self, flags: Settings) -> Settings {
        if flags.look.is_some() {
            self.look = flags.look;
            self.picture = flags.picture;
        } else {
            self.picture.overlay(flags.picture);
        }
        self.game = flags.game.or(self.game);
        self.saves = flags.saves.or(self.saves);
        self.runtime = flags.runtime.or(self.runtime);
        self.retroarch = flags.retroarch.or(self.retroarch);
        self.core_dir = flags.core_dir.or(self.core_dir);
        self.shader_dir = flags.shader_dir.or(self.shader_dir);
        self
    }

    /// The picture for this run: the look's values, then the options set on
    /// their own.
    pub fn picture(&self, _screen: Option<(u32, u32)>) -> Picture {
        self.picture
    }

    /// The core to use, with Auto resolved.
    pub fn core(&self) -> &'static str {
        match self.picture.core.unwrap_or("auto") {
            "bsnes_hd" => "bsnes_hd",
            "snes9x" => "snes9x",
            _ if self.picture.widescreen.unwrap_or(false) => "bsnes_hd",
            _ => "snes9x",
        }
    }
}

/// Reads `launcher.cfg` text; the warnings name each line it could not use.
pub fn parse_file(text: &str) -> (Settings, Vec<String>) {
    let mut settings = Settings::default();
    let warnings = spec::parse_file(text, FILE_NAME, OPTIONS, |key, value| {
        settings.apply(key, value)
    });
    (settings, warnings)
}

/// Reads the command line: the settings its flags set, and the switches
/// (`--no-window`, ...) in the order given.
pub fn parse_args(
    args: impl IntoIterator<Item = String>,
) -> Result<(Settings, Vec<String>), ArgError> {
    let mut settings = Settings::default();
    let switches = spec::parse_args(args, STYLE, OPTIONS, SWITCHES, |key, value| {
        settings.apply(key, value)
    })?;
    Ok((settings, switches))
}

/// A new `launcher.cfg`: every option commented out, with its description.
pub fn template() -> String {
    spec::template(HEADING, OPTIONS)
}

/// The first run's `launcher.cfg`: the template with the recommended look's
/// values saved explicitly, which is how the default becomes the
/// recommended experience (STANDARDS "The game's own settings stay the
/// game's").
pub fn first_run_text(screen: Option<(u32, u32)>) -> String {
    let look = Look::recommended(screen);
    let mut text = set(&template(), "look", look.key());
    for (key, value) in look.picture(screen).values() {
        if let Some(value) = value {
            text = set(&text, key, &value);
        }
    }
    text
}

/// [`spec::set_key`] for the row of `key`.
pub fn set(text: &str, key: &str, value: &str) -> String {
    match spec::find(OPTIONS, key) {
        Some(row) => spec::set_key(text, row, value),
        None => cfg::set_key(text, key, value),
    }
}

/// `--help`.
pub fn usage() -> String {
    format!(
        "SAILauncher {version}: plays your own copy of Super Adventure Island on modern Windows and Linux.\n\
         \n\
         Usage: SAILauncher [options]\n\
         \n\
         Without options the settings window opens; Play saves them and starts the game.\n\
         \n\
         Options (each is also a {FILE_NAME} key):\n\
         {options}\n\
         Launcher switches:\n  \
           --settings   only the settings window (exit 0 after Play, 3 when closed)\n  \
           --no-window  play with the saved settings and these options, no window\n  \
           --check      check the ROM and its edition, then exit (see launcher.log)\n  \
           --linux      started by the Linux helper (linux/sai-play)\n  \
           --help       this text\n",
        version = env!("CARGO_PKG_VERSION"),
        options = spec::usage(OPTIONS, STYLE),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_template_parses_to_nothing_and_lists_every_option() {
        let text = template();
        assert_eq!(parse_file(&text), (Settings::default(), Vec::new()));
        for row in OPTIONS {
            assert!(text.contains(&format!("#{} =", row.key)), "{}", row.key);
        }
    }

    #[test]
    fn the_first_run_saves_the_recommended_look_explicitly() {
        let (settings, warnings) = parse_file(&first_run_text(Some((2560, 1440))));
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(settings.look, Some(Look::Enhanced));
        assert_eq!(settings.picture.aspect, Some("pixel"));
        assert_eq!(settings.picture.integer_scale, Some(true));
        assert_eq!(settings.picture.core, Some("snes9x"));
        let deck = parse_file(&first_run_text(Some((1280, 800)))).0;
        assert_eq!(deck.look, Some(Look::SteamDeck));
        assert_eq!(deck.picture.core, Some("snes9x"));
    }

    #[test]
    fn a_look_sets_its_values_and_later_options_win() {
        let (settings, _) = parse_file("look = original\nshader = crt\nvsync = off\n");
        assert_eq!(settings.look, Some(Look::Original));
        assert_eq!(settings.picture.shader, Some("crt"));
        assert_eq!(settings.picture.vsync, Some(false));
        let before = parse_file("shader = crt\nlook = original\n").0;
        assert_eq!(before.picture.shader, Some("off"), "the later look wins");
    }

    #[test]
    fn flags_win_over_the_file_and_a_look_flag_replaces_its_values() {
        let (file, _) = parse_file("game = /roms\nlook = original\nshader = off\nvsync = on\n");
        let (flags, _) = parse_args(["--look".into(), "crt".into()]).unwrap();
        let merged = file.clone().merged(flags);
        assert_eq!(merged.game, file.game);
        assert_eq!(merged.picture.shader, Some("crt"));
        assert_eq!(merged.picture.aspect, Some("pixel"));
        let (flags, _) = parse_args(["--vsync".into(), "off".into()]).unwrap();
        let merged = file.merged(flags);
        assert_eq!(merged.picture.vsync, Some(false));
        assert_eq!(merged.picture.aspect, Some("4:3"));
    }

    #[test]
    fn auto_core_follows_widescreen() {
        let (settings, _) = parse_file("look = enhanced\n");
        assert_eq!(settings.core(), "snes9x");
        let (settings, _) = parse_file("widescreen = on\n");
        assert_eq!(settings.core(), "bsnes_hd");
        let (settings, _) = parse_file("core = snes9x\nwidescreen = on\n");
        assert_eq!(settings.core(), "snes9x");
    }

    #[test]
    fn every_picture_option_round_trips_through_the_flags() {
        for (key, flag, value) in [
            ("overclock", "--overclock", "max"),
            ("mode7_hd", "--mode7-hd", "4x"),
            ("hires_blend", "--hires-blend", "off"),
            ("reduce_flicker", "--reduce-flicker", "on"),
            ("run_ahead", "--run-ahead", "1"),
            ("fullscreen", "--fullscreen", "off"),
            ("show_fps", "--show-fps", "on"),
            ("core_dir", "--core-dir", "/usr/lib/libretro"),
            (
                "shader_dir",
                "--shader-dir",
                "/usr/share/libretro/shaders/shaders_slang",
            ),
        ] {
            let (settings, _) = parse_args([flag.to_owned(), value.to_owned()]).unwrap();
            let ok = match key {
                "overclock" => settings.picture.overclock == Some("max"),
                "mode7_hd" => settings.picture.mode7_hd == Some("4x"),
                "hires_blend" => settings.picture.hires_blend == Some(false),
                "reduce_flicker" => settings.picture.reduce_flicker == Some(true),
                "run_ahead" => settings.picture.run_ahead == Some("1"),
                "fullscreen" => settings.picture.fullscreen == Some(false),
                "show_fps" => settings.picture.show_fps == Some(true),
                "core_dir" => settings.core_dir == Some(PathBuf::from("/usr/lib/libretro")),
                "shader_dir" => {
                    settings.shader_dir
                        == Some(PathBuf::from("/usr/share/libretro/shaders/shaders_slang"))
                }
                _ => false,
            };
            assert!(ok, "{key} did not round-trip through {flag}");
            assert!(usage().contains(flag), "--help lacks {flag}");
        }
    }

    #[test]
    fn bad_lines_are_reported_and_kept() {
        let (settings, warnings) = parse_file("vsync = maybe\nfps = 60\nshader = crt\n");
        assert_eq!(settings.picture.shader, Some("crt"));
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(warnings[0].contains("on or off"), "{warnings:?}");
        assert!(warnings[1].contains("unknown setting"), "{warnings:?}");
    }
}
