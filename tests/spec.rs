//! Tests for spec file parsing (requires the `build` feature).

#![cfg(feature = "build")]

use librpm::build::{BuildFlags, Section, SourceFlags, Spec, SpecFlags};

mod common;

fn spec_path() -> std::path::PathBuf {
    common::get_assets_path().join("test.spec")
}

fn load_spec() -> Spec {
    common::configure();
    Spec::parse(spec_path().to_str().unwrap(), SpecFlags::FORCE, None)
        .expect("failed to parse test.spec")
}

// Spec::parse

#[test]
fn test_parse_success() {
    let _spec = load_spec();
}

#[test]
fn test_parse_nonexistent_returns_none() {
    common::configure();
    assert!(Spec::parse("/nonexistent/test.spec", SpecFlags::NONE, None).is_none());
}

// Spec::source_header

#[test]
fn test_source_header() {
    let spec = load_spec();
    let hdr = spec.source_header();
    assert_eq!(hdr.name(), "test-package");
    assert_eq!(hdr.version(), "1.0");
}

// Spec::get_section

#[test]
fn test_get_section_prep() {
    let spec = load_spec();
    let section = spec.get_section(Section::PREP);
    assert!(section.is_some());
}

#[test]
fn test_get_section_preprocessed() {
    let spec = load_spec();
    let full = spec.get_section(Section::NONE);
    assert!(full.is_some());
    let text = full.unwrap();
    assert!(text.contains("test-package"));
}

// Spec::packages

#[test]
fn test_packages_iter() {
    let spec = load_spec();
    let pkgs: Vec<_> = spec.packages().collect();
    assert!(!pkgs.is_empty());
    assert_eq!(pkgs[0].name(), "test-package");
}

#[test]
fn test_package_header() {
    let spec = load_spec();
    let pkg = spec.packages().next().expect("at least one package");
    let hdr = pkg.header();
    assert_eq!(hdr.name(), "test-package");
    assert_eq!(hdr.version(), "1.0");
    assert_eq!(hdr.summary(), "A test package for librpm.rs");
}

// Spec::sources

#[test]
fn test_sources_iter() {
    let spec = load_spec();
    let srcs: Vec<_> = spec.sources().collect();
    assert!(srcs.len() >= 2, "expected source + patch");
}

#[test]
fn test_source_entry() {
    let spec = load_spec();
    let srcs: Vec<_> = spec.sources().collect();
    let source = srcs
        .iter()
        .find(|s| s.is_source())
        .expect("no source entry");
    assert_eq!(source.num(), 0);
    assert_eq!(source.filename(), "test-package-1.0.tar.gz");
    assert!(source.flags().contains(SourceFlags::SOURCE));
}

#[test]
fn test_patch_entry() {
    let spec = load_spec();
    let srcs: Vec<_> = spec.sources().collect();
    let patch = srcs.iter().find(|s| s.is_patch()).expect("no patch entry");
    assert_eq!(patch.num(), 0);
    assert_eq!(patch.filename(), "fix-build.patch");
    assert!(patch.flags().contains(SourceFlags::PATCH));
}

// Flag types

#[test]
fn test_spec_flags_bitor() {
    let flags = SpecFlags::ANYARCH | SpecFlags::FORCE;
    assert_ne!(flags, SpecFlags::NONE);
}

#[test]
fn test_build_flags_bitor() {
    let flags = BuildFlags::PREP | BuildFlags::BUILD | BuildFlags::INSTALL;
    assert_ne!(flags, BuildFlags::NONE);
}

// Debug

#[test]
fn test_spec_debug() {
    let spec = load_spec();
    let s = format!("{spec:?}");
    assert!(s.contains("Spec"));
}
