# Third-party components and assets

Nothing from Super Adventure Island itself is part of this repository or
its packages. The owner's ROM is read in place.

| Component | Version | What it does here | Licence | Source | Bundled? |
|---|---|---|---|---|---|
| [rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk) | v0.4.3 | Shared launcher code, Linux helper, settings window, packaging | MIT OR Apache-2.0 | https://github.com/akitaonrails/rea-launcher-sdk | compiled in, `sdk/` |
| Barlow fonts | from the SDK | The settings window's type | SIL OFL 1.1 (`licenses/Barlow-OFL.txt`) | https://github.com/jpt/barlow | compiled in |
| [RetroArch](https://www.retroarch.com) | 1.22.2 | Runs the ROM with the selected core | GPLv3 | https://github.com/libretro/RetroArch/releases/tag/v1.22.2 | Windows package; system package on Linux |
| [Snes9x libretro core](https://github.com/libretro/snes9x) | the Windows 1.22.2 build / Arch `libretro-snes9x` | The default SNES core | Snes9x non-commercial licence (see the core `snes9x_libretro.info`) | https://github.com/libretro/snes9x | Windows package; system package on Linux |
| [bsnes-hd beta](https://github.com/DerKoun/bsnes-hd) | the Windows 1.22.2 build / Arch `libretro-bsnes-hd` | The Widescreen look | GPLv3 | https://github.com/DerKoun/bsnes-hd | Windows package; system package on Linux |
| [libretro slang shaders](https://github.com/libretro/slang-shaders) | the RetroArch 1.22.2 set / Arch `libretro-shaders-slang` | CRT, Sharp and Smooth looks | each shader's own licence (see the shader files) | https://github.com/libretro/slang-shaders | Windows package; system package on Linux |
| RetroArch DualSense autoconfig | the player's RetroArch profile | Linux pad mapping; bundled for Windows | RetroArch's licence | https://github.com/libretro/RetroArch | a profile copied into the launcher's runtime |

The Windows package inputs and their SHA-256 are pinned in
`packaging/windows-pins.txt`. The RetroArch and core source is offered in
`SOURCE-OFFER.txt` inside the Windows package.

## Images

| File | Author | Licence | Source |
|---|---|---|---|
| `docs/images/launcher-play.jpg`, `launcher-picture.jpg`, `launcher-controls.jpg`, `launcher-about.jpg` | the launcher project | MIT OR Apache-2.0 (the launcher's own UI) | this repository |
| `docs/images/gameplay.jpg`, `docs/images/gameplay-crt.jpg` | the owner's copy | Super Adventure Island © Hudson Soft / Nintendo, used with the owner's decision for this README; not bundled in the packages | the owner's ROM |
| `packaging/appimage/sai-launcher.png` | the launcher project | MIT OR Apache-2.0 | this repository |

## Owner decisions

| Date | Decision | Why |
|---|---|---|
| 2026-11-07 | Use screenshots of the owner's own copy in the README. | The README needs to show the verified Enhanced look; the screenshots are not shipped in the game/Windows package and no ROM is included. |
| 2026-11-07 | The edition table contains the owner's PT-BR translation by Juliano_Dio. | That is the ROM the owner has; the original USA ROM is added only when the owner supplies it. |
