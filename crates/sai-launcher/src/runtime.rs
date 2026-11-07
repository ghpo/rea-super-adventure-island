//! Where the launcher keeps its own files: the generated RetroArch
//! configuration, logs, save files and states. The player's ROM folder is
//! never written to; the launcher only reads the ROM.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Settings;
use crate::edition::Found;

/// The application name in the XDG/AppData folders.
pub const APP: &str = "sai-launcher";

/// The Windows folder name (`%APPDATA%\SAILauncher`).
#[cfg(windows)]
const WINDOWS_NAME: &str = "SAILauncher";

/// The DualSense autoconfig bundled for the Windows package. Linux uses the
/// player's own RetroArch autoconfig folder when it exists, so other pads
/// keep working.
const DUALSENSE_CFG: &str = include_str!("../assets/autoconfig/udev/dualsense.cfg");

/// The save the existing RetroArch setup keeps for this ROM.
#[cfg(not(windows))]
const RETROARCH_SAVE: &str = "Super Adventure Island.srm";
const IMPORT_MARKER: &str = ".imported-from-retroarch";

/// `$XDG_CONFIG_HOME` (or `%APPDATA%`) plus the application folder.
pub fn config_home() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(WINDOWS_NAME)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(std::env::temp_dir)
            .join(APP)
    }
}

/// `$XDG_DATA_HOME` (or `%LOCALAPPDATA%`) plus the application folder.
pub fn data_home() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(WINDOWS_NAME)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
            .unwrap_or_else(std::env::temp_dir)
            .join(APP)
    }
}

/// `launcher.cfg`: beside the program when a portable copy is there, else
/// in [`config_home`].
pub fn settings_path() -> PathBuf {
    let portable = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(crate::config::FILE_NAME)));
    match portable {
        Some(path) if path.is_file() => path,
        _ => config_home().join(crate::config::FILE_NAME),
    }
}

/// `launcher.log`, beside the settings file.
pub fn log_path() -> PathBuf {
    settings_path().with_file_name("launcher.log")
}

/// `controls.cfg`, beside the settings file.
pub fn controls_path() -> PathBuf {
    settings_path().with_file_name(crate::controls::FILE_NAME)
}

pub fn default_runtime() -> PathBuf {
    data_home().join("runtime")
}

pub fn default_saves() -> PathBuf {
    data_home().join("saves")
}

/// The folders this run uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub root: PathBuf,
    pub config_dir: PathBuf,
    pub saves: PathBuf,
    pub states: PathBuf,
    pub screenshots: PathBuf,
    pub logs: PathBuf,
    pub system: PathBuf,
    /// Where RetroArch reads the pad autoconfig profiles.
    pub autoconfig: PathBuf,
    pub retroarch_cfg: PathBuf,
    pub retroarch_log: PathBuf,
}

/// Creates the runtime and saves folders, picks the autoconfig folder and
/// imports a save from the player's previous RetroArch setup once.
pub fn prepare(found: &Found, settings: &Settings) -> Result<Paths, String> {
    prepare_with(found, settings, true)
}

/// [`prepare`], optionally importing the old RetroArch save (tests disable
/// the import so they never read the player's home).
pub fn prepare_with(found: &Found, settings: &Settings, import: bool) -> Result<Paths, String> {
    let root = settings.runtime.clone().unwrap_or_else(default_runtime);
    let saves = settings.saves.clone().unwrap_or_else(default_saves);
    let game_root = if found.rom.is_file() {
        found.rom.parent().unwrap_or_else(|| Path::new(""))
    } else {
        found.rom.as_path()
    };
    if root.starts_with(game_root) {
        return Err(format!(
            "The runtime folder {} is inside the ROM folder. Choose another folder: the \
             game folder must stay read-only.",
            root.display()
        ));
    }
    if saves.starts_with(game_root) {
        return Err(format!(
            "The saves folder {} is inside the ROM folder. Choose another folder: the \
             game folder must stay read-only.",
            saves.display()
        ));
    }

    let config_dir = root.join("config");
    let states = root.join("states");
    let screenshots = root.join("screenshots");
    let logs = root.join("logs");
    let system = root.join("system");
    for dir in [
        &root,
        &config_dir,
        &saves,
        &states,
        &screenshots,
        &logs,
        &system,
    ] {
        fs::create_dir_all(dir)
            .map_err(|error| format!("Cannot create {}: {error}", dir.display()))?;
    }

    let autoconfig = autoconfig_dir(&root)?;
    if import {
        import_retroarch_save(&saves)?;
    }

    Ok(Paths {
        retroarch_cfg: root.join("retroarch.cfg"),
        retroarch_log: logs.join("retroarch.log"),
        root,
        config_dir,
        saves,
        states,
        screenshots,
        logs,
        system,
        autoconfig,
    })
}

