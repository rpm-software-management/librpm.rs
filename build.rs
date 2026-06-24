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

// Forward librpm/librpmbuild/librpmsign constant availability as cfg flags.
//
// Each -sys crate scans its bindgen output and emits Cargo metadata for every
// relevant constant found in the system headers.  Cargo surfaces these as
// DEP_<LINKS>_<KEY> environment variables here.  We re-export each one as a
// rustc-cfg so that enum variants can be gated with e.g.
// #[cfg(has_rpmtag_payloadsha3_256)] or #[cfg(has_rpmsignflag_rpmv6)],
// allowing the crate to build against older librpm versions that lack newer
// constants.
fn main() {
    for (key, _) in std::env::vars() {
        let name = if let Some(n) = key.strip_prefix("DEP_RPMBUILD_") {
            n
        } else if let Some(n) = key.strip_prefix("DEP_RPMSIGN_") {
            n
        } else if let Some(n) = key.strip_prefix("DEP_RPM_") {
            n
        } else {
            continue;
        };
        let cfg = format!("has_{}", name.to_lowercase());
        println!("cargo:rustc-check-cfg=cfg({cfg})");
        println!("cargo:rustc-cfg={cfg}");
    }
}
