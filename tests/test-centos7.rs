use librpm::{config::set_db_path, db::installed_packages, Package};

mod common;

#[test]
fn test_centos_7_rpm_database() {
    common::configure();
    set_db_path(&common::get_assets_path().join("centos7")).unwrap();

    let mut packages: Vec<Package> = installed_packages().collect();
    packages.sort_by_key(|p| p.name().to_string());

    assert_eq!(packages.len(), 148);
    let sample_package = &packages[0];
    assert_eq!(sample_package.name(), "acl");
    assert_eq!(sample_package.epoch(), None);
    assert_eq!(sample_package.version(), "2.2.51");
    assert_eq!(sample_package.release(), "15.el7");
    assert_eq!(sample_package.arch(), Some("x86_64"));
    assert_eq!(sample_package.license(), "GPLv2+");
    assert_eq!(sample_package.summary(), "Access control list utilities");
    assert_eq!(
        sample_package.description(),
        "This package contains the getfacl and setfacl utilities needed for\nmanipulating access control lists."
    );
}
