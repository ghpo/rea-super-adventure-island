# Super Adventure Island launcher — instructions for coding agents

This repository makes the player's own **Super Adventure Island** SNES ROM
run on Linux and Windows through RetroArch, with a native settings window,
sensible defaults and no manual emulator setup. It is built on
[rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk) v0.4.3.

**Read `sdk/docs/STANDARDS.md` before any work; the playbooks are in
`sdk/docs/playbooks/`** (vendored at the tag in `sdk/VERSION`).

## Always

- The ROM is read-only: never committed, copied into the repo or published.
- Everything outside this repository is the user's and read-only (ROMs,
  RetroArch configuration, saves, home configs, containers, the SDK
  checkout) unless the user asked for or approved that specific change.
- Headless by default; open a visible window only when a check needs a
  person, and say so first.
- Never print, log or commit credentials.
- Push, tag, release or post only with the user's go-ahead.
- Never attach the ROM, a save or any game file to an issue.
- Never edit `sdk/`; SDK gaps and bugs go to the SDK repository.

## Layout

| Path | What |
|---|---|
| `crates/sai-launcher/src/edition.rs` | Known ROM builds (SHA-256), the unknown-build refusal |
| `crates/sai-launcher/src/config.rs` | The option table: `launcher.cfg`, flags, `--help`, template, looks |
| `crates/sai-launcher/src/cli.rs` | Modes (`--settings`, `--no-window`, `--check`, `--help`), exit codes |
| `crates/sai-launcher/src/controls.rs` | Keyboard bindings and controller layout (`controls.cfg`) written into the private RetroArch input keys |
| `crates/sai-launcher/src/form.rs`, `ui.rs`, `palette.rs` | The native settings window (glow on Linux, wgpu/D3D12 on Windows) and its headless tests |
| `crates/sai-launcher/src/retroarch.rs` | Finds RetroArch/the cores, writes the private config and core options, starts the game, the Hyprland scale guard |
| `crates/sai-launcher/src/runtime.rs` | The launcher's config/data/save folders and the one-time RetroArch-save import |
| `crates/sai-launcher/src/main.rs` | Native Linux/Windows wiring: commands, window, log, process |
| `linux/sai-play`, `linux/tests/` | The Linux helper (sources `sdk/rea-play.sh`) and its tests |
| `packaging/` | The Windows package pins and inputs; the AppImage desktop entry and icon |
| `sdk/` | Vendored by the SDK's `scripts/sdk-sync`; never edit by hand |
| `docs/RESEARCH.md`, `docs/RUNTIME_NOTES.md`, `docs/CONTRIBUTIONS.md` | Findings, verification, drafts to share |

## Checks

Host (Linux):

```sh
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
for t in linux/tests/*.sh packaging/tests/*.sh; do sh "$t"; done
../rea-launcher-sdk/scripts/sdk-sync --check .
sdk/scripts/launcher-check
```

Windows target (the host has no rustup target; the owner used a Docker
image with `rust:1.99-bookworm`, `gcc-mingw-w64-x86-64`,
`binutils-mingw-w64-x86-64`, `lld` and the target):

```sh
cargo build --release --locked --target x86_64-pc-windows-gnu
cargo clippy --locked --target x86_64-pc-windows-gnu --workspace --all-targets -- -D warnings
cargo test --locked --target x86_64-pc-windows-gnu --no-run
```

Run the Windows `*.exe` test binaries with **Wine >= 9** (the host has Wine
11; the SDK's `wine-test` container with Wine 8 fails on
`bcryptprimitives.dll`). The `wine-test` script works with a Wine >= 9
container. `cargo deny check`/`cargo audit` need the tooling installed.

Conventional Commits; every behaviour change gets a CHANGELOG "Unreleased"
entry, and every change that still needs a real-game check a line in
`docs/RUNTIME_NOTES.md`.

## What is specific to this game

- **Edition:** the owner's USA/NTSC **PT-BR translation by Juliano_Dio**,
  SHA-256 `6ee5917881c330b64f4e6284e1361031fe08c4a0ab583aa71b958b7d3a50e54f`.
  The original USA ROM is not in the table yet; add its SHA-256 only after
  the owner supplies the file.
- **Emulator:** RetroArch 1.22.2 with Snes9x 1.61 (Linux core) or the
  Windows stable core; bsnes-hd beta for the Widescreen look. The game is
  NTSC/USA, ~60.1 Hz; no 60 fps patch exists or is needed.
- **Input/audio:** RetroArch's udev driver on Linux (the player's DualSense
  autoconfig is reused) and XInput on Windows; PulseAudio on Linux.
- **Files the game writes:** none in the ROM folder; RetroArch saves and
  states go to the launcher's `saves`/`states` folders.
- **Community components:** RetroArch, Snes9x, bsnes-hd, libretro slang
  shaders, Barlow fonts.
- **Patches:** none. The launcher writes a RetroArch configuration; it
  never patches the ROM or the game files.
- **Owner decisions:** README screenshots are of the owner's own copy
  (2026-11-07); the ROM is the PT-BR translation; no official box art is
  bundled.
- **Known problems and status:** widescreen only widens Mode 7 scenes;
  Hyprland fractional scale is handled by temporarily setting scale 1;
  the Windows VM visual run is pending; the Windows settings window needs
  OpenGL (the `--no-window` fallback exists).
