//! librpm.rs integration tests
//!
//! Tests here exercise functionality that doesn't fit neatly into the
//! per-distro assertion pattern in common.rs — either because they depend
//! on specific CS9 package contents, or because they test tag data access
//! and macro operations rather than index queries.

use librpm::db::{Index, MatchMode};
use librpm::{Db, Package, Tag};

mod common;

#[test]
fn test_scalar_int32_tag() {
    let db = common::init(&common::CENTOS_STREAM_9);

    let results: Vec<Package> = db.find(Index::Name, "alternatives").collect();
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

    let results: Vec<Package> = db.find(Index::Name, "alternatives").collect();
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

    let results: Vec<Package> = db.find(Index::Name, "alternatives").collect();
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

#[test]
fn db_find_test_multiple() {
    let db = common::init(&common::CENTOS_STREAM_9);

    let mut matches = db.find(Index::Name, "glibc-common");
    if let Some(package) = matches.next() {
        assert_eq!(package.name(), "glibc-common");
        assert!(matches.next().is_none(), "expected one result, got more!");
    } else {
        panic!(
            "glibc-common package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?"
        );
    }

    let mut matches = db.find(Index::Name, "glibc");
    if let Some(package) = matches.next() {
        assert_eq!(package.name(), "glibc");
        assert!(matches.next().is_none(), "expected one result, got more!");
    } else {
        panic!(
            "glibc package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?"
        );
    }
}

#[test]
fn find_re_glob_returns_superset() {
    let db = common::init(&common::CENTOS_STREAM_9);

    let results: Vec<Package> = db.find_re(Index::Name, "glibc*", MatchMode::Glob).collect();
    assert!(
        results.len() >= 2,
        "glob 'glibc*' should match glibc and glibc-common (got {})",
        results.len(),
    );
    for pkg in &results {
        assert!(
            pkg.name().starts_with("glibc"),
            "all results should start with 'glibc', got '{}'",
            pkg.name(),
        );
    }
}

#[test]
fn db_find_test_multiple_packages() {
    let db = common::init(&common::CENTOS_STREAM_9);

    assert!(db.find(Index::Name, "bash").next().is_some());
    assert!(db.find(Index::Name, "filesystem").next().is_some());
}

#[test]
fn iterator_outlives_db() {
    let packages: Vec<Package> = {
        let db = common::init(&common::CENTOS_STREAM_9);
        db.installed_packages().collect()
    };
    // Db (and its TransactionSet) is dropped. The collected Packages must
    // still be valid — each owns a refcounted Header independent of the rpmts.
    assert!(!packages.is_empty());
    for pkg in &packages {
        assert!(!pkg.name().is_empty());
    }
}

#[test]
fn find_re_regex_returns_superset() {
    let db = common::init(&common::CENTOS_STREAM_9);

    let results: Vec<Package> = db
        .find_re(Index::Name, "^glibc", MatchMode::Regex)
        .collect();
    assert!(
        results.len() >= 2,
        "regex '^glibc' should match glibc and glibc-common (got {})",
        results.len(),
    );
    for pkg in &results {
        assert!(
            pkg.name().starts_with("glibc"),
            "all results should start with 'glibc', got '{}'",
            pkg.name(),
        );
    }
}

#[test]
fn multiple_db_instances() {
    let db1 = common::init(&common::CENTOS_STREAM_9);
    let db2 = common::init(&common::CENTOS_STREAM_9);

    let count1 = db1.installed_packages().count();
    let count2 = db2.installed_packages().count();
    assert_eq!(count1, count2);

    let results1: Vec<Package> = db1.find(Index::Name, "bash").collect();
    let results2: Vec<Package> = db2.find(Index::Name, "bash").collect();
    assert_eq!(results1.len(), results2.len());
    assert_eq!(results1[0].name(), results2[0].name());
}

#[test]
fn concurrent_db_instances() {
    common::init(&common::CENTOS_STREAM_9);

    let handles: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                let db = Db::open().unwrap();
                let packages: Vec<Package> = db.installed_packages().collect();
                assert!(!packages.is_empty());
                packages.len()
            })
        })
        .collect();

    let counts: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(counts.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn find_re_glob_by_providename() {
    let db = common::init(&common::CENTOS_STREAM_9);

    let results: Vec<Package> = db
        .find_re(Index::Providename, "glibc*", MatchMode::Glob)
        .collect();
    assert!(
        !results.is_empty(),
        "glob on Providename for 'glibc*' should match at least one package",
    );
}

#[test]
fn iter_match_count_agrees_with_iteration() {
    let db = common::init(&common::CENTOS_STREAM_9);

    let mut iter = db.find(Index::Providename, "bash");
    let count = iter.match_count();
    let actual = std::iter::from_fn(|| iter.next()).count();
    assert_eq!(
        count, actual,
        "match_count should equal actual iteration count"
    );
}
