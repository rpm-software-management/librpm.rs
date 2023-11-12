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

//! librpm.rs integration tests

use librpm::db::installed_packages;
use librpm::Package;
use std::process::Command;

mod common;

#[derive(Debug, PartialEq)]
struct PartialPackage {
    name: String,
    version: String,
    release: String,
    summary: String,
}

fn fetch_system_packages() -> Vec<PartialPackage> {
    let rpm_info = Command::new("rpm")
        .arg("-qa")
        .arg("--queryformat")
        .arg("%{NAME}~%{VERSION}~%{RELEASE}~%{SUMMARY}\n")
        .output()
        .unwrap()
        .stdout;

    let text = String::from_utf8(rpm_info).unwrap();
    let mut packages = Vec::new();
    for line in text.lines() {
        let mut parts = line.split('~');
        let name = parts.next().unwrap();
        let version = parts.next().unwrap();
        let release = parts.next().unwrap();
        let summary = parts.next().unwrap();
        packages.push(PartialPackage {
            name: name.to_string(),
            version: version.to_string(),
            release: release.to_string(),
            summary: summary.to_string(),
        });
    }

    packages
}

#[test]
fn test_against_installed_packages() {
    common::configure();

    let mut expected_install_packages = fetch_system_packages();
    let mut found_packages: Vec<Package> = installed_packages().collect();

    expected_install_packages.sort_by_key(|p| p.name.to_string());
    found_packages.sort_by_key(|p| p.name().to_string());

    assert!(
        expected_install_packages.len() > 0,
        "Couldn't find any installed packages using the RPM CLI"
    );
    assert_eq!(expected_install_packages.len(), found_packages.len());

    for (expected, found) in expected_install_packages.iter().zip(found_packages.iter()) {
        assert_eq!(expected.name, found.name());
        assert_eq!(expected.version, found.version());
        assert_eq!(expected.release, found.release());
        assert_eq!(expected.summary, found.summary());
    }
}
