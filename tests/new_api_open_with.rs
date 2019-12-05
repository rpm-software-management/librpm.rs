use librpm::Db;

#[test]
fn db_open_test() {
    Db::open_with().config("/usr/lib/rpm/rpmrc").open().unwrap();
}
