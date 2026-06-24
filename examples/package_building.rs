//! Build binary and source RPM packages from a spec file.
//!
//! Run with: cargo run --example package_building --features build

use std::path::Path;

use librpm::build::{BuildArgs, Spec, SpecFlags};
use librpm::macro_context::MacroContext;

fn main() {
    librpm::init().expect("failed to initialize librpm");

    let testdata = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
    let spec_path = testdata.join("rpm-basic.spec");
    let build_dir = tempfile::tempdir().expect("failed to create temp dir");
    let topdir = build_dir.path();

    let macros = MacroContext::default();
    macros
        .define(&format!("_sourcedir {}", testdata.display()), 0)
        .unwrap();
    macros
        .define(&format!("_topdir {}", topdir.display()), 0)
        .unwrap();

    std::fs::create_dir_all(topdir.join("BUILD")).unwrap();
    std::fs::create_dir_all(topdir.join("RPMS")).unwrap();
    std::fs::create_dir_all(topdir.join("SRPMS")).unwrap();

    let args = BuildArgs::new();
    let mut spec = Spec::parse_for_build(spec_path.to_str().unwrap(), SpecFlags::NONE, &args, None)
        .expect("failed to parse spec");

    print!("Building packages... ");
    match spec.build(&args) {
        Ok(()) => {
            println!("ok");
            println!("\nOutput packages:");
            for entry in walkdir(topdir) {
                if entry.to_string_lossy().ends_with(".rpm") {
                    println!("  {}", entry.display());
                }
            }
        }
        Err(e) => {
            eprintln!("failed: {e}");
            std::process::exit(1);
        }
    }
}

fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(walkdir(&path));
            } else {
                results.push(path);
            }
        }
    }
    results
}
