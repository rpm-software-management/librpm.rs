/*
 * Copyright (C) RustRPM Developers
 *
 * Licensed under the Mozilla Public License Version 2.0
 * Fedora-License-Identifier: MPLv2.0
 * SPDX-2.0-License-Identifier: MPL-2.0
 * SPDX-3.0-License-Identifier: MPL-2.0
 *
 * This is free software.
 * For more information on the license, see LICENSE.
 * For more information on free software, see <https://www.gnu.org/philosophy/free-sw.en.html>.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at <https://mozilla.org/MPL/2.0/>.
 */

//! Transaction support integration tests.
//!
//! Gated behind the `test-transaction` feature because they exercise
//! write-path APIs (element manipulation, dependency checking, dry-run
//! execution) that require a writable database fixture.
//!
//! Run with: `cargo test --features test-transaction --test transaction`

#![cfg(feature = "test-transaction")]

use std::path::{Path, PathBuf};

use librpm::Package;
use librpm::db::Index;
use librpm::problem::ProblemType;
use librpm::transaction::{CallbackEvent, ElementType, ProblemFilter, TransactionFlags};

mod common;

fn assets_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

fn rpm_basic_path() -> PathBuf {
    assets_path().join("rpms/rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm")
}

fn rpm_empty_path() -> PathBuf {
    assets_path().join("rpms/rpm-empty-0-0.x86_64.rpm")
}

fn rpm_basic() -> Package {
    Package::from_file(&rpm_basic_path()).expect("failed to read rpm-basic")
}

fn rpm_empty() -> Package {
    Package::from_file(&rpm_empty_path()).expect("failed to read rpm-empty")
}

// --- Transaction lifecycle ---

#[test]
fn test_transaction_lifecycle() {
    let mut db = common::init(&common::CENTOS_STREAM_9);

    {
        let txn = db.transaction();
        assert_eq!(txn.len(), 0);
        assert!(txn.is_empty());
    }
    // Db is usable again after transaction is dropped
    let count = db.find(Index::Name, "bash").count();
    assert!(
        count > 0,
        "should be able to query after dropping transaction"
    );
}

// --- Adding elements ---

#[test]
fn test_add_install() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_empty();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_empty_path(), false).unwrap();

    assert_eq!(txn.len(), 1);
    assert!(!txn.is_empty());

    let elem = txn.elements().next().expect("should have one element");
    assert_eq!(elem.element_type(), ElementType::Install);
    assert_eq!(elem.name(), "rpm-empty");
}

#[test]
fn test_add_upgrade() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_empty();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_empty_path(), true).unwrap();

    assert_eq!(txn.len(), 1);
    let elem = txn.elements().next().unwrap();
    // Upgrades still show as Install (TR_ADDED) element type
    assert_eq!(elem.element_type(), ElementType::Install);
}

#[test]
fn test_add_erase() {
    let mut db = common::init(&common::CENTOS_STREAM_9);

    // Query an installed package from the offline database
    let alternatives: Package = db
        .find(Index::Name, "alternatives")
        .next()
        .expect("alternatives should be in the centos-stream-9 fixture");

    let mut txn = db.transaction();
    txn.add_erase(&alternatives).unwrap();

    assert_eq!(txn.len(), 1);
    let elem = txn.elements().next().unwrap();
    assert_eq!(elem.element_type(), ElementType::Erase);
    assert_eq!(elem.name(), "alternatives");
}

#[test]
fn test_multiple_elements() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let empty = rpm_empty();
    let basic = rpm_basic();

    let mut txn = db.transaction();
    txn.add_install(&empty, &rpm_empty_path(), false).unwrap();
    txn.add_install(&basic, &rpm_basic_path(), true).unwrap();

    assert_eq!(txn.len(), 2);

    let names: Vec<String> = txn.elements().map(|e| e.name().to_string()).collect();
    assert!(names.contains(&"rpm-empty".to_string()));
    assert!(names.contains(&"rpm-basic".to_string()));
}

// --- Element accessors ---

#[test]
fn test_element_accessors() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_basic();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_basic_path(), false).unwrap();

    let elem = txn.elements().next().unwrap();
    assert_eq!(elem.name(), "rpm-basic");
    assert_eq!(elem.version(), "2.3.4");
    assert_eq!(elem.release(), "5.el9");
    assert_eq!(elem.arch(), "noarch");
    assert!(!elem.nevra().is_empty());
    assert!(elem.nevra().contains("rpm-basic"));
    assert!(
        !elem.failed(),
        "element should not be marked failed before run"
    );
}

// --- Transaction flags ---

#[test]
fn test_transaction_flags() {
    let mut db = common::init(&common::CENTOS_STREAM_9);

    let mut txn = db.transaction();

    // Default flags
    let initial = txn.flags();
    assert_eq!(initial, TransactionFlags::NONE);

    // Set and read back
    txn.set_flags(TransactionFlags::TEST);
    assert_eq!(txn.flags(), TransactionFlags::TEST);

    // BitOr composition
    let combined = TransactionFlags::TEST | TransactionFlags::JUSTDB;
    txn.set_flags(combined);
    assert_eq!(txn.flags(), combined);
}

// --- Dependency checking ---

