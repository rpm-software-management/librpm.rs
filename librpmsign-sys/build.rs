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

//! bindgen configuration for librpmsign-sys

use bindgen::Builder;
use std::{env, path::PathBuf};

/// Bind to librpmsign.so
fn main() {
    println!("cargo:rustc-link-lib=rpmsign");

    let builder = Builder::default()
        .header("include/librpmsign.hpp")
        // rpmsign.h
        .allowlist_function("rpmPkgSign")
        .allowlist_function("rpmPkgDelSign")
        .allowlist_function("rpmPkgDelFileSign")
        // rpmsign.h — sign args and flags
        .allowlist_type("rpmSignArgs")
        .allowlist_type("rpmSignFlags_e");

    // Write generated bindings to OUT_DIR (to be included in the crate)
    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("binding.rs");

    builder
        .generate()
        .unwrap()
        .write_to_file(&output_path)
        .unwrap();

    let bindings_src = std::fs::read_to_string(&output_path).unwrap();
    for line in bindings_src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub const ") {
            let Some((name, _)) = rest.split_once(':') else {
                continue;
            };
            if let Some(flag) = name.strip_prefix("rpmSignFlags_e_RPMSIGN_FLAG_") {
                println!("cargo:rpmsignflag_{}=1", flag.to_lowercase());
            } else if let Some(algo) = name.strip_prefix("pgpHashAlgo_e_PGPHASHALGO_") {
                println!("cargo:pgphashalgo_{}=1", algo.to_lowercase());
            }
        } else if let Some(rest) = trimmed.strip_prefix("pub fn ") {
            let name = rest.split(&['(', ' '][..]).next().unwrap_or("");
            if let Some(func) = name.strip_prefix("rpmPkg") {
                println!("cargo:rpmpkg_{}=1", func.to_lowercase());
            }
        }
    }
}
