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

//! Package verification options
//!
//! [`VerificationFlags`] controls which digest and signature checks are
//! performed when reading a package file. [`VerifyOptions`] bundles these
//! flags with an optional [`Keyring`](crate::keyring::Keyring) to form a
//! reusable configuration for [`Package::from_file`](crate::Package::from_file).
//!
//! # Example
//!
//! ```no_run
//! use librpm::verify::{VerifyOptions, VerificationFlags};
//! use librpm::keyring::Keyring;
//! use std::path::Path;
//!
//! librpm::init().unwrap();
//! let keyring = Keyring::from_rpmdb().unwrap();
//!
//! let opts = VerifyOptions::skip_signatures()  // digests only
//!     .keyring(keyring);
//!
//! let pkg = librpm::PackageHeader::from_file(
//!     Path::new("package.rpm"),
//!     Some(&opts),
//! );
//! ```

use crate::keyring::Keyring;

/// Flags controlling which verification checks are performed when
/// reading a package file.
///
/// By default (`DEFAULT`), all available checks are performed. Individual
/// checks can be disabled by combining flags with `|`.
///
/// These correspond to `rpmVSFlags_e` in librpm.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct VerificationFlags(u32);

impl VerificationFlags {
    /// Verify everything (no checks disabled).
    pub const DEFAULT: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_DEFAULT);

    /// Skip header integrity check.
    pub const NOHDRCHK: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NOHDRCHK);

    /// Skip SHA-1 header digest.
    pub const NOSHA1HEADER: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA1HEADER);

    /// Skip SHA-256 header digest.
    pub const NOSHA256HEADER: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA256HEADER);

    /// Skip DSA header signature.
    pub const NODSAHEADER: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NODSAHEADER);

    /// Skip RSA header signature.
    pub const NORSAHEADER: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NORSAHEADER);

    /// Skip MD5 payload digest.
    pub const NOMD5: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NOMD5);

    /// Skip DSA payload signature.
    pub const NODSA: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NODSA);

    /// Skip RSA payload signature.
    pub const NORSA: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NORSA);

    /// Skip OpenPGP signature verification.
    #[cfg(has_rpmvsflag_noopenpgp)]
    pub const NOOPENPGP: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NOOPENPGP);

    /// Skip SHA-256 payload digest.
    #[cfg(has_rpmvsflag_nosha256payload)]
    pub const NOSHA256PAYLOAD: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA256PAYLOAD);

    /// Skip SHA-512 payload digest.
    #[cfg(has_rpmvsflag_nosha512payload)]
    pub const NOSHA512PAYLOAD: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA512PAYLOAD);

    /// Skip SHA3-256 header digest.
    #[cfg(has_rpmvsflag_nosha3_256header)]
    pub const NOSHA3_256HEADER: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA3_256HEADER);

    /// Skip SHA3-256 payload digest.
    #[cfg(has_rpmvsflag_nosha3_256payload)]
    pub const NOSHA3_256PAYLOAD: Self = Self(librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA3_256PAYLOAD);

    /// Skip all signature checks (header and payload).
    #[allow(unused_mut)]
    pub fn mask_nosignatures() -> Self {
        let mut flags = Self::NODSAHEADER.0 | Self::NORSAHEADER.0 | Self::NODSA.0 | Self::NORSA.0;
        #[cfg(has_rpmvsflag_noopenpgp)]
        {
            flags |= Self::NOOPENPGP.0;
        }
        Self(flags)
    }

    /// Skip all digest checks (header and payload).
    #[allow(unused_mut)]
    pub fn mask_nodigests() -> Self {
        let mut flags =
            Self::NOHDRCHK.0 | Self::NOSHA1HEADER.0 | Self::NOSHA256HEADER.0 | Self::NOMD5.0;
        #[cfg(has_rpmvsflag_nosha256payload)]
        {
            flags |= Self::NOSHA256PAYLOAD.0;
        }
        #[cfg(has_rpmvsflag_nosha512payload)]
        {
            flags |= Self::NOSHA512PAYLOAD.0;
        }
        #[cfg(has_rpmvsflag_nosha3_256header)]
        {
            flags |= Self::NOSHA3_256HEADER.0;
        }
        #[cfg(has_rpmvsflag_nosha3_256payload)]
        {
            flags |= Self::NOSHA3_256PAYLOAD.0;
        }
        Self(flags)
    }

    /// Skip all verification checks (signatures and digests).
    #[allow(unused_mut)]
    pub fn all_disabled() -> Self {
        Self(Self::mask_nodigests().bits() | Self::mask_nosignatures().bits())
    }

    /// Return the raw bits.
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for VerificationFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for VerificationFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for VerificationFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for VerificationFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::Not for VerificationFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

/// Options for package verification.
///
/// Bundles [`VerificationFlags`] with an optional
/// [`Keyring`](crate::keyring::Keyring). Pass to
/// [`Package::from_file`](crate::Package::from_file) to control how
/// a package file is verified when read.
///
/// The struct is cheaply cloneable: `Keyring` clone is a refcount
/// increment.
///
/// # Example
///
/// ```no_run
/// use librpm::verify::{VerifyOptions, VerificationFlags};
/// use std::path::Path;
///
/// // Skip all verification (equivalent to pre-0.3 from_file behavior)
/// let opts = VerifyOptions::skip_verification();
/// let pkg = librpm::PackageHeader::from_file(
///     Path::new("package.rpm"),
///     Some(&opts),
/// );
/// ```
#[derive(Clone, Debug)]
pub struct VerifyOptions {
    pub(crate) flags: VerificationFlags,
    pub(crate) keyring: Option<Keyring>,
}

impl VerifyOptions {
    /// Create default verification options.
    ///
    /// Uses `VerificationFlags::DEFAULT` (verify everything) with no
    /// custom keyring (the system keyring is auto-loaded).
    pub fn new() -> Self {
        Self {
            flags: VerificationFlags::DEFAULT,
            keyring: None,
        }
    }

    /// Create options that skip all verification checks.
    pub fn skip_verification() -> Self {
        Self {
            flags: VerificationFlags::all_disabled(),
            keyring: None,
        }
    }

    /// Create options that skip all digest verification checks.
    pub fn skip_digests() -> Self {
        Self {
            flags: VerificationFlags::mask_nodigests(),
            keyring: None,
        }
    }

    /// Create options that skip all signature verification checks.
    pub fn skip_signatures() -> Self {
        Self {
            flags: VerificationFlags::mask_nosignatures(),
            keyring: None,
        }
    }

    /// Set the verification flags.
    pub fn flags(mut self, flags: VerificationFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set a custom keyring for signature verification.
    ///
    /// If not set, the system keyring is auto-loaded from the RPM
    /// database on first use.
    pub fn keyring(mut self, keyring: Keyring) -> Self {
        self.keyring = Some(keyring);
        self
    }
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self::new()
    }
}
