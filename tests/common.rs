//! Shared test infrastructure for per-distro integration tests.
//!
//! Provides distro database definitions and a common assertion helper so that
//! each per-distro test binary is a one-liner delegating to [`assert_distro`].

#![allow(dead_code)]

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use std::time;

use librpm::db::MatchMode;
use librpm::{Db, Index, Package};

static INIT: OnceLock<()> = OnceLock::new();
static DISTRO: OnceLock<&'static str> = OnceLock::new();

pub fn configure() -> Db {
    INIT.get_or_init(|| {
        librpm::init().unwrap();
    });
    Db::open().unwrap()
}

pub fn init(distro: &DistroTestCase) -> Db {
    let prev = DISTRO.get_or_init(|| {
        librpm::init_with(None, Some(&get_assets_path().join(distro.db_subdir))).unwrap();
        distro.db_subdir
    });
    assert_eq!(
        *prev, distro.db_subdir,
        "cannot use two different distro databases in one process"
    );
    Db::open().unwrap()
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

pub fn assert_distro(distro: &DistroTestCase) {
    let db = init(distro);

    let mut packages: Vec<Package> = db.installed_packages().collect();
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
    let db = init(distro);

    let results: Vec<Package> = db.find(Index::Name, distro.sample.name).collect();
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
    let db = init(distro);

    let results: Vec<Package> = db.find(Index::Name, "nonexistent-package-xyz").collect();
    assert_eq!(
        results.len(),
        0,
        "{}: expected no results for nonexistent package",
        distro.name,
    );
}

pub fn assert_find_by_providename(distro: &DistroTestCase) {
    let db = init(distro);

    let results: Vec<Package> = db.find(Index::Providename, distro.sample.name).collect();
    assert!(
        results.iter().any(|p| p.name() == distro.sample.name),
        "{}: '{}' should provide itself",
        distro.name,
        distro.sample.name,
    );
}

pub fn assert_find_by_requirename(distro: &DistroTestCase) {
    let db = init(distro);

    let results: Vec<Package> = db.find(Index::Requirename, "glibc").collect();
    assert!(
        !results.is_empty(),
        "{}: at least one package should require glibc",
        distro.name,
    );
}

pub fn assert_find_by_dirnames(distro: &DistroTestCase) {
    let db = init(distro);

    let results: Vec<Package> = db.find(Index::Dirnames, "/etc/").collect();
    assert!(
        !results.is_empty(),
        "{}: at least one package should own files in /etc/",
        distro.name,
    );
}

// NOTE: We intentionally do not test `Index::Basenames` or `Index::Instfilenames`
// with exact-match lookups (`Db::find`) against offline database snapshots.
//
// These indices use `rpmdbFindByFile` internally, which performs filesystem
// fingerprinting via `stat()` / `fpLookup()` to resolve symlinks before matching
// (see rpm/lib/rpmdb.cc:rpmdbFindByFile).  When the files referenced by the
// package headers do not exist on the local filesystem — as is the case with our
// offline test databases captured from container images — the fingerprint
// comparison always fails and the lookup silently returns zero results.
//
// The `find_re` (glob/regex) path works because `rpmdbSetIteratorRE` filters
// headers purely by tag string comparison without filesystem access.  However,
// this iterates every matching header and is too slow for CI when applied to
// file-based indices.
//
// `Index::Dirnames` exact-match lookups work fine because they use simple string
// index lookups without fingerprinting.

pub fn assert_find_re_glob(distro: &DistroTestCase) {
    let db = init(distro);

    let results: Vec<Package> = db
        .find_re(Index::Name, "alternatives*", MatchMode::Glob)
        .collect();
    assert!(
        results.iter().any(|p| p.name() == "alternatives"),
        "{}: glob 'alternatives*' should match the alternatives package",
        distro.name,
    );
}

pub fn assert_find_re_regex(distro: &DistroTestCase) {
    let db = init(distro);

    let results: Vec<Package> = db
        .find_re(Index::Name, "^alternatives$", MatchMode::Regex)
        .collect();
    assert_eq!(
        results.len(),
        1,
        "{}: regex '^alternatives$' should match exactly one package",
        distro.name,
    );
    assert_eq!(results[0].name(), "alternatives", "{}", distro.name);
}

pub fn assert_find_re_no_match(distro: &DistroTestCase) {
    let db = init(distro);

    let results: Vec<Package> = db
        .find_re(Index::Name, "zzz-nonexistent*", MatchMode::Glob)
        .collect();
    assert_eq!(
        results.len(),
        0,
        "{}: glob should return no results for nonexistent pattern",
        distro.name,
    );
}

pub fn assert_iter_match_count(distro: &DistroTestCase) {
    let db = init(distro);

    let iter = db.find(Index::Name, distro.sample.name);
    assert_eq!(
        iter.match_count(),
        1,
        "{}: '{}' should have match_count 1",
        distro.name,
        distro.sample.name,
    );

    let iter = db.find(Index::Name, "nonexistent-xyz");
    assert_eq!(
        iter.match_count(),
        0,
        "{}: nonexistent package should have match_count 0",
        distro.name,
    );
}

pub fn assert_iter_offset(distro: &DistroTestCase) {
    let db = init(distro);

    let mut iter = db.find(Index::Name, distro.sample.name);
    assert_eq!(
        iter.offset(),
        0,
        "{}: offset should be 0 before first next()",
        distro.name,
    );
    if let Some(_pkg) = iter.next() {
        assert_ne!(
            iter.offset(),
            0,
            "{}: offset should be non-zero after next()",
            distro.name,
        );
    }
}

pub fn assert_buildtimes_valid(distro: &DistroTestCase) {
    let db = init(distro);

    // 2020-01-01 as a reasonable lower bound for all test databases
    let year_2020 = time::SystemTime::UNIX_EPOCH + time::Duration::from_secs(1577836800);

    for pkg in db.installed_packages() {
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
