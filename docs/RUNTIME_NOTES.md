# Runtime notes: Super Adventure Island

What was verified in the real game, how, and what was not. An option reaches
the settings window only after its effect has been seen here
(rea-launcher-sdk `docs/STANDARDS.md`, "No unverified options").

Most runs used the test seams `SAI_MAX_FRAMES=N` and
`SAI_TEST_SCREENSHOT=<path>`: RetroArch runs N frames headless of the
desktop and saves its last frame; the launcher/helper then exits normally,
including the monitor-scale restore. The gameplay captures used
`hyprctl dispatch 'hl.dsp.send_shortcut(...)'` to send Start and A into the
RetroArch window without opening a settings window.

Status: ✅ verified · ⚠️ partly (see notes) · ❌ fails · — not checked.

## Editions

| Edition | `--check` | Starts | Plays a session | Notes |
|---|---|---|---|---|
| PT-BR translation by Juliano_Dio (`6ee59178…a50e54f`) | ✅ | ✅ | ✅ first level | The title, the intro and the first level ran at 60.01–60.05 fps. |
| Original USA ROM | — | — | — | Not in the table; add its SHA-256 when the owner supplies the file. |
| Unknown file | ✅ refused with SHA-256 and the issue link | — | — | The `cli`/`edition` tests and the Windows exe test cover it. |

The ROM is headerless (1,048,576 bytes), region byte `01` (USA), and was
read in place; the runtime never copied or changed it.

## Looks and options

| Look / option | Window → `launcher.cfg` → config | Effect seen in the game | Notes |
|---|---|---|---|
| Original | 4:3, no shader, no overclock, no HD Mode 7 | ✅ core geometry `Aspect: 1.333`, `snes9x_aspect=4:3`; no separate display capture | RetroArch's `--max-frames-ss` screenshot is the raw 8:7 core frame, not the 4:3 display viewport. |
| **Enhanced** | 8:7, integer 4×, no shader, `compatible`, Mode 7 2×, hi-res merge, reduce flicker, run-ahead 2 | ✅ title and first level, 60.01–60.05 fps, 1024×896 internal | The default look; the screenshots `docs/images/gameplay.jpg`. |
| CRT | Enhanced + `zfast-crt` | ✅ title screen; shader visibly applied | `crt-guest-advanced-fast` crashed under Vulkan on this Intel driver; `zfast-crt` is stable on GL/Vulkan. |
| Widescreen | bsnes-hd, `16:9`, Mode 7 only | ⚠️ the core reports a 768×432 geometry; no Mode 7 gameplay capture yet | Off by default; ordinary 2D levels stay 4:3. |
| Steam Deck | Enhanced values | — | No Deck hardware available; the look is the same picture profile. |
| `aspect` | pixel/4:3/widescreen | ✅ 8:7 verified; 4:3 configured | |
| `integer_scale` | on/off | ✅ 4× at 1080p verified | The fractional-scale guard is below. |
| `shader` | off/sharp/crt/smooth | ✅ off and CRT; sharp/smooth presets were checked by the shader test | |
| `overclock` | off/light/compatible/max | ✅ `compatible` in all Enhanced runs | |
| `mode7_hd` | off/2x/4x | ✅ `2x` configured in the Enhanced runs | No Mode 7 boss screenshot yet. |
| `hires_blend` | on/off | ✅ `merge` in the Enhanced runs | |
| `reduce_flicker` | on/off | ✅ `enabled` in the Enhanced runs | |
| `widescreen` | on/off | ⚠️ see the Widescreen look | |
| `vsync` | on/off | ✅ on in the fullscreen runs | |
| `run_ahead` | 0/1/2 | ✅ `2` in the Enhanced runs | |
| `fullscreen` | on/off | ✅ fullscreen; windowed configured | The Linux fractional-scale guard only matters in fullscreen. |
| `core` | auto/snes9x/bsnes_hd | ✅ Snes9x; bsnes-hd loaded for Widescreen | |
| `show_fps` | on/off | ✅ the FPS overlay was seen in the exploratory windowed runs; the option writes `fps_show` | The internal screenshot does not include the overlay. |
| `saves`/`runtime`/`retroarch`/`core_dir`/`shader_dir` | paths | ✅ the installed run used the default paths; the flags parse | The one-time import has no SAI save to import yet. |
| Controls: keyboard | Controls tab -> `controls.cfg` -> `input_player1_*` | ✅ headless UI test: Change + key K sets A; the controls file round-trips; the private config test writes `input_player1_a = \"x\"` | The player's real keyboard in the real game is the user's next check; the defaults are RetroArch's. |
| Controls: controller | Controls tab -> `pad_layout` -> `input_player1_*_btn` | ✅ the layout switch and the private config test (Nintendo A=1/B=0, Xbox A=0/B=1); Auto writes no pad keys | Auto (system autoconfig) is the default; the presets assume DualSense indices on Linux and XInput on Windows; a fully custom pad stays in RetroArch's Input settings. |

## Platforms

