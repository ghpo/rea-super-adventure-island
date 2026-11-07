//! Which ROM builds this launcher knows, by SHA-256
//! (`rea_launcher::identify`).
//!
//! The owner has the USA/NTSC translation by Juliano_Dio. Any other ROM is
//! refused with its digest and the link to the "unknown edition" issue
//! template, so a different translation or a bad dump is never launched as
//! if it were the verified one (docs/STANDARDS.md, "Every edition").
//!
//! The folder search is forgiving: the exact `Super Adventure Island.sfc`,
//! a `roms` subfolder (the owner's Windows layout), a `roms` folder beside
//! a `SAILauncher-Windows` folder, and common names such as
//! `Super Adventure Island (USA).sfc`/`.smc`.

use std::fs;
use std::path::{Path, PathBuf};

use rea_launcher::identify::{self, Known};
use rea_launcher::paths;

/// The ROM's file name, as shipped in the owner's collection.
pub const ROM_NAME: &str = "Super Adventure Island.sfc";

/// Where players report a ROM the launcher does not know.
pub const UNKNOWN_EDITION_ISSUE: &str =
    "https://github.com/ghpo/rea-super-adventure-island/issues/new?template=unknown-edition.yml";

/// What the launcher knows about one build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edition {
    /// For the log and the window.
    pub name: &'static str,
}

/// Every known build of [`ROM_NAME`].
///
/// `6ee59178…` is the PT-BR translation by Juliano_Dio, verified in the
/// real game on this machine (title screen and first level, 60 fps).
///
/// Add a row for the original USA ROM only after the owner supplies the
/// file and its SHA-256 is recorded here; never guess a hash.
pub static EDITIONS: &[Known<Edition>] = &[Known::new(
    "6ee5917881c330b64f4e6284e1361031fe08c4a0ab583aa71b958b7d3a50e54f",
    Edition {
        name: "SNES USA (PT-BR translation by Juliano_Dio)",
    },
)];

/// The build of these ROM bytes, or their digest when it is unknown.
pub fn identify(bytes: &[u8]) -> Result<&'static Edition, [u8; 32]> {
    identify::identify(bytes, EDITIONS)
}

/// The message for a ROM that is not in [`EDITIONS`]: what happened, the
/// SHA-256 and where to report it.
pub fn unknown_message(digest: &[u8; 32]) -> String {
    format!(
        "This {ROM_NAME} is not a ROM this launcher knows, so it was left alone.\n\n\
         SHA-256: {}\n\n\
         Please report it, with that SHA-256 and where your copy came from, so it can \
         be supported:\n{UNKNOWN_EDITION_ISSUE}",
        identify::hex(digest)
    )
}

/// The ROM and its known build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// The ROM, spelled as it is on disk.
    pub rom: PathBuf,
    pub edition: &'static Edition,
}

/// Finds the ROM at `path` (a folder, a `roms` subfolder, a common file
/// name, or the ROM file itself) and identifies it. The error is a message
/// for the player.
pub fn find(path: &Path) -> Result<Found, String> {
    if path.is_file() {
        return identify_rom(path.to_path_buf());
    }
    if !path.is_dir() {
        return Err(format!("There is no {} at {}.", ROM_NAME, path.display()));
    }

    let dirs = candidate_dirs(path);
    let mut first_unknown: Option<[u8; 32]> = None;

    // The exact name in the folder or its roms/ folders first, so a big
    // collection is not scanned when the expected name is there.
    for dir in &dirs {
        if let Some(rom) = paths::find_ignoring_case(dir, ROM_NAME).filter(|rom| rom.is_file()) {
            match classify(rom) {
                Ok(Ok(found)) => return Ok(found),
                Ok(Err(digest)) => {
                    first_unknown.get_or_insert(digest);
                }
                Err(error) => return Err(error),
            }
        }
    }

    // Then common names such as `Super Adventure Island (USA).sfc`.
    for dir in &dirs {
        for rom in variant_roms(dir) {
            match classify(rom) {
                Ok(Ok(found)) => return Ok(found),
                Ok(Err(digest)) => {
                    first_unknown.get_or_insert(digest);
                }
                Err(error) => return Err(error),
            }
        }
    }

    match first_unknown {
        Some(digest) => Err(unknown_message(&digest)),
        None => Err(format!(
            "There is no {ROM_NAME} in {} or its roms folder.",
            path.display()
        )),
    }
}