/// The player's RetroArch autoconfig folder on Linux (their DualSense and
/// other profiles), else a private folder holding the bundled DualSense
/// profile (the Windows package).
fn autoconfig_dir(root: &Path) -> Result<PathBuf, String> {
    #[cfg(not(windows))]
    {
        let xdg = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));
        if let Some(dir) = xdg.map(|dir| dir.join("retroarch/autoconfig"))
            && dir.join("udev").is_dir()
        {
            return Ok(dir);
        }
    }

    let dir = root.join("autoconfig");
    let udev = dir.join("udev");
    fs::create_dir_all(&udev)
        .map_err(|error| format!("Cannot create {}: {error}", udev.display()))?;
    let profile = udev.join("dualsense.cfg");
    if !profile.is_file() {
        fs::write(&profile, DUALSENSE_CFG)
            .map_err(|error| format!("Cannot write {}: {error}", profile.display()))?;
    }
    Ok(dir)
}

/// Copies the save the player already has in RetroArch, once. A save that
/// appears later (the player deleted it here) is never brought back.
fn import_retroarch_save(saves: &Path) -> Result<(), String> {
    let marker = saves.join(IMPORT_MARKER);
    if marker.is_file() {
        return Ok(());
    }
    #[cfg(not(windows))]
    if let Some(config) = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
    {
        let source = config.join("retroarch/saves/Snes9x").join(RETROARCH_SAVE);
        let destination = saves.join(RETROARCH_SAVE);
        if source.is_file() && !destination.is_file() {
            fs::copy(&source, &destination).map_err(|error| {
                format!(
                    "Cannot import the existing save {}: {error}",
                    source.display()
                )
            })?;
        }
    }
    fs::write(&marker, b"imported\n")
        .map_err(|error| format!("Cannot write {}: {error}", marker.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::Edition;

    fn found(rom: PathBuf) -> Found {
        static EDITION: Edition = Edition {
            name: "test edition",
        };
        Found {
            rom,
            edition: &EDITION,
        }
    }

    #[test]
    fn the_runtime_folders_are_created() {
        let root = std::env::temp_dir().join(format!("sai-runtime-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let game = root.join("roms");
        let rom = game.join("Super Adventure Island.sfc");
        fs::create_dir_all(&game).unwrap();
        fs::write(&rom, b"rom").unwrap();
        let settings = Settings {
            runtime: Some(root.join("runtime")),
            saves: Some(root.join("saves")),
            ..Settings::default()
        };
        let paths = prepare_with(&found(rom.clone()), &settings, false).unwrap();
        assert!(paths.config_dir.is_dir());
        assert!(paths.saves.is_dir());
        assert!(paths.autoconfig.is_dir());
        // The ROM folder was not touched.
        assert_eq!(fs::read(&rom).unwrap(), b"rom");

        let paths = prepare_with(&found(rom), &settings, false).unwrap();
        assert!(paths.saves.is_dir());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_runtime_inside_the_rom_folder_is_refused() {
        let root = std::env::temp_dir().join(format!("sai-runtime-refuse-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let rom = root.join("Super Adventure Island.sfc");
        fs::create_dir_all(&root).unwrap();
        fs::write(&rom, b"rom").unwrap();
        let settings = Settings {
            runtime: Some(root.join("runtime")),
            saves: Some(root.join("saves")),
            ..Settings::default()
        };
        assert!(prepare_with(&found(rom), &settings, false).is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}
