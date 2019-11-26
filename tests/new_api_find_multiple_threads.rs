use librpm::{Db, Index};

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn db_find_test_multiple() {
    let db = Arc::new(Mutex::new(Db::open::<&Path>().unwrap()));

    {
        let db = db.clone();
        thread::spawn(move || {
            let mut matches = Index::Name.find(&db.lock().unwrap(), "glibc-common");
            if let Some(package) = matches.next() {
                assert_eq!(package.name, "glibc-common");
                assert!(matches.next().is_none(), "expected one result, got more!");
            } else {
                panic!("glibc-common package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?");
            }
        });
    }

    let mut matches = Index::Name.find(&db.lock().unwrap(), "glibc");
    if let Some(package) = matches.next() {
        assert_eq!(package.name, "glibc");
        assert!(matches.next().is_none(), "expected one result, got more!");
    } else {
        panic!("glibc package not installed, are you running on RPM hosted system (RHEL, Fedora, CentOS)?");
    }
}
