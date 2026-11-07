//! The emulator side: find RetroArch and the game's core, write the private
//! RetroArch configuration and the core options, then run the ROM.
//!
//! Nothing here writes to the player's RetroArch configuration: the
//! launcher passes its own `--config`, its own `rgui_config_directory` and
//! its own save folders.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rea_launcher::log::Log;

use crate::config::{Picture, Settings};
use crate::controls::{Controls, SnesButton};
use crate::edition::Found;
use crate::runtime::Paths;

/// A found core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Core {
    /// `snes9x_libretro.so` and friends.
    pub path: PathBuf,
    /// The name RetroArch uses for the per-core options folder
    /// (`Snes9x`, `bsnes-hd beta`).
    pub name: &'static str,
}

impl Core {
    pub fn display(&self) -> &'static str {
        self.name
    }
}

/// Everything one launch needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub program: PathBuf,
    pub core: Core,
    pub rom: PathBuf,
    pub config: PathBuf,
    pub shader: Option<PathBuf>,
    pub retroarch_log: PathBuf,
}

/// Finds RetroArch and the core, writes the configuration and the core
/// options, and returns what to run.
pub fn prepare(
    settings: &Settings,
    found: &Found,
    paths: &Paths,
    controls: &Controls,
) -> Result<Launch, String> {
    let program = find_retroarch(settings)?;
    let core = find_core(settings, settings.core(), &program)?;
    let shader = settings.picture.shader.unwrap_or("off");
    let shader_dir = if shader == "off" {
        None
    } else {
        Some(find_shader_dir(settings, &program)?)
    };
    let shader = match (shader_dir.as_deref(), shader_preset(&settings.picture)) {
        (Some(dir), Some(relative)) => {
            let path = dir.join(relative);
            if !path.is_file() {
                return Err(format!(
                    "The shader preset {} is missing. Install the libretro slang shaders, or \
                     choose shader = off.",
                    path.display()
                ));
            }
            Some(path)
        }
        _ => None,
    };

    let text = config_text(settings, paths, &core, shader_dir.as_deref(), controls);
    fs::write(&paths.retroarch_cfg, text)
        .map_err(|error| format!("Cannot write {}: {error}", paths.retroarch_cfg.display()))?;
    write_core_options(paths, &core, &settings.picture)?;

    Ok(Launch {
        program,
        core,
        rom: found.rom.clone(),
        config: paths.retroarch_cfg.clone(),
        shader,
        retroarch_log: paths.retroarch_log.clone(),
    })
}

/// Runs the game and waits. The process output goes to files in the
/// runtime's log folder; the launcher's own log records the command.
pub fn start(launch: &Launch, paths: &Paths, log: &Log) -> Result<i32, String> {
    let stdout = fs::File::create(paths.logs.join("retroarch-stdout.log"))
        .map_err(|error| format!("Cannot write the RetroArch stdout log: {error}"))?;
    let stderr = fs::File::create(paths.logs.join("retroarch-stderr.log"))
        .map_err(|error| format!("Cannot write the RetroArch stderr log: {error}"))?;
    let _scale = ScaleGuard::pixel_perfect(log);
    let mut command = Command::new(&launch.program);
    command
        .arg("-L")
        .arg(&launch.core.path)
        .arg("--config")
        .arg(&launch.config)
        .arg("--verbose")
        .arg("--log-file")
        .arg(&launch.retroarch_log);
    if let Some(shader) = &launch.shader {
        command.arg(format!("--set-shader={}", display_path(shader)));
    }
    // Test seams: the real-game checks run RetroArch for a fixed number of
    // frames and save the last frame (docs/RUNTIME_NOTES.md).
    if let Ok(frames) = std::env::var("SAI_MAX_FRAMES")
        && frames.chars().all(|c| c.is_ascii_digit())
        && !frames.is_empty()
    {
        command.arg("--max-frames").arg(&frames);
    }
    if let Ok(path) = std::env::var("SAI_TEST_SCREENSHOT") {
        command
            .arg("--max-frames-ss")
            .arg("--max-frames-ss-path")
            .arg(path);
    }
    command
        .arg(&launch.rom)
        .current_dir(&paths.root)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    log.line(&format!(
        "retroarch: {:?} -L {} --config {} {}",
        launch.program,
        launch.core.path.display(),
        launch.config.display(),
        launch
            .shader
            .as_ref()
            .map_or(String::new(), |s| format!("--set-shader={}", s.display()))
    ));
    let status = command.status().map_err(|error| {
        format!(
            "Cannot start RetroArch ({}): {error}",
            launch.program.display()
        )
    })?;
    Ok(status.code().unwrap_or(1))
}

