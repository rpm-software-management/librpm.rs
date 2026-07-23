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

//! Tests that librpm::init() handles failure and retry correctly.
//!
//! This is a separate test binary because init() modifies
//! process-global state — it must run in isolation from other tests
//! that call configure().
//!
//! All assertions are in one test because the global state is
//! shared across tests within a binary.

use std::path::Path;

#[test]
fn test_config_behavior() {
    // A failed call should not prevent subsequent configuration
    let result = librpm::init_with(Some(Path::new("/nonexistent/rpmrc")), None);
    assert!(result.is_err(), "should fail for nonexistent path");

    // Db::open() should fail before configuration
    let result = librpm::Db::open();
    assert!(result.is_err(), "Db::open() should fail before config");

    // Retry with default config should succeed
    let result = librpm::init();
    assert!(
        result.is_ok(),
        "should succeed after prior failure, not get 'already configured': {result:?}"
    );

    // Db::open() should succeed after configuration
    let db = librpm::Db::open().expect("Db::open() should succeed after config");
    let results: Vec<librpm::PackageHeader> = db
        .find(librpm::Index::Name, "nonexistent-pkg-xyz")
        .collect();
    assert_eq!(results.len(), 0);

    // But a second successful configure should be rejected
    let result = librpm::init();
    assert!(result.is_err(), "double configure should fail");
}
