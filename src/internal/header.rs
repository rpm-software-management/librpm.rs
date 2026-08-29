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
use std::ffi::{CStr, CString};
use std::mem;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use super::rc::{RpmErrorKind, RpmReturnCode};
use super::ts::TransactionSet;
use super::{tag::Tag, td::TagData};

unsafe extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

/// RPM package header
pub(crate) struct Header(librpm_sys::Header); // *mut librpm_sys::headerToken_s

impl Header {
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

    /// Get the raw librpm Header pointer for passing to C functions.
    pub(crate) fn as_ptr(&self) -> librpm_sys::Header {
        self.0
    }

    pub(crate) fn from_file(
        path: &Path,
        options: Option<&crate::verify::VerifyOptions>,
    ) -> Result<Self, RpmErrorKind> {
        let (header, fd) = Self::read_package_file(path, options)?;
        unsafe { librpm_sys::Fclose(fd) };
        Ok(header)
    }

    /// Open an RPM file, read its header, and return both the header and the
    /// still-open file descriptor. After `rpmReadPackageFile`, the fd is
    /// positioned at the start of the payload — callers that need to read the
    /// archive content should keep it open.
    pub(crate) fn read_package_file(
        path: &Path,
        options: Option<&crate::verify::VerifyOptions>,
    ) -> Result<(Self, librpm_sys::FD_t), RpmErrorKind> {
        let txn = TransactionSet::create();

        let filename = CString::new(path.as_os_str().as_bytes()).unwrap();
        let fmode = CString::new("r.ufdio").unwrap();

        // Safety: filename and fmode are valid CStrings kept alive for the call
        let fd: librpm_sys::FD_t = unsafe { librpm_sys::Fopen(filename.as_ptr(), fmode.as_ptr()) };

        if fd.is_null() || unsafe { librpm_sys::Ferror(fd) } != 0 {
            let msg = if fd.is_null() {
                "failed to open file".to_string()
            } else {
                let s = unsafe { CStr::from_ptr(librpm_sys::Fstrerror(fd)) }
                    .to_string_lossy()
                    .into_owned();
                unsafe { librpm_sys::Fclose(fd) };
                s
            };
            return Err(RpmErrorKind::Io(msg));
        }

        let vsflags = match options {
            Some(opts) => opts.flags.bits(),
            None => librpm_sys::rpmVSFlags_e_RPMVSF_DEFAULT,
        };

        // Safety: rpmReadPackageFile takes a `Header *hdrp` out-parameter.
        // It sets `*hdrp = NULL`, then on success sets `*hdrp = headerLink(h)`.
        // We pass a null pointer — not a headerNew() result — to avoid leaking
        // the overwritten header. On error, Fclose is called before returning.
        unsafe {
            let raw_ts = txn.as_ptr();
            librpm_sys::rpmtsSetVSFlags(raw_ts, vsflags);

            if let Some(opts) = options {
                if let Some(ref kr) = opts.keyring {
                    librpm_sys::rpmtsSetKeyring(raw_ts, kr.as_ptr());
                }
            }

            let mut hdr_ptr: librpm_sys::Header = std::ptr::null_mut();
            // rpmReadPackageFile looks like pure file I/O, but with signature
            // checking enabled (RPMVSF_DEFAULT) and no explicit keyring it
            // calls rpmtsGetKeyring(ts, 1) -> loadKeyringFromDB, which opens
            // the database (rpmdbRock) and creates a match iterator (rpmmiRock),
            // mutating the RPM <= 4.18 global tracking lists. Hold
            // rpm_global_lock across the call. See docs/locking.md.
            let _lock = crate::internal::rpm_global_lock();
            let rc = librpm_sys::rpmReadPackageFile(raw_ts, fd, std::ptr::null(), &mut hdr_ptr);

            match RpmReturnCode::from_raw(rc) {
                Some(RpmReturnCode::Ok) => {
                    assert!(!hdr_ptr.is_null());
                    // rpmReadPackageFile already called headerLink; the header
                    // has refcount 1. Wrap it directly — do NOT call headerLink
                    // again (Header::from_ptr would double-link).
                    Ok((Header(hdr_ptr), fd))
                }
                err => {
                    librpm_sys::Fclose(fd);
                    match err {
                        Some(RpmReturnCode::NotFound) => Err(RpmErrorKind::NotFound),
                        Some(RpmReturnCode::NotTrusted) => Err(RpmErrorKind::NotTrusted),
                        Some(RpmReturnCode::NoKey) => Err(RpmErrorKind::NoKey),
                        _ => Err(RpmErrorKind::Fail),
                    }
                }
            }
        }
    }

    /// Get the data that corresponds to the given header tag.
    ///
    /// # Safety invariant: `HEADERGET_MINMEM`
    ///
    /// We use `HEADERGET_MINMEM`, which returns pointers directly into
    /// the header's in-memory blob rather than copying. The returned
    /// `TagData<'_>` borrows from `&self`, so the header (and its blob)
    /// stays alive as long as the `TagData` exists. `rpmtdFreeData` is
    /// called before returning, but with `HEADERGET_MINMEM` it only
    /// frees the pointer array allocated for `STRING_ARRAY` types —
    /// the string data itself points into the blob and is not freed.
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

    /// Format the header using an RPM query format string (`%{TAG}` syntax).
    ///
    /// Returns the formatted string, or an error if the format string is invalid
    /// (e.g. references an unknown tag).
    pub(crate) fn format(&self, fmt: &str) -> Result<String, crate::error::Error> {
        use crate::error::ErrorKind;

        let fmt_cstr =
            CString::new(fmt).map_err(|e| format_err!(ErrorKind::InvalidArg, "{}", e))?;

        // errmsg_t is *const c_char; headerFormat sets it to point into a
        // static buffer inside librpm on failure — do NOT free it.
        let mut errmsg: librpm_sys::errmsg_t = std::ptr::null();

        // Safety: self.0 is a valid Header pointer (invariant of Header).
        // fmt_cstr is a valid null-terminated C string kept alive for the call.
        // The returned pointer (if non-null) is malloc'd by librpm and must be
        // freed with free().
        let result = unsafe { librpm_sys::headerFormat(self.0, fmt_cstr.as_ptr(), &mut errmsg) };

        if result.is_null() {
            let msg = if errmsg.is_null() {
                "headerFormat failed".to_string()
            } else {
                unsafe { CStr::from_ptr(errmsg) }
                    .to_string_lossy()
                    .into_owned()
            };
            return Err(format_err!(ErrorKind::FormatString, "{}", msg));
        }

        let s = unsafe { CStr::from_ptr(result) }
            .to_string_lossy()
            .into_owned();
        unsafe { free(result.cast()) };
        Ok(s)
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
