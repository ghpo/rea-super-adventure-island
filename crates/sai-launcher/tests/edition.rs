//! Finding and identifying the ROM in a folder.

use std::path::PathBuf;

use rea_launcher::identify;
use sai_launcher::edition::{self, EDITIONS};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("sai-edition-{name}-{}", std::process::id()))
        .join("Games d'Été");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_missing_folder_or_exe_is_named() {
    let dir = scratch("missing");
    let none = edition::find(&dir.join("nowhere")).unwrap_err();
    assert!(none.contains("There is no"), "{none}");
    let no_exe = edition::find(&dir).unwrap_err();
    assert!(no_exe.contains(edition::ROM_NAME), "{no_exe}");
}

#[test]
fn an_unknown_build_in_any_letter_case_is_refused_with_its_digest() {
    let dir = scratch("unknown");
    let bytes = b"MZ an unknown build";
    std::fs::write(dir.join(edition::ROM_NAME.to_uppercase()), bytes).unwrap();
    let refused = edition::find(&dir).unwrap_err();
    assert!(
        refused.contains(&identify::hex(&identify::sha256(bytes))),
        "{refused}"
    );
    assert!(
        refused.contains(edition::UNKNOWN_EDITION_ISSUE),
        "{refused}"
    );
}

#[test]
fn the_table_holds_valid_distinct_digests() {
    for known in EDITIONS {
        assert_ne!(known.sha256, [0; 32], "{}", known.value.name);
        assert_eq!(
            EDITIONS.iter().filter(|k| k.sha256 == known.sha256).count(),
            1,
            "{} is listed twice",
            known.value.name
        );
    }
}

#[test]
fn a_rom_in_a_roms_subfolder_is_discovered() {
    let dir = scratch("roms-sub");
    std::fs::create_dir_all(dir.join("roms")).unwrap();
    let bytes = b"fake rom in roms";
    std::fs::write(dir.join("roms").join(edition::ROM_NAME), bytes).unwrap();
    let refused = edition::find(&dir).unwrap_err();
    assert!(
        refused.contains(&identify::hex(&identify::sha256(bytes))),
        "{refused}"
    );
    assert!(!refused.contains("There is no"), "{refused}");
}

#[test]
fn a_variant_usa_smc_name_is_discovered() {
    let dir = scratch("variant");
    let bytes = b"fake usa smc";
    std::fs::write(dir.join("Super Adventure Island (USA).smc"), bytes).unwrap();
    let refused = edition::find(&dir).unwrap_err();
    assert!(
        refused.contains(&identify::hex(&identify::sha256(bytes))),
        "{refused}"
    );
    assert!(!refused.contains("There is no"), "{refused}");
}

#[test]
fn a_roms_folder_beside_the_exe_folder_is_discovered() {
    let root = scratch("beside");
    let exe_dir = root.join("SAILauncher-Windows");
    std::fs::create_dir_all(&exe_dir).unwrap();
    std::fs::create_dir_all(root.join("roms")).unwrap();
    let bytes = b"fake rom beside the exe";
    std::fs::write(root.join("roms").join(edition::ROM_NAME), bytes).unwrap();
    let refused = edition::find(&exe_dir).unwrap_err();
    assert!(
        refused.contains(&identify::hex(&identify::sha256(bytes))),
        "{refused}"
    );
    assert!(!refused.contains("There is no"), "{refused}");
}
