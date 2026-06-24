//! Parse a spec file and inspect its contents.
//!
//! Run with: cargo run --example spec_parsing --features build

use std::path::Path;

use librpm::build::{Section, Spec, SpecFlags};

fn main() {
    librpm::init().expect("failed to initialize librpm");

    let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/test.spec");
    let spec = Spec::parse(spec_path.to_str().unwrap(), SpecFlags::FORCE, None)
        .expect("failed to parse spec");

    // Source RPM header
    let srpm = spec.source_header();
    println!("SRPM name:    {}", srpm.name());
    println!("SRPM version: {}", srpm.version());
    println!("SRPM summary: {}", srpm.summary());
    println!();

    // Binary subpackages
    println!("Binary packages:");
    for pkg in spec.packages() {
        let hdr = pkg.header();
        println!("  {} ({}-{})", hdr.name(), hdr.version(), hdr.release());
    }
    println!();

    // Source and patch entries
    println!("Sources and patches:");
    for src in spec.sources() {
        let kind = if src.is_source() { "Source" } else { "Patch" };
        println!("  {kind}{}: {}", src.num(), src.filename());
    }
    println!();

    // Script sections
    if let Some(prep) = spec.get_section(Section::PREP) {
        println!("%prep section:\n{prep}");
    }
}
