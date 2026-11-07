//! The Windows executable's version information, icon and manifest
//! (rea-launcher-build): a named, versioned program with the launcher's
//! icon, instead of an anonymous one that antivirus heuristics distrust.
fn main() {
    rea_launcher_build::Resources::new(
        "SAILauncher",
        "Launcher for Super Adventure Island on modern systems",
    )
    .company("Gustavo Oliveira")
    .file_name("SAILauncher.exe")
    .icon("../../packaging/appimage/sai-launcher.png")
    .embed()
    .unwrap();
}
