// This is separate test as there is global state in native librpm.
// It can't be cleared w/o process restart.
// Every integration test source gets compiled to separate binary, so it works that way.

use librpm::{Db, error};

use std::path::Path;

#[test]
fn db_open_twice_test() {
    Db::open::<&Path>().unwrap();
    assert_eq!(
        Db::open::<&Path>(),
        Err(error::Error {
            kind: error::ErrorKind::AlreadyConfigured,
            msg: Some("librpm is already configured, global state can\'t be configured again".into())
        })
    );
}
