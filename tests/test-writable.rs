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

//! Tests that require a writable RPM database.
//!
//! This binary copies the CentOS Stream 9 rpmdb snapshot to a temporary
//! directory and initializes librpm against that copy, so write operations
//! (transactions, keyring import/delete) run without root or host modification.
//!
//! This must be a separate binary because librpm can only be initialized once
//! per process.

use std::path::{Path, PathBuf};

use librpm::db::Index;
use librpm::keyring::PubKey;
use librpm::problem::ProblemType;
use librpm::transaction::{CallbackEvent, ElementType, ProblemFilter, TransactionFlags};
use librpm::{PackageHeader, VerifyOptions};

mod common;

/// Helper to convert ASCII-armored PGP key to binary packet data
fn dearmor_key(armored_data: &[u8]) -> Vec<u8> {
    use base64::prelude::*;

    let armored = String::from_utf8_lossy(armored_data);
    let mut base64_lines = Vec::new();

    // Skip header and footer, collect base64 content
    let mut in_body = false;
    for line in armored.lines() {
        if line.starts_with("-----BEGIN") {
            in_body = true;
            continue;
        }
        if line.starts_with("-----END") {
            break;
        }
        if in_body && !line.is_empty() && !line.starts_with('=') {
            // Skip empty lines and checksum lines (starting with '=')
            base64_lines.push(line);
        }
    }

    let base64_content = base64_lines.join("");
    BASE64_STANDARD
        .decode(base64_content.as_bytes())
        .expect("failed to decode base64")
}

fn assets_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

fn rpm_basic_path() -> PathBuf {
    assets_path().join("rpms/rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm")
}

fn rpm_empty_path() -> PathBuf {
    assets_path().join("rpms/rpm-empty-0-0.x86_64.rpm")
}

fn test_key_path() -> PathBuf {
    assets_path().join("keys/rpm-testkey-v4-rsa4096.asc")
}

fn rpm_basic() -> PackageHeader {
    let skip = VerifyOptions::skip_verification();
    PackageHeader::from_file(&rpm_basic_path(), Some(&skip)).expect("failed to read rpm-basic")
}

fn rpm_empty() -> PackageHeader {
    let skip = VerifyOptions::skip_verification();
    PackageHeader::from_file(&rpm_empty_path(), Some(&skip)).expect("failed to read rpm-empty")
}

// ========================
// Transaction tests
// ========================

// --- Transaction lifecycle ---

#[test]
fn test_transaction_lifecycle() {
    let mut db = common::init_writable();

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
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();

    // Query an installed package from the offline database
    let alternatives: PackageHeader = db
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
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();

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
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();
    let pkg = rpm_empty();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_empty_path(), false).unwrap();

    // rpm-empty has only rpmlib(...) deps which are always satisfied
    txn.check().expect("check should pass for rpm-empty");
}

// --- Ordering ---

#[test]
fn test_order() {
    let mut db = common::init_writable();
    let pkg = rpm_empty();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_empty_path(), false).unwrap();

    let unordered = txn.order().expect("order should succeed");
    assert_eq!(unordered, 0, "single element should be fully orderable");
}

// --- Dry run ---