/// The configuration RetroArch reads. All paths are absolute; the player's
/// own RetroArch configuration is not touched.
pub fn config_text(
    settings: &Settings,
    paths: &Paths,
    core: &Core,
    shader_dir: Option<&Path>,
    controls: &Controls,
) -> String {
    let p = &settings.picture;
    let shader = p.shader.unwrap_or("off");
    let core_dir = core.path.parent().unwrap_or_else(|| Path::new(""));
    let mut out = String::new();
    let mut put = |key: &str, value: &str| {
        out.push_str(key);
        out.push_str(" = \"");
        out.push_str(value);
        out.push_str("\"\n");
    };

    put("libretro_directory", &display_path(core_dir));
    put("rgui_config_directory", &display_path(&paths.config_dir));
    put(
        "core_options_path",
        &display_path(&paths.config_dir.join("core-options.cfg")),
    );
    put("joypad_autoconfig_dir", &display_path(&paths.autoconfig));
    put("system_directory", &display_path(&paths.system));
    put("savefile_directory", &display_path(&paths.saves));
    put("savestate_directory", &display_path(&paths.states));
    put("screenshot_directory", &display_path(&paths.screenshots));
    put("log_dir", &display_path(&paths.logs));
    // D3D11 works in Windows VMs with WARP; OpenGL is the tested Linux path.
    put("video_driver", if cfg!(windows) { "d3d11" } else { "gl" });
    put("video_scale", "4");
    put("video_windowed_fullscreen", "false");
    put("video_smooth", "false");
    put("aspect_ratio_index", "21");
    put("custom_viewport_width", "0");
    put("custom_viewport_height", "0");
    put("custom_viewport_x", "0");
    put("custom_viewport_y", "0");
    put(
        "video_shader_enable",
        if shader == "off" { "false" } else { "true" },
    );
    if let Some(dir) = shader_dir {
        put("video_shader_dir", &display_path(dir));
    }
    put("audio_sync", "true");
    put("audio_rate_control", "true");
    put("audio_rate_control_delta", "0.005");
    put("config_save_on_exit", "false");
    // The Controls tab: keyboard keys always, the controller explicitly only
    // for a preset (Auto keeps RetroArch's system autoconfig).
    for button in SnesButton::ALL {
        put(button.input(), controls.key(button));
    }
    for (key, value) in controls.pad_keys() {
        put(key, value);
    }
    // Explicit quit/menu keys: the private config is the only one RetroArch
    // loads, so the defaults are written out and are visible to the player.
    put("input_exit_emulator", "escape");
    put("input_menu_toggle", "f1");
    put("pause_nonactive", "true");
    put("input_autodetect_enable", "true");
    #[cfg(not(windows))]
    {
        put("input_joypad_driver", "udev");
        put("audio_driver", "pulse");
    }
    if let Some(on) = p.fullscreen {
        put("video_fullscreen", if on { "true" } else { "false" });
    }
    if let Some(on) = p.vsync {
        put("video_vsync", if on { "true" } else { "false" });
    }
    if let Some(on) = p.integer_scale {
        put("video_scale_integer", if on { "true" } else { "false" });
    }
    if let Some(on) = p.show_fps {
        put("fps_show", if on { "true" } else { "false" });
    }
    if let Some(frames) = p.run_ahead {
        let enabled = frames != "0";
        put("run_ahead_enabled", if enabled { "true" } else { "false" });
        put("run_ahead_frames", frames);
    }
    out
}

