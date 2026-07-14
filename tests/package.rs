//! Tests for Package::from_file(), TagData type coverage, error paths,
//! and Package trait implementations.

use std::collections::HashSet;
use std::path::Path;

use librpm::{Package, Tag, error::ErrorKind};

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

// Package::files

#[test]
fn test_files_count_and_paths() {
    let pkg = load_basic();
    let files = pkg.files();

    assert_eq!(files.len(), 11);
    assert!(!files.is_empty());

    let paths: Vec<String> = files.iter().map(|f| f.path()).collect();
    assert_eq!(paths.len(), 11);
    assert!(paths.contains(&"/etc/rpm-basic/example_config.toml".to_string()));
    assert!(paths.contains(&"/usr/bin/rpm-basic".to_string()));
    assert!(paths.contains(&"/usr/share/doc/rpm-basic/README".to_string()));
    assert!(paths.contains(&"/var/log/rpm-basic/basic.log".to_string()));
}

#[test]
fn test_file_entry_metadata() {
    let pkg = load_basic();
    let files = pkg.files();

    let config = files
        .iter()
        .find(|f| f.path() == "/etc/rpm-basic/example_config.toml")
        .expect("config file should exist");
    assert_eq!(config.size(), 31);
    assert_eq!(config.user(), "root");
    assert_eq!(config.group(), "root");
    assert!(config.flags().is_config());
    assert!(!config.flags().is_doc());
    assert_eq!(config.basename(), "example_config.toml");
    assert_eq!(config.dirname(), "/etc/rpm-basic/");
}

#[test]
fn test_file_flags() {
    let pkg = load_basic();
    let files = pkg.files();

    let readme = files
        .iter()
        .find(|f| f.path() == "/usr/share/doc/rpm-basic/README")
        .expect("README should exist");
    assert!(readme.flags().is_doc());
    assert!(!readme.flags().is_config());

    let ghost = files
        .iter()
        .find(|f| f.path() == "/var/log/rpm-basic/basic.log")
        .expect("ghost file should exist");
    assert!(ghost.flags().is_ghost());
}

#[test]
fn test_file_digest() {
    let pkg = load_basic();
    let files = pkg.files();

    assert!(files.digest_algo() > 0);

    let config = files
        .iter()
        .find(|f| f.path() == "/etc/rpm-basic/example_config.toml")
        .expect("config file should exist");
    let digest = config.digest().expect("regular file should have a digest");
    assert!(!digest.is_empty());
}

#[test]
fn test_files_empty_package() {
    common::configure();
    let pkg = Package::from_file(&assets_path().join("rpm-empty-0-0.x86_64.rpm")).unwrap();
    let files = pkg.files();

    assert_eq!(files.len(), 0);
    assert!(files.is_empty());
    assert_eq!(files.iter().count(), 0);
}

// Package::requires / provides / conflicts / obsoletes / recommends / suggests

#[test]
fn test_requires() {
    let pkg = load_basic();
    let requires = pkg.requires();

    assert!(!requires.is_empty());

    let names: Vec<&str> = requires.iter().map(|d| d.name()).collect();
    assert!(names.contains(&"methylamine"));
    assert!(names.contains(&"morality"));
    assert!(names.contains(&"regret"));
    assert!(names.contains(&"rpmlib(CompressedFileNames)"));

    let methylamine = requires.iter().find(|d| d.name() == "methylamine").unwrap();
    assert_eq!(methylamine.evr(), Some("1.0.0-1"));
    assert!(methylamine.flags().is_greater());
    assert!(methylamine.flags().is_equal());
    assert!(!methylamine.flags().is_less());
    assert_eq!(methylamine.flags().version_cmp_str(), ">=");

    let morality = requires.iter().find(|d| d.name() == "morality").unwrap();
    assert_eq!(morality.evr(), Some("2"));
    assert_eq!(morality.flags().version_cmp_str(), "<=");

    let regret = requires.iter().find(|d| d.name() == "regret").unwrap();
    assert_eq!(regret.evr(), None);
    assert_eq!(regret.flags().version_cmp_str(), "");

    let rpmlib_dep = requires
        .iter()
        .find(|d| d.name() == "rpmlib(CompressedFileNames)")
        .unwrap();
    assert!(rpmlib_dep.flags().is_rpmlib());
    assert_eq!(rpmlib_dep.flags().version_cmp_str(), "<=");
}

