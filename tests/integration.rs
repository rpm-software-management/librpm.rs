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
use librpm::{config, package, Index, Package};
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

fn fetch_package_info(package_name: &str, query_param: &str) -> Option<String> {
    let rpm_info = Command::new("rpm")
        .arg("-q")
        .arg("rpm-devel")
        .arg(format!("--queryformat=%{{{}}}", query_param))
        .output()
        .unwrap()
        .stdout;

    let text = String::from_utf8(rpm_info).unwrap();
    Some(text).filter(|c| c != "(none)" && c != "")
}

#[test]
fn system_installed_find_db_test() {
    configure();

    let PACKAGE_NAME = "rpm-devel";
    let PACKAGE_NEVRA = fetch_package_info(PACKAGE_NAME, "NEVRA").unwrap();

    let mut matches = Index::Name.find(PACKAGE_NAME);

    if let Some(package) = matches.next() {
        assert_eq!(package.name(), "rpm-devel");
        assert_eq!(
            package.epoch(),
            fetch_package_info(PACKAGE_NAME, "EPOCH")
                .unwrap()
                .parse::<i32>()
                .ok()
        );
        assert_eq!(
            package.version(),
            fetch_package_info(PACKAGE_NAME, "VERSION").unwrap()
        );
        assert_eq!(
            package.release(),
            fetch_package_info(PACKAGE_NAME, "RELEASE").unwrap()
        );
        assert_eq!(
            package.summary(),
            fetch_package_info(PACKAGE_NAME, "SUMMARY").unwrap()
        );
        assert_eq!(
            package.license(),
            fetch_package_info(PACKAGE_NAME, "LICENSE").unwrap()
        );

        assert_eq!(package.nevra(), PACKAGE_NEVRA);
        assert_eq!(package.to_string(), PACKAGE_NEVRA);

        assert!(matches.next().is_none(), "expected one result, got more!");
    } else {
        if librpm::db::installed_packages().count() == 0 {
            eprintln!("*** warning: No RPMs installed! Tests skipped!")
        } else {
            panic!("some RPMs installed, but not `rpm-devel`?!");
        }
    }
}

fn get_assets_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
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
