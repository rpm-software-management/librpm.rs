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

//! RPM package signing and signature removal
//!
//! Requires the `sign` feature and links against `librpmsign.so`.
//!
//! # Example
//!
//! ```no_run
//! use librpm::sign::{self, SignArgs, SignFlags};
//!
//! librpm::init();
//!
//! // Sign with default settings (uses the GPG key configured in RPM macros)
//! sign::sign_package("/path/to/package.rpm", None).unwrap();
//!
//! // Sign with a specific key ID
//! let args = SignArgs::new().key_id("DEADBEEF");
//! sign::sign_package("/path/to/package.rpm", Some(&args)).unwrap();
//! ```

use std::ffi::CString;
use std::fmt;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Flags controlling signing behaviour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct SignFlags(u32);

impl SignFlags {
    /// No special flags.
    pub const NONE: Self = Self(librpmsign_sys::rpmSignFlags_e_RPMSIGN_FLAG_NONE);
    /// Include IMA file signatures.
    pub const IMA: Self = Self(librpmsign_sys::rpmSignFlags_e_RPMSIGN_FLAG_IMA);
    /// Produce an RPM v3 header+payload signature.
    #[cfg(has_rpmsignflag_rpmv3)]
    pub const RPMV3: Self = Self(librpmsign_sys::rpmSignFlags_e_RPMSIGN_FLAG_RPMV3);
    /// Include fsverity file signatures.
    #[cfg(has_rpmsignflag_fsverity)]
    pub const FSVERITY: Self = Self(librpmsign_sys::rpmSignFlags_e_RPMSIGN_FLAG_FSVERITY);
    /// Re-sign an already-signed package.
    #[cfg(has_rpmsignflag_resign)]
    pub const RESIGN: Self = Self(librpmsign_sys::rpmSignFlags_e_RPMSIGN_FLAG_RESIGN);
    /// Produce an RPM v4 header-only signature.
    #[cfg(has_rpmsignflag_rpmv4)]
    pub const RPMV4: Self = Self(librpmsign_sys::rpmSignFlags_e_RPMSIGN_FLAG_RPMV4);
    /// Produce an RPM v6 signature.
    #[cfg(has_rpmsignflag_rpmv6)]
    pub const RPMV6: Self = Self(librpmsign_sys::rpmSignFlags_e_RPMSIGN_FLAG_RPMV6);
}

impl std::ops::BitOr for SignFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SignFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Hash algorithm for signing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct HashAlgo(u32);

impl HashAlgo {
    /// SHA-1 (legacy, not recommended).
    pub const SHA1: Self = Self(librpmsign_sys::pgpHashAlgo_e_PGPHASHALGO_SHA1);
    /// SHA-256.
    pub const SHA256: Self = Self(librpmsign_sys::pgpHashAlgo_e_PGPHASHALGO_SHA256);
    /// SHA-384.
    pub const SHA384: Self = Self(librpmsign_sys::pgpHashAlgo_e_PGPHASHALGO_SHA384);
    /// SHA-512.
    pub const SHA512: Self = Self(librpmsign_sys::pgpHashAlgo_e_PGPHASHALGO_SHA512);
    /// SHA3-256.
    #[cfg(has_pgphashalgo_sha3_256)]
    pub const SHA3_256: Self = Self(librpmsign_sys::pgpHashAlgo_e_PGPHASHALGO_SHA3_256);
    /// SHA3-512.
    #[cfg(has_pgphashalgo_sha3_512)]
    pub const SHA3_512: Self = Self(librpmsign_sys::pgpHashAlgo_e_PGPHASHALGO_SHA3_512);
}

/// Parameters for signing operations.
///
/// Use the builder-style methods to configure, then pass to
/// [`sign_package`], [`del_sign`], or [`del_file_sign`].
pub struct SignArgs {
    key_id_c: Option<CString>,
    hashalgo: u32,
    signflags: u32,
}

