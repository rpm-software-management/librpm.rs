//! RPM package headers

use super::{tag::Tag, td::TagData};
use crate::Package;
use std::{mem, borrow::BorrowMut};

/// RPM package header
pub struct Header(*mut librpm_sys::headerToken_s);

impl Header {
    pub(crate) unsafe fn from_ptr(ffi_header: librpm_sys::Header) -> Self {
        assert!(!ffi_header.is_null());
        // Increment librpm's internal reference count for this header
        librpm_sys::headerLink(ffi_header);
        Header(ffi_header)
    }

    pub(crate) fn as_ptr(&self) -> *mut librpm_sys::headerToken_s {
        self.0
    }

    /// Get the data that corresponds to the given header tag.
    pub(crate) fn get(&self, tag: Tag) -> Option<TagData> {
        let mut data = TagData::create();
        let td = data.to_ptr();

        let rc = unsafe {
            librpm_sys::headerGet(
                self.0,
                tag as i32,
                td,
                librpm_sys::headerGetFlags_e_HEADERGET_MINMEM,
            )
        };

        if rc == 0 {
            return None;
        }

        Some(data)
    }

    pub(crate) fn set(&mut self, mut data: TagData) {
        let td = data.to_ptr();

        let rc = unsafe {
            librpm_sys::headerMod(
                self.0,
                td,
            )
        };

        if rc == 0 {
            panic!("failed to set header tag");
        }
    }

    /// Convert this `Header` into a `Package`
    pub fn to_package(&self) -> Package {
        Package {
            name: self.get(Tag::NAME).unwrap().str().to_owned(),
            epoch: self.get(Tag::EPOCH).map(|mut d| d.str().to_owned()),
            version: self.get(Tag::VERSION).unwrap().str().to_owned(),
            release: self.get(Tag::RELEASE).unwrap().str().to_owned(),
            // BUG: Architecture is an enum and not to be converted into a string. Please fix.
            arch: self.get(Tag::ARCH).map(|mut d| d.str().to_owned()),
            license: self.get(Tag::LICENSE).unwrap().str().to_owned(),
            summary: self.get(Tag::SUMMARY).unwrap().str().into(),
            description: self.get(Tag::DESCRIPTION).unwrap().str().into(),
            buildtime: self.get(Tag::BUILDTIME).unwrap().int32(),
        }
    }
    /// Turn the given `Package` into a `Header`
    pub fn from_package(package: Package) -> Self {
        let mut header = Header {
            0: unsafe { librpm_sys::headerNew() },
        };
        // TODO: Either ditch the Package struct or add a way to convert it into a Header
        // fingers crossed, i don't know what i'm doing -cappy
        unsafe {
            //header.set(TagData::string(package.name.as_bytes()).to_ptr());
        };
        header
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
