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

/// Find all `pub const {prefix}{NAME} :` patterns in the bindgen output,
/// returning the NAME suffix for each match.
///
/// Bindgen may or may not insert newlines between items depending on the
/// libclang version and formatting settings, so we search the raw text
/// instead of iterating by line.
fn find_consts<'a>(src: &'a str, prefix: &'a str) -> impl Iterator<Item = &'a str> {
    let needle = format!("pub const {prefix}");
    let needle_len = needle.len();
    let mut start = 0;
    std::iter::from_fn(move || {
        let pos = src[start..].find(&needle)?;
        let abs = start + pos + needle_len;
        start = abs;
        let rest = &src[abs..];
        let end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
        Some(&rest[..end])
    })
}

/// Find all `pub fn {prefix}{NAME}(` patterns in the bindgen output,
/// returning the NAME suffix for each match.
fn find_fns<'a>(src: &'a str, prefix: &'a str) -> impl Iterator<Item = &'a str> {
    let needle = format!("pub fn {prefix}");
    let needle_len = needle.len();
    let mut start = 0;
    std::iter::from_fn(move || {
        let pos = src[start..].find(&needle)?;
        let abs = start + pos + needle_len;
        start = abs;
        let rest = &src[abs..];
        let end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
        Some(&rest[..end])
    })
}

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
    for cap in find_consts(&bindings_src, "rpmSignFlags_e_RPMSIGN_FLAG_") {
        println!("cargo:rpmsignflag_{}=1", cap.to_lowercase());
    }
    for cap in find_consts(&bindings_src, "pgpHashAlgo_e_PGPHASHALGO_") {
        println!("cargo:pgphashalgo_{}=1", cap.to_lowercase());
    }
    for cap in find_fns(&bindings_src, "rpmPkg") {
        println!("cargo:rpmpkg_{}=1", cap.to_lowercase());
    }
}