impl SignArgs {
    /// Create default signing arguments.
    pub fn new() -> Self {
        SignArgs {
            key_id_c: None,
            hashalgo: 0,
            signflags: 0,
        }
    }

    /// Set the signer key ID.
    pub fn key_id(mut self, key_id: &str) -> Self {
        self.key_id_c = Some(CString::new(key_id).expect("key ID contains NUL byte"));
        self
    }

    /// Set the hash algorithm.
    pub fn hash_algo(mut self, algo: HashAlgo) -> Self {
        self.hashalgo = algo.0;
        self
    }

    /// Re-sign an already-signed package. Default: `false`.
    #[cfg(has_rpmsignflag_resign)]
    pub fn resign(mut self, yes: bool) -> Self {
        if yes {
            self.signflags |= SignFlags::RESIGN.0;
        } else {
            self.signflags &= !SignFlags::RESIGN.0;
        }
        self
    }

    /// Set signing flags.
    pub fn flags(mut self, flags: SignFlags) -> Self {
        self.signflags |= flags.0;
        self
    }

    fn to_raw(&self) -> librpmsign_sys::rpmSignArgs {
        librpmsign_sys::rpmSignArgs {
            keyid: self
                .key_id_c
                .as_ref()
                .map_or(std::ptr::null_mut(), |c| c.as_ptr().cast_mut()),
            hashalgo: self.hashalgo,
            signflags: self.signflags,
        }
    }
}

impl Default for SignArgs {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned by signing operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignError;

impl fmt::Display for SignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RPM signing operation failed")
    }
}

impl std::error::Error for SignError {}

/// Sign an RPM package.
///
/// Pass `None` for `args` to use default settings (key and algorithm
/// are taken from RPM macro configuration).
///
/// # Panics
///
/// Panics if `path` contains interior NUL bytes.
pub fn sign_package(path: impl AsRef<Path>, args: Option<&SignArgs>) -> Result<(), SignError> {
    let path_c =
        CString::new(path.as_ref().as_os_str().as_bytes()).expect("path contains NUL byte");
    let raw_args;
    let args_ptr = match args {
        Some(a) => {
            raw_args = a.to_raw();
            &raw_args
        }
        None => std::ptr::null(),
    };
    let rc = unsafe { librpmsign_sys::rpmPkgSign(path_c.as_ptr(), args_ptr) };
    if rc == 0 { Ok(()) } else { Err(SignError) }
}

/// Delete signature(s) from an RPM package.
///
/// # Panics
///
/// Panics if `path` contains interior NUL bytes.
pub fn del_sign(path: impl AsRef<Path>, args: Option<&SignArgs>) -> Result<(), SignError> {
    let path_c =
        CString::new(path.as_ref().as_os_str().as_bytes()).expect("path contains NUL byte");
    let raw_args;
    let args_ptr = match args {
        Some(a) => {
            raw_args = a.to_raw();
            &raw_args
        }
        None => std::ptr::null(),
    };
    let rc = unsafe { librpmsign_sys::rpmPkgDelSign(path_c.as_ptr(), args_ptr) };
    if rc == 0 { Ok(()) } else { Err(SignError) }
}

/// Delete file signature(s) from an RPM package.
///
/// # Panics
///
/// Panics if `path` contains interior NUL bytes.
#[cfg(has_rpmpkg_delfilesign)]
pub fn del_file_sign(path: impl AsRef<Path>, args: Option<&SignArgs>) -> Result<(), SignError> {
    let path_c =
        CString::new(path.as_ref().as_os_str().as_bytes()).expect("path contains NUL byte");
    let raw_args;
    let args_ptr = match args {
        Some(a) => {
            raw_args = a.to_raw();
            &raw_args
        }
        None => std::ptr::null(),
    };
    let rc = unsafe { librpmsign_sys::rpmPkgDelFileSign(path_c.as_ptr(), args_ptr) };
    if rc == 0 { Ok(()) } else { Err(SignError) }
}
