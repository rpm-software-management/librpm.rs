//! librpm.rs integration tests

use librpm::{config, Index};
use std::io::BufRead;
use std::process::Command;
use std::sync::Once;

static CONFIGURE: Once = Once::new();

// Read the default config
// TODO: create a mock RPM database for testing
fn configure() {
    CONFIGURE.call_once(|| {
        config::read_file(None).unwrap();
    });
}

fn fetch_package_info(package_name: &str, query_param: &str) -> Vec<Option<String>> {
    let rpm_info = Command::new("rpm")
        .arg("-q")
        .arg(package_name)
        .arg(format!("--queryformat=%{{{}}}\n", query_param))
        .output()
        .unwrap()
        .stdout;

    let text = String::from_utf8(rpm_info).unwrap();
    text.lines()
        .map(|c| {
            if c.starts_with("(none)") {
                None
            } else {
                Some(c.to_owned())
            }
        })
        .collect()
}

#[test]
fn db_find_test() {
    configure();

    let PACKAGE_NAME = "rpm-devel";
    let PACKAGE_NEVRA = fetch_package_info(PACKAGE_NAME, "NEVRA")[0]
        .clone()
        .unwrap();

    let mut matches = Index::Name.find(PACKAGE_NAME);

    if let Some(package) = matches.next() {
        assert_eq!(package.name(), PACKAGE_NAME);
        assert_eq!(
            package.epoch().map(|e| e.to_owned()),
            fetch_package_info(PACKAGE_NAME, "EPOCH")[0].clone()
        );
        assert_eq!(
            package.version(),
            fetch_package_info(PACKAGE_NAME, "VERSION")[0]
                .clone()
                .unwrap()
        );
        assert_eq!(
            package.release(),
            fetch_package_info(PACKAGE_NAME, "RELEASE")[0]
                .clone()
                .unwrap()
        );
        assert_eq!(
            package.summary(),
            fetch_package_info(PACKAGE_NAME, "SUMMARY")[0]
                .clone()
                .unwrap()
        );
        assert_eq!(
            package.license(),
            fetch_package_info(PACKAGE_NAME, "LICENSE")[0]
                .clone()
                .unwrap()
        );

        assert_eq!(package.nevra(), PACKAGE_NEVRA);
        assert_eq!(package.to_string(), PACKAGE_NEVRA);

        assert!(matches.next().is_none(), "expected one result, got more!");
    } else {
        if librpm::db::installed_packages().count() == 0 {
            eprintln!("*** warning: No RPMs installed! Tests skipped!")
        } else {
            panic!("some RPMs installed, but not `rpm-devel`?!");
        }
    }
}

#[test]
fn db_find_test_multiple_packages() {
    configure();
    assert!(Index::Name.find("kernel").next().is_some());
    assert!(Index::Name.find("rpm-devel").next().is_some());
}

#[test]
fn db_find_test_multiple_matching() {
    configure();
    let matches: Vec<librpm::Package> = Index::Name.find("kernel").collect();
    assert!(matches.len() > 1);

    for package in matches {
        assert_eq!(package.name(), "kernel");
    }
}