| Platform | Status | Notes |
|---|---|---|
| Linux, `linux/sai-play --play` | ✅ | Scale 1.25 -> 1 -> 1.25; the game ran; the helper exited 0. |
| Linux, menu/desktop entry | ⚠️ installed; not yet started from the menu | The `.desktop` files and the helper are installed in the owner's home. |
| Linux, settings window | ✅ | The native window and its headless tests; the screenshot `docs/images/launcher-play.jpg`. |
| Linux, fractional scale | ✅ | Hyprland `hyprctl eval`; gamescope rejected (47 fps). |
| Windows build/tests | ✅ | `x86_64-pc-windows-gnu` release; `cargo clippy`; the Windows test binaries pass under Wine 11. |
| Windows settings window | ⚠️ Wine ✅ / VM ❌ (old build) | The glow window opened under Wine; on the owner's VM it did not (no OpenGL), so the auto-detected ROM played without a menu. The wgpu/D3D12 exe now builds and its tests pass; the VM run is pending. |
| Windows package | ⚠️ under Wine | The launcher found `RetroArch-Win64`, the Snes9x core, D3D11 at 1920×1080 and the core ran; the Wine D3D11 screenshot was black. With a fresh prefix and no `--game`, the auto-detect found `roms/Super Adventure Island.sfc` above the exe folder and played (the log shows `auto-detected ROM`). |
| Native Windows 10/11 (VM) | ⚠️ | The game opened from `roms` with the auto-detect (the owner's report). The old glow settings window did not open (no OpenGL); the picture was Enhanced (sharp integer, no shader), so it looked like "no graphics improvement". The new wgpu/D3D12 exe is built; the menu and the CRT look need the VM re-test. |
| Steam Deck Game Mode | — | No hardware. |
| AppImage | — | Not built for this native design; the Linux install/helper is the current deliverable. |

## Scale and performance

- Linux display: 1920×1080 @ 60.02 Hz, Hyprland scale 1.25, Intel UHD 630 +
  GTX 1050 Ti. Enhanced at 60.01–60.05 fps; the helper did not use the
  NVIDIA offload by default (`SAI_GPU=` empty).
- The fullscreen guard: the launcher (direct start) or the helper (menu/Steam)
  sets the focused monitor to scale 1 through `hyprctl eval` and restores
  the original on drop/trap. Both paths were run; the scale was 1.25 again
  after each.
- The gamescope experiment used `-W 1920 -H 1080 -w 1920 -h 1080 -f`; it
  produced the correct shape at about 47 fps, so it is off.
- The global `~/.config/retroarch/retroarch.cfg` and
  `~/.config/retroarch/config/Snes9x/Snes9x.opt` were not changed: the
  launcher always passes its own `--config` and `rgui_config_directory`.

## Quitting and leftovers

| Quit path | Result |
|---|---|
| Quit from the game menu | The SNES game has no quit menu; the equivalent is RetroArch's Quick Menu → Quit (F1) or the window close. The window close was verified; the synthetic Escape test did not reach RetroArch in this harness (the physical key should). |
| Window close / Alt+F4 | ✅ focused the RetroArch window and closed it: helper exit 0, no `retroarch`/`SAILauncher`, scale back at 1.25. |
| `sai-play stop` | ✅ the stand-in test (`linux/tests/play.sh`); it also cleaned a real leftover RetroArch after a killed helper. |
| SIGINT (Ctrl+C) to `sai-play` | — a background helper ignores SIGINT (the SDK note); not run as a foreground real game. |
| SIGTERM to `sai-play` | ✅ helper exit 143; the SDK trap killed `retroarch`/`SAILauncher`; scale back at 1.25. |
| Launcher window closed without Play | `--settings` returns 3; a direct window close returns 0; nothing starts. |
| Second start | The SDK single-instance guard refuses with exit 5 (the Linux/window/Wine tests). |
| Hostile path | The ROM path has spaces; the launcher and the RetroArch arguments handled it. |

After a normal quit, `pgrep -x retroarch` and `pgrep -x SAILauncher` are
empty and the monitor scale is back at 1.25. The game has no in-game
"quit" menu; Escape/Quick Menu Quit and the window close are the paths
above.

## The owner's Windows VM report (2026-11-07)

- The game opened; the ROM at `C:\Games\SAILauncher\roms\Super Adventure Island.sfc` was found by the auto-detect (1,048,576 bytes, exact name).
- There was no settings menu: the VM has no OpenGL for eframe's default glow backend, and the launcher played with the auto-detected settings.
- The picture was **Enhanced**, which is 8:7 integer 4× with **no shader**; the visible CRT/Smooth looks were only reachable from the menu/CLI.
- The new Windows exe selects **wgpu → D3D12** for the settings window; the Windows target tests pass under Wine. The VM menu and a CRT `--look crt` run are the next checks.

## Not yet checked

- The native Windows VM run of the new wgpu/D3D12 settings window, a CRT/Smooth look and the Controls tab.
- The real-game keyboard/controller remap: the UI and the private config are tested; press the keys in the game to confirm.
- Steam Deck Game Mode, suspend/resume, the Deck's controller.
- The original USA ROM in the edition table.
- A full playthrough: the save file, save states, reset, and a second session.
- A Mode 7 boss/scene screenshot for the Widescreen look.
- A real Windows install without Wine for the helper-less `--no-window`
  fallback when the VM has no OpenGL.
