# Research: Super Adventure Island

What was found before writing the launcher, and what the ROM and the
emulator showed. Observations, inferences and decisions are kept apart.

## The game

- **Super Adventure Island** (Hudson Soft, 1992) is a SNES platformer with
  ordinary 2D levels and some Mode 7 scenes. It uses no SuperFX/SA-1 chip.
- The owner's ROM is the **USA/NTSC PT-BR translation by Juliano_Dio**,
  1 MiB, SHA-256
  `6ee5917881c330b64f4e6284e1361031fe08c4a0ab583aa71b958b7d3a50e54f`.
  The title screen and the first level were played on this machine (see
  `RUNTIME_NOTES.md`).
- NTSC runs at about 60.1 Hz. **No 60 fps patch exists or is needed**; on a
  60 Hz screen 60 is the ceiling. The relevant work is keeping the frame
  rate stable (overclock) and the input latency low (run-ahead).

## Where we looked

- RetroArch 1.22.2 and the libretro core options for Snes9x and bsnes-hd.
- The libretro slang-shaders presets (`crt/zfast-crt`, `crt/crt-guest-advanced-fast`,
  `pixel-art-scaling/sharp-bilinear-simple`, `edge-smoothing/xbr/xbr-lv2`).
- The libretro database (`Nintendo - Super Nintendo Entertainment System.rdb`)
  for the USA title and serial.
- GitHub and romhacking searches for a Super Adventure Island widescreen or
  enhancement patch. The only related project found was
  `abyssonym/sai2_rando` (a Super Adventure Island **II** randomizer), not
  relevant to this game.
- No PCGamingWiki entry applies: this is a console ROM, not a PC port. The
  usual PC fixes (dgVoodoo2, DXVK, Xidi, widescreen patches) do not apply.
- RetroArch/Hyprland behavior with fractional monitor scaling; gamescope was
  measured as an alternative and rejected (see below).

## Community components

| Project | What it gives us | Licence | Maintained | Decision |
|---|---|---|---|---|
| RetroArch | Frontend, config, run-ahead, shaders, saves | GPLv3 | yes | use the system package on Linux; bundle the 1.22.2 Windows build |
| Snes9x (libretro) | Accurate SNES core with overclock, HD Mode 7, hi-res blending, sprite-flicker reduction | Snes9x non-commercial licence | yes | default core |
| bsnes-hd beta (libretro) | HD Mode 7 and experimental Mode 7 widescreen | GPLv3 | yes | optional Widescreen look |
| libretro slang shaders | CRT/Sharp/Smooth presets | per shader | yes | optional shader looks |
| DualSense autoconfig from RetroArch | Pad mapping | RetroArch's licence | yes | reuse the player's profile on Linux; bundle a copy for Windows |

No game patch, crack or DRM workaround is involved. The ROM is read in
place; the launcher writes only its own RetroArch configuration.

## Measurements

- **Enhanced** on the Linux laptop (Intel UHD 630): the first level at
  60.01–60.05 fps; 256×224 at 8:7, integer 4× = 1024×896.
- **CRT:** `zfast-crt` works on GL and Vulkan; `crt-guest-advanced-fast`
  works on GL but **segfaulted under Vulkan** on this Intel driver. The CRT
  look therefore uses `zfast-crt` (the safer, lighter preset).
- **Widescreen:** bsnes-hd reports a 768×432 geometry with its default
  `16:9`; its own option is "Only for Mode 7 scenes", so ordinary 2D levels
  stay 4:3.
- **Fractional scaling:** Hyprland at scale 1.25 gives fullscreen RetroArch
  a 1536×864 logical surface and resamples the 1024×896 picture.
  - gamescope with a 1920×1080 internal surface fixed the shape but dropped
    the frame rate to about 47 fps on the Intel GPU.
  - Setting the focused monitor to scale 1.0 through `hyprctl eval` while
    the game runs, and restoring it afterwards, keeps 60 fps and integer
    scaling. This is what the launcher and the helper do.
- **Windows:** the Windows build and the Windows-target tests pass under
  Wine 11; the package ran the core and D3D11 device (the Wine D3D11
  screenshot was black; native Windows is the remaining visual test).

## Decision summary

- Use **RetroArch** rather than a PC-game Wine/Proton pipeline: the SDK's
  native-launcher path (as in the Screamer DOSBox launcher) fits.
- **Enhanced** is the default: 8:7, integer 4×, no shader, Snes9x
  overclock `Compatible`, HD Mode 7 2×, hi-res blending, reduce flicker,
  run-ahead 2, VSync.
- **CRT** uses `zfast-crt`; **Sharp** is `sharp-bilinear-simple`; **Smooth**
  is `xbr-lv2`.
- **Widescreen** uses bsnes-hd and stays off by default because it only
  affects Mode 7 scenes.
- The Linux launcher uses the system RetroArch; the Windows package bundles
  RetroArch and the two cores, with the pins in `packaging/windows-pins.txt`.

## Owner decisions

| Date | Decision | Why |
|---|---|---|
| 2026-11-07 | Target the owner's PT-BR translation ROM. | It is the copy the owner has and it was verified end to end. |
| 2026-11-07 | Use screenshots of the owner's copy in the README. | The README must show the verified look; no ROM is shipped. |
