//! Validate package iteration against an offline Fedora 44 database snapshot.
//!
//! This must be a separate binary because librpm caches the database connection
//! on the global transaction set, so each test database needs its own process.

mod common;

#[test]
fn test_fedora_44_rpm_database() {
    common::assert_distro(&common::FEDORA_44);
}

#[test]
fn test_fedora_44_find_by_name() {
    common::assert_find_by_name(&common::FEDORA_44);
}

#[test]
fn test_fedora_44_find_nonexistent() {
    common::assert_find_nonexistent(&common::FEDORA_44);
}

#[test]
fn test_fedora_44_buildtimes() {
    common::assert_buildtimes_valid(&common::FEDORA_44);
}
