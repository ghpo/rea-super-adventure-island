# Super Adventure Island launcher — instructions for coding agents

This repository makes the player's own copy of **Super Adventure Island** run on modern
Windows and Linux (Proton/Wine, AppImage) with a settings window, sensible
defaults and no manual steps. It is built on
[rea-launcher-sdk](https://github.com/akitaonrails/rea-launcher-sdk) v0.4.3.

**Read `sdk/docs/STANDARDS.md` before any work; the playbooks are in
`sdk/docs/playbooks/`** (vendored from the SDK at the tag in `sdk/VERSION`).

## Always

- The game's files are read-only: never committed, copied into the repo or published.
- Everything outside this repository is the user's and read-only (game installs,
  other repos, Wine prefixes, saves, home configs, containers, the SDK checkout)
  unless the user asked for or approved that specific change: back up first,
  never delete by a broad glob, ask when unsure.
- Headless by default; open a visible window only when a check needs a person, and say so first.
- Never print, log or commit credentials.
- Push, tag, release or post only with the user's go-ahead.
- Research community fixes first (`docs/RESEARCH.md`); patch only what nobody solved.
- Never edit `sdk/`; SDK gaps and bugs go to the SDK repository as issues or PRs.
- Clean up after a release or a long session: `sdk/scripts/clean`, then `--yes` (log scratch in `.work/.task-created`).

Running the real game in a test: `sdk/docs/STANDARDS.md`, "Running the real
game in a test" (under "Testing").

## Layout

| Path | What |
|---|---|
| `crates/sai-launcher/src/edition.rs` | Known builds (SHA-256), the unknown-edition refusal |
| `crates/sai-launcher/src/config.rs` | The option table: `launcher.cfg`, flags, `--help`, template, looks |
| `crates/sai-launcher/src/cli.rs` | Modes (`--settings`, `--no-window`, `--check`, `--help`), exit codes |
| `crates/sai-launcher/src/form.rs`, `ui.rs`, `palette.rs` | The settings window and its headless tests |
| `crates/sai-launcher/src/runtime.rs` | The runtime copy's `Policy` |
| `crates/sai-launcher/src/main.rs` | Windows wiring: console, window, `launcher.log`, the game's process |
| `linux/sai-play`, `linux/tests/` | The Linux helper (sources `sdk/rea-play.sh`) and its tests |
| `packaging/` | Pins, allowlist, package script, AppImage desktop entry and icon |
| `sdk/` | Vendored by the SDK's `scripts/sdk-sync`; never edit by hand |
| `docs/RESEARCH.md`, `docs/RUNTIME_NOTES.md`, `docs/CONTRIBUTIONS.md` | Findings, verification, drafts to share |

## Checks (all must pass before a commit)

```sh
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --target i686-pc-windows-gnu --workspace --all-targets -- -D warnings
cargo deny check
for t in linux/tests/*.sh packaging/tests/*.sh; do sh "$t"; done
../rea-launcher-sdk/scripts/sdk-sync --check .
sdk/scripts/launcher-check                     # family rules; every error blocks the commit
cargo build --release --locked --target i686-pc-windows-gnu
sdk/scripts/wine-test --locked --workspace     # Windows-target tests under Wine, headless
```

MinGW, lld and Wine may live in the `gaming` distrobox: run the last two as
`distrobox enter gaming -- sh -c 'cd /absolute/path && …'`.

Conventional Commits; every behaviour change gets a CHANGELOG "Unreleased"
entry, and every change that still needs a real-game check a line in
`docs/RUNTIME_NOTES.md`.

## Super Adventure Island: what is specific to this game

Fill this in as the playbook's phases find it, and keep it current:

- **Editions:** (each known build, its SHA-256 and where it comes from)
- **Renderer, input, audio:** (D3D version, DirectInput/XInput, DirectSound/EAX)
- **Files the game writes:** (config, saves, registry keys; decides `runtime::GamePolicy`)
- **Community components used:** (see `docs/RESEARCH.md`)
- **Patches:** (only what nobody else solved; byte sites per edition)
- **Owner decisions:** (licence-less assets, official art, DRM; dated, also
  in `docs/RESEARCH.md`. An exception to a `launcher-check` rule is a line
  `rule: reason` in `.rea-check-waivers`, only with the owner's decision.)
- **Known problems and their status:**
