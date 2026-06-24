//! Tests for version parsing, comparison, and the `vercmp` free function.

use std::cmp::Ordering;

use librpm::version::{self, Version};

// vercmp — segmented string comparison

#[test]
fn test_vercmp_comparisons() {
    assert_eq!(version::vercmp("1.0", "1.0"), Ordering::Equal);

    assert_eq!(version::vercmp("1.1", "1.0"), Ordering::Greater);
    assert_eq!(version::vercmp("1.0", "1.1"), Ordering::Less);

    // longer wins
    assert_eq!(version::vercmp("1.0.1", "1.0"), Ordering::Greater);

    // numeric beats alpha
    assert_eq!(version::vercmp("1.1", "1.a"), Ordering::Greater);

    // alpha comparison
    assert_eq!(version::vercmp("1.0.a", "1.0.b"), Ordering::Less);

    // tilde ordering
    assert_eq!(version::vercmp("1.0~rc1", "1.0"), Ordering::Less);
    assert_eq!(version::vercmp("1.0~rc1", "1.0~rc2"), Ordering::Less);

    // caret ordering
    assert_eq!(version::vercmp("1.0^post1", "1.0"), Ordering::Greater);
    assert_eq!(version::vercmp("1.0^post1", "1.0^post2"), Ordering::Less);

    // tilde + caret
    assert_eq!(version::vercmp("1.0~rc1", "1.0^post1"), Ordering::Less);
}

// Version::parse

#[test]
fn test_parse_version_only() {
    let v = Version::parse("2.3.4").unwrap();
    assert_eq!(v.epoch(), None);
    assert_eq!(v.version(), "2.3.4");
    assert_eq!(v.release(), None);
}

#[test]
fn test_parse_version_release() {
    let v = Version::parse("2.3.4-5.el9").unwrap();
    assert_eq!(v.epoch(), None);
    assert_eq!(v.version(), "2.3.4");
    assert_eq!(v.release(), Some("5.el9"));
}

#[test]
fn test_parse_epoch_version_release() {
    let v = Version::parse("1:2.3.4-5.el9").unwrap();
    assert_eq!(v.epoch(), Some("1"));
    assert_eq!(v.version(), "2.3.4");
    assert_eq!(v.release(), Some("5.el9"));
}

#[test]
fn test_parse_evr_string() {
    let v = Version::parse("1:2.3.4-5.el9").unwrap();
    assert_eq!(v.evr(), "1:2.3.4-5.el9");
}

#[test]
fn test_parse_evr_no_epoch() {
    let v = Version::parse("2.3.4-5.el9").unwrap();
    assert_eq!(v.evr(), "2.3.4-5.el9");
}

#[test]
fn test_parse_evr_no_release() {
    let v = Version::parse("2.3.4").unwrap();
    assert_eq!(v.evr(), "2.3.4");
}

// Version::new

#[test]
fn test_new_full() {
    let v = Version::new(Some("1"), "2.3.4", Some("5.el9")).unwrap();
    assert_eq!(v.epoch(), Some("1"));
    assert_eq!(v.version(), "2.3.4");
    assert_eq!(v.release(), Some("5.el9"));
}

#[test]
fn test_new_no_epoch_no_release() {
    let v = Version::new(None, "1.0", None).unwrap();
    assert_eq!(v.epoch(), None);
    assert_eq!(v.version(), "1.0");
    assert_eq!(v.release(), None);
}

// Version comparison (Ord / PartialOrd / Eq)

#[test]
fn test_version_ord_equal() {
    let a = Version::parse("1.0-1").unwrap();
    let b = Version::parse("1.0-1").unwrap();
    assert_eq!(a, b);
    assert_eq!(a.cmp(&b), Ordering::Equal);
}

#[test]
fn test_version_ord_version_newer() {
    let a = Version::parse("1.1-1").unwrap();
    let b = Version::parse("1.0-1").unwrap();
    assert!(a > b);
}

#[test]
fn test_version_ord_epoch_wins() {
    let a = Version::parse("1:1.0-1").unwrap();
    let b = Version::parse("2.0-1").unwrap();
    assert!(a > b);
}

#[test]
fn test_version_ord_release() {
    let a = Version::parse("1.0-2").unwrap();
    let b = Version::parse("1.0-1").unwrap();
    assert!(a > b);
}

// Display / Debug

#[test]
fn test_display() {
    let v = Version::parse("1:2.3.4-5.el9").unwrap();
    assert_eq!(format!("{v}"), "1:2.3.4-5.el9");
}

#[test]
fn test_debug() {
    let v = Version::parse("1:2.3.4-5.el9").unwrap();
    let debug = format!("{v:?}");
    assert!(debug.contains("2.3.4"));
    assert!(debug.contains("5.el9"));
}

// Send + Sync

#[test]
fn test_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<Version>();
    assert_sync::<Version>();
}