/// The shader preset's path under the shader root.
pub fn shader_preset(picture: &Picture) -> Option<&'static str> {
    match picture.shader.unwrap_or("off") {
        "off" => None,
        "sharp" => Some("pixel-art-scaling/sharp-bilinear-simple.slangp"),
        "crt" => Some("crt/zfast-crt.slangp"),
        "smooth" => Some("edge-smoothing/xbr/xbr-lv2.slangp"),
        _ => None,
    }
}

/// Writes the per-core options. Only the keys the launcher owns are written;
/// the core keeps its defaults for everything else.
fn write_core_options(paths: &Paths, core: &Core, picture: &Picture) -> Result<(), String> {
    let dir = paths.config_dir.join(core.name);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Cannot create {}: {error}", dir.display()))?;
    let options = dir.join(format!("{}.opt", core.name));
    let text = match core.name {
        "Snes9x" => snes9x_options(picture),
        "bsnes-hd beta" => bsnes_options(picture),
        _ => String::new(),
    };
    fs::write(&options, text)
        .map_err(|error| format!("Cannot write {}: {error}", options.display()))?;
    // The global core-options file is empty: the per-core file above is the
    // one the launcher owns.
    let globals = paths.config_dir.join("core-options.cfg");
    if !globals.is_file() {
        fs::write(
            &globals,
            "# Managed by the Super Adventure Island launcher.\n",
        )
        .map_err(|error| format!("Cannot write {}: {error}", globals.display()))?;
    }
    Ok(())
}

/// Snes9x's own values.
pub fn snes9x_options(p: &Picture) -> String {
    let mut out = String::new();
    let mut put = |key: &str, value: &str| {
        out.push_str(key);
        out.push_str(" = \"");
        out.push_str(value);
        out.push_str("\"\n");
    };
    if let Some(aspect) = p.aspect {
        put(
            "snes9x_aspect",
            match aspect {
                "4:3" => "4:3",
                "pixel" => "uncorrected",
                // Snes9x has no widescreen; keep it in 4:3.
                _ => "4:3",
            },
        );
    }
    if let Some(overclock) = p.overclock {
        put(
            "snes9x_overclock_cycles",
            match overclock {
                "off" => "disabled",
                "light" => "light",
                "max" => "max",
                _ => "compatible",
            },
        );
    }
    if let Some(mode7) = p.mode7_hd {
        put(
            "snes9x_mode7_hires",
            match mode7 {
                "off" => "disabled",
                "4x" => "4x",
                _ => "2x",
            },
        );
    }
    put(
        "snes9x_hires_blend",
        if p.hires_blend.unwrap_or(false) {
            "merge"
        } else {
            "disabled"
        },
    );
    put(
        "snes9x_reduce_sprite_flicker",
        if p.reduce_flicker.unwrap_or(false) {
            "enabled"
        } else {
            "disabled"
        },
    );
    put("snes9x_overscan", "enabled");
    out
}

/// bsnes-hd's own values.
pub fn bsnes_options(p: &Picture) -> String {
    let mut out = String::new();
    let mut put = |key: &str, value: &str| {
        out.push_str(key);
        out.push_str(" = \"");
        out.push_str(value);
        out.push_str("\"\n");
    };
    if let Some(aspect) = p.aspect {
        put(
            "bsnes_video_aspectcorrection",
            if aspect == "4:3" { "ON" } else { "OFF" },
        );
    }
    if let Some(overclock) = p.overclock {
        put(
            "bsnes_cpu_overclock",
            match overclock {
                "off" => "100",
                "light" => "110",
                "max" => "200",
                _ => "150",
            },
        );
    }
    if let Some(mode7) = p.mode7_hd {
        put(
            "bsnes_mode7_scale",
            match mode7 {
                "off" => "disable",
                "4x" => "4x",
                _ => "2x",
            },
        );
    }
    if let Some(on) = p.widescreen {
        put("bsnes_mode7_widescreen", if on { "16:9" } else { "none" });
    }
    put("bsnes_mode7_wsMode", "Mode 7");
    put(
        "bsnes_ppu_no_sprite_limit",
        if p.reduce_flicker.unwrap_or(false) {
            "ON"
        } else {
            "OFF"
        },
    );
    put("bsnes_ppu_show_overscan", "OFF");
    out
}

