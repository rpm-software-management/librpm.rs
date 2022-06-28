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

    pub(crate) fn to_ptr(&mut self) -> &mut AtomicPtr<librpm_sys::rpmtd_s> {
        &mut self.0
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

    pub(crate) unsafe fn char(&mut self) -> char {
        let td = *self.0.get_mut();

        assert_eq!((*td).type_, TagType::CHAR as u32);
        let ix = if (*td).ix >= 0 { (*td).ix as isize } else { 0 };
        *((*td).data as *const char).offset(ix)
    }

    pub(crate) unsafe fn set_char(&mut self, mut value: char) {
        let mut container = **self.0.get_mut();
        container.size = 1;
        container.data = &mut value as *mut _ as *mut c_void;
    }

    pub(crate) unsafe fn int8(&mut self) -> i8 {
        let td = *self.0.get_mut();

        assert_eq!((*td).type_, TagType::INT8 as u32);
        let ix = if (*td).ix >= 0 { (*td).ix as isize } else { 0 };
        *((*td).data as *const i8).offset(ix)
    }

    pub(crate) unsafe fn set_int8(&mut self, mut value: i8) {
        let mut container = **self.0.get_mut();
        container.size = 1;
        container.data = &mut value as *mut _ as *mut c_void;
    }

    pub(crate) unsafe fn int16(&mut self) -> i16 {
        let td = *self.0.get_mut();

        assert_eq!((*td).type_, TagType::INT16 as u32);
        let ix = if (*td).ix >= 0 { (*td).ix as isize } else { 0 };
        *((*td).data as *const i16).offset(ix)
    }

    pub(crate) unsafe fn set_int16(&mut self, mut value: i16) {
        let mut container = **self.0.get_mut();
        container.size = 1;
        container.data = &mut value as *mut _ as *mut c_void;
    }

    pub(crate) unsafe fn int32(&mut self) -> i32 {
        let td = *self.0.get_mut();

        assert_eq!((*td).type_, TagType::INT32 as u32);
        let ix = if (*td).ix >= 0 { (*td).ix as isize } else { 0 };
        *((*td).data as *const i32).offset(ix)
    }

    pub(crate) unsafe fn set_int32(&mut self, mut value: i32) {
        let mut container = **self.0.get_mut();
        container.size = 1;
        container.data = &mut value as *mut _ as *mut c_void;
    }

    pub(crate) unsafe fn int64(&mut self) -> i64 {
        let td = *self.0.get_mut();

        assert_eq!((*td).type_, TagType::INT64 as u32);
        let ix = if (*td).ix >= 0 { (*td).ix as isize } else { 0 };
        *((*td).data as *const i64).offset(ix)
    }

    pub(crate) unsafe fn set_int64(&mut self, mut value: i64) {
        let mut container = **self.0.get_mut();
        container.size = 1;
        container.data = &mut value as *mut _ as *mut c_void;
    }

    pub(crate) unsafe fn str(&mut self) -> &str {
        let td = *self.0.get_mut();

        assert_eq!((*td).type_, TagType::STRING as u32);
        let cstr = CStr::from_ptr((*td).data as *const c_char);

        let s = str::from_utf8(cstr.to_bytes()).unwrap_or_else(|e| {
            panic!(
                "failed to decode RPM_STRING_TYPE as UTF-8 (tag: {}): {}",
                (*td).tag, e
            );
        });
        debug!("decoded RPM_STRING_TYPE as UTF-8: {}", s);
        s
    }

    pub(crate) unsafe fn set_str(&mut self, mut value: &str) {
        let mut container = **self.0.get_mut();
        let string = CString::new(value).expect("could not convert to c string");
        container.size = 1;
        container.data = &mut string.as_c_str().as_ptr() as *mut _ as *mut c_void;
    }

    pub(crate) unsafe fn string_array(&mut self) -> ! {
        panic!("RPM_STRING_ARRAY_TYPE unsupported!");
    }

    pub(crate) unsafe fn i18n_string(&mut self) -> &str {
        let td = *self.0.get_mut();

        assert_eq!((*td).type_, TagType::I18NSTRING as u32);
        let cstr = CStr::from_ptr((*td).data as *const c_char);

        str::from_utf8(cstr.to_bytes()).unwrap_or_else(|e| {
            panic!(
                "failed to decode RPM_I18NSTRING_TYPE as UTF-8 (tag: {}): {}",
                (*td).tag, e
            );
        })
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
            librpm_sys::rpmtdFree(*self.0.get_mut());
        }
    }
}