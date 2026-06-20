//! Validate package iteration against an offline CentOS Stream 10 database snapshot.
//!
//! This must be a separate binary because librpm caches the database connection
//! on the global transaction set, so each test database needs its own process.

mod common;

#[test]
fn test_centos_stream_10_rpm_database() {
    common::assert_distro(&common::CENTOS_STREAM_10);
}

#[test]
fn test_centos_stream_10_find_by_name() {
    common::assert_find_by_name(&common::CENTOS_STREAM_10);
}

#[test]
fn test_centos_stream_10_find_nonexistent() {
    common::assert_find_nonexistent(&common::CENTOS_STREAM_10);
}

#[test]
fn test_centos_stream_10_buildtimes() {
    common::assert_buildtimes_valid(&common::CENTOS_STREAM_10);
}
