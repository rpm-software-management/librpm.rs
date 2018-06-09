//! librpm.rs integration tests

extern crate librpm;

use librpm::{config, Index};
use std::sync::{Once, ONCE_INIT};

/// The `.rpm` containing librpm itself
const PACKAGE_NAME: &str = "rpm-devel";
const PACKAGE_SUMMARY: &str = "Development files for manipulating RPM packages";
const PACKAGE_LICENSE: &str = "GPLv2+ and LGPLv2+ with exceptions";

static CONFIGURE: Once = ONCE_INIT;

// Read the default config
// TODO: create a mock RPM database for testing
fn configure() {
    CONFIGURE.call_once(|| {
        config::read_file(None).unwrap();
    });
}

#[test]
fn db_find_test() {
    configure();

    let mut matches = Index::Name.find(PACKAGE_NAME);

    if let Some(package) = matches.next() {
        assert_eq!(package.name, PACKAGE_NAME);
        assert_eq!(package.summary, PACKAGE_SUMMARY);
        assert_eq!(package.license.as_str(), PACKAGE_LICENSE);
    } else {
        panic!("expected 1 result, got 0!");
    }

    assert!(matches.next().is_none(), "expected one result, got more!");
}
