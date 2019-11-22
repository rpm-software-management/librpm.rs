use librpm::Db;

use std::path::Path;

#[test]
fn db_open_test() {
    Db::open::<&Path>().unwrap();
}
