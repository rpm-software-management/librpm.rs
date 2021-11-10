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
}

impl Drop for Header {
    fn drop(&mut self) {
        // Decrement librpm's internal reference count for this header
        unsafe {
            librpm_sys::headerFree(self.0);
        }
    }
}
