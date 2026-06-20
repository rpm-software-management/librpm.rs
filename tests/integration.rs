//! librpm.rs integration tests

use librpm::{Index, config};
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

fn fetch_package_info(package_name: &str, query_param: &str) -> Option<String> {
    let rpm_info = Command::new("rpm")
        .arg("-q")
        .arg(package_name)
        .arg(format!("--queryformat=%{{{}}}", query_param))
        .output()
        .unwrap()
        .stdout;

    let text = String::from_utf8(rpm_info).unwrap();
    Some(text).filter(|c| c != "(none)" && c != "")
}

#[test]
fn db_find_test() {
    configure();

    let package_name = "rpm-devel";
    let package_nevra = fetch_package_info(package_name, "NEVRA").unwrap();

    let mut matches = Index::Name.find(package_name);

    if let Some(package) = matches.next() {
        assert_eq!(package.name(), "rpm-devel");
        assert_eq!(
            package.epoch(),
            fetch_package_info(package_name, "EPOCH").map(|s| s.parse::<i32>().unwrap())
        );
        assert_eq!(
            package.version(),
            fetch_package_info(package_name, "VERSION").unwrap()
        );
        assert_eq!(
            package.release(),
            fetch_package_info(package_name, "RELEASE").unwrap()
        );
        assert_eq!(
            package.summary(),
            fetch_package_info(package_name, "SUMMARY").unwrap()
        );
        assert_eq!(
            package.license(),
            fetch_package_info(package_name, "LICENSE").unwrap()
        );

        assert_eq!(package.nevra(), package_nevra);
        assert_eq!(package.to_string(), package_nevra);

        assert!(matches.next().is_none(), "expected one result, got more!");
    } else {
        if librpm::db::installed_packages().count() == 0 {
            eprintln!("*** warning: No RPMs installed! Tests skipped!")
        } else {
            panic!("some RPMs installed, but not `rpm-devel`?!");
        }
    }
}

// TODO: This will deadlock: https://github.com/rpm-software-management/librpm.rs/issues/15
// #[test]
// fn db_find_test_multiple() {

//     configure();

//     let mut matches = Index::Name.find("glibc-common");
//     if let Some(package) = matches.next() {
//         assert_eq!(package.name(), "glibc-common");
//         assert!(matches.next().is_none(), "expected one result, got more!");
//     } else {
//         panic!("glibc-common package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?");
//     }

//     let mut matches = Index::Name.find("glibc");
//     if let Some(package) = matches.next() {
//         assert_eq!(package.name(), "glibc");
//         assert!(matches.next().is_none(), "expected one result, got more!");
//     } else {
//         panic!("glibc package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?");
//     }
// }
