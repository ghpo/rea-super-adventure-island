// A GUI program on Windows: no console window beside the settings window on
// a double click. Commands reach the terminal they were started from
// through the SDK's console attach.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use rea_launcher::cfg;
use rea_launcher::fsx;
use rea_launcher::log::Log;
#[cfg(windows)]
use rea_launcher_win32::report;
use rea_launcher_win32::report::{Audience, Run};

use sai_launcher::cli::{self, Invocation, Mode};
use sai_launcher::controls::Controls;
use sai_launcher::{config, edition, retroarch, runtime, ui};

/// Who may see this run's messages, decided by the SDK.
fn audience(run: Run) -> Audience {
    #[cfg(windows)]
    {
        rea_launcher_win32::report::audience(run)
    }
    #[cfg(not(windows))]
    {
        Audience::of(run, false)
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let invocation = match cli::parse(args.clone()) {
        Ok(invocation) => invocation,
        Err(message) => {
            report_error(&message, audience(Run::bad_command_line(&args)));
            return exit_code(cli::USAGE_ERROR);
        }
    };
    if invocation.mode == Mode::Help {
        command_output(&config::usage(), audience(invocation.run()), true);
        return ExitCode::SUCCESS;
    }
    let says = audience(invocation.run());
    let _instance = match single_instance(&invocation, says) {
        Ok(guard) => guard,
        Err(code) => return code,
    };
    match run(invocation, says) {
        Ok(code) => exit_code(code),
        Err(message) => {
            report_error(&message, says);
            exit_code(cli::REFUSED)
        }
    }
}

fn exit_code(code: i32) -> ExitCode {
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}

/// `launcher.log`.
fn log() -> Log {
    Log::new(runtime::log_path())
}

/// A command's output (`--help`, `--check`): the terminal, or a dialog
/// when nobody sees it and a person started the launcher.
fn command_output(text: &str, says: Audience, dialog: bool) {
    #[cfg(windows)]
    {
        let route = report::route_here(says.mode());
        if route.print {
            println!("{text}");
        }
        if dialog && route.dialog {
            use rea_launcher_win32::dialog::{self, Icon};
            dialog::message(sai_launcher::WINDOW_TITLE, text, Icon::Info);
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (says, dialog);
        println!("{text}");
    }
}

/// A fatal error: always in `launcher.log`, then through the SDK's report
/// for this run's audience.
fn report_error(message: &str, says: Audience) {
    let log = log();
    log.line(&format!("error: {message}"));
    #[cfg(windows)]
    {
        let text = format!("{message}\n\nDetails: {}", log.path().display());
        rea_launcher_win32::report::fatal(sai_launcher::WINDOW_TITLE, &text, says.mode());
    }
    #[cfg(not(windows))]
    {
        let details = format!(
            "SAILauncher {}\nlauncher.log: {}",
            env!("CARGO_PKG_VERSION"),
            log.path().display()
        );
        ui::fatal(message, &details, says);
    }
}

/// One launcher at a time (STANDARDS "Clean exit, single instance").
#[cfg(windows)]
fn single_instance(
    invocation: &Invocation,
    says: Audience,
) -> Result<rea_launcher_win32::instance::Guard, ExitCode> {
    use rea_launcher_win32::instance::{self, Window};
    match instance::single(sai_launcher::INSTANCE) {
        Ok(guard) => Ok(guard),
        Err(running) => {
            let second = running.second_start(
                invocation.is_action(),
                Window::Own(sai_launcher::WINDOW_TITLE),
            );
            log().line(second.log_line());
            if let Some(text) = second.message(sai_launcher::WINDOW_TITLE) {
                report_error(&text, says);
            }
            Err(exit_code(second.exit_code()))
        }
    }
}

/// The native Linux lock (the helper's session lock does not cover a
/// launcher started straight from a menu or the AppImage).
#[cfg(not(windows))]
fn single_instance(
    _invocation: &Invocation,
    says: Audience,
) -> Result<rea_launcher::instance::Guard, ExitCode> {
    match rea_launcher::instance::single(sai_launcher::INSTANCE) {
        Ok(guard) => Ok(guard),
        Err(running) => {
            log().line("second start: another launcher holds the instance; refused");
            report_error(
                &format!(
                    "The {} launcher or its game is already running. Quit the game or close \
                     that window first, then start again.",
                    sai_launcher::WINDOW_TITLE
                ),
                says,
            );
            Err(exit_code(running.exit_code()))
        }
    }
}

/// The ROM beside the launcher, only when the saved choice is missing or
/// no longer playable. A valid saved choice always wins.
fn auto_detect(settings: &config::Settings, cwd: &Path) -> Option<edition::Found> {
    if settings.game.is_some() && cli::check(settings, cwd).is_ok() {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    edition::find_near(exe.parent()?)
}

fn run(invocation: Invocation, says: Audience) -> Result<i32, String> {
    let log = log();
    log.line(concat!("SAILauncher ", env!("CARGO_PKG_VERSION")));
    let cfg_path = runtime::settings_path();
    let controls_path = runtime::controls_path();
    let cwd = std::env::current_dir().unwrap_or_default();
    let (mut controls, control_warnings) = Controls::read(&controls_path);
    for warning in &control_warnings {
        log.warn(warning);
    }
    let existing = cfg::read_file(&cfg_path);
    let first_run = matches!(&existing, Ok(None));
    let mut text = match existing {
        Ok(Some(text)) => text,
        Ok(None) => {
            let text = config::first_run_text(None);
            if let Some(parent) = cfg_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fsx::write_atomic(&cfg_path, &text)
                .map_err(|error| format!("Cannot save {}: {error}", cfg_path.display()))?;
            log.line("first run: wrote launcher.cfg with the recommended look");
            text
        }
        Err(error) => {
            log.warn(&format!(
                "{}: {error}; using the defaults",
                cfg_path.display()
            ));
            config::template()
        }
    };

    // If the player has not chosen a working ROM yet, look in the package
    // layout (roms/ next to or above the exe). This also lets a VM without
    // OpenGL play when the settings window cannot open.
    let (early, _) = config::parse_file(&text);
    if let Some(found) = auto_detect(&early, &cwd) {
        log.line(&format!(
            "auto-detected ROM: {} ({})",
            found.rom.display(),
            found.edition.name
        ));
        text = config::set(&text, "game", &found.rom.display().to_string());
        if let Some(parent) = cfg_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fsx::write_atomic(&cfg_path, &text)
            .map_err(|error| format!("Cannot save {}: {error}", cfg_path.display()))?;
    }

    if let Mode::Window { settings_only } = invocation.mode {
        match ui::show(
            &cfg_path,
            Some(&text),
            &controls_path,
            &controls,
            invocation.linux,
        ) {
            Ok(ui::Outcome::Play) => {
                text = cfg::read_file(&cfg_path).ok().flatten().unwrap_or(text);
                controls = Controls::read(&controls_path).0;
                if settings_only {
                    return Ok(cli::OK);
                }
            }
            Ok(ui::Outcome::Closed) => {
                return Ok(if settings_only {
                    cli::WINDOW_CLOSED
                } else {
                    cli::OK
                });
            }
            Err(ui::ShowError::NoWindow(error)) => {
                log.warn(&format!("settings window unavailable: {error}"));
                if settings_only {
                    return Ok(cli::WINDOW_UNAVAILABLE);
                }
            }
            Err(ui::ShowError::Failed(error)) => {
                return Err(format!("The settings window failed: {error}"));
            }
        }
    }

    let (saved, warnings) = config::parse_file(&text);
    for warning in &warnings {
        log.warn(warning);
    }
    let settings = saved.clone().merged(invocation.flags.clone());
    let (game, found) = cli::check(&settings, &cwd)?;
    log.line(&format!(
        "game: {} ({}, {})",
        found.rom.display(),
        found.edition.name,
        game.display()
    ));
    if invocation.mode == Mode::Check {
        command_output(&cli::check_report(&game, &found), says, true);
        return Ok(cli::OK);
    }

    if first_run || saved.game.as_ref() != Some(&game) {
        let text = config::set(&text, "game", &game.display().to_string());
        if let Some(parent) = cfg_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fsx::write_atomic(&cfg_path, text)
            .map_err(|error| format!("Cannot save {}: {error}", cfg_path.display()))?;
    }

    log.line(&format!("picture: {}", settings.picture(None).describe()));
    let paths = runtime::prepare(&found, &settings)?;
    log.line(&format!(
        "runtime: {}; saves: {}; autoconfig: {}",
        paths.root.display(),
        paths.saves.display(),
        paths.autoconfig.display()
    ));
    let launch = retroarch::prepare(&settings, &found, &paths, &controls)?;
    log.line(&format!(
        "core: {} ({}); shader: {}",
        launch.core.display(),
        launch.core.path.display(),
        launch
            .shader
            .as_ref()
            .map_or("off".to_owned(), |path| path.display().to_string())
    ));
    let code = retroarch::start(&launch, &paths, &log)?;
    log.line(&format!("game exited with {code}"));
    Ok(code)
}
