use librpm::{Package, config::set_db_path, db::installed_packages};

mod common;

#[test]
fn test_centos_stream_9_rpm_database() {
    common::configure();
    set_db_path(&common::get_assets_path().join("centos-stream-9")).unwrap();

    let mut packages: Vec<Package> = installed_packages().collect();
    packages.sort_by_key(|p| p.name().to_string());

    assert_eq!(packages.len(), 137);
    let sample_package = &packages[0];
    assert_eq!(sample_package.name(), "alternatives");
    assert_eq!(sample_package.epoch(), None);
    assert_eq!(sample_package.version(), "1.24");
    assert_eq!(sample_package.release(), "2.el9");
    assert_eq!(sample_package.arch(), Some("x86_64"));
    assert_eq!(sample_package.license(), "GPL-2.0-only");
    assert_eq!(
        sample_package.summary(),
        "A tool to maintain symbolic links determining default commands"
    );
    assert_eq!(
        sample_package.description(),
        "alternatives creates, removes, maintains and displays information about the\nsymbolic links comprising the alternatives system. It is possible for several\nprograms fulfilling the same or similar functions to be installed on a single\nsystem at the same time."
    );
}