/// Looks beside the launcher: the exe's folder and its parent, each with
/// the `roms` fallback. Used when the player has not chosen a ROM yet (or
/// the chosen one no longer works).
pub fn find_near(exe_dir: &Path) -> Option<Found> {
    let mut dirs = vec![exe_dir.to_path_buf()];
    if let Some(parent) = exe_dir.parent() {
        dirs.push(parent.to_path_buf());
    }
    for dir in dirs {
        if let Ok(found) = find(&dir) {
            return Some(found);
        }
    }
    None
}

fn read_digest(rom: &Path) -> Result<[u8; 32], String> {
    identify::sha256_file(rom).map_err(|error| format!("Cannot read {}: {error}", rom.display()))
}

/// `Ok(Ok(found))` for a known ROM, `Ok(Err(digest))` for an unknown file,
/// `Err` when the file cannot be read.
fn classify(rom: PathBuf) -> Result<Result<Found, [u8; 32]>, String> {
    let digest = read_digest(&rom)?;
    Ok(match identify::identify_digest(digest, EDITIONS) {
        Ok(edition) => Ok(Found { rom, edition }),
        Err(digest) => Err(digest),
    })
}

fn identify_rom(rom: PathBuf) -> Result<Found, String> {
    match classify(rom)? {
        Ok(found) => Ok(found),
        Err(digest) => Err(unknown_message(&digest)),
    }
}

/// The folder, its `roms` subfolder and the `roms` folder beside it.
fn candidate_dirs(path: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![path.to_path_buf()];
    if let Some(roms) = child_dir(path, "roms") {
        dirs.push(roms);
    }
    if let Some(parent) = path.parent()
        && let Some(roms) = child_dir(parent, "roms")
    {
        dirs.push(roms);
    }
    dirs.dedup();
    dirs
}

fn child_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    let direct = dir.join(name);
    if direct.is_dir() {
        return Some(direct);
    }
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(name)
            && entry.path().is_dir()
        {
            return Some(entry.path());
        }
    }
    None
}

/// Files with a SNES extension whose name starts with the game's title, so
/// `Super Adventure Island (USA).sfc` and `.smc` copies are found too. The
/// exact name is tried before this; duplicates are harmless (same digest).
fn variant_roms(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if !extension.eq_ignore_ascii_case("sfc") && !extension.eq_ignore_ascii_case("smc") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if stem.to_lowercase().contains("super adventure island") {
            out.push(path);
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_known_build_has_its_own_digest() {
        for (i, known) in EDITIONS.iter().enumerate() {
            assert!(!known.value.name.is_empty());
            assert!(
                EDITIONS[i + 1..].iter().all(|k| k.sha256 != known.sha256),
                "{} is listed twice",
                known.value.name
            );
            assert_eq!(
                identify::identify_digest(known.sha256, EDITIONS),
                Ok(&known.value)
            );
        }
    }

    #[test]
    fn the_recorded_digest_is_64_hex_digits() {
        assert_eq!(identify::hex(&EDITIONS[0].sha256).len(), 64);
    }

    #[test]
    fn an_unknown_build_is_refused_with_its_digest_and_the_issue_link() {
        let digest = identify::sha256(b"not the game");
        assert_eq!(identify(b"not the game"), Err(digest));
        let message = unknown_message(&digest);
        assert!(message.contains(&identify::hex(&digest)), "{message}");
        assert!(message.contains(UNKNOWN_EDITION_ISSUE), "{message}");
        assert!(UNKNOWN_EDITION_ISSUE.starts_with("https://github.com/"));
    }
}
