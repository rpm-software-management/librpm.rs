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

//! Transaction problem reporting
//!
//! When a transaction check or run encounters issues (unsatisfied
//! dependencies, file conflicts, disk space, etc.), librpm reports them
//! as a problem set (`Problems`) containing individual `Problem`
//! entries.

use std::ffi::CStr;
use std::fmt;

unsafe extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

/// Classification of a transaction problem.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProblemType {
    /// Package architecture is incompatible with the system.
    BadArch,
    /// Package OS is incompatible with the system.
    BadOs,
    /// Package is already installed.
    PackageInstalled,
    /// Invalid file relocation.
    BadRelocate,
    /// Unsatisfied dependency (requires).
    Requires,
    /// Dependency conflict.
    Conflict,
    /// New file conflicts with an existing file from another package.
    NewFileConflict,
    /// File conflicts between packages in the transaction.
    FileConflict,
    /// Installing an older version of an already-installed package.
    OldPackage,
    /// Insufficient disk space.
    DiskSpace,
    /// Insufficient disk inodes.
    DiskNodes,
    /// Package is obsoleted by another package.
    Obsoletes,
    /// Package verification failure.
    Verify,
    /// Unrecognized problem type from a newer librpm version.
    Unknown,
}

impl ProblemType {
    fn from_raw(raw: librpm_sys::rpmProblemType_e) -> Self {
        match raw {
            librpm_sys::rpmProblemType_e_RPMPROB_BADARCH => Self::BadArch,
            librpm_sys::rpmProblemType_e_RPMPROB_BADOS => Self::BadOs,
            librpm_sys::rpmProblemType_e_RPMPROB_PKG_INSTALLED => Self::PackageInstalled,
            librpm_sys::rpmProblemType_e_RPMPROB_BADRELOCATE => Self::BadRelocate,
            librpm_sys::rpmProblemType_e_RPMPROB_REQUIRES => Self::Requires,
            librpm_sys::rpmProblemType_e_RPMPROB_CONFLICT => Self::Conflict,
            librpm_sys::rpmProblemType_e_RPMPROB_NEW_FILE_CONFLICT => Self::NewFileConflict,
            librpm_sys::rpmProblemType_e_RPMPROB_FILE_CONFLICT => Self::FileConflict,
            librpm_sys::rpmProblemType_e_RPMPROB_OLDPACKAGE => Self::OldPackage,
            librpm_sys::rpmProblemType_e_RPMPROB_DISKSPACE => Self::DiskSpace,
            librpm_sys::rpmProblemType_e_RPMPROB_DISKNODES => Self::DiskNodes,
            librpm_sys::rpmProblemType_e_RPMPROB_OBSOLETES => Self::Obsoletes,
            librpm_sys::rpmProblemType_e_RPMPROB_VERIFY => Self::Verify,
            _ => Self::Unknown,
        }
    }
}

/// A single transaction problem reported by librpm.
///
/// Each `Problem` owns a refcounted reference to the underlying
/// `rpmProblem`. Cloning increments the refcount via `rpmProblemLink`.
pub struct Problem {
    ptr: librpm_sys::rpmProblem,
}

impl Problem {
    pub(crate) unsafe fn from_ptr(ptr: librpm_sys::rpmProblem) -> Self {
        debug_assert!(!ptr.is_null());
        unsafe { librpm_sys::rpmProblemLink(ptr) };
        Self { ptr }
    }

    /// The type/classification of this problem.
    pub fn problem_type(&self) -> ProblemType {
        ProblemType::from_raw(unsafe { librpm_sys::rpmProblemGetType(self.ptr) })
    }

    /// NEVR of the primary package involved in this problem.
    pub fn package_nevr(&self) -> &str {
        let c_str = unsafe { librpm_sys::rpmProblemGetPkgNEVR(self.ptr) };
        if c_str.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or("")
    }

    /// NEVR of the other (conflicting/providing) package, if any.
    pub fn alt_nevr(&self) -> &str {
        let c_str = unsafe { librpm_sys::rpmProblemGetAltNEVR(self.ptr) };
        if c_str.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or("")
    }

    /// Additional description string (e.g. filename for file conflicts).
    pub fn description(&self) -> &str {
        let c_str = unsafe { librpm_sys::rpmProblemGetStr(self.ptr) };
        if c_str.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or("")
    }

    /// Disk space/inode need (for `DiskSpace`/`DiskNodes` problems).
    pub fn disk_need(&self) -> u64 {
        unsafe { librpm_sys::rpmProblemGetDiskNeed(self.ptr) }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c_str = unsafe { librpm_sys::rpmProblemString(self.ptr) };
        if c_str.is_null() {
            return write!(f, "(unknown problem)");
        }
        let msg = unsafe { CStr::from_ptr(c_str) }
            .to_str()
            .unwrap_or("(invalid UTF-8)");
        let result = write!(f, "{msg}");
        unsafe { free(c_str.cast()) };
        result
    }
}

impl fmt::Debug for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Problem")
            .field("type", &self.problem_type())
            .field("package", &self.package_nevr())
            .field("description", &self.description())
            .finish()
    }
}

impl Clone for Problem {
    fn clone(&self) -> Self {
        unsafe { librpm_sys::rpmProblemLink(self.ptr) };
        Self { ptr: self.ptr }
    }
}

impl Drop for Problem {
    fn drop(&mut self) {
        unsafe { librpm_sys::rpmProblemFree(self.ptr) };
    }
}

/// A set of transaction problems returned by
/// [`Transaction::run`](crate::transaction::Transaction::run) or
/// [`Transaction::check`](crate::transaction::Transaction::check).
///
/// Owns a refcounted `rpmps` handle; freed on drop.
pub struct Problems {
    ptr: librpm_sys::rpmps,
}

impl Problems {
    pub(crate) unsafe fn from_ptr(ptr: librpm_sys::rpmps) -> Self {
        Self { ptr }
    }

    /// Number of problems in the set.
    pub fn len(&self) -> usize {
        if self.ptr.is_null() {
            return 0;
        }
        unsafe { librpm_sys::rpmpsNumProblems(self.ptr) as usize }
    }

    /// Whether the problem set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over the problems.
    pub fn iter(&self) -> ProblemIter<'_> {
        let psi = if self.ptr.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { librpm_sys::rpmpsInitIterator(self.ptr) }
        };
        ProblemIter {
            psi,
            _marker: std::marker::PhantomData,
        }
    }
}

impl fmt::Display for Problems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, problem) in self.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{problem}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Problems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl Drop for Problems {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { librpm_sys::rpmpsFree(self.ptr) };
        }
    }
}

/// Iterator over [`Problem`] entries in a [`Problems`] set.
pub struct ProblemIter<'ps> {
    psi: librpm_sys::rpmpsi,
    _marker: std::marker::PhantomData<&'ps Problems>,
}

impl<'ps> Iterator for ProblemIter<'ps> {
    type Item = Problem;

    fn next(&mut self) -> Option<Problem> {
        if self.psi.is_null() {
            return None;
        }
        let prob = unsafe { librpm_sys::rpmpsiNext(self.psi) };
        if prob.is_null() {
            return None;
        }
        Some(unsafe { Problem::from_ptr(prob) })
    }
}

impl Drop for ProblemIter<'_> {
    fn drop(&mut self) {
        if !self.psi.is_null() {
            unsafe { librpm_sys::rpmpsFreeIterator(self.psi) };
        }
    }
}
