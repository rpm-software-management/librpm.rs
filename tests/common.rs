//! Shared test infrastructure for per-distro integration tests.
//!
//! Provides distro database definitions and a common assertion helper so that
//! each per-distro test binary is a one-liner delegating to [`assert_distro`].

#![allow(dead_code)]

use std::{
    path::{Path, PathBuf},
    sync::Once,
};

use std::time;

use librpm::{Index, Package, config};

static CONFIGURE: Once = Once::new();

pub fn configure() {
    CONFIGURE.call_once(|| {
        config::read_file(None).unwrap();
    });
}

pub fn get_assets_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

pub struct DistroTestCase {
    pub name: &'static str,
    pub db_subdir: &'static str,
    pub expected_count: usize,
    pub sample: SamplePackage,
}

pub struct SamplePackage {
    pub name: &'static str,
    pub epoch: Option<i32>,
    pub version: &'static str,
    pub release: &'static str,
    pub arch: Option<&'static str>,
    pub license: &'static str,
    pub summary: &'static str,
    pub description: &'static str,
}

pub fn setup_distro(distro: &DistroTestCase) {
    configure();
    config::set_db_path(&get_assets_path().join(distro.db_subdir)).unwrap();
}

pub fn assert_distro(distro: &DistroTestCase) {
    setup_distro(distro);

    let mut packages: Vec<Package> = librpm::db::installed_packages().collect();
    packages.sort_by_key(|p| p.name().to_string());

    assert_eq!(
        packages.len(),
        distro.expected_count,
        "{}: unexpected package count",
        distro.name,
    );

    let first = &packages[0];
    let sample = &distro.sample;
    assert_eq!(first.name(), sample.name, "{}", distro.name);
    assert_eq!(first.epoch(), sample.epoch, "{}", distro.name);
    assert_eq!(first.version(), sample.version, "{}", distro.name);
    assert_eq!(first.release(), sample.release, "{}", distro.name);
    assert_eq!(first.arch(), sample.arch, "{}", distro.name);
    assert_eq!(first.license(), sample.license, "{}", distro.name);
    assert_eq!(first.summary(), sample.summary, "{}", distro.name);
    assert_eq!(first.description(), sample.description, "{}", distro.name);
}

pub fn assert_find_by_name(distro: &DistroTestCase) {
    setup_distro(distro);

    let results: Vec<Package> = Index::Name.find(distro.sample.name).collect();
    assert_eq!(
        results.len(),
        1,
        "{}: expected exactly 1 result for '{}'",
        distro.name,
        distro.sample.name,
    );
    assert_eq!(results[0].name(), distro.sample.name, "{}", distro.name);
    assert_eq!(
        results[0].version(),
        distro.sample.version,
        "{}",
        distro.name
    );
}

pub fn assert_find_nonexistent(distro: &DistroTestCase) {
    setup_distro(distro);

    let results: Vec<Package> = Index::Name.find("nonexistent-package-xyz").collect();
    assert_eq!(
        results.len(),
        0,
        "{}: expected no results for nonexistent package",
        distro.name,
    );
}

pub fn assert_buildtimes_valid(distro: &DistroTestCase) {
    setup_distro(distro);

    // 2020-01-01 as a reasonable lower bound for all test databases
    let year_2020 = time::SystemTime::UNIX_EPOCH + time::Duration::from_secs(1577836800);

    for pkg in librpm::db::installed_packages() {
        let bt = pkg.buildtime();
        assert!(
            bt > year_2020,
            "{}: package '{}' has suspiciously old buildtime",
            distro.name,
            pkg.name(),
        );
    }
}

pub const CENTOS_STREAM_9: DistroTestCase = DistroTestCase {
    name: "centos-stream-9",
    db_subdir: "centos-stream-9",
    expected_count: 137,
    sample: SamplePackage {
        name: "alternatives",
        epoch: None,
        version: "1.24",
        release: "2.el9",
        arch: Some("x86_64"),
        license: "GPL-2.0-only",
        summary: "A tool to maintain symbolic links determining default commands",
        description: "alternatives creates, removes, maintains and displays information about the\nsymbolic links comprising the alternatives system. It is possible for several\nprograms fulfilling the same or similar functions to be installed on a single\nsystem at the same time.",
    },
};

pub const CENTOS_STREAM_10: DistroTestCase = DistroTestCase {
    name: "centos-stream-10",
    db_subdir: "centos-stream-10",
    expected_count: 162,
    sample: SamplePackage {
        name: "alternatives",
        epoch: None,
        version: "1.30",
        release: "2.el10",
        arch: Some("x86_64"),
        license: "GPL-2.0-only",
        summary: "A tool to maintain symbolic links determining default commands",
        description: "alternatives creates, removes, maintains and displays information about the\nsymbolic links comprising the alternatives system. It is possible for several\nprograms fulfilling the same or similar functions to be installed on a single\nsystem at the same time.",
    },
};

pub const FEDORA_44: DistroTestCase = DistroTestCase {
    name: "fedora-44",
    db_subdir: "fedora-44",
    expected_count: 147,
    sample: SamplePackage {
        name: "alternatives",
        epoch: None,
        version: "1.33",
        release: "5.fc44",
        arch: Some("x86_64"),
        license: "GPL-2.0-only",
        summary: "A tool to maintain symbolic links determining default commands",
        description: "alternatives creates, removes, maintains and displays information about the\nsymbolic links comprising the alternatives system. It is possible for several\nprograms fulfilling the same or similar functions to be installed on a single\nsystem at the same time.",
    },
};
