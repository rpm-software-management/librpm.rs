//! Tests for package building (requires the `build` feature).
//!
//! These tests modify global RPM macro state (_topdir, _sourcedir) and
//! must run in their own binary to avoid races with other spec-parsing tests.

#![cfg(feature = "build")]

use std::sync::Mutex;

use librpm::build::{BuildArgs, BuildFlags, Spec, SpecFlags};
use librpm::macro_context::MacroContext;

mod common;

// RPM's macro context is process-global. Tests that set _topdir/_sourcedir
// must not run concurrently or they overwrite each other's values.
static BUILD_LOCK: Mutex<()> = Mutex::new(());

fn testdata() -> std::path::PathBuf {
    common::get_assets_path()
}

#[test]
fn test_build_nobuild() {
    let _guard = BUILD_LOCK.lock().unwrap();
    common::configure();
    let build_dir = tempfile::tempdir().expect("failed to create temp dir");
    let macros = MacroContext::default();
    macros
        .define(&format!("_topdir {}", build_dir.path().display()), 0)
        .unwrap();

    let spec_file = testdata().join("test.spec");
    let args = BuildArgs::from_flags(BuildFlags::NOBUILD);
    let mut spec = Spec::parse_for_build(spec_file.to_str().unwrap(), SpecFlags::NONE, &args, None)
        .expect("failed to parse spec");
    spec.build(&args).expect("NOBUILD dry-run failed");
}

#[test]
fn test_build_packages() {
    let _guard = BUILD_LOCK.lock().unwrap();
    common::configure();
    let testdata = testdata();
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

    let spec_file = testdata.join("rpm-basic.spec");
    let args = BuildArgs::new();
    let mut spec = Spec::parse_for_build(spec_file.to_str().unwrap(), SpecFlags::NONE, &args, None)
        .expect("failed to parse rpm-basic.spec");

    spec.build(&args).expect("build failed");

    let has_binary = walkdir(topdir)
        .iter()
        .any(|p| p.to_string_lossy().contains("/RPMS/") && p.to_string_lossy().ends_with(".rpm"));
    let has_source = walkdir(topdir)
        .iter()
        .any(|p| p.to_string_lossy().contains("/SRPMS/") && p.to_string_lossy().ends_with(".rpm"));
    assert!(has_binary, "no binary RPM found in {}", topdir.display());
    assert!(has_source, "no source RPM found in {}", topdir.display());
}

fn walkdir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
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
