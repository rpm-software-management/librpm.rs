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

use super::{tag::Tag, td::TagData};
use crate::Package;
use std::ffi::{CStr, OsStr};
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::null_mut;

/// RPM package header
pub(crate) struct Header(*mut librpm_sys::headerToken_s);

impl Header {
    pub(crate) unsafe fn from_ptr(ffi_header: librpm_sys::Header) -> Self {
        assert!(!ffi_header.is_null());
        // Increment librpm's internal reference count for this header
        unsafe {
            librpm_sys::headerLink(ffi_header);
        }
        Header(ffi_header)
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
            librpm_sys::rpmTagType_e_RPM_STRING_ARRAY_TYPE => unsafe { TagData::string_array(&td) },
            librpm_sys::rpmTagType_e_RPM_I18NSTRING_TYPE => unsafe { TagData::i18n_string(&td) },
            librpm_sys::rpmTagType_e_RPM_BIN_TYPE => unsafe { TagData::bin(&td) },
            other => panic!("unsupported rpmtd tag type: {other}"),
        };

        Some(data)
    }

    /// Convert this `Header` into a `Package`
    pub(crate) fn to_package(&self) -> Package {
        Package {
            name: self.get(Tag::NAME).unwrap().as_str().unwrap().to_owned(),
            epoch: self
                .get(Tag::EPOCH)
                .map(|d| d.to_int32().unwrap().to_owned()),
            version: self.get(Tag::VERSION).unwrap().as_str().unwrap().to_owned(),
            release: self.get(Tag::RELEASE).unwrap().as_str().unwrap().to_owned(),
            arch: self.get(Tag::ARCH).map(|d| d.as_str().unwrap().to_owned()),
            license: self.get(Tag::LICENSE).unwrap().as_str().unwrap().to_owned(),
            summary: self.get(Tag::SUMMARY).unwrap().as_str().unwrap().into(),
            description: self.get(Tag::DESCRIPTION).unwrap().as_str().unwrap().into(),
            buildtime: self.get(Tag::BUILDTIME).unwrap().to_int32().unwrap(),
            filenames: FileNameIterator::from_header(self).collect(),
        }
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

/// Iterator over a single file info query
struct FileNameIterator {
    fi: *mut librpm_sys::rpmfi_s,
}

impl Iterator for FileNameIterator {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: see `Self::from_header`
        if unsafe { librpm_sys::rpmfiNext(self.fi) } < 0 {
            return None;
        }
        // SAFETY: see `Self::from_header`. Return value can
        // be an arbitrary null-terminated byte sequence.
        // Return value (`name`) stays valid until the next rpmfiFn call.
        let name = unsafe {
            let name = librpm_sys::rpmfiFN(self.fi);
            assert!(!name.is_null());
            CStr::from_ptr(name).to_bytes()
        };
        let name = PathBuf::from(OsStr::from_bytes(name));
        Some(name)
    }
}

impl FileNameIterator {
    fn from_header(header: &Header) -> Self {
        // SAFETY: once constructed, rpmfiNew return value stays valid
        // until rpmfiFree is called. Additionally:
        // 1. Ensure it is not NULL so that a valid Rust reference can be created
        // 2. Memory-safety of fi is not dependent on the header per docs, however,
        //    it does not mean actual file list will be up to date per the latest package
        //    version. Right now it doesn't matter as we collect file names eagerly during
        //    construction.
        let fi = unsafe {
            let fi = librpm_sys::rpmfiNew(null_mut(), header.0, 0, 0);
            assert!(!fi.is_null());
            fi
        };
        FileNameIterator { fi }
    }
}

impl Drop for FileNameIterator {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: see `Self::from_header`
            librpm_sys::rpmfiFree(self.fi);
        }
    }
}
