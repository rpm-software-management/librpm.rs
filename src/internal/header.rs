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

//! RPM package headers
use std::ffi::CString;
use std::mem;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use super::rc::{RpmErrorKind, RpmReturnCode};
use super::ts::GlobalTS;
use super::{tag::Tag, td::TagData};

/// RPM package header
pub(crate) struct Header(librpm_sys::Header); // *mut librpm_sys::headerToken_s

impl Header {
    pub(crate) fn new() -> Self {
        let ffi_header = unsafe { librpm_sys::headerNew() };
        assert!(!ffi_header.is_null());

        // No need to increment refcount, starts with refcount=1
        Header(ffi_header)
    }

    /// Create a Header handle in Rust from a raw pointer
    ///
    /// SAFETY: The input pointer must not be used after passing ownership from Rust, except for dropping
    /// the live reference if one existed. Once the original pointer goes out of scope, Rust should own
    /// the only reference.
    pub(crate) unsafe fn from_ptr(ffi_header: librpm_sys::Header) -> Self {
        assert!(!ffi_header.is_null());
        // Increment librpm's internal reference count for this header
        unsafe {
            librpm_sys::headerLink(ffi_header);
        }
        Header(ffi_header)
    }

    /// Get a pointer to the original librpm C struct for use by C functions.
    ///
    /// SAFETY: This pointer should not be copied and must not passed to headerFree().
    pub(crate) unsafe fn as_mut_ptr(&mut self) -> &mut librpm_sys::Header {
        &mut self.0
    }

    pub(crate) fn from_file(path: &Path) -> Result<Self, RpmErrorKind> {
        let mut txn = GlobalTS::create();

        let filename = CString::new(path.as_os_str().as_bytes()).unwrap();
        let fmode = CString::new("r.ufdio").unwrap();

        // Safety: filename and fmode are valid CStrings kept alive for the call
        let fd: librpm_sys::FD_t = unsafe { librpm_sys::Fopen(filename.as_ptr(), fmode.as_ptr()) };
        let mut hdr = Header::new();

        let mut vsflags = librpm_sys::rpmVSFlags_e_RPMVSF_NOHDRCHK
            | librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA1HEADER
            | librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA256HEADER
            | librpm_sys::rpmVSFlags_e_RPMVSF_NOMD5
            | librpm_sys::rpmVSFlags_e_RPMVSF_NODSAHEADER
            | librpm_sys::rpmVSFlags_e_RPMVSF_NORSAHEADER
            | librpm_sys::rpmVSFlags_e_RPMVSF_NODSA
            | librpm_sys::rpmVSFlags_e_RPMVSF_NORSA;

        #[cfg(has_rpmvsflag_nosha256payload)]
        {
            vsflags |= librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA256PAYLOAD;
        }
        #[cfg(has_rpmvsflag_nosha512payload)]
        {
            vsflags |= librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA512PAYLOAD;
        }
        #[cfg(has_rpmvsflag_nosha3_256payload)]
        {
            vsflags |= librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA3_256PAYLOAD;
        }
        #[cfg(has_rpmvsflag_nosha3_256header)]
        {
            vsflags |= librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA3_256HEADER;
        }
        #[cfg(has_rpmvsflag_noopenpgp)]
        {
            vsflags |= librpm_sys::rpmVSFlags_e_RPMVSF_NOOPENPGP;
        }

        // Safety: raw_ts and fd are valid pointers obtained above; hdr is
        // initialized by headerNew(). Fclose is called on all paths after
        // rpmReadPackageFile returns, regardless of success or failure.
        unsafe {
            let raw_ts = txn.as_mut_ptr();
            librpm_sys::rpmtsSetVSFlags(raw_ts, vsflags);

            let rc = librpm_sys::rpmReadPackageFile(raw_ts, fd, std::ptr::null(), hdr.as_mut_ptr());
            librpm_sys::Fclose(fd);

            match RpmReturnCode::from_raw(rc) {
                Some(RpmReturnCode::Ok) => Ok(hdr),
                Some(RpmReturnCode::NotFound) => Err(RpmErrorKind::NotFound),
                Some(RpmReturnCode::NotTrusted) => Err(RpmErrorKind::NotTrusted),
                Some(RpmReturnCode::NoKey) => Err(RpmErrorKind::NoKey),
                _ => Err(RpmErrorKind::Fail),
            }
        }
    }

    /// Get the data that corresponds to the given header tag.
    pub(crate) fn get(&self, tag: Tag) -> Option<TagData<'_>> {
        // Create a zeroed `rpmtd_s` and then immediately initialize it
        let mut td: librpm_sys::rpmtd_s = unsafe { mem::zeroed() };
        unsafe {
            librpm_sys::rpmtdReset(&mut td);
        }

        let rc = unsafe {
            librpm_sys::headerGet(
                self.0,
                tag.into(),
                &mut td,
                librpm_sys::headerGetFlags_e_HEADERGET_MINMEM,
            )
        };

        if rc == 0 {
            return None;
        }

        let data = match td.type_ {
            librpm_sys::rpmTagType_e_RPM_NULL_TYPE => TagData::Null,
            librpm_sys::rpmTagType_e_RPM_CHAR_TYPE => unsafe { TagData::char(&td) },
            librpm_sys::rpmTagType_e_RPM_INT8_TYPE => unsafe { TagData::int8(&td) },
            librpm_sys::rpmTagType_e_RPM_INT16_TYPE => unsafe { TagData::int16(&td) },
            librpm_sys::rpmTagType_e_RPM_INT32_TYPE => unsafe { TagData::int32(&td) },
            librpm_sys::rpmTagType_e_RPM_INT64_TYPE => unsafe { TagData::int64(&td) },
            librpm_sys::rpmTagType_e_RPM_STRING_TYPE => unsafe { TagData::string(&td) },
            librpm_sys::rpmTagType_e_RPM_STRING_ARRAY_TYPE => unsafe {
                TagData::string_array(&mut td)
            },
            librpm_sys::rpmTagType_e_RPM_I18NSTRING_TYPE => unsafe {
                TagData::i18n_string(&mut td)
            },
            librpm_sys::rpmTagType_e_RPM_BIN_TYPE => unsafe { TagData::bin(&td) },
            other => panic!("unsupported rpmtd tag type: {other}"),
        };

        // Safety: rpmtdFreeData is always safe to call — it only frees data
        // that was malloc'd by headerGet (e.g. the pointer array for
        // STRING_ARRAY). With HEADERGET_MINMEM, string/binary data points
        // directly into the header blob and is not freed, so our TagData
        // references remain valid.
        unsafe {
            librpm_sys::rpmtdFreeData(&mut td);
        }

        Some(data)
    }
}

impl Clone for Header {
    fn clone(&self) -> Self {
        // Safety: self.0 is a valid header pointer (invariant of Header).
        // headerLink increments the refcount; Drop calls headerFree to
        // decrement it, so the new Header owns one reference.
        unsafe {
            librpm_sys::headerLink(self.0);
        }
        Header(self.0)
    }
}

impl Drop for Header {
    fn drop(&mut self) {
        // Decrement librpm's internal reference count for this header
        unsafe {
            librpm_sys::headerFree(self.0);
        }
    }
}
