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

//! File information for RPM packages

use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::Tag;
use crate::internal::header::Header;

unsafe extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

/// File information set for an RPM package.
///
/// Wraps librpm's `rpmfiles` handle. Owns the underlying allocation
/// and frees it on drop. Packages with no files produce an empty set.
pub struct Files {
    ptr: Option<NonNull<librpm_sys::rpmfiles_s>>,
}

// Safety: `rpmfiles_s` is a self-contained, immutable-after-construction
// data structure. All fields are populated during `rpmfilesNew` and all
// accessor functions (`rpmfilesFN`, `rpmfilesBN`, etc.) are pure index
// lookups with no internal mutation. The reference count (`nrefs`) is
// `std::atomic_int`, making refcount operations safe across threads.
unsafe impl Send for Files {}
unsafe impl Sync for Files {}

impl Files {
    pub(crate) fn from_header(header: &Header) -> Self {
        let ptr = unsafe {
            librpm_sys::rpmfilesNew(
                std::ptr::null_mut(),
                header.as_ptr(),
                Tag::BASENAMES as librpm_sys::rpmTagVal,
                0,
            )
        };
        Files {
            ptr: NonNull::new(ptr),
        }
    }

    /// Number of files in the set.
    pub fn len(&self) -> usize {
        match self.ptr {
            Some(ptr) => unsafe { librpm_sys::rpmfilesFC(ptr.as_ptr()) as usize },
            None => 0,
        }
    }

    /// Returns `true` if the package contains no files.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Digest algorithm used for file checksums.
    ///
    /// Returns the algorithm identifier (e.g. 8 for SHA-256), or 0 if
    /// the package has no files.
    pub fn digest_algo(&self) -> i32 {
        match self.ptr {
            Some(ptr) => unsafe { librpm_sys::rpmfilesDigestAlgo(ptr.as_ptr()) },
            None => 0,
        }
    }

    /// Iterate over the files in this set.
    pub fn iter(&self) -> FileIter<'_> {
        FileIter {
            ptr: self.ptr,
            index: 0,
            count: self.len() as i32,
            _marker: PhantomData,
        }
    }
}

impl Drop for Files {
    fn drop(&mut self) {
        if let Some(ptr) = self.ptr {
            unsafe {
                librpm_sys::rpmfilesFree(ptr.as_ptr());
            }
        }
    }
}

impl fmt::Debug for Files {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Files").field("len", &self.len()).finish()
    }
}

impl<'a> IntoIterator for &'a Files {
    type Item = FileEntry<'a>;
    type IntoIter = FileIter<'a>;

    fn into_iter(self) -> FileIter<'a> {
        self.iter()
    }
}

/// A single file entry within an RPM package.
pub struct FileEntry<'a> {
    ptr: NonNull<librpm_sys::rpmfiles_s>,
    index: i32,
    _marker: PhantomData<&'a Files>,
}

