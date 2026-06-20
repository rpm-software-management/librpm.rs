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

//! bindgen configuration for librpmbuild-sys

use bindgen::Builder;
use std::{env, path::PathBuf};

/// Bind to librpmbuild.so
fn main() {
    println!("cargo:rustc-link-lib=rpmbuild");

    // No functions are consumed from this crate yet.
    // Uncomment as safe wrappers are added.
    let builder = Builder::default()
        .header("include/librpmbuild.hpp")
        // rpmbuild.h
        // .allowlist_function("rpmSpecParse")
        // .allowlist_function("rpmSpecBuild")
        // .allowlist_function("rpmSpecCheckDeps")
        // .allowlist_function("rpmSpecDS")
        // .allowlist_function("rpmSpecSourceHeader")
        // rpmspec.h (included transitively by rpmbuild.h)
        // .allowlist_function("rpmSpecFree")
        // .allowlist_function("rpmSpecGetSection")
        // .allowlist_function("rpmSpecPkgGetSection")
        // .allowlist_function("rpmSpecPkgHeader")
        // .allowlist_function("rpmSpecPkgIterInit")
        // .allowlist_function("rpmSpecPkgIterNext")
        // .allowlist_function("rpmSpecPkgIterFree")
        // .allowlist_function("rpmspecQuery")
        // .allowlist_function("rpmSpecSrcFilename")
        // .allowlist_function("rpmSpecSrcFlags")
        // .allowlist_function("rpmSpecSrcIterInit")
        // .allowlist_function("rpmSpecSrcIterNext")
        // .allowlist_function("rpmSpecSrcIterFree")
        // .allowlist_function("rpmSpecSrcNum")
        // rpmbuild.h — build flags
        // .allowlist_type("rpmBuildFlags_e")
        // .allowlist_type("rpmBuildPkgFlags_e")
        // rpmspec.h — source/spec flags
        // .allowlist_type("rpmSourceFlags_e")
        // .allowlist_type("rpmSpecFlags_e")
        .allowlist_function("__librpmbuild_sys_placeholder__");

    // Write generated bindings to OUT_DIR (to be included in the crate)
    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("binding.rs");

    builder
        .generate()
        .unwrap()
        .write_to_file(output_path)
        .unwrap();
}
