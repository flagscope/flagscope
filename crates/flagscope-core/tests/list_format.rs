//! Integration test of the list format through the public API (AD-16).

use flagscope_core::{Build, Feature, FeatureId, ListHeader, read_list, write_list};

#[test]
fn a_list_with_header_is_read_back_through_the_public_api() {
    let header = ListHeader {
        build: Build {
            major: 26_300,
            revision: 10_075,
        },
        architecture: "x64".to_owned(),
        generator: "flagscope 0.1.0, light scan".to_owned(),
        scanned: "2026-11-27".to_owned(),
    };
    let features = vec![
        Feature {
            name: "Alpha".to_owned(),
            id: FeatureId(1),
        },
        Feature {
            name: "Beta".to_owned(),
            id: FeatureId(40_767_182),
        },
    ];

    let mut text = header.lines().join("\n") + "\n";
    let mut entries = Vec::new();
    write_list(&mut entries, &features).unwrap();
    text.push_str(&String::from_utf8(entries).unwrap());

    assert_eq!(ListHeader::parse(&text), Some(header));
    assert_eq!(read_list(text.as_bytes()).unwrap(), features);
}
