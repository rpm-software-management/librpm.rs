//! Tests for the signing module (requires the `sign` feature).
//!
//! These tests only verify the API surface compiles and that error paths
//! work correctly. Actually signing requires GPG keys, so we test that
//! signing a nonexistent file returns an error rather than panicking.

#![cfg(feature = "sign")]

use librpm::sign::{self, HashAlgo, SignArgs, SignError, SignFlags};

mod common;

// SignFlags

#[test]
fn test_sign_flags_default() {
    assert_eq!(SignFlags::default(), SignFlags::NONE);
}

// SignArgs builder

#[test]
fn test_sign_args_default() {
    let _args = SignArgs::default();
}

#[test]
fn test_sign_args_builder() {
    let _args = SignArgs::new()
        .key_id("DEADBEEF")
        .hash_algo(HashAlgo::SHA256)
        .flags(SignFlags::IMA);
}

// Error paths — operations on nonexistent files

#[test]
fn test_sign_nonexistent_file() {
    common::configure();
    let result = sign::sign_package("/nonexistent/package.rpm", None);
    assert_eq!(result, Err(SignError));
}

#[test]
fn test_del_sign_nonexistent_file() {
    common::configure();
    let result = sign::del_sign("/nonexistent/package.rpm", None);
    assert_eq!(result, Err(SignError));
}

#[test]
#[cfg(has_rpmpkg_delfilesign)]
fn test_del_file_sign_nonexistent_file() {
    common::configure();
    let result = sign::del_file_sign("/nonexistent/package.rpm", None);
    assert_eq!(result, Err(SignError));
}

// SignError Display

#[test]
fn test_sign_error_display() {
    let err = SignError;
    assert_eq!(format!("{err}"), "RPM signing operation failed");
}

// HashAlgo constants exist

#[test]
fn test_hash_algo_constants() {
    assert_ne!(HashAlgo::SHA256, HashAlgo::SHA512);
    assert_ne!(HashAlgo::SHA256, HashAlgo::SHA1);
}
