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

    // Write generated bindings to OUT_DIR (to be included in the crate)
    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("binding.rs");

    // TODO: whitelist types and functions we actually use
    Builder::default()
        .header("include/librpmsign.hpp")
        .generate()
        .unwrap()
        .write_to_file(output_path)
        .unwrap();
}
