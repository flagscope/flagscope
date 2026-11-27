//! Integration test of saved lists and contributions through the public API,
//! without network access: the list is saved first, so nothing is downloaded.

use std::fs;

use flagscope_scanner::{Error, ListSource, NoObserver, ScanOptions};

#[test]
fn a_saved_scanned_list_is_loaded_and_prepared_for_contribution() {
    let root = std::env::temp_dir().join(format!("flagscope-saved-list-{}", std::process::id()));
    let mut options = ScanOptions::for_this_machine().expect("Windows environment");
    options.lists_directory = root.join("lists");
    let build = flagscope_scanner::installed_build().expect("installed build");

    // A list as `flagscope scan` saves it (AD-16 header, AD-22 source line).
    fs::create_dir_all(&options.lists_directory).unwrap();
    let saved = format!(
        "# flagscope feature list\n# Build: {build}\n# Architecture: x64\n\
         # Generator: flagscope 0.1.0, light scan\n# Scanned: 2026-11-27\n\
         # Source: scanned on this machine\nAlpha: 1\nBeta: 2\n"
    );
    fs::write(options.lists_directory.join(format!("{build}.txt")), &saved).unwrap();

    let list = flagscope_scanner::feature_list(&options, false, &mut NoObserver).unwrap();
    assert!(matches!(list.source, ListSource::Saved));
    assert_eq!(list.features.len(), 2);

    let repository = root.join("feature-lists");
    let written = flagscope_scanner::prepare_contribution(&options, &repository).unwrap();
    let expected = repository
        .join("x64")
        .join(build.major.to_string())
        .join(format!("{build}.txt"));
    assert_eq!(written, expected);
    let contribution = fs::read_to_string(&written).unwrap();
    assert!(!contribution.contains("# Source:"));
    assert!(contribution.ends_with("Alpha: 1\nBeta: 2\n"));
    assert!(matches!(
        flagscope_scanner::prepare_contribution(&options, &repository),
        Err(Error::ContributionExists { .. })
    ));

    fs::remove_dir_all(&root).unwrap();
}
