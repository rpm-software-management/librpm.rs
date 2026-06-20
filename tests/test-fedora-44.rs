use librpm::{Package, Tag, config::set_db_path, db::installed_packages};

mod common;

#[test]
fn test_fedora_44_rpm_database() {
    common::configure();
    set_db_path(&common::get_assets_path().join("fedora-44")).unwrap();

    let mut packages: Vec<Package> = installed_packages().collect();
    packages.sort_by_key(|p| p.name().to_string());

    assert_eq!(packages.len(), 147);
    let sample_package = &packages[0];
    assert_eq!(sample_package.name(), "alternatives");
    assert_eq!(sample_package.epoch(), None);
    assert_eq!(sample_package.version(), "1.33");
    assert_eq!(sample_package.release(), "5.fc44");
    assert_eq!(sample_package.arch(), Some("x86_64"));
    assert_eq!(sample_package.license(), "GPL-2.0-only");
    assert_eq!(
        sample_package.summary(),
        "A tool to maintain symbolic links determining default commands"
    );
    assert_eq!(
        sample_package.description(),
        "alternatives creates, removes, maintains and displays information about the\nsymbolic links comprising the alternatives system. It is possible for several\nprograms fulfilling the same or similar functions to be installed on a single\nsystem at the same time."
    );
}

#[test]
fn test_fedora_44_array_tag_data() {
    common::configure();
    set_db_path(&common::get_assets_path().join("fedora-44")).unwrap();

    let mut packages: Vec<Package> = installed_packages().collect();
    packages.sort_by_key(|p| p.name().to_string());

    let pkg = &packages[0]; // "alternatives"
    assert_eq!(pkg.name(), "alternatives");

    // BASENAMES is a STRING_ARRAY tag — the current code handles these correctly
    // because string_array() iterates with rpmtdNextString
    let basenames = pkg.get(Tag::BASENAMES).expect("BASENAMES tag missing");
    let basenames = basenames
        .as_str_array()
        .expect("BASENAMES should be a string array");
    assert_eq!(basenames.len(), 12, "alternatives package has 12 files");

    // FILESIZES is an INT32 tag with count > 1 (one entry per file).
    // BUG: the current TagData::int32() implementation reads only the first
    // element as a scalar Int32, discarding the remaining entries.
    // This is the bug fixed by the TagData array rework — when that lands,
    // this assertion should change to verify all 12 entries are returned.
    let filesizes = pkg.get(Tag::FILESIZES).expect("FILESIZES tag missing");
    assert!(
        filesizes.to_int32().is_some(),
        "FILESIZES currently returns a scalar Int32 (known bug: should be an array of 12 values)"
    );
}