#[test]
fn test_provides() {
    let pkg = load_basic();
    let provides = pkg.provides();

    assert!(!provides.is_empty());

    let names: Vec<&str> = provides.iter().map(|d| d.name()).collect();
    assert!(names.contains(&"rpm-basic"));
    assert!(names.contains(&"aaronpaul"));
    assert!(names.contains(&"shock"));

    let shock = provides.iter().find(|d| d.name() == "shock").unwrap();
    assert_eq!(shock.evr(), Some("33"));
    assert_eq!(shock.flags().version_cmp_str(), "=");
}

#[test]
fn test_conflicts() {
    let pkg = load_basic();
    let conflicts = pkg.conflicts();

    assert_eq!(conflicts.len(), 1);

    let hank = conflicts.iter().find(|d| d.name() == "hank").unwrap();
    assert_eq!(hank.evr(), Some("35"));
    assert_eq!(hank.flags().version_cmp_str(), ">");
}

#[test]
fn test_obsoletes() {
    let pkg = load_basic();
    let obsoletes = pkg.obsoletes();

    assert_eq!(obsoletes.len(), 2);

    let names: Vec<&str> = obsoletes.iter().map(|d| d.name()).collect();
    assert!(names.contains(&"gusfring"));
    assert!(names.contains(&"tucosalamanca"));

    let gusfring = obsoletes.iter().find(|d| d.name() == "gusfring").unwrap();
    assert_eq!(gusfring.evr(), Some("32.1-0"));
    assert_eq!(gusfring.flags().version_cmp_str(), "<");
}

#[test]
fn test_recommends() {
    let pkg = load_basic();
    let recommends = pkg.recommends();

    assert!(!recommends.is_empty());

    let names: Vec<&str> = recommends.iter().map(|d| d.name()).collect();
    assert!(names.contains(&"huel"));
    assert!(names.contains(&"SaulGoodman(CriminalLawyer)"));

    let huel = recommends.iter().find(|d| d.name() == "huel").unwrap();
    assert_eq!(huel.evr(), Some("9:11.0-0"));
    assert_eq!(huel.flags().version_cmp_str(), ">");
}

#[test]
fn test_suggests() {
    let pkg = load_basic();
    let suggests = pkg.suggests();

    assert_eq!(suggests.len(), 1);

    let chili = suggests.iter().find(|d| d.name() == "chilipowder").unwrap();
    assert_eq!(chili.evr(), None);
}

#[test]
fn test_dependencies_empty_package() {
    common::configure();
    let pkg = Package::from_file(&assets_path().join("rpm-empty-0-0.x86_64.rpm")).unwrap();

    assert!(pkg.requires().iter().all(|d| d.flags().is_rpmlib()));
    assert!(pkg.provides().len() >= 1); // self-provide always exists
    assert!(pkg.conflicts().is_empty());
    assert!(pkg.obsoletes().is_empty());
    assert!(pkg.recommends().is_empty());
    assert!(pkg.suggests().is_empty());
}

#[test]
fn test_dependency_display() {
    let pkg = load_basic();
    let requires = pkg.requires();

    let methylamine = requires.iter().find(|d| d.name() == "methylamine").unwrap();
    assert_eq!(format!("{methylamine}"), "methylamine >= 1.0.0-1");

    let regret = requires.iter().find(|d| d.name() == "regret").unwrap();
    assert_eq!(format!("{regret}"), "regret");
}

#[test]
fn test_dependencies_into_iter() {
    let pkg = load_basic();
    let conflicts = pkg.conflicts();

    let names: Vec<String> = conflicts.into_iter().map(|d| d.name().to_owned()).collect();
    assert_eq!(names, vec!["hank"]);
}

// Package::format

#[test]
fn test_format_nvra() {
    let pkg = load_basic();
    let result = pkg.format("%{NAME}-%{VERSION}-%{RELEASE}.%{ARCH}").unwrap();
    assert_eq!(result, "rpm-basic-2.3.4-5.el9.noarch");
}

#[test]
fn test_format_nevra() {
    let pkg = load_basic();
    let result = pkg.format("%{NEVRA}").unwrap();
    assert_eq!(result, "rpm-basic-1:2.3.4-5.el9.noarch");
}

#[test]
fn test_format_epoch_missing_display() {
    common::configure();
    let pkg = Package::from_file(&assets_path().join("rpm-empty-0-0.x86_64.rpm")).unwrap();
    // librpm renders a missing EPOCH as "(none)" in the format string
    let result = pkg.format("%{EPOCH}").unwrap();
    assert_eq!(result, "(none)");
}

#[test]
fn test_format_invalid_tag_returns_error() {
    let pkg = load_basic();
    let err = pkg.format("%{NOSUCHTAG}").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::FormatString);
}

#[test]
fn test_format_literal_text() {
    let pkg = load_basic();
    let result = pkg.format("hello world").unwrap();
    assert_eq!(result, "hello world");
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
