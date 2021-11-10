//! RPM package headers

use super::archive::{RpmReturnCode, RpmErrorKind};
use super::{tag::Tag, td::TagData, ts::TransactionSet};
use std::ffi::CString;
use std::mem;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

/// RPM package header
pub(crate) struct Header(*mut librpm_sys::headerToken_s);

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
    /// the live reference if one existed.
    pub(crate) unsafe fn from_ptr(ffi_header: librpm_sys::Header) -> Self {
        assert!(!ffi_header.is_null());
        // Increment librpm's internal reference count for this header
        librpm_sys::headerLink(ffi_header);
        Header(ffi_header)
    }

    pub(crate) fn from_file(path: &Path) -> Result<Self, RpmErrorKind> {
        let filename = CString::new(path.as_os_str().as_bytes()).unwrap();
        let fmode = CString::new("r.ufdio").unwrap();

        let fd: librpm_sys::FD_t = unsafe { librpm_sys::Fopen(filename.as_ptr(), fmode.as_ptr()) };
        // GlobalTS.lock() ?  Or can the transaction sets be independent because we're not touching a global database?
        let ts = TransactionSet::create();

        let mut hdr = Header::new();

        let mut vsflags: librpm_sys::rpmVSFlags;
        // These are #defines, need to recreate them :(
        vsflags |= librpm_sys::_RPMVSF_NODIGESTS;
        vsflags |= librpm_sys::_RPMVSF_NOSIGNATURES;
        vsflags |= librpm_sys::_RPMVSF_NOHDRCHK;

        unsafe {
            let raw_ts = *ts.as_mut_ptr();
            librpm_sys::rpmtsSetVSFlags(raw_ts, vsflags);

            // TODO: implement Header::as_ptr() to avoid needing field accesses?
            match librpm_sys::rpmReadPackageFile(raw_ts, fd, std::ptr::null(), &mut hdr.0) {
                RpmReturnCode::Ok => Ok(hdr),
                RpmReturnCode::Fail => Err(RpmErrorKind::Fail),
                RpmReturnCode::NotFound => Err(RpmErrorKind::NotFound),
                RpmReturnCode::NotTrusted => Err(RpmErrorKind::NotTrusted),
                RpmReturnCode::NoKey => Err(RpmErrorKind::NoKey),
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
