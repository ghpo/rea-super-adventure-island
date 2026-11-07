# Super Adventure Island on Linux

The launcher is a native Linux program (not Wine/Proton). It uses the
system RetroArch with the Snes9x core and writes a private configuration;
your `~/.config/retroarch` is not changed. The ROM is only read.

## Packages

On Arch:

```sh
sudo pacman -S retroarch libretro-snes9x libretro-shaders-slang libretro-bsnes-hd
```

`libretro-shaders-slang` is only needed for the CRT/Sharp/Smooth looks;
`libretro-bsnes-hd` only for the Widescreen look. `jq` is used by the
Hyprland fractional-scale fix; without it the game still runs, at the
monitor's scale.

## The helper

`linux/sai-play` (next to the launcher in a package, or one folder up with
`../target/release/SAILauncher` in a checkout) sources
`sdk/rea-play.sh` or a `rea-play.sh` beside it:

```sh
linux/sai-play             # settings window, then the game
linux/sai-play --play      # game with the saved settings, no window
linux/sai-play stop        # end the session
linux/sai-play status      # running or idle
```

Launcher options after `--play` apply to that run, for example
`linux/sai-play --play --look crt`. `--check` identifies the ROM (the helper forwards it to the launcher):

```sh
linux/sai-play --check --game "/path/to/roms"
```

The helper runs the launcher with `--settings --linux` for the window and
with `--no-window --linux` for the game, so Play-time errors still reach a
message window. It keeps one session at a time (`~/.config/sai-launcher/running.pid`)
and uses the launcher's single-instance guard too.

## The package on this machine

The owner's local install is in `~/.local/share/sai-launcher/`:

- `SAILauncher` (the release binary)
- `sai-play` and `rea-play.sh`
- the menu entries in `~/.local/share/applications/`:
  `sai-launcher.desktop` (settings window) and `sai-launcher-game.desktop`
  (`sai-play --play`)
- the icon in `~/.local/share/icons/hicolor/256x256/apps/sai-launcher.png`

Add `~/.local/share/sai-launcher/sai-play --play` as a non-Steam game in
Steam for a direct game shortcut; the `.desktop` game entry is another way.
`~/.local/bin/sai-play` is a convenience symlink.

## Environment

| Variable | Meaning |
| --- | --- |
| `SAI_LAUNCHER` | the `SAILauncher` program (default: beside the helper, one folder up, or the repository's release/debug build) |
| `SAI_WAIT` | accepted for AppImage compatibility; every launch waits |
| `SAI_GPU` | the GPU environment for the game (`SAI_GPU=` for none, the default here; set the NVIDIA offload variables to use the discrete GPU) |
| `SAI_MAX_FRAMES`, `SAI_TEST_SCREENSHOT` | test seams: run RetroArch for N frames and save the last frame (used in `docs/RUNTIME_NOTES.md`) |

## Fractional scaling (Hyprland)

At monitor scale 1.25 a fullscreen RetroArch gets a 1536×864 logical
surface, so the 256×224 picture is resampled. While the game runs the
helper sets the focused monitor to scale 1 through `hyprctl eval` and
restores it on every exit path; the launcher has the same guard for a
direct start. Gamescope was tested and is deliberately off: it cost frames
on this Intel GPU.

## Where things are

| What | Where |
| --- | --- |
| Settings, controls and log | `~/.config/sai-launcher/launcher.cfg`, `controls.cfg`, `launcher.log` |
| Generated RetroArch config, states, logs | `~/.local/share/sai-launcher/runtime/` |
| Saves | `~/.local/share/sai-launcher/saves/` |
| The existing RetroArch save imported once | `~/.config/retroarch/saves/Snes9x/` |
| The ROM | wherever you keep it |

## Troubleshooting

- **`RetroArch was not found`**: install the packages above or pass
  `--retroarch <program>`, `--core-dir <folder>`, `--shader-dir <folder>`.
- **The window does not open**: the helper plays with the saved settings
  when the settings window exits with 4; check `settings.log` in
  `~/.config/sai-launcher`.
- **The scale looks wrong after a hard kill**: restore the monitor scale
  in your Hyprland config; the helper's trap covers normal exits,
  `stop`, Ctrl+C and `--play`.
- **The game log**: `~/.local/share/sai-launcher/runtime/logs/retroarch.log`.