/// Finds `retroarch` (or `retroarch.exe`).
pub fn find_retroarch(settings: &Settings) -> Result<PathBuf, String> {
    if let Some(path) = &settings.retroarch {
        let program = if path.is_dir() {
            path.join(program_name())
        } else {
            path.clone()
        };
        if program.is_file() {
            return Ok(program);
        }
        return Err(format!(
            "RetroArch was not found at {}. Point --retroarch at the program.",
            program.display()
        ));
    }
    if let Some(program) = search_path(program_name()) {
        return Ok(program);
    }
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_default();
    let mut candidates = vec![
        exe_dir.join(program_name()),
        exe_dir.join("RetroArch-Win64").join(program_name()),
        exe_dir.join("retroarch").join(program_name()),
    ];
    if cfg!(unix) {
        candidates.extend([
            PathBuf::from("/usr/bin/retroarch"),
            PathBuf::from("/usr/local/bin/retroarch"),
        ]);
    }
    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(if cfg!(windows) {
        "RetroArch was not found. The Windows package should include retroarch.exe next to \
         the launcher; reinstall the package, or pass --retroarch <path>."
            .to_owned()
    } else {
        "RetroArch was not found. On Arch, install it with:\n\n  \
         sudo pacman -S retroarch libretro-snes9x libretro-shaders-slang\n\n\
         Or pass --retroarch <path>."
            .to_owned()
    })
}

/// Finds the selected core (Snes9x or bsnes-hd).
pub fn find_core(settings: &Settings, core: &str, program: &Path) -> Result<Core, String> {
    let (file, name) = match core {
        "bsnes_hd" => (bsnes_file(), "bsnes-hd beta"),
        _ => (snes9x_file(), "Snes9x"),
    };
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(dir) = &settings.core_dir {
        if dir.is_file() {
            if dir.file_name().is_some_and(|n| n == file) {
                return Ok(Core {
                    path: dir.clone(),
                    name,
                });
            }
        } else {
            dirs.push(dir.clone());
        }
    }
    if let Some(parent) = program.parent() {
        dirs.push(parent.join("cores"));
        if let Some(grand) = parent.parent() {
            dirs.push(grand.join("cores"));
        }
    }
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        dirs.push(exe_dir.join("cores"));
        dirs.push(exe_dir.join("RetroArch-Win64/cores"));
        dirs.push(exe_dir.join("retroarch/cores"));
    }
    if cfg!(unix) {
        for dir in [
            "/usr/lib/libretro",
            "/usr/lib64/libretro",
            "/usr/local/lib/libretro",
        ] {
            dirs.push(PathBuf::from(dir));
        }
        if let Some(home) = std::env::var_os("HOME") {
            dirs.push(PathBuf::from(&home).join(".config/retroarch/cores"));
            dirs.push(PathBuf::from(&home).join(".local/lib/libretro"));
        }
    }
    for dir in dirs {
        let path = dir.join(file);
        if path.is_file() {
            return Ok(Core { path, name });
        }
    }
    Err(if core == "bsnes_hd" {
        "The bsnes-hd core was not found. On Arch, install it with:\n\n  \
         sudo pacman -S libretro-bsnes-hd\n\n\
         Or choose core = snes9x."
            .to_owned()
    } else if cfg!(windows) {
        format!(
            "The Snes9x core ({file}) was not found. The Windows package should include it in \
             cores/ next to the launcher."
        )
    } else {
        format!(
            "The Snes9x core ({file}) was not found. On Arch, install it with:\n\n  \
             sudo pacman -S libretro-snes9x"
        )
    })
}

