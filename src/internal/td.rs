//! Tag data, i.e. data fields found in RPM headers

// Take this as a sign this code is not properly tested
#![allow(dead_code)]

use super::tag::TagType;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::{slice, str};

/// Data found in RPM headers, associated with a particular `Tag` value.
#[derive(Debug)]
pub(crate) enum TagData<'hdr> {
    /// No data associated with this tag
    Null,

    /// Character
    Char(Vec<char>),

    /// 8-bit integer
    Int8(Vec<i8>),

    /// 16-bit integer
    Int16(Vec<i16>),

    /// 32-bit integer
    Int32(Vec<i32>),

    /// 64-bit integer
    Int64(Vec<i64>),

    /// String
    Str(&'hdr str),

    /// String array
    StrArray(Vec<&'hdr str>),

    /// Internationalized string (UTF-8?)
    I18NStr(&'hdr str),

    /// Binary data
    Bin(&'hdr [u8]),
}

impl<'hdr> TagData<'hdr> {
    /// Convert an `rpmtd_s` into a `TagData::Char`
    pub(crate) unsafe fn char(_td: &librpm_sys::rpmtd_s) -> Self {
        panic!("RPM_CHAR_TYPE unimplemented!");
    }

    /// Convert an `rpmtd_s` into an `TagData::Int8`
    pub(crate) unsafe fn int8(_td: &librpm_sys::rpmtd_s) -> Self {
        panic!("RPM_INT8_TYPE unimplemented!");
    }

    /// Convert an `rpmtd_s` int an `TagData::Int16`
    pub(crate) unsafe fn int16(_td: &librpm_sys::rpmtd_s) -> Self {
        panic!("RPM_INT16_TYPE unimplemented!");
    }

    /// Convert an `rpmtd_s` int an `TagData::Int32`
    pub(crate) unsafe fn int32(_td: &librpm_sys::rpmtd_s) -> Self {
        panic!("RPM_INT32_TYPE unimplemented!");
    }

    /// Convert an `rpmtd_s` int an `Int64`
    pub(crate) unsafe fn int64(_td: &librpm_sys::rpmtd_s) -> Self {
        panic!("RPM_INT64_TYPE unimplemented!");
    }

    /// Convert an `rpmtd_s` into a `Str`
    pub(crate) unsafe fn string(td: &librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::STRING as u32);
        let cstr = CStr::from_ptr(td.data as *const c_char);

        // RPM_STRING_TYPE is ASCII-only. We presently treat it as UTF-8.
        TagData::Str(str::from_utf8(cstr.to_bytes()).unwrap_or_else(|e| {
            panic!(
                "failed to decode RPM_STRING_TYPE as UTF-8 (tag: {}): {}",
                td.tag, e
            );
        }))
    }

    /// Convert an `rpmtd_s` into a `StrArray`
    pub(crate) unsafe fn string_array(_td: &librpm_sys::rpmtd_s) -> Self {
        panic!("RPM_STRING_ARRAY_TYPE unsupported!");
    }

    /// Convert an `rpmtd_s` into an `I18NStr`
    pub(crate) unsafe fn i18n_string(td: &librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::I18NSTRING as u32);
        let cstr = CStr::from_ptr(td.data as *const c_char);

        TagData::I18NStr(str::from_utf8(cstr.to_bytes()).unwrap_or_else(|e| {
            panic!(
                "failed to decode RPM_I18NSTRING_TYPE as UTF-8 (tag: {}): {}",
                td.tag, e
            );
        }))
    }

    /// Convert an `rpmtd_s` into a `Bin`
    pub(crate) unsafe fn bin(td: &librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::BIN as u32);

        assert!(
            !td.data.is_null(),
            "rpmtd.data is NULL! (tag type: {})",
            td.tag
        );

        assert_ne!(
            td.type_,
            TagType::NULL as u32,
            "can't get slice of NULL data (tag type: {})",
            td.tag
        );

        TagData::Bin(slice::from_raw_parts(
            td.data as *const u8,
            td.count as usize,
        ))
    }

    /// Is this tag data NULL?
    pub fn is_null(&self) -> bool {
	matches!(*self, TagData::Null)
    }

    /// Obtain a char value, if this is a char
    pub fn as_char(&self) -> Option<&[char]> {
        match *self {
            TagData::Char(ref c) => Some(c),
            _ => None,
        }
    }

    /// Is this value a char?
    pub fn is_char(&self) -> bool {
        self.as_char().is_some()
    }

    /// Obtain an int8 value, if this is an int8
    pub fn as_int8(&self) -> Option<&[i8]> {
        match *self {
            TagData::Int8(ref i) => Some(i),
            _ => None,
        }
    }

    /// Is this value an int8?
    pub fn is_int8(&self) -> bool {
        self.as_int8().is_some()
    }

    /// Obtain an int16 value, if this is an int16
    pub fn as_int16(&self) -> Option<&[i16]> {
        match *self {
            TagData::Int16(ref i) => Some(i),
            _ => None,
        }
    }

    /// Is this value an int16?
    pub fn is_int16(&self) -> bool {
        self.as_int16().is_some()
    }

    /// Obtain an int32 value, if this is an int32
    pub fn as_int32(&self) -> Option<&[i32]> {
        match *self {
            TagData::Int32(ref i) => Some(i),
            _ => None,
        }
    }

    /// Is this value an int32?
    pub fn is_int32(&self) -> bool {
        self.as_int32().is_some()
    }

    /// Obtain an int64 value, if this is an int64
    pub fn as_int64(&self) -> Option<&[i64]> {
        match *self {
            TagData::Int64(ref i) => Some(i),
            _ => None,
        }
    }

    /// Is this value an int64?
    pub fn is_int64(&self) -> bool {
        self.as_int64().is_some()
    }

    /// Obtain a string reference, so long as this value is a string type
    pub fn as_str(&self) -> Option<&'hdr str> {
        // We presently treat `STRING` and `I18NSTRING` equivalently
        match *self {
            TagData::Str(s) => Some(s),
            TagData::I18NStr(s) => Some(s),
            _ => None,
        }
    }

    /// Is this value a string?
    pub fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    /// Obtain a slice of string references, if this value is a string array
    pub fn as_str_array(&self) -> Option<&[&'hdr str]> {
        match *self {
            TagData::StrArray(ref sa) => Some(&sa[..]),
            _ => None,
        }
    }

    /// Is this value a string array?
    pub fn is_str_array(&self) -> bool {
        self.as_str_array().is_some()
    }

    /// Obtain a byte slice, if this value contains binary data
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match *self {
            TagData::Bin(b) => Some(b),
            _ => None,
        }
    }

    /// Is this value binary data?
    pub fn is_bytes(&self) -> bool {
        self.as_bytes().is_some()
    }
}