impl FileEntry<'_> {
    /// Full path of the file.
    pub fn path(&self) -> String {
        let p = unsafe { librpm_sys::rpmfilesFN(self.ptr.as_ptr(), self.index) };
        assert!(!p.is_null());
        let s = unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("file path is not UTF-8")
            .to_owned();
        unsafe { free(p.cast()) };
        s
    }

    /// Base name of the file (filename without directory).
    pub fn basename(&self) -> &str {
        let p = unsafe { librpm_sys::rpmfilesBN(self.ptr.as_ptr(), self.index) };
        assert!(!p.is_null());
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("file basename is not UTF-8")
    }

    /// Directory name of the file (including trailing slash).
    pub fn dirname(&self) -> &str {
        let di = unsafe { librpm_sys::rpmfilesDI(self.ptr.as_ptr(), self.index) };
        let p = unsafe { librpm_sys::rpmfilesDN(self.ptr.as_ptr(), di as i32) };
        assert!(!p.is_null());
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("file dirname is not UTF-8")
    }

    /// File size in bytes.
    pub fn size(&self) -> u64 {
        unsafe { librpm_sys::rpmfilesFSize(self.ptr.as_ptr(), self.index) }
    }

    /// File mode (Unix permission bits and file type).
    pub fn mode(&self) -> u16 {
        unsafe { librpm_sys::rpmfilesFMode(self.ptr.as_ptr(), self.index) }
    }

    /// Owner (user name) of the file.
    pub fn user(&self) -> &str {
        let p = unsafe { librpm_sys::rpmfilesFUser(self.ptr.as_ptr(), self.index) };
        assert!(!p.is_null());
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("file user is not UTF-8")
    }

    /// Group of the file.
    pub fn group(&self) -> &str {
        let p = unsafe { librpm_sys::rpmfilesFGroup(self.ptr.as_ptr(), self.index) };
        assert!(!p.is_null());
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("file group is not UTF-8")
    }

    /// File attribute flags (config, doc, ghost, license, etc.).
    pub fn flags(&self) -> FileAttrs {
        let raw = unsafe { librpm_sys::rpmfilesFFlags(self.ptr.as_ptr(), self.index) };
        FileAttrs(raw)
    }

    /// Symlink target, or `None` if this is not a symbolic link.
    pub fn link_target(&self) -> Option<&str> {
        let p = unsafe { librpm_sys::rpmfilesFLink(self.ptr.as_ptr(), self.index) };
        if p.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("link target is not UTF-8");
        if s.is_empty() { None } else { Some(s) }
    }

    /// File capabilities string, or `None` if no capabilities are set.
    pub fn caps(&self) -> Option<&str> {
        let p = unsafe { librpm_sys::rpmfilesFCaps(self.ptr.as_ptr(), self.index) };
        if p.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("file caps is not UTF-8");
        if s.is_empty() { None } else { Some(s) }
    }

    /// Binary digest of the file, or `None` if no digest is available.
    pub fn digest(&self) -> Option<&[u8]> {
        let mut algo: std::ffi::c_int = 0;
        let mut len: usize = 0;
        let p = unsafe {
            librpm_sys::rpmfilesFDigest(self.ptr.as_ptr(), self.index, &mut algo, &mut len)
        };
        if p.is_null() || len == 0 {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(p, len) })
        }
    }
}

/// Iterator over files in a package.
pub struct FileIter<'a> {
    ptr: Option<NonNull<librpm_sys::rpmfiles_s>>,
    index: i32,
    count: i32,
    _marker: PhantomData<&'a Files>,
}

impl<'a> Iterator for FileIter<'a> {
    type Item = FileEntry<'a>;

    fn next(&mut self) -> Option<FileEntry<'a>> {
        let ptr = self.ptr?;
        if self.index >= self.count {
            return None;
        }
        let entry = FileEntry {
            ptr,
            index: self.index,
            _marker: PhantomData,
        };
        self.index += 1;
        Some(entry)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.index).max(0) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for FileIter<'_> {}

/// File attribute flags from the RPM header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileAttrs(u32);

impl FileAttrs {
    /// Raw flag bits.
    pub fn bits(self) -> u32 {
        self.0
    }

    /// File is a configuration file (`%config`).
    pub fn is_config(self) -> bool {
        self.0 & librpm_sys::rpmfileAttrs_e_RPMFILE_CONFIG != 0
    }

    /// File is documentation (`%doc`).
    pub fn is_doc(self) -> bool {
        self.0 & librpm_sys::rpmfileAttrs_e_RPMFILE_DOC != 0
    }

    /// File may be missing on disk (`%config(missingok)`).
    pub fn is_missingok(self) -> bool {
        self.0 & librpm_sys::rpmfileAttrs_e_RPMFILE_MISSINGOK != 0
    }

    /// Existing file should not be replaced (`%config(noreplace)`).
    pub fn is_noreplace(self) -> bool {
        self.0 & librpm_sys::rpmfileAttrs_e_RPMFILE_NOREPLACE != 0
    }

    /// File is a ghost (`%ghost`).
    pub fn is_ghost(self) -> bool {
        self.0 & librpm_sys::rpmfileAttrs_e_RPMFILE_GHOST != 0
    }

    /// File is a license file (`%license`).
    pub fn is_license(self) -> bool {
        self.0 & librpm_sys::rpmfileAttrs_e_RPMFILE_LICENSE != 0
    }

    /// File is an artifact (`%artifact`).
    pub fn is_artifact(self) -> bool {
        self.0 & librpm_sys::rpmfileAttrs_e_RPMFILE_ARTIFACT != 0
    }
}
