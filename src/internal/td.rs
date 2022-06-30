use core::ffi;
use std::sync::atomic::AtomicPtr;
use super::tag::{TagType, Tag};
use std::ffi::{CStr, c_void, CString};
use std::os::raw::c_char;
use std::{slice, str, ptr};
use std::convert::TryInto;
use log::{debug, info};


pub(crate) struct TagData(AtomicPtr<librpm_sys::rpmtd_s>);

impl TagData {
    pub(crate) unsafe fn from_ptr(ffi_tagdata: librpm_sys::rpmtd) -> Self {
        TagData(AtomicPtr::new(ffi_tagdata))
    }
    
    pub(crate) fn create() -> Self {
        TagData(AtomicPtr::new(unsafe {
            librpm_sys::rpmtdNew()
        }))
    }

    pub(crate) fn to_ptr(&mut self) -> *mut librpm_sys::rpmtd_s {
        *self.0.get_mut()
    }

    pub(crate) fn tag_type(&mut self) -> TagType {
        unsafe {
            num::FromPrimitive::from_u32(librpm_sys::rpmtdType(*self.0.get_mut())).unwrap()
        }
    }

    pub(crate) fn tag(&mut self) -> Tag {
        unsafe {
            num::FromPrimitive::from_i32(librpm_sys::rpmtdTag(*self.0.get_mut())).unwrap()
        }
    }

    pub(crate) fn set_tag(&mut self, tag: Tag) {
        unsafe {
            let code = librpm_sys::rpmtdSetTag(*self.0.get_mut(), tag as i32);

            if code != 1 {
                panic!("failed to set tag, since container is not empty and types are incompatible");
            }
        }
    }

    pub(crate) fn reset(&mut self) {
        unsafe {
            librpm_sys::rpmtdReset(*self.0.get_mut());
        }
    }

    pub(crate) fn count(&mut self) -> u32 {
        unsafe {
            librpm_sys::rpmtdCount(*self.0.get_mut())
        }
    }

    pub(crate) fn char(&mut self) -> char {
        assert_eq!(self.tag_type(), TagType::CHAR);

        let chr = unsafe { *librpm_sys::rpmtdGetChar(*self.0.get_mut()) };
        chr as char
    }

    pub(crate) fn set_char(&mut self, value: char) {
        assert_eq!(self.tag_type(), TagType::CHAR);

        let boxed = Box::new(value as u8);
        let pointer = Box::into_raw(boxed);

        let result = unsafe {
            librpm_sys::rpmtdFromUint8(*self.0.get_mut(), self.tag() as i32, pointer, 1)
        };

        assert!(result == 1, "the tag type is not compatible with char");
    }

    pub(crate) fn int8(&mut self) -> i8 {
        assert_eq!(self.tag_type(), TagType::INT8);

        let chr = unsafe { librpm_sys::rpmtdGetNumber(*self.0.get_mut()) };
        chr as i8
    }

    pub(crate) fn set_int8(&mut self, value: i8) {
        assert_eq!(self.tag_type(), TagType::INT8);

        let boxed = Box::new(value as u8);
        let pointer = Box::into_raw(boxed);

        let result = unsafe {
            librpm_sys::rpmtdFromUint8(*self.0.get_mut(), self.tag() as i32, pointer, 1)
        };

        assert!(result == 1, "the tag type is not compatible with u8");
    }

    pub(crate) fn int16(&mut self) -> i16 {
        assert_eq!(self.tag_type(), TagType::INT16);

        let chr = unsafe { librpm_sys::rpmtdGetNumber(*self.0.get_mut()) };
        chr as i16
    }

    pub(crate) fn set_int16(&mut self, value: i16) {
        assert_eq!(self.tag_type(), TagType::INT16);

        let boxed = Box::new(value as u16);
        let pointer = Box::into_raw(boxed);

        let result = unsafe {
            librpm_sys::rpmtdFromUint16(*self.0.get_mut(), self.tag() as i32, pointer, 1)
        };

        assert!(result == 1, "the tag type is not compatible with u16");
    }

    pub(crate) fn int32(&mut self) -> i32 {
        assert_eq!(self.tag_type(), TagType::INT32);

        let chr = unsafe { librpm_sys::rpmtdGetNumber(*self.0.get_mut()) };
        chr as i32
    }

    pub(crate) fn set_int32(&mut self, value: i32) {
        assert_eq!(self.tag_type(), TagType::INT32);

        let boxed = Box::new(value as u32);
        let pointer = Box::into_raw(boxed);

        let result = unsafe {
            librpm_sys::rpmtdFromUint32(*self.0.get_mut(), self.tag() as i32, pointer, 1)
        };

        assert!(result == 1, "the tag type is not compatible with u32");
    }

    pub(crate) unsafe fn int64(&mut self) -> i64 {
        assert_eq!(self.tag_type(), TagType::INT64);

        let chr = unsafe { librpm_sys::rpmtdGetNumber(*self.0.get_mut()) };
        chr as i64
    }

    pub(crate) fn set_int64(&mut self, value: i64) {
        assert_eq!(self.tag_type(), TagType::INT64);

        let boxed = Box::new(value as u64);
        let pointer = Box::into_raw(boxed);

        let result = unsafe {
            librpm_sys::rpmtdFromUint64(*self.0.get_mut(), self.tag() as i32, pointer, 1)
        };

        assert!(result == 1, "the tag type is not compatible with u32");
    }

    pub(crate) fn str(&mut self) -> String {
        assert_eq!(self.tag_type(), TagType::STRING);

        let chr = unsafe { librpm_sys::rpmtdGetString(*self.0.get_mut()) };
        let cstr = unsafe { CStr::from_ptr(chr) };

        let str = cstr.to_string_lossy().into_owned();
        str
    }

    pub(crate) fn set_str(&mut self, value: &str) {
        assert_eq!(self.tag_type(), TagType::STRING);

        let string = CString::new(value).expect("could not convert to c string");
        let pointer = string.into_raw();

        let result = unsafe {
            librpm_sys::rpmtdFromString(*self.0.get_mut(), self.tag() as i32, pointer)
        };

        assert!(result == 1, "the tag type is not compatible with str");
    }

    pub(crate) fn string_array(&mut self) -> ! {
        panic!("RPM_STRING_ARRAY_TYPE unsupported!");
    }

    pub(crate) fn i18n_string(&mut self) -> String {
        assert_eq!(self.tag_type(), TagType::STRING);

        let chr = unsafe { librpm_sys::rpmtdGetString(*self.0.get_mut()) };
        let cstr = unsafe { CStr::from_ptr(chr) };

        let str = cstr.to_string_lossy().into_owned();
        str
    }

    pub(crate) unsafe fn bin(&mut self) -> &[u8] {
        let td = *self.0.get_mut();

        assert_eq!((*td).type_, TagType::BIN as u32);

        assert!(
            !(*td).data.is_null(),
            "rpmtd.data is NULL! (tag type: {})",
            (*td).tag
        );

        assert_ne!(
            (*td).type_,
            TagType::NULL as u32,
            "can't get slice of NULL data (tag type: {})",
            (*td).tag
        );

        slice::from_raw_parts(
            (*td).data as *const u8,
            (*td).count as usize,
        )
    }
}

impl Drop for TagData {
    fn drop(&mut self) {
        unsafe {
            // librpm_sys::rpmtdFree(*self.0.get_mut());
        }
    }
}