use librpm::Db;

use std::path::Path;

#[test]
fn db_open_test() {
    Db::open_with().config("/usr/lib/rpm/rpmrc").open().unwrap();
}
