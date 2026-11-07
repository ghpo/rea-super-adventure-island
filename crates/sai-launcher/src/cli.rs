//! What a command line asks for (docs/STANDARDS.md "Family consistency":
//! `--game`, `--settings`, `--no-window`, `--check`, `--linux`, `--look`).

use std::path::{Path, PathBuf};

use rea_launcher::cfg::ArgError;
use rea_launcher::paths;
use rea_launcher_win32::report::{Audience, Run};

use crate::config::{self, Settings};
use crate::edition::{self, Found};

/// Exit codes. The Linux helper reads the `--settings` ones.
pub const OK: i32 = 0;
/// Refused: a bad ROM, no ROM folder, a failed `--check`.
pub const REFUSED: i32 = 1;
/// A bad command line (the family code, `report::BAD_COMMAND_LINE`).
pub const USAGE_ERROR: i32 = rea_launcher_win32::report::BAD_COMMAND_LINE;
/// `--settings`: the window was closed without Play.
pub const WINDOW_CLOSED: i32 = 3;
/// `--settings`: no window could be made (the helper then plays anyway).
pub const WINDOW_UNAVAILABLE: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `--help`, `-h`, `/?`.
    Help,
    /// The settings window; then the game, unless `settings_only`
    /// (`--settings`).
    Window { settings_only: bool },
    /// `--check`: identify the ROM and report, never launch.
    Check,
    /// `--no-window`, or any option flag: play with the saved settings and
    /// the flags on top.
    Play,
}

/// A parsed command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub mode: Mode,
    /// Started by the Linux helper (`--linux`).
    pub linux: bool,
    /// `--no-window` was given (a script, or the helper's game run).
    pub no_window: bool,
    /// The settings the flags set, to go on top of `launcher.cfg`.
    pub flags: Settings,
}

impl Invocation {
    /// The family flags of this run, for the SDK's audience rule.
    pub fn run(&self) -> Run {
        Run {
            command: matches!(self.mode, Mode::Help | Mode::Check),
            no_window: self.no_window,
            settings_only: self.mode
                == (Mode::Window {
                    settings_only: true,
                }),
            linux: self.linux,
        }
    }

    /// Who may see this run's messages (`wine`: running under Wine). The
    /// SDK decides it; never write `!no_window` or `!linux` here.
    pub fn audience(&self, wine: bool) -> Audience {
        Audience::of(self.run(), wine)
    }

    /// Whether the run does work without the window (`--check`).
    pub fn is_action(&self) -> bool {
        self.mode == Mode::Check
    }
}

/// Reads the command line (without the program name). The error is the
/// message for the player.
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Invocation, String> {
    let args: Vec<String> = args.into_iter().collect();
    let (flags, switches) = match config::parse_args(args.clone()) {
        Ok(parsed) => parsed,
        Err(ArgError::Help(_)) => {
            return Ok(Invocation {
                mode: Mode::Help,
                linux: args.iter().any(|arg| arg == "--linux"),
                no_window: args.iter().any(|arg| arg == "--no-window"),
                flags: Settings::default(),
            });
        }
        Err(error) => {
            return Err(format!(
                "{error}\n\nRun SAILauncher --help for the options."
            ));
        }
    };
    let has = |switch: &str| switches.iter().any(|s| s == switch);
    let options = flags != Settings::default();
    let mode = if has("--check") {
        Mode::Check
    } else if has("--settings") {
        if has("--no-window") {
            return Err("--settings and --no-window cannot be used together.".into());
        }
        Mode::Window {
            settings_only: true,
        }
    } else if has("--no-window") || options {
        Mode::Play
    } else {
        Mode::Window {
            settings_only: false,
        }
    };
    Ok(Invocation {
        mode,
        linux: has("--linux"),
        no_window: has("--no-window"),
        flags,
    })
}

/// The ROM folder/file from the settings, made absolute against `cwd`.
pub fn game_path(settings: &Settings, cwd: &Path) -> Result<PathBuf, String> {
    let game = settings.game.as_deref().ok_or_else(|| {
        format!(
            "No ROM yet. Choose it in the settings window, or run:\n\n\
             SAILauncher --game <the folder with {}>",
            edition::ROM_NAME
        )
    })?;
    paths::resolve(game, cwd)
}

/// `--check`: the ROM and its edition, or why it cannot be played.
pub fn check(settings: &Settings, cwd: &Path) -> Result<(PathBuf, Found), String> {
    let game = game_path(settings, cwd)?;
    let found = edition::find(&game)?;
    Ok((game, found))
}

/// The `--check` report for a playable ROM.
pub fn check_report(game: &Path, found: &Found) -> String {
    format!(
        "Ready to play: {} ({}) from {}",
        found.rom.file_name().map_or(edition::ROM_NAME, |name| name
            .to_str()
            .unwrap_or(edition::ROM_NAME)),
        found.edition.name,
        game.display()
    )
}
