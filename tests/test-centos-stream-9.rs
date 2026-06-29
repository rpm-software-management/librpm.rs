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

#[test]
fn test_centos_stream_9_find_by_providename() {
    common::assert_find_by_providename(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_find_by_requirename() {
    common::assert_find_by_requirename(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_find_by_dirnames() {
    common::assert_find_by_dirnames(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_find_re_glob() {
    common::assert_find_re_glob(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_find_re_regex() {
    common::assert_find_re_regex(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_find_re_no_match() {
    common::assert_find_re_no_match(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_iter_match_count() {
    common::assert_iter_match_count(&common::CENTOS_STREAM_9);
}

#[test]
fn test_centos_stream_9_iter_offset() {
    common::assert_iter_offset(&common::CENTOS_STREAM_9);
}
