use librpm::{Db, Index};

use std::path::Path;

#[test]
fn db_find_test_multiple() {
    let db = Db::open::<&Path>().unwrap();
    let mut matches = Index::Name.find(&db, "glibc-common");

    if let Some(package) = matches.next() {
        assert_eq!(package.name, "glibc-common");
        assert!(matches.next().is_none(), "expected one result, got more!");
    } else {
        panic!("glibc-common package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?");
    }

    let mut matches = Index::Name.find(&db, "glibc");
    if let Some(package) = matches.next() {
        assert_eq!(package.name, "glibc");
        assert!(matches.next().is_none(), "expected one result, got more!");
    } else {
        panic!("glibc package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?");
    }
}
