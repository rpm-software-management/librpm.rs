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

//! bindgen configuration for librpm-sys
//!
//! For more on using librpm, see "Chapter 15. Programming RPM with C" from the
//! Fedora RPM Guide (Draft 0.1):
//!
//! https://docs.fedoraproject.org/en-US/Fedora_Draft_Documentation/0.1/html/RPM_Guide/ch-programming-c.html

use bindgen::Builder;
use std::{env, path::PathBuf};

/// Bind to librpm.so + librpmio.so
fn main() {
    // Link with librpm.so + librpmio.so
    //
    // See "Table 16-3: Required rpm libraries" from the "Compiling and Linking
    // RPM Programs" section of "Programming RPM with C" (see above).
    //
    // We don't yet link against librpmbuild.so or librpmsign.so because bindgen
    // is having trouble generating bindings for these libraries. See
    // `librpm.hpp` for more information.
    println!("cargo:rustc-link-lib=rpm");
    println!("cargo:rustc-link-lib=rpmio");

    // TODO: whitelist types and functions we actually use
    let builder = Builder::default()
        .header("include/librpm.hpp")
        .blocklist_type("timex")
        .blocklist_function("clock_adjtime");

    // Write generated bindings to OUT_DIR (to be included in the crate)
    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("binding.rs");

    builder
        .generate()
        .unwrap()
        .write_to_file(&output_path)
        .unwrap();

    let bindings_src = std::fs::read_to_string(&output_path).unwrap();
    for line in bindings_src.lines() {
        let Some(rest) = line.strip_prefix("pub const ") else {
            continue;
        };
        let Some((name, _)) = rest.split_once(':') else {
            continue;
        };
        if let Some(tag) = name.strip_prefix("rpmTag_e_RPMTAG_") {
            println!("cargo:rpmtag_{}=1", tag.to_lowercase());
        } else if let Some(tag) = name.strip_prefix("rpmSigTag_e_RPMSIGTAG_") {
            println!("cargo:rpmsigtag_{}=1", tag.to_lowercase());
        }
    }
}