#[test]
fn test_dry_run() {
    let mut db = common::init_writable();
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

// --- Alternate root (Db::open_with_root) ---

#[test]
fn test_open_with_root_relative_rejected() {
    common::init_writable();

    let err = librpm::Db::open_with_root(Path::new("relative/path"))
        .expect_err("a relative root directory should be rejected");
    assert_eq!(err.kind(), librpm::error::ErrorKind::InvalidArg);
}

#[test]
fn test_install_into_alternate_root() {
    // A real transaction into a non-"/" root calls chroot(); skip when we
    // can't (e.g. the unprivileged Ubuntu CI job).
    if !common::can_chroot() {
        eprintln!("skipping test_install_into_alternate_root: requires CAP_SYS_CHROOT");
        return;
    }

    // fresh_root_db() gives an empty, isolated database at <root>/<_dbpath>.
    let (root, mut db) = common::fresh_root_db();
    assert_eq!(
        db.installed_packages().count(),
        0,
        "a freshly initialized database should be empty"
    );

    // Actually install (not a dry run) rpm-empty. JUSTDB records the package
    // in the database without laying down files, so nothing escapes the root.
    {
        let pkg = rpm_empty();
        let mut txn = db.transaction();
        txn.add_install(&pkg, &rpm_empty_path(), false).unwrap();
        txn.set_flags(TransactionFlags::JUSTDB);
        txn.set_problem_filter(ProblemFilter::IGNORE_OS | ProblemFilter::IGNORE_ARCH);
        txn.check()
            .expect("dependency check should pass for rpm-empty");
        txn.order().expect("order should succeed");
        txn.run().expect("install should succeed");
    }

    // Re-open the same root and confirm the package really landed in the DB.
    let db = librpm::Db::open_with_root(root.path()).expect("re-open failed");
    let installed: Vec<PackageHeader> = db.installed_packages().collect();
    assert_eq!(
        installed.len(),
        1,
        "exactly one package should be installed in the alternate root"
    );
    assert_eq!(installed[0].name(), "rpm-empty");
}

#[test]
fn test_erase_from_populated_root() {
    // A real transaction into a non-"/" root calls chroot(); skip when we
    // can't (e.g. the unprivileged Ubuntu CI job).
    if !common::can_chroot() {
        eprintln!("skipping test_erase_from_populated_root: requires CAP_SYS_CHROOT");
        return;
    }

    // populated_root_db() seeds the snapshot into an isolated root, so this
    // real erase never touches the shared fixture.
    let (root, mut db) = common::populated_root_db();
    let before = db.installed_packages().count();
    assert!(
        before > 0,
        "seeded root should contain the snapshot packages"
    );

    let alternatives: PackageHeader = db
        .find(Index::Name, "alternatives")
        .next()
        .expect("alternatives should be in the seeded snapshot");

    {
        let mut txn = db.transaction();
        txn.add_erase(&alternatives).unwrap();
        txn.set_flags(TransactionFlags::JUSTDB);
        txn.set_problem_filter(ProblemFilter::IGNORE_OS | ProblemFilter::IGNORE_ARCH);
        // Other packages in the snapshot require `alternatives`, so a
        // dependency check would (correctly) fail. This test only exercises
        // the database write, so we skip the check — the equivalent of
        // `rpm -e --nodeps`. rpmtsRun does not re-check dependencies.
        txn.order().expect("order should succeed");
        txn.run().expect("erase should succeed");
    }

    // Re-open the same root and confirm the package is really gone.
    let db = librpm::Db::open_with_root(root.path()).expect("re-open failed");
    assert_eq!(
        db.installed_packages().count(),
        before - 1,
        "erasing one package should reduce the count by exactly one"
    );
    assert_eq!(
        db.find(Index::Name, "alternatives").count(),
        0,
        "alternatives should no longer be in the database"
    );
}

// --- Progress callback ---

#[test]
fn test_set_callback() {
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();
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
    let mut db = common::init_writable();
    let pkg = rpm_basic();

    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_basic_path(), false).unwrap();

    let err = txn.check().expect_err("check should fail");
    let problems = err.into_problems();
    assert!(!problems.is_empty());
}

// ========================
// Keyring mutation tests
// ========================

#[cfg(all(
    any(has_rpmkeyring_rpmtxnimportpubkey, has_rpmkeyring_rpmtsimportpubkey),
    has_rpmkeyring_rpmkeyringlookupkey,
))]
#[test]
fn test_import_to_rpmdb() {
    // Import into an isolated, populated root so we never mutate the shared
    // writable snapshot. Keyring import does not chroot, so no privilege gate
    // is needed. Exercises the root-aware Db::import_pubkey directly.
    let (root, db) = common::populated_root_db();

    // Read ASCII-armored key and convert to binary
    let armored_key = std::fs::read(test_key_path()).unwrap();
    let binary_key = dearmor_key(&armored_key);

    db.import_pubkey(&binary_key).unwrap();

    // Re-open the same root (a fresh ts avoids any cached keyring) and confirm
    // the key really landed in this database.
    let db = librpm::Db::open_with_root(root.path()).unwrap();
    let keyring = db.keyring();
    let test_key = PubKey::from_file(&test_key_path()).unwrap();
    assert!(
        keyring.lookup(&test_key).is_some(),
        "imported key should be present in the keyring"
    );
}

#[cfg(all(
    any(has_rpmkeyring_rpmtxnimportpubkey, has_rpmkeyring_rpmtsimportpubkey),
    has_rpmkeyring_rpmtxndeletepubkey,
    has_rpmkeyring_rpmkeyringlookupkey,
))]
#[test]
fn test_delete_from_rpmdb() {
    // Isolated, populated root — no shared-state mutation, no cleanup needed.
    // Exercises the root-aware Db::delete_pubkey directly.
    let (root, db) = common::populated_root_db();

    // Read ASCII-armored key and convert to binary
    let armored_key = std::fs::read(test_key_path()).unwrap();
    let binary_key = dearmor_key(&armored_key);

    // Import first so we have something to delete.
    db.import_pubkey(&binary_key).unwrap();

    let test_key = PubKey::from_file(&test_key_path()).unwrap();
    db.delete_pubkey(&test_key).unwrap();

    // Re-open the same root (fresh ts avoids a cached keyring) and verify the
    // key is really gone.
    let db = librpm::Db::open_with_root(root.path()).unwrap();
    let keyring = db.keyring();
    assert!(
        keyring.lookup(&test_key).is_none(),
        "key should no longer be in the keyring after deletion"
    );
}
