//! RPM package headers

use super::{tag::Tag, td::TagData};
use crate::{License, Package, Version};
use std::mem;

/// RPM package header
pub(crate) struct Header(*mut librpm_sys::headerToken_s);

impl Header {
    /// Create a new header from an `librpm_sys::Header`.
    pub(crate) fn new(ffi_header: librpm_sys::Header) -> Self {
        unsafe {
            // Increment librpm's internal reference count for this header
            librpm_sys::headerLink(ffi_header);
        }

        Header(ffi_header)
    }

    /// Get the data that corresponds to the given header tag.
    pub(crate) fn get(&self, tag: Tag) -> TagData<'_> {
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

        assert_ne!(rc, 0, "headerGet returned non-zero status: {}", rc);

        match td.type_ {
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
        }
    }

    /// Convert this `Header` into a `Package`
    pub(crate) fn to_package(&self) -> Package {
        Package {
            name: self.get(Tag::NAME).as_str().unwrap().to_owned(),
            version: Version::new(self.get(Tag::VERSION).as_str().unwrap()),
            license: License::new(self.get(Tag::LICENSE).as_str().unwrap()),
            summary: self.get(Tag::SUMMARY).as_str().unwrap().into(),
            description: self.get(Tag::DESCRIPTION).as_str().unwrap().into(),
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
