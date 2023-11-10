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

use librpm::config::set_db_path;
use librpm::db::installed_packages;
use librpm::{config, Package};
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use std::time::SystemTime;

static CONFIGURE: Once = Once::new();

// Read the default config
fn configure() {
    CONFIGURE.call_once(|| {
        config::read_file(None).unwrap();
    });
}

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
        .arg("'%{NAME}~%{VERSION}~%{RELEASE}~%{SUMMARY}\n'")
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
    configure();

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
        assert_eq!(expected.summary, found.summary());
    }
}

fn get_assets_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

#[test]
fn test_centos_7_rpm_database() {
    configure();
    set_db_path(&get_assets_path().join("centos7")).unwrap();

    let mut packages: Vec<Package> = installed_packages().collect();
    packages.sort_by_key(|p| p.name().to_string());

    assert_eq!(packages.len(), 148);
    let sample_package = &packages[0];
    assert_eq!(sample_package.name(), "acl");
    assert_eq!(sample_package.epoch(), None);
    assert_eq!(sample_package.version(), "2.2.51");
    assert_eq!(sample_package.release(), "15.el7");
    assert_eq!(sample_package.arch(), Some("x86_64"));
    assert_eq!(sample_package.license(), "GPLv2+");
    assert_eq!(sample_package.summary(), "Access control list utilities");
    assert_eq!(
        sample_package.description(),
        "This package contains the getfacl and setfacl utilities needed for\nmanipulating access control lists."
    );
}