/// Finds the root of the slang shaders that contains the presets.
pub fn find_shader_dir(settings: &Settings, program: &Path) -> Result<PathBuf, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = &settings.shader_dir {
        candidates.push(dir.clone());
        candidates.push(dir.join("shaders_slang"));
    }
    if let Some(parent) = program.parent() {
        candidates.push(parent.join("shaders/shaders_slang"));
        candidates.push(parent.join("shaders"));
    }
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        candidates.push(exe_dir.join("shaders/shaders_slang"));
        candidates.push(exe_dir.join("RetroArch-Win64/shaders/shaders_slang"));
    }
    if cfg!(unix) {
        candidates.push(PathBuf::from("/usr/share/libretro/shaders/shaders_slang"));
        if let Some(home) = std::env::var_os("HOME") {
            candidates.push(PathBuf::from(home).join(".config/retroarch/shaders/shaders_slang"));
        }
    }
    for dir in &candidates {
        if dir.join("crt/zfast-crt.slangp").is_file()
            || dir
                .join("pixel-art-scaling/sharp-bilinear-simple.slangp")
                .is_file()
        {
            return Ok(dir.clone());
        }
    }
    Err(
        "The slang shaders were not found. On Arch, install them with:\n\n  \
         sudo pacman -S libretro-shaders-slang\n\n\
         Or choose shader = off."
            .to_owned(),
    )
}

fn search_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

fn program_name() -> &'static str {
    if cfg!(windows) {
        "retroarch.exe"
    } else {
        "retroarch"
    }
}

fn snes9x_file() -> &'static str {
    if cfg!(windows) {
        "snes9x_libretro.dll"
    } else {
        "snes9x_libretro.so"
    }
}

