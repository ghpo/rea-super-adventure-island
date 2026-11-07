//! The Super Adventure Island launcher: everything that can be tested on a
//! Linux host lives in this library; `main.rs` wires it to the settings
//! window and to RetroArch.
//!
//! - [`edition`]: which ROM builds are known (SHA-256), and the refusal for
//!   any other build.
//! - [`config`]: the one option table behind `launcher.cfg`, the flags,
//!   `--help`, the template and the window rows.
//! - [`cli`]: what a command line asks for, and the `--check` report.
//! - [`form`]: the settings window's state as `launcher.cfg` text.
//! - [`controls`]: the keyboard bindings and controller layout (`controls.cfg`).
//! - [`retroarch`]: the private RetroArch configuration, core options and
//!   the launch.
//! - [`runtime`]: the launcher's own folders and the save import.
//! - `palette`, `ui`: the settings window.

/// The settings window's title. It must differ from RetroArch's window
/// title: a second start brings this window forward on Windows.
pub const WINDOW_TITLE: &str = "Super Adventure Island Launcher";

/// The single-instance name: one launcher at a time per user session.
pub const INSTANCE: &str = "rea-sai-launcher";

pub mod cli;
pub mod config;
pub mod controls;
pub mod edition;
pub mod form;
pub mod palette;
pub mod retroarch;
pub mod runtime;
pub mod ui;
