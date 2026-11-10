# Changelog

All notable changes to the Super Adventure Island launcher. Versions are git tags
(`vX.Y.Z`) matching the `sai-launcher` crate version.

## [Unreleased]

## [0.1.0] - 2026-11-07

### Added

- A native Linux/Windows launcher on rea-launcher-sdk v0.4.3: the ROM
  edition table (the owner's PT-BR translation, SHA-256), the option table,
  the tropical settings window (Play/Picture/About), `--check`,
  `--settings`, `--no-window` and the single-instance guard.
- `retroarch.rs`: finds RetroArch and the Snes9x or bsnes-hd core, writes
  a private `retroarch.cfg` and the per-core options, and starts the game.
  The player's RetroArch configuration and ROM folder are never changed.
- The looks Original, Enhanced (the verified default), CRT, Widescreen and
  Steam Deck; the Enhanced look runs the first level at 60 fps (8:7,
  integer 4x, overclock Compatible, HD Mode 7 2x, hi-res blending,
  reduce flicker, run-ahead 2).
- The Linux helper `linux/sai-play` (the SDK `rea-play.sh`), including a
  Hyprland fractional-scale guard and the `--check` forwarding; the menu
  entries and the Steam recipe.
- The Windows package: the `x86_64-pc-windows-gnu` release, RetroArch
  1.22.2, the Snes9x/bsnes-hd cores and the slang shaders, with the pins in
  `packaging/windows-pins.txt` and a source offer.
- The **Controls** tab and `controls.cfg`: keyboard remapping for the
  twelve SNES buttons (key capture, reset) and the controller layout
  (Auto/system autoconfig, Nintendo SNES A/B, Xbox literal A/B), written
  into the private RetroArch input keys.
- The README, Linux README, AGENTS.md, THIRD_PARTY.md, RESEARCH.md,
  RUNTIME_NOTES.md and the README screenshots.

### Verified

- Linux: the PT-BR ROM, the first level at 60.01–60.05 fps, the fullscreen
  scale guard, the helper exit/scale restore, the native window.
- Windows: the release build and clippy, the Windows test binaries under
  Wine 11, the settings window and the package layout under Wine. The
  native Windows VM visual run is pending.

### Fixed

- The Windows settings window now uses eframe's `wgpu` backend and selects
  D3D12, so it opens in VMs without OpenGL (the owner's VM). Linux keeps the
  verified glow backend.
- The launcher now finds the ROM in a `roms` subfolder, in a `roms` folder
  beside a `SAILauncher-Windows` folder, and under common
  `Super Adventure Island (USA).sfc`/`.smc` names, not only the exact file
  in the chosen folder. If the saved choice is missing or unplayable it
  auto-detects beside the exe and saves the found path.

### Known limitations

- The edition table has only the owner's PT-BR translation; the original
  USA ROM is added when the owner supplies it.
- Widescreen is bsnes-hd's Mode 7-only experimental widening.
- The Steam Deck was not tested on hardware; the AppImage was not built for
  this native design (the Linux helper/install is the current deliverable).
