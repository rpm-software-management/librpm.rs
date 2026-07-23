//! Read metadata from an .rpm file.
//!
//! Run with: cargo run --example package_reading

use std::path::Path;

fn main() {
    librpm::init().expect("failed to initialize librpm");

    // Host platform as RPM sees it
    println!("Host arch: {}", librpm::arch().unwrap_or("(unknown)"));
    println!("Host OS:   {}", librpm::os().unwrap_or("(unknown)"));
    println!();

    let rpm_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/rpms/rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm");

    let pkg = librpm::PackageHeader::from_file(&rpm_path, None).expect("failed to read RPM");
    println!("Name:    {}", pkg.name());
    println!("Version: {}", pkg.version());
    println!("Release: {}", pkg.release());
    println!("Arch:    {}", pkg.arch().unwrap_or("(none)"));
    println!("License: {}", pkg.license());
    println!("Summary: {}", pkg.summary());
    println!("NEVRA:   {}", pkg.nevra());
    println!("Is Source Package:   {}", pkg.is_source());
    println!();

    // Check for tag presence without decoding
    println!("Has EPOCH tag: {}", pkg.has_tag(librpm::Tag::EPOCH));
    println!();

    // Access raw tag data
    if let Some(data) = pkg.get(librpm::Tag::BUILDTIME) {
        println!("BUILDTIME tag: {:?}", data.as_int64());
    }
    println!();

    // Dependencies with predicates
    println!("=== Requires ===");
    for dep in pkg.requires().iter() {
        let mut tags = Vec::new();
        if dep.is_rich() {
            tags.push("rich");
        }
        if dep.is_weak() {
            tags.push("weak");
        }
        let suffix = if tags.is_empty() {
            String::new()
        } else {
            format!("  [{}]", tags.join(", "))
        };
        println!("  {dep}{suffix}");
    }
    println!();

    // File lookup by path
    println!("=== Files ===");
    let files = pkg.files();
    println!("{} files total", files.len());

    if let Some(entry) = files.find("/etc/rpm-basic/example_config.toml") {
        println!(
            "  Found config: {} ({} bytes, config={})",
            entry.path(),
            entry.size(),
            entry.flags().is_config(),
        );
    }
}
