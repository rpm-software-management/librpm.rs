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
use std::mem;

/// RPM package header
pub(crate) struct Header(*mut librpm_sys::headerToken_s);

impl Header {
    pub(crate) unsafe fn from_ptr(ffi_header: librpm_sys::Header) -> Self {
        assert!(!ffi_header.is_null());
        // Increment librpm's internal reference count for this header
        librpm_sys::headerLink(ffi_header);
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
                tag as i32,
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
            other => panic!("unsupported rpmtd tag type: {}", other),
        };

        Some(data)
    }

    /// Convert this `Header` into a `Package`
    pub(crate) fn to_package(&self) -> Package {
        Package {
            name: self.get(Tag::NAME).unwrap().as_str().unwrap().to_owned(),
            epoch: self.get(Tag::EPOCH).map(|d| d.as_str().unwrap().to_owned()),
            version: self.get(Tag::VERSION).unwrap().as_str().unwrap().to_owned(),
            release: self.get(Tag::RELEASE).unwrap().as_str().unwrap().to_owned(),
            arch: self.get(Tag::ARCH).map(|d| d.as_str().unwrap().to_owned()),
            license: self.get(Tag::LICENSE).unwrap().as_str().unwrap().to_owned(),
            summary: self.get(Tag::SUMMARY).unwrap().as_str().unwrap().into(),
            description: self.get(Tag::DESCRIPTION).unwrap().as_str().unwrap().into(),
            buildtime: self.get(Tag::BUILDTIME).unwrap().to_int32().unwrap(),
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
