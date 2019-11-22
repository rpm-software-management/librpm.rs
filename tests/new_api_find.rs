//! librpm.rs integration tests

use librpm::{Db, Index};

use std::path::Path;

/// The `.rpm` containing librpm itself
const PACKAGE_NAME: &str = "rpm-devel";
const PACKAGE_SUMMARY: &str = "Development files for manipulating RPM packages";
const PACKAGE_LICENSE: &str = "GPLv2+ and LGPLv2+ with exceptions";

#[test]
fn db_find_test() {
    let db = Db::open::<&Path>().unwrap();
    let mut matches = Index::Name.find(&db, PACKAGE_NAME);

    if let Some(package) = matches.next() {
        assert_eq!(package.name, PACKAGE_NAME);
        assert_eq!(package.summary, PACKAGE_SUMMARY);
        assert_eq!(package.license.as_str(), PACKAGE_LICENSE);
        assert!(matches.next().is_none(), "expected one result, got more!");
    } else {
        if librpm::db::installed_packages().count() == 0 {
            eprintln!("*** warning: No RPMs installed! Tests skipped!")
        } else {
            panic!("some RPMs installed, but not `rpm-devel`?!");
        }
    }
}