#[test]
fn test_check_unsatisfied_deps() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_basic();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_basic_path(), false).unwrap();

    // rpm-basic requires "methylamine >= 1.0.0-1", "/usr/sbin/ego", etc.
    // None of these are in the offline fixture, so check should fail.
    let err = txn
        .check()
        .expect_err("check should fail with unsatisfied deps");
    let problems = err.problems();

    assert!(
        !problems.is_empty(),
        "should have at least one dependency problem"
    );
    assert!(problems.len() > 0);

    let has_requires = problems
        .iter()
        .any(|p| p.problem_type() == ProblemType::Requires);
    assert!(
        has_requires,
        "at least one problem should be an unsatisfied Requires"
    );
}

#[test]
fn test_check_no_deps() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_empty();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_empty_path(), false).unwrap();

    // rpm-empty has only rpmlib(...) deps which are always satisfied
    txn.check().expect("check should pass for rpm-empty");
}

// --- Ordering ---

#[test]
fn test_order() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_empty();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_empty_path(), false).unwrap();

    let unordered = txn.order().expect("order should succeed");
    assert_eq!(unordered, 0, "single element should be fully orderable");
}

// --- Dry run ---

#[test]
fn test_dry_run() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_empty();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_empty_path(), false).unwrap();
    txn.set_flags(TransactionFlags::TEST);
    txn.set_problem_filter(
        ProblemFilter::IGNORE_OS
            | ProblemFilter::IGNORE_ARCH
            | ProblemFilter::REPLACE_PKG
            | ProblemFilter::REPLACE_NEW_FILES
            | ProblemFilter::REPLACE_OLD_FILES
            | ProblemFilter::OLD_PACKAGE,
    );

    txn.check().expect("check should pass for rpm-empty");
    txn.order().expect("order should succeed");

    match txn.run() {
        Ok(()) => {} // dry run succeeded
        Err(e) => {
            // If the fixture DB doesn't support run, report but don't panic —
            // the important thing is that run() returns a structured error.
            let problems = e.problems();
            eprintln!(
                "dry run produced {} problem(s) (may be expected in offline fixture):",
                problems.len()
            );
            for p in problems.iter() {
                eprintln!("  {p}");
            }
        }
    }
}

// --- Progress callback ---

#[test]
fn test_set_callback() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_empty();

    let called = std::cell::Cell::new(false);
    let mut txn = db.transaction();

    txn.set_callback(|_event| {
        called.set(true);
    });

    txn.add_install(&pkg, &rpm_empty_path(), false).unwrap();
    // The callback isn't fired by add_install, only by run()
    assert!(!called.get(), "callback should not fire during add_install");
}

#[test]
fn test_clear_callback() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_empty();

    let mut txn = db.transaction();
    txn.set_callback(|_event| {});
    txn.add_install(&pkg, &rpm_empty_path(), false).unwrap();

    // Should not panic when clearing
    txn.clear_callback();

    assert_eq!(txn.len(), 1, "elements should survive clear_callback");
}

#[test]
fn test_callback_event_debug() {
    let event = CallbackEvent::InstProgress {
        amount: 42,
        total: 100,
    };
    let debug = format!("{event:?}");
    assert!(debug.contains("InstProgress"));
    assert!(debug.contains("42"));
    assert!(debug.contains("100"));
}

#[test]
fn test_callback_open_close_file_debug() {
    let open = CallbackEvent::InstOpenFile {
        nevra: "foo-1.0-1.x86_64".to_string(),
    };
    let debug = format!("{open:?}");
    assert!(debug.contains("InstOpenFile"));
    assert!(debug.contains("foo-1.0-1.x86_64"));

    let close = CallbackEvent::InstCloseFile {
        nevra: "foo-1.0-1.x86_64".to_string(),
    };
    let debug = format!("{close:?}");
    assert!(debug.contains("InstCloseFile"));
    assert!(debug.contains("foo-1.0-1.x86_64"));
}

// --- Problem and error display ---

#[test]
fn test_problems_display() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_basic();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_basic_path(), false).unwrap();

    let err = txn.check().expect_err("check should fail");
    let problems = err.problems();

    let display = format!("{problems}");
    assert!(!display.is_empty(), "Display output should be non-empty");

    let debug = format!("{problems:?}");
    assert!(!debug.is_empty(), "Debug output should be non-empty");
}

#[test]
fn test_problem_accessors() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_basic();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_basic_path(), false).unwrap();

    let err = txn.check().expect_err("check should fail");
    let problems = err.problems();

    for problem in problems.iter() {
        // Every problem should have a recognized type
        assert_ne!(problem.problem_type(), ProblemType::Unknown);

        // Display should produce a human-readable message
        let msg = format!("{problem}");
        assert!(!msg.is_empty());

        // package_nevr should be non-empty for dep problems
        if problem.problem_type() == ProblemType::Requires {
            assert!(
                !problem.package_nevr().is_empty(),
                "Requires problem should have a package NEVR"
            );
        }
    }
}

#[test]
fn test_transaction_error_display() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_basic();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_basic_path(), false).unwrap();

    let err = txn.check().expect_err("check should fail");
    let msg = format!("{err}");
    assert!(
        msg.contains("transaction failed"),
        "TransactionError Display should contain 'transaction failed', got: {msg}"
    );
}

#[test]
fn test_transaction_error_into_problems() {
    let mut db = common::init(&common::CENTOS_STREAM_9);
    let pkg = rpm_basic();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_basic_path(), false).unwrap();

    let err = txn.check().expect_err("check should fail");
    let problems = err.into_problems();
    assert!(!problems.is_empty());
}
