//! Read metadata from an .rpm file.
//!
//! Run with: cargo run --example package_reading

use std::path::Path;

fn main() {
    librpm::init().expect("failed to initialize librpm");

    let rpm_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/rpms/rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm");

    let pkg = librpm::PackageHeader::from_file(&rpm_path).expect("failed to read RPM");
    println!("Name:    {}", pkg.name());
    println!("Version: {}", pkg.version());
    println!("Release: {}", pkg.release());
    println!("Arch:    {}", pkg.arch().unwrap_or("(none)"));
    println!("License: {}", pkg.license());
    println!("Summary: {}", pkg.summary());
    println!("NEVRA:   {}", pkg.nevra());
    println!();

    // Access raw tag data
    if let Some(data) = pkg.get(librpm::Tag::BUILDTIME) {
        println!("BUILDTIME tag: {data:?}");
    }
}
