//! librpm.rs integration tests

use librpm::{Package, Tag};

mod common;

// TODO: This will deadlock: https://github.com/rpm-software-management/librpm.rs/issues/15
// #[test]
// fn db_find_test_multiple() {
//     let db = common::configure();
//
//     let mut matches = db.find(librpm::Index::Name, "glibc-common");
//     if let Some(package) = matches.next() {
//         assert_eq!(package.name(), "glibc-common");
//         assert!(matches.next().is_none(), "expected one result, got more!");
//     } else {
//         panic!("glibc-common package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?");
//     }
//
//     let mut matches = db.find(librpm::Index::Name, "glibc");
//     if let Some(package) = matches.next() {
//         assert_eq!(package.name(), "glibc");
//         assert!(matches.next().is_none(), "expected one result, got more!");
//     } else {
//         panic!("glibc package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?");
//     }
// }

#[test]
fn test_scalar_int32_tag() {
    let db = common::init(&common::CENTOS_STREAM_9);

    let results: Vec<Package> = db.find(librpm::Index::Name, "alternatives").collect();
    let package = &results[0];

    let buildtime = package.get(Tag::BUILDTIME).expect("BUILDTIME should exist");
    let value = buildtime.as_int32().expect("BUILDTIME should be Int32");
    assert!(value > 0, "BUILDTIME should be a positive timestamp");

    let values = buildtime
        .as_int32_array()
        .expect("BUILDTIME should be Int32");
    assert_eq!(values.len(), 1, "BUILDTIME is a scalar tag (count=1)");
}

#[test]
fn test_array_tag_data() {
    let db = common::init(&common::CENTOS_STREAM_9);

    let results: Vec<Package> = db.find(librpm::Index::Name, "alternatives").collect();
    let pkg = &results[0];

    let basenames = pkg.get(Tag::BASENAMES).expect("BASENAMES tag missing");
    let basenames = basenames
        .as_str_array()
        .expect("BASENAMES should be a string array");
    assert_eq!(basenames.len(), 12, "alternatives package has 12 files");

    let filesizes = pkg.get(Tag::FILESIZES).expect("FILESIZES tag missing");
    let sizes = filesizes
        .as_int32_array()
        .expect("FILESIZES should be Int32 array");
    assert_eq!(
        sizes.len(),
        basenames.len(),
        "FILESIZES and BASENAMES should have the same count",
    );

    let filemodes = pkg.get(Tag::FILEMODES).expect("FILEMODES tag missing");
    let modes = filemodes
        .as_int16_array()
        .expect("FILEMODES should be Int16 array");
    assert_eq!(
        modes.len(),
        basenames.len(),
        "FILEMODES and BASENAMES should have the same count",
    );
}

#[test]
fn test_tag_type_mismatch_returns_none() {
    let db = common::init(&common::CENTOS_STREAM_9);

    let results: Vec<Package> = db.find(librpm::Index::Name, "alternatives").collect();
    let pkg = &results[0];

    let name = pkg.get(Tag::NAME).expect("NAME should exist");
    assert!(name.as_int32().is_none());
    assert!(name.as_int16().is_none());
    assert!(name.as_int64().is_none());
    assert!(name.as_bytes().is_none());
    assert!(name.as_str_array().is_none());

    let buildtime = pkg.get(Tag::BUILDTIME).expect("BUILDTIME should exist");
    assert!(buildtime.as_str().is_none());
    assert!(buildtime.as_bytes().is_none());
    assert!(buildtime.as_str_array().is_none());
}