fn bsnes_file() -> &'static str {
    if cfg!(windows) {
        "bsnes_hd_beta_libretro.dll"
    } else {
        "bsnes_hd_beta_libretro.so"
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// On Hyprland with a fractional monitor scale, a fullscreen client gets a
/// smaller logical surface and the 256×224 picture is resampled. While the
/// game runs, set the focused monitor to scale 1.0 and restore it on drop.
#[cfg(target_os = "linux")]
mod scale {
    use std::process::Command;

    use rea_launcher::log::Log;
    use serde_json::Value;

    pub struct ScaleGuard {
        monitor: String,
        mode: String,
        position: String,
        scale: f64,
    }

    impl ScaleGuard {
        pub fn pixel_perfect(log: &Log) -> Option<ScaleGuard> {
            std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;
            let output = Command::new("hyprctl")
                .args(["monitors", "-j"])
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let monitors: Value = serde_json::from_slice(&output.stdout).ok()?;
            let monitor = monitors
                .as_array()?
                .iter()
                .find(|m| m.get("focused").and_then(Value::as_bool) == Some(true))
                .or_else(|| monitors.as_array()?.first())?;
            let scale = monitor.get("scale")?.as_f64()?;
            if (scale - 1.0).abs() < 0.001 {
                return None;
            }
            let name = monitor.get("name")?.as_str()?.to_owned();
            let width = monitor.get("width")?.as_u64()?;
            let height = monitor.get("height")?.as_u64()?;
            let refresh = monitor
                .get("refreshRate")
                .and_then(Value::as_f64)
                .unwrap_or(60.0);
            let x = monitor.get("x").and_then(Value::as_i64).unwrap_or(0);
            let y = monitor.get("y").and_then(Value::as_i64).unwrap_or(0);
            let mode = format!("{width}x{height}@{refresh}");
            let position = format!("{x}x{y}");
            let guard = ScaleGuard {
                monitor: name,
                mode,
                position,
                scale,
            };
            if guard.set(1.0) {
                log.line(&format!(
                    "monitor {}: scale {} -> 1 for integer scaling (restored after the game)",
                    guard.monitor, guard.scale
                ));
                Some(guard)
            } else {
                None
            }
        }

        fn set(&self, scale: f64) -> bool {
            let lua = format!(
                "hl.monitor({{output=\"{}\", mode=\"{}\", position=\"{}\", scale={}}})",
                self.monitor, self.mode, self.position, scale
            );
            Command::new("hyprctl")
                .args(["eval", &lua])
                .output()
                .is_ok_and(|output| output.status.success())
        }
    }

    impl Drop for ScaleGuard {
        fn drop(&mut self) {
            self.set(self.scale);
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod scale {
    use rea_launcher::log::Log;

    pub struct ScaleGuard;

    impl ScaleGuard {
        pub fn pixel_perfect(_log: &Log) -> Option<ScaleGuard> {
            None
        }
    }
}

use scale::ScaleGuard;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Look;
    use crate::config::Settings;
    use crate::controls::{Controls, PadLayout};
    use crate::runtime::Paths;

    #[test]
    fn the_enhanced_look_writes_the_verified_snes9x_values() {
        let p = Look::Enhanced.picture(Some((1920, 1080)));
        let text = snes9x_options(&p);
        assert!(text.contains("snes9x_aspect = \"uncorrected\""), "{text}");
        assert!(
            text.contains("snes9x_overclock_cycles = \"compatible\""),
            "{text}"
        );
        assert!(text.contains("snes9x_mode7_hires = \"2x\""), "{text}");
        assert!(text.contains("snes9x_hires_blend = \"merge\""), "{text}");
        assert!(
            text.contains("snes9x_reduce_sprite_flicker = \"enabled\""),
            "{text}"
        );
    }

    #[test]
    fn the_original_look_keeps_the_core_defaults() {
        let p = Look::Original.picture(Some((1920, 1080)));
        let text = snes9x_options(&p);
        assert!(text.contains("snes9x_aspect = \"4:3\""), "{text}");
        assert!(
            text.contains("snes9x_overclock_cycles = \"disabled\""),
            "{text}"
        );
    }

    #[test]
    fn the_widescreen_look_writes_bsnes_hd_values() {
        let p = Look::Widescreen.picture(Some((1920, 1080)));
        let text = bsnes_options(&p);
        assert!(text.contains("bsnes_mode7_widescreen = \"16:9\""), "{text}");
        assert!(text.contains("bsnes_mode7_wsMode = \"Mode 7\""), "{text}");
        assert!(
            text.contains("bsnes_video_aspectcorrection = \"OFF\""),
            "{text}"
        );
    }

    fn test_paths() -> Paths {
        Paths {
            root: "/tmp/sai-test".into(),
            config_dir: "/tmp/sai-test/config".into(),
            saves: "/tmp/sai-test/saves".into(),
            states: "/tmp/sai-test/states".into(),
            screenshots: "/tmp/sai-test/screenshots".into(),
            logs: "/tmp/sai-test/logs".into(),
            system: "/tmp/sai-test/system".into(),
            autoconfig: "/tmp/sai-test/autoconfig".into(),
            retroarch_cfg: "/tmp/sai-test/retroarch.cfg".into(),
            retroarch_log: "/tmp/sai-test/logs/retroarch.log".into(),
        }
    }

    #[test]
    fn the_controls_are_written_into_the_retroarch_config() {
        let core = Core {
            path: "/usr/lib/libretro/snes9x_libretro.so".into(),
            name: "Snes9x",
        };
        let controls = Controls::default();
        let text = config_text(&Settings::default(), &test_paths(), &core, None, &controls);
        assert!(text.contains("input_player1_a = \"x\""), "{text}");
        assert!(text.contains("input_player1_start = \"enter\""), "{text}");
        assert!(!text.contains("input_player1_a_btn"), "{text}");

        let mut nintendo = controls.clone();
        nintendo.pad = PadLayout::Nintendo;
        let text = config_text(&Settings::default(), &test_paths(), &core, None, &nintendo);
        assert!(text.contains("input_player1_a_btn = \"1\""), "{text}");
        assert!(text.contains("input_player1_b_btn = \"0\""), "{text}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn the_shader_presets_exist_in_the_installed_package() {
        let dir = find_shader_dir(
            &crate::config::Settings::default(),
            Path::new("/usr/bin/retroarch"),
        );
        if let Ok(dir) = dir {
            for look in [Look::Enhanced, Look::Crt] {
                let p = look.picture(None);
                if let Some(relative) = shader_preset(&p) {
                    assert!(
                        dir.join(relative).is_file(),
                        "{}",
                        dir.join(relative).display()
                    );
                }
            }
        }
    }
}
