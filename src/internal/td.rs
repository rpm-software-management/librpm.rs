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

//! Tag data, i.e. data fields found in RPM headers

// Take this as a sign this code is not properly tested
#![allow(dead_code)]

use super::tag::TagType;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::{slice, str};

/// Data found in RPM headers, associated with a particular `Tag` value.
#[derive(Debug)]
pub enum TagData<'hdr> {
    /// No data associated with this tag
    Null,

    /// Character array (single-byte values, corresponds to RPM_CHAR_TYPE)
    Char(&'hdr [u8]),

    /// 8-bit integer array
    Int8(&'hdr [i8]),

    /// 16-bit integer array
    Int16(&'hdr [i16]),

    /// 32-bit integer array
    Int32(&'hdr [i32]),

    /// 64-bit integer array
    Int64(&'hdr [i64]),

    /// String
    Str(&'hdr str),

    /// String array
    StrArray(Vec<&'hdr str>),

    /// Internationalized string array (one per locale)
    I18NStr(Vec<&'hdr str>),

    /// Binary data
    Bin(&'hdr [u8]),
}

impl<'hdr> TagData<'hdr> {
    /// Convert an `rpmtd_s` into a `TagData::Char`
    pub(crate) unsafe fn char(td: &librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::CHAR as u32);
        // Safety: type assertion above guarantees data points to `count` u8 values
        let data = unsafe { slice::from_raw_parts(td.data as *const u8, td.count as usize) };
        TagData::Char(data)
    }

    /// Convert an `rpmtd_s` into a `TagData::Int8`
    pub(crate) unsafe fn int8(td: &librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::INT8 as u32);
        // Safety: type assertion above guarantees data points to `count` i8 values
        let data = unsafe { slice::from_raw_parts(td.data as *const i8, td.count as usize) };
        TagData::Int8(data)
    }

    /// Convert an `rpmtd_s` into a `TagData::Int16`
    pub(crate) unsafe fn int16(td: &librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::INT16 as u32);
        // Safety: type assertion above guarantees data points to `count` i16 values
        let data = unsafe { slice::from_raw_parts(td.data as *const i16, td.count as usize) };
        TagData::Int16(data)
    }

    /// Convert an `rpmtd_s` into a `TagData::Int32`
    pub(crate) unsafe fn int32(td: &librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::INT32 as u32);
        // Safety: type assertion above guarantees data points to `count` i32 values
        let data = unsafe { slice::from_raw_parts(td.data as *const i32, td.count as usize) };
        TagData::Int32(data)
    }

    /// Convert an `rpmtd_s` into a `TagData::Int64`
    pub(crate) unsafe fn int64(td: &librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::INT64 as u32);
        // Safety: type assertion above guarantees data points to `count` i64 values
        let data = unsafe { slice::from_raw_parts(td.data as *const i64, td.count as usize) };
        TagData::Int64(data)
    }

    /// Convert an `rpmtd_s` into a `Str`
    pub(crate) unsafe fn string(td: &librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::STRING as u32);
        // Safety: protected by the above line
        let cstr = unsafe { CStr::from_ptr(td.data as *const c_char) };

        // RPM_STRING_TYPE is ASCII-only. We presently treat it as UTF-8.
        TagData::Str(str::from_utf8(cstr.to_bytes()).unwrap_or_else(|e| {
            panic!(
                "failed to decode RPM_STRING_TYPE as UTF-8 (tag: {}): {}",
                td.tag, e
            );
        }))
    }

    /// Convert an `rpmtd_s` into a `StrArray`
    pub(crate) unsafe fn string_array(td: &mut librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::STRING_ARRAY as u32);
        let mut result = Vec::new();
        loop {
            // Safety: rpmtdNextString is always safe to call with a valid td;
            // it advances the internal index and returns a pointer into the
            // header blob (with HEADERGET_MINMEM), or NULL when exhausted.
            let cstr = unsafe { librpm_sys::rpmtdNextString(td) };
            if cstr.is_null() {
                break;
            }
            // Safety: rpmtdNextString returned a non-null pointer to a
            // null-terminated C string within the header blob.
            let cstr = unsafe { CStr::from_ptr(cstr as *const c_char) };
            result.push(str::from_utf8(cstr.to_bytes()).unwrap_or_else(|e| {
                panic!(
                    "failed to decode an item from RPM_STRING_ARRAY as UTF-8 (tag: {}): {}",
                    td.tag, e
                );
            }));
        }
        TagData::StrArray(result)
    }

    /// Convert an `rpmtd_s` into an `I18NStr`
    pub(crate) unsafe fn i18n_string(td: &mut librpm_sys::rpmtd_s) -> Self {
        assert_eq!(td.type_, TagType::I18NSTRING as u32);
        let mut result = Vec::new();
        loop {
            // Safety: rpmtdNextString is always safe to call with a valid td;
            // it advances the internal index and returns a pointer into the
            // header blob (with HEADERGET_MINMEM), or NULL when exhausted.
            let cstr = unsafe { librpm_sys::rpmtdNextString(td) };
            if cstr.is_null() {
                break;
            }
            // Safety: rpmtdNextString returned a non-null pointer to a
            // null-terminated C string within the header blob.
            let cstr = unsafe { CStr::from_ptr(cstr as *const c_char) };
            result.push(str::from_utf8(cstr.to_bytes()).unwrap_or_else(|e| {
                panic!(
                    "failed to decode an item from RPM_I18NSTRING_TYPE as UTF-8 (tag: {}): {}",
                    td.tag, e
                );
            }));
        }
        TagData::I18NStr(result)
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

        // Safety: protected by 3 assertions above
        let bin = unsafe { slice::from_raw_parts(td.data as *const u8, td.count as usize) };
        TagData::Bin(bin)
    }

    /// Is this tag data NULL?
    pub fn is_null(&self) -> bool {
        matches!(*self, TagData::Null)
    }

    /// Obtain the first char value, if this is a char
    pub fn as_char(&self) -> Option<u8> {
        self.as_char_array().and_then(|s| s.first().copied())
    }

    /// Obtain a char slice, if this is a char
    pub fn as_char_array(&self) -> Option<&'hdr [u8]> {
        match *self {
            TagData::Char(c) => Some(c),
            _ => None,
        }
    }

    /// Is this value a char?
    pub fn is_char(&self) -> bool {
        self.as_char_array().is_some()
    }

    /// Obtain the first int8 value, if this is an int8
    pub fn as_int8(&self) -> Option<i8> {
        self.as_int8_array().and_then(|s| s.first().copied())
    }

    /// Obtain an int8 slice, if this is an int8
    pub fn as_int8_array(&self) -> Option<&'hdr [i8]> {
        match *self {
            TagData::Int8(i) => Some(i),
            _ => None,
        }
    }

    /// Is this value an int8?
    pub fn is_int8(&self) -> bool {
        self.as_int8_array().is_some()
    }

    /// Obtain the first int16 value, if this is an int16
    pub fn as_int16(&self) -> Option<i16> {
        self.as_int16_array().and_then(|s| s.first().copied())
    }

    /// Obtain an int16 slice, if this is an int16
    pub fn as_int16_array(&self) -> Option<&'hdr [i16]> {
        match *self {
            TagData::Int16(i) => Some(i),
            _ => None,
        }
    }

    /// Is this value an int16?
    pub fn is_int16(&self) -> bool {
        self.as_int16_array().is_some()
    }

    /// Obtain the first int32 value, if this is an int32
    pub fn as_int32(&self) -> Option<i32> {
        self.as_int32_array().and_then(|s| s.first().copied())
    }

    /// Obtain an int32 slice, if this is an int32
    pub fn as_int32_array(&self) -> Option<&'hdr [i32]> {
        match *self {
            TagData::Int32(i) => Some(i),
            _ => None,
        }
    }

    /// Is this value an int32?
    pub fn is_int32(&self) -> bool {
        self.as_int32_array().is_some()
    }

    /// Obtain the first int64 value, if this is an int64
    pub fn as_int64(&self) -> Option<i64> {
        self.as_int64_array().and_then(|s| s.first().copied())
    }

    /// Obtain an int64 slice, if this is an int64
    pub fn as_int64_array(&self) -> Option<&'hdr [i64]> {
        match *self {
            TagData::Int64(i) => Some(i),
            _ => None,
        }
    }

    /// Is this value an int64?
    pub fn is_int64(&self) -> bool {
        self.as_int64_array().is_some()
    }

    /// Obtain a string reference, so long as this value is a string type.
    /// For I18NStr, returns the first (locale-selected) string.
    pub fn as_str(&self) -> Option<&'hdr str> {
        match self {
            TagData::Str(s) => Some(s),
            TagData::I18NStr(s) => s.first().copied(),
            _ => None,
        }
    }

    /// Is this value a string?
    pub fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    /// Obtain the full array of locale strings, if this is an I18NStr
    pub fn as_i18n_str_array(&self) -> Option<&[&'hdr str]> {
        match self {
            TagData::I18NStr(sa) => Some(&sa[..]),
            _ => None,
        }
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
