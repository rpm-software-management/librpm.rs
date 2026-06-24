//! Tests for Package::from_file(), TagData type coverage, error paths,
//! and Package trait implementations.

use std::collections::HashSet;
use std::path::Path;

use librpm::{Package, Tag};

mod common;

fn assets_path() -> std::path::PathBuf {
    common::get_assets_path().join("rpms")
}

fn load_basic() -> Package {
    common::configure();
    Package::from_file(&assets_path().join("rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm"))
        .unwrap()
}

// Package::from_file

#[test]
fn test_from_file_basic_metadata() {
    let pkg = load_basic();

    assert_eq!(pkg.name(), "rpm-basic");
    assert_eq!(pkg.epoch(), Some(1));
    assert_eq!(pkg.version(), "2.3.4");
    assert_eq!(pkg.release(), "5.el9");
    assert_eq!(pkg.arch(), Some("noarch"));
    assert_eq!(pkg.license(), "MPL-2.0");
    assert_eq!(
        pkg.summary(),
        "A package for exercising basic features of RPM"
    );
    assert_eq!(
        pkg.description(),
        "This package attempts to exercise basic features of RPM packages."
    );
    assert_eq!(pkg.nevra(), "rpm-basic-1:2.3.4-5.el9.noarch");
    assert_eq!(pkg.evr(), "1:2.3.4-5.el9");
}

#[test]
fn test_from_file_empty_package() {
    common::configure();
    let pkg = Package::from_file(&assets_path().join("rpm-empty-0-0.x86_64.rpm")).unwrap();

    assert_eq!(pkg.name(), "rpm-empty");
    assert_eq!(pkg.epoch(), None);
    assert_eq!(pkg.version(), "0");
    assert_eq!(pkg.release(), "0");
    assert_eq!(pkg.arch(), Some("x86_64"));
    assert_eq!(pkg.nevra(), "rpm-empty-0-0.x86_64");
    assert_eq!(pkg.evr(), "0-0");
}

#[test]
fn test_from_file_nonexistent() {
    common::configure();
    let result = Package::from_file(Path::new("/nonexistent/path/to/package.rpm"));
    assert!(result.is_err());
}

// TagData type coverage

#[test]
fn test_tag_int32_scalar_and_array() {
    let pkg = load_basic();

    let buildtime = pkg.get(Tag::BUILDTIME).expect("BUILDTIME should exist");
    assert_eq!(buildtime.as_int32(), Some(1681068559));
    let arr = buildtime.as_int32_array().expect("should be Int32");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], 1681068559);
}

#[test]
fn test_tag_int32_array() {
    let pkg = load_basic();

    let filesizes = pkg.get(Tag::FILESIZES).expect("FILESIZES should exist");
    let sizes = filesizes
        .as_int32_array()
        .expect("FILESIZES should be Int32");
    assert_eq!(sizes.len(), 11);
    assert_eq!(sizes[0], 31);
    assert_eq!(sizes[1], 120);
}

#[test]
fn test_tag_int16_array() {
    let pkg = load_basic();

    let filemodes = pkg.get(Tag::FILEMODES).expect("FILEMODES should exist");
    let modes = filemodes
        .as_int16_array()
        .expect("FILEMODES should be Int16");
    assert_eq!(modes.len(), 11);
}

#[test]
fn test_tag_string() {
    let pkg = load_basic();

    let name = pkg.get(Tag::NAME).expect("NAME should exist");
    assert_eq!(name.as_str(), Some("rpm-basic"));
    assert!(name.is_str());
    assert!(name.as_int32().is_none());
}

#[test]
fn test_tag_string_array() {
    let pkg = load_basic();

    let basenames = pkg.get(Tag::BASENAMES).expect("BASENAMES should exist");
    let names = basenames
        .as_str_array()
        .expect("BASENAMES should be StrArray");
    assert_eq!(names.len(), 11);
    assert!(names.contains(&"README"));
}

#[test]
fn test_tag_bin() {
    let pkg = load_basic();

    let sigmd5 = pkg.get(Tag::SIGMD5).expect("SIGMD5 should exist");
    let bytes = sigmd5.as_bytes().expect("SIGMD5 should be Bin");
    assert_eq!(bytes.len(), 16, "MD5 digest is 16 bytes");
    assert!(sigmd5.is_bytes());
    assert!(sigmd5.as_str().is_none());
}

#[test]
fn test_tag_missing_returns_none() {
    let pkg = load_basic();
    assert!(pkg.get(Tag::EPOCH).is_some());

    common::configure();
    let empty = Package::from_file(&assets_path().join("rpm-empty-0-0.x86_64.rpm")).unwrap();
    assert!(empty.get(Tag::EPOCH).is_none());
}

// Package trait implementations

#[test]
fn test_package_clone() {
    let pkg = load_basic();
    let cloned = pkg.clone();

    assert_eq!(pkg.name(), cloned.name());
    assert_eq!(pkg.epoch(), cloned.epoch());
    assert_eq!(pkg.version(), cloned.version());
    assert_eq!(pkg.release(), cloned.release());
    assert_eq!(pkg.arch(), cloned.arch());
}

#[test]
fn test_package_display() {
    let pkg = load_basic();
    assert_eq!(format!("{}", pkg), "rpm-basic-1:2.3.4-5.el9.noarch");
}

#[test]
fn test_package_debug() {
    let pkg = load_basic();
    let debug = format!("{:?}", pkg);
    assert!(debug.contains("rpm-basic"));
    assert!(debug.contains("2.3.4"));
}

#[test]
fn test_package_partial_eq() {
    let pkg1 = load_basic();
    let pkg2 = load_basic();
    assert_eq!(pkg1, pkg2);

    common::configure();
    let empty = Package::from_file(&assets_path().join("rpm-empty-0-0.x86_64.rpm")).unwrap();
    assert_ne!(pkg1, empty);
}

#[test]
fn test_package_hash() {
    let pkg1 = load_basic();
    let pkg2 = load_basic();

    let mut set = HashSet::new();
    set.insert(pkg1);
    assert!(set.contains(&pkg2));
}
