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

//! RPM version parsing and comparison
//!
//! This module wraps librpm's `rpmver.h` API, providing:
//!
//! - [`vercmp`]: compare two version or release strings using RPM's
//!   segmented comparison algorithm.
//! - [`Version`]: a parsed `[epoch:]version[-release]` with full
//!   `Ord`/`Eq` support.

use std::cmp::Ordering;
use std::ffi::{CStr, CString};
use std::fmt;

unsafe extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

/// Compare two version or release strings using RPM's segmented comparison.
///
/// This is a direct wrapper around `rpmvercmp(3)`. It splits each string
/// into alphabetic and numeric segments and compares them pairwise.
///
/// Returns `Ordering::Greater` if `a` is "newer", `Ordering::Less` if `b`
/// is "newer", or `Ordering::Equal` if they are identical.
///
/// # Panics
///
/// Panics if either string contains an interior NUL byte.
pub fn vercmp(a: &str, b: &str) -> Ordering {
    let a = CString::new(a).expect("version string contains NUL byte");
    let b = CString::new(b).expect("version string contains NUL byte");
    let rc = unsafe { librpm_sys::rpmvercmp(a.as_ptr(), b.as_ptr()) };
    rc.cmp(&0)
}

/// A parsed RPM version: `[epoch:]version[-release]`.
///
/// Wraps librpm's opaque `rpmver` handle. Owns the underlying allocation
/// and frees it on drop.
pub struct Version {
    ptr: librpm_sys::rpmver,
}

// Safety: the rpmver struct is a self-contained heap allocation with no
// references to global state. It is safe to send across threads and to
// share (all accessors are read-only).
unsafe impl Send for Version {}
unsafe impl Sync for Version {}

impl Version {
    /// Parse an `[epoch:]version[-release]` string.
    ///
    /// Returns `None` if the string is not a valid EVR.
    ///
    /// # Panics
    ///
    /// Panics if `evr` contains an interior NUL byte.
    pub fn parse(evr: &str) -> Option<Self> {
        let evr = CString::new(evr).expect("EVR string contains NUL byte");
        let ptr = unsafe { librpm_sys::rpmverParse(evr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(Version { ptr })
        }
    }

    /// Construct a version from individual components.
    ///
    /// `epoch` and `release` are optional. Returns `None` if the inputs
    /// are invalid.
    ///
    /// # Panics
    ///
    /// Panics if any argument contains an interior NUL byte.
    pub fn new(epoch: Option<&str>, version: &str, release: Option<&str>) -> Option<Self> {
        let epoch_c = epoch.map(|e| CString::new(e).expect("epoch contains NUL byte"));
        let version_c = CString::new(version).expect("version contains NUL byte");
        let release_c = release.map(|r| CString::new(r).expect("release contains NUL byte"));

        let e_ptr = epoch_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr());
        let r_ptr = release_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr());

        let ptr = unsafe { librpm_sys::rpmverNew(e_ptr, version_c.as_ptr(), r_ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(Version { ptr })
        }
    }

    /// Epoch string (e.g. `"1"`), or `None` if no epoch was specified.
    pub fn epoch(&self) -> Option<&str> {
        let p = unsafe { librpm_sys::rpmverE(self.ptr) };
        if p.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(p) }
                    .to_str()
                    .expect("epoch is not UTF-8"),
            )
        }
    }

    /// Version string (e.g. `"2.3.4"`).
    pub fn version(&self) -> &str {
        let p = unsafe { librpm_sys::rpmverV(self.ptr) };
        assert!(!p.is_null());
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("version is not UTF-8")
    }

    /// Release string (e.g. `"5.el9"`), or `None` if no release was specified.
    pub fn release(&self) -> Option<&str> {
        let p = unsafe { librpm_sys::rpmverR(self.ptr) };
        if p.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(p) }
                    .to_str()
                    .expect("release is not UTF-8"),
            )
        }
    }

    /// Formatted `[E:]V[-R]` string.
    pub fn evr(&self) -> String {
        let p = unsafe { librpm_sys::rpmverEVR(self.ptr) };
        assert!(!p.is_null());
        let s = unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("EVR is not UTF-8")
            .to_owned();
        unsafe { free(p.cast()) };
        s
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let rc = unsafe { librpm_sys::rpmverCmp(self.ptr, other.ptr) };
        rc.cmp(&0)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Version {}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.evr())
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Version")
            .field("epoch", &self.epoch())
            .field("version", &self.version())
            .field("release", &self.release())
            .finish()
    }
}

impl Drop for Version {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::rpmverFree(self.ptr);
        }
    }
}
