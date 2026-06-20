//! Validate package iteration against an offline CentOS Stream 9 database snapshot.
//!
//! This must be a separate binary because librpm caches the database connection
//! on the global transaction set, so each test database needs its own process.

mod common;

#[test]
fn test_centos_stream_9_rpm_database() {
    common::assert_distro(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_find_by_name() {
    common::assert_find_by_name(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_find_nonexistent() {
    common::assert_find_nonexistent(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_buildtimes() {
    common::assert_buildtimes_valid(&common::CENTOS_STREAM_9);
}
