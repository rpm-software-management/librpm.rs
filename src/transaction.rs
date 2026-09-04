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

//! RPM transaction support: install, upgrade, and erase packages
//!
//! A `Transaction` is created from a [`Db`](crate::Db) via
//! [`Db::transaction()`](crate::Db::transaction). It borrows the `Db`
//! exclusively (`&mut`), preventing new queries from being started through the
//! `Db` while a transaction is active. Existing database iterators are
//! independent of the `Db` and may remain alive.
//!
//! # Lifecycle
//!
//! 1. Create the transaction: `let mut txn = db.transaction();`
//! 2. Add elements: `add_install`, `add_erase`, etc.
//! 3. Optionally `check` dependencies.
//! 4. Optionally `order` the elements.
//! 5. `run` the transaction.
//!
//! On drop, the transaction cleans up all elements via `rpmtsEmpty` and
//! restores the original transaction flags, verification flags, and keyring.
//!
//! # Thread safety
//!
//! `Transaction::run` acquires the process-wide `mutation_lock()` and
//! RPM's cross-process `.rpm.lock` (via `rpmtxnBegin`/`rpmtxnEnd`).
//! See `docs/locking.md` for details.

use std::ffi::{CStr, CString, c_void};
use std::marker::PhantomData;
use std::os::unix::ffi::OsStrExt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::{fmt, ptr};

use crate::db::Db;
use crate::error::Error;
use crate::internal::{mutation_lock, rpm_global_lock};
use crate::package::PackageHeader;
use crate::problem::Problems;
use crate::verify::{VerificationFlags, VerifyOptions};

#[cfg(not(has_rpmts_set_notify_style))]
unsafe extern "C" {
    fn free(ptr: *mut c_void);
}

/// Flags controlling transaction execution.
///
/// These correspond to `rpmtransFlags_e` in librpm. Combine with `|`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct TransactionFlags(u32);

impl TransactionFlags {
    /// No special flags.
    pub const NONE: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_NONE as u32);
    /// Perform a dry run: check and report problems without modifying the system.
    pub const TEST: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_TEST as u32);
    /// Build the problem set without aborting on the first error.
    pub const BUILD_PROBS: Self =
        Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_BUILD_PROBS as u32);
    /// Do not execute package scriptlets.
    pub const NOSCRIPTS: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_NOSCRIPTS as u32);
    /// Only update the database, do not modify the filesystem.
    pub const JUSTDB: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_JUSTDB as u32);
    /// Do not execute trigger scriptlets.
    pub const NOTRIGGERS: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_NOTRIGGERS as u32);
    /// Do not install documentation files.
    pub const NODOCS: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_NODOCS as u32);
    /// Install all files, even configuration files that were modified.
    pub const ALLFILES: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_ALLFILES as u32);
    /// Do not run plugins.
    pub const NOPLUGINS: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_NOPLUGINS as u32);
    /// Do not install/update SELinux file contexts.
    pub const NOCONTEXTS: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_NOCONTEXTS as u32);
    /// Do not set file capabilities.
    pub const NOCAPS: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_NOCAPS as u32);
    /// Do not update the database (filesystem-only).
    #[cfg(has_rpmtransflag_nodb)]
    pub const NODB: Self = Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_NODB as u32);
    /// Skip file digest verification during install.
    pub const NOFILEDIGEST: Self =
        Self(librpm_sys::rpmtransFlags_e_RPMTRANS_FLAG_NOFILEDIGEST as u32);

    /// Return the raw bits.
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for TransactionFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for TransactionFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Filter flags that tell `rpmtsRun` to ignore certain problem types.
///
/// These correspond to `rpmprobFilterFlags_e` in librpm. Combine with `|`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct ProblemFilter(u32);

impl ProblemFilter {
    /// No filtering — report all problems.
    pub const NONE: Self = Self(librpm_sys::rpmprobFilterFlags_e_RPMPROB_FILTER_NONE);
    /// Ignore OS incompatibility problems.
    pub const IGNORE_OS: Self = Self(librpm_sys::rpmprobFilterFlags_e_RPMPROB_FILTER_IGNOREOS);
    /// Ignore architecture incompatibility problems.
    pub const IGNORE_ARCH: Self = Self(librpm_sys::rpmprobFilterFlags_e_RPMPROB_FILTER_IGNOREARCH);
    /// Allow replacing an already-installed package.
    pub const REPLACE_PKG: Self = Self(librpm_sys::rpmprobFilterFlags_e_RPMPROB_FILTER_REPLACEPKG);
    /// Allow replacing files owned by other packages (new files).
    pub const REPLACE_NEW_FILES: Self =
        Self(librpm_sys::rpmprobFilterFlags_e_RPMPROB_FILTER_REPLACENEWFILES);
    /// Allow replacing files owned by other packages (old files).
    pub const REPLACE_OLD_FILES: Self =
        Self(librpm_sys::rpmprobFilterFlags_e_RPMPROB_FILTER_REPLACEOLDFILES);
    /// Allow installing an older version over a newer one.
    pub const OLD_PACKAGE: Self = Self(librpm_sys::rpmprobFilterFlags_e_RPMPROB_FILTER_OLDPACKAGE);
    /// Ignore disk space problems.
    pub const DISK_SPACE: Self = Self(librpm_sys::rpmprobFilterFlags_e_RPMPROB_FILTER_DISKSPACE);
    /// Ignore disk inode problems.
    pub const DISK_NODES: Self = Self(librpm_sys::rpmprobFilterFlags_e_RPMPROB_FILTER_DISKNODES);

    /// Return the raw bits.
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for ProblemFilter {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ProblemFilter {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Type of a transaction element.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ElementType {
    /// Package to be installed or upgraded.
    Install,
    /// Package to be erased.
    Erase,
    /// Package from the rpmdb (used internally for triggers and verify scripts).
    Rpmdb,
    /// Package to be restored from the database.
    #[cfg(has_rpmelementtype_tr_restored)]
    Restore,
    /// Unrecognized element type from a newer librpm version.
    Unknown,
}

impl ElementType {
    fn from_raw(raw: librpm_sys::rpmElementType_e) -> Self {
        match raw {
            librpm_sys::rpmElementType_e_TR_ADDED => Self::Install,
            librpm_sys::rpmElementType_e_TR_REMOVED => Self::Erase,
            librpm_sys::rpmElementType_e_TR_RPMDB => Self::Rpmdb,
            #[cfg(has_rpmelementtype_tr_restored)]
            librpm_sys::rpmElementType_e_TR_RESTORED => Self::Restore,
            _ => Self::Unknown,
        }
    }
}

/// A transaction element: a single package install, erase, or restore within a [`Transaction`].
///
/// Elements borrow the transaction's internal state and are valid only while the transaction is
/// alive.
pub struct Element<'txn> {
    ptr: librpm_sys::rpmte,
    _marker: PhantomData<&'txn Transaction<'txn>>,
}

impl<'txn> Element<'txn> {
    /// The type of operation for this element.
    pub fn element_type(&self) -> ElementType {
        ElementType::from_raw(unsafe { librpm_sys::rpmteType(self.ptr) })
    }

    /// Package name.
    pub fn name(&self) -> &str {
        let c_str = unsafe { librpm_sys::rpmteN(self.ptr) };
        if c_str.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or("")
    }

    /// Package epoch (as a string), or `None` if unset.
    pub fn epoch(&self) -> Option<&str> {
        let c_str = unsafe { librpm_sys::rpmteE(self.ptr) };
        if c_str.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or(""))
    }

    /// Package version.
    pub fn version(&self) -> &str {
        let c_str = unsafe { librpm_sys::rpmteV(self.ptr) };
        if c_str.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or("")
    }

    /// Package release.
    pub fn release(&self) -> &str {
        let c_str = unsafe { librpm_sys::rpmteR(self.ptr) };
        if c_str.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or("")
    }

    /// Package architecture.
    pub fn arch(&self) -> &str {
        let c_str = unsafe { librpm_sys::rpmteA(self.ptr) };
        if c_str.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or("")
    }

    /// Full NEVR string (name-[epoch:]version-release).
    pub fn nevr(&self) -> &str {
        let c_str = unsafe { librpm_sys::rpmteNEVR(self.ptr) };
        if c_str.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or("")
    }

    /// Full NEVRA string (name-[epoch:]version-release.arch).
    pub fn nevra(&self) -> &str {
        let c_str = unsafe { librpm_sys::rpmteNEVRA(self.ptr) };
        if c_str.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().unwrap_or("")
    }

    /// Size of the package file in bytes.
    pub fn pkg_file_size(&self) -> u64 {
        unsafe { librpm_sys::rpmtePkgFileSize(self.ptr) }
    }

    /// Database offset (record number) for erase operations, or `0`.
    pub fn db_offset(&self) -> i32 {
        unsafe { librpm_sys::rpmteDBOffset(self.ptr) }
    }

    /// Whether this element failed during transaction execution.
    pub fn failed(&self) -> bool {
        unsafe { librpm_sys::rpmteFailed(self.ptr) != 0 }
    }

    /// Problems specific to this element.
    pub fn problems(&self) -> Problems {
        let ps = unsafe { librpm_sys::rpmteProblems(self.ptr) };
        unsafe { Problems::from_ptr(ps) }
    }
}

impl fmt::Debug for Element<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Element")
            .field("type", &self.element_type())
            .field("nevra", &self.nevra())
            .field("failed", &self.failed())
            .finish()
    }
}

/// Iterator over transaction elements.
pub struct ElementIter<'txn> {
    tsi: librpm_sys::rpmtsi,
    _marker: PhantomData<&'txn Transaction<'txn>>,
}

impl<'txn> Iterator for ElementIter<'txn> {
    type Item = Element<'txn>;

    fn next(&mut self) -> Option<Element<'txn>> {
        if self.tsi.is_null() {
            return None;
        }
        // type 0 = iterate all element types
        let te = unsafe { librpm_sys::rpmtsiNext(self.tsi, 0) };
        if te.is_null() {
            return None;
        }
        Some(Element {
            ptr: te,
            _marker: PhantomData,
        })
    }
}

impl Drop for ElementIter<'_> {
    fn drop(&mut self) {
        if !self.tsi.is_null() {
            unsafe { librpm_sys::rpmtsiFree(self.tsi) };
        }
    }
}

/// Error returned when a transaction fails.
///
/// Contains the problem set describing what went wrong.
#[derive(Debug)]
pub struct TransactionError {
    problems: Problems,
}

impl TransactionError {
    /// The problems that caused the transaction to fail.
    pub fn problems(&self) -> &Problems {
        &self.problems
    }

    /// Consume this error and return the problem set.
    pub fn into_problems(self) -> Problems {
        self.problems
    }
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "transaction failed: {}", self.problems)
    }
}

impl std::error::Error for TransactionError {}

/// A progress or status event fired during transaction execution.
///
/// Passed to the callback set via [`Transaction::set_callback`]. Progress variants carry
/// `amount` (bytes/packages processed so far) and `total` (total bytes/packages).
/// Start/stop variants carry the NEVRA of the package involved. Error variants carry
/// available context.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CallbackEvent {
    /// Install progress: `amount` of `total` bytes processed.
    InstProgress {
        /// Bytes processed so far.
        amount: u64,
        /// Total bytes to process.
        total: u64,
    },
    /// Install started for a package.
    InstStart {
        /// Package NEVRA string.
        nevra: String,
    },
    /// Install completed for a package.
    InstStop {
        /// Package NEVRA string.
        nevra: String,
    },
    /// Uninstall progress: `amount` of `total` bytes processed.
    UninstProgress {
        /// Bytes processed so far.
        amount: u64,
        /// Total bytes to process.
        total: u64,
    },
    /// Uninstall started for a package.
    UninstStart {
        /// Package NEVRA string.
        nevra: String,
    },
    /// Uninstall completed for a package.
    UninstStop {
        /// Package NEVRA string.
        nevra: String,
    },
    /// Overall transaction progress: `amount` of `total` packages processed.
    TransProgress {
        /// Packages processed so far.
        amount: u64,
        /// Total packages to process.
        total: u64,
    },
    /// Transaction processing started.
    TransStart {
        /// Total packages to process.
        total: u64,
    },
    /// Transaction processing completed.
    TransStop {
        /// Total packages processed.
        total: u64,
    },
    /// Per-element progress: `amount` of `total` bytes processed.
    ElemProgress {
        /// Bytes processed so far.
        amount: u64,
        /// Total bytes to process.
        total: u64,
    },
    /// Verify progress: `amount` of `total` packages verified.
    VerifyProgress {
        /// Packages verified so far.
        amount: u64,
        /// Total packages to verify.
        total: u64,
    },
    /// Verify phase started.
    VerifyStart {
        /// Total packages to verify.
        total: u64,
    },
    /// Verify phase completed.
    VerifyStop {
        /// Total packages verified.
        total: u64,
    },
    /// Scriptlet execution started.
    ScriptStart {
        /// Package NEVRA string.
        nevra: String,
        /// RPM tag of the scriptlet being executed.
        tag: u64,
    },
    /// Scriptlet execution completed.
    ScriptStop {
        /// Package NEVRA string.
        nevra: String,
        /// Scriptlet exit status.
        return_code: u64,
    },
    /// Scriptlet execution error.
    ScriptError {
        /// Package NEVRA string.
        nevra: String,
    },
    /// Package unpack error.
    UnpackError {
        /// Package NEVRA string.
        nevra: String,
    },
    /// CPIO payload error.
    CpioError {
        /// Package NEVRA string.
        nevra: String,
    },
    /// File opened for package install (internal I/O).
    InstOpenFile {
        /// Package NEVRA string.
        nevra: String,
    },
    /// File closed after package install (internal I/O).
    InstCloseFile {
        /// Package NEVRA string.
        nevra: String,
    },
}

struct CallbackState<'a> {
    user_callback: Option<Box<dyn FnMut(CallbackEvent) + 'a>>,
    open_fd: Option<librpm_sys::FD_t>,
    callback_panic: Option<Box<dyn std::any::Any + Send>>,
}

impl CallbackState<'_> {
    fn call(&mut self, event: CallbackEvent) {
        if self.callback_panic.is_some() {
            return;
        }
        if let Some(callback) = &mut self.user_callback {
            if let Err(panic) = catch_unwind(AssertUnwindSafe(|| callback(event))) {
                self.callback_panic = Some(panic);
            }
        }
    }
}

/// Extract NEVRA from callback's `h` parameter (style 1: rpmte pointer, RPM >= 4.17).
#[cfg(has_rpmts_set_notify_style)]
fn nevra_from_callback(h: *const c_void) -> String {
    if h.is_null() {
        return String::new();
    }
    let c_str = unsafe { librpm_sys::rpmteNEVRA(h as librpm_sys::rpmte) };
    if c_str.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(c_str) }
        .to_str()
        .unwrap_or("")
        .to_owned()
}

/// Extract NEVRA from callback's `h` parameter (style 0: Header pointer, RPM < 4.17).
#[cfg(not(has_rpmts_set_notify_style))]
fn nevra_from_callback(h: *const c_void) -> String {
    if h.is_null() {
        return String::new();
    }
    let c_str = unsafe {
        librpm_sys::headerFormat(
            h as librpm_sys::Header,
            c"%{NEVRA}".as_ptr(),
            ptr::null_mut(),
        )
    };
    if c_str.is_null() {
        return String::new();
    }
    let result = unsafe { CStr::from_ptr(c_str) }
        .to_str()
        .unwrap_or("")
        .to_owned();
    unsafe { free(c_str as *mut c_void) };
    result
}

unsafe extern "C" fn callback_trampoline(
    h: *const c_void,
    what: librpm_sys::rpmCallbackType,
    amount: librpm_sys::rpm_loff_t,
    total: librpm_sys::rpm_loff_t,
    key: librpm_sys::fnpyKey,
    data: librpm_sys::rpmCallbackData,
) -> *mut c_void {
    if data.is_null() {
        return ptr::null_mut();
    }

    let state = unsafe { &mut *(data as *mut CallbackState) };

    if what == librpm_sys::rpmCallbackType_e_RPMCALLBACK_INST_OPEN_FILE {
        if key.is_null() {
            return ptr::null_mut();
        }
        let fd = unsafe { librpm_sys::Fopen(key as *const _, c"r.ufdio".as_ptr()) };
        if fd.is_null() || unsafe { librpm_sys::Ferror(fd) } != 0 {
            if !fd.is_null() {
                unsafe { librpm_sys::Fclose(fd) };
            }
            return ptr::null_mut();
        }
        state.open_fd = Some(fd);
        state.call(CallbackEvent::InstOpenFile {
            nevra: nevra_from_callback(h),
        });
        return fd as *mut c_void;
    }

    if what == librpm_sys::rpmCallbackType_e_RPMCALLBACK_INST_CLOSE_FILE {
        if let Some(fd) = state.open_fd.take() {
            unsafe { librpm_sys::Fclose(fd) };
        }
        state.call(CallbackEvent::InstCloseFile {
            nevra: nevra_from_callback(h),
        });
        return ptr::null_mut();
    }

    let event = match what {
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_INST_PROGRESS => {
            CallbackEvent::InstProgress { amount, total }
        }
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_INST_START => CallbackEvent::InstStart {
            nevra: nevra_from_callback(h),
        },
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_INST_STOP => CallbackEvent::InstStop {
            nevra: nevra_from_callback(h),
        },
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_UNINST_PROGRESS => {
            CallbackEvent::UninstProgress { amount, total }
        }
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_UNINST_START => CallbackEvent::UninstStart {
            nevra: nevra_from_callback(h),
        },
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_UNINST_STOP => CallbackEvent::UninstStop {
            nevra: nevra_from_callback(h),
        },
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_TRANS_PROGRESS => {
            CallbackEvent::TransProgress { amount, total }
        }
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_TRANS_START => {
            CallbackEvent::TransStart { total }
        }
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_TRANS_STOP => CallbackEvent::TransStop { total },
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_ELEM_PROGRESS => {
            CallbackEvent::ElemProgress { amount, total }
        }
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_VERIFY_PROGRESS => {
            CallbackEvent::VerifyProgress { amount, total }
        }
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_VERIFY_START => {
            CallbackEvent::VerifyStart { total }
        }
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_VERIFY_STOP => {
            CallbackEvent::VerifyStop { total }
        }
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_SCRIPT_START => CallbackEvent::ScriptStart {
            nevra: nevra_from_callback(h),
            tag: amount,
        },
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_SCRIPT_STOP => CallbackEvent::ScriptStop {
            nevra: nevra_from_callback(h),
            return_code: total,
        },
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_SCRIPT_ERROR => CallbackEvent::ScriptError {
            nevra: nevra_from_callback(h),
        },
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_UNPACK_ERROR => CallbackEvent::UnpackError {
            nevra: nevra_from_callback(h),
        },
        librpm_sys::rpmCallbackType_e_RPMCALLBACK_CPIO_ERROR => CallbackEvent::CpioError {
            nevra: nevra_from_callback(h),
        },
        _ => return ptr::null_mut(),
    };

    state.call(event);

    ptr::null_mut()
}

/// An RPM transaction for installing, upgrading, and erasing packages.
///
/// Created via [`Db::transaction()`](crate::Db::transaction). Borrows the `Db` exclusively
/// — all queries must be completed before creating the transaction.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use librpm::{Db, PackageHeader};
/// use librpm::transaction::{TransactionFlags, ProblemFilter};
///
/// librpm::init().unwrap();
/// let mut db = Db::open().unwrap();
///
/// // Load a package from a file
/// let path = Path::new("/path/to/package.rpm");
/// let pkg = PackageHeader::from_file(path, None).unwrap();
///
/// // Create a transaction and add the package as an upgrade
/// let mut txn = db.transaction();
/// txn.add_install(&pkg, path, true).unwrap();
/// txn.set_flags(TransactionFlags::TEST);
///
/// // Dry-run: check and report problems without modifying the system
/// txn.check().unwrap();
/// txn.order().unwrap();
/// txn.run().unwrap();
/// ```
pub struct Transaction<'db> {
    ts: *mut librpm_sys::rpmts_s,
    saved_flags: u32,
    saved_verification_flags: u32,
    saved_keyring: librpm_sys::rpmKeyring,
    problem_filter: ProblemFilter,
    callback_state: Box<CallbackState<'db>>,
    paths: Vec<CString>,
    _marker: PhantomData<&'db mut Db>,
}

impl<'db> Transaction<'db> {
    pub(crate) fn new(db: &'db mut Db) -> Self {
        let ts = db.ts_ptr();
        let saved_flags = unsafe { librpm_sys::rpmtsFlags(ts) };
        let saved_verification_flags = unsafe { librpm_sys::rpmtsVSFlags(ts) };
        // rpmtsGetKeyring(ts, 0) does not auto-load the system keyring and
        // returns an owned reference when a keyring is already configured.
        let saved_keyring = {
            // Keep transaction-set initialization consistent with other RPM
            // calls that may touch global database/iterator state. See
            // docs/locking.md.
            let _lock = rpm_global_lock();
            unsafe { librpm_sys::rpmtsGetKeyring(ts, 0) }
        };
        let mut callback_state = Box::new(CallbackState {
            user_callback: None,
            open_fd: None,
            callback_panic: None,
        });
        let data_ptr: *mut c_void = {
            let ptr: *mut CallbackState = &mut *callback_state;
            ptr as *mut c_void
        };
        unsafe {
            // Style 1 (RPM >= 4.17) passes rpmte to callbacks; style 0 passes Header.
            // The trampoline handles both via cfg-gated nevra_from_callback().
            #[cfg(has_rpmts_set_notify_style)]
            librpm_sys::rpmtsSetNotifyStyle(ts, 1);
            librpm_sys::rpmtsSetNotifyCallback(ts, Some(callback_trampoline), data_ptr);
        }
        Self {
            ts,
            saved_flags,
            saved_verification_flags,
            saved_keyring,
            problem_filter: ProblemFilter::NONE,
            callback_state,
            paths: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Add a package to be installed or upgraded.
    ///
    /// `path` must point to the `.rpm` file on disk — it is used to open the package payload
    /// during [`run()`](Transaction::run). If `upgrade` is true, the package replaces any
    /// older version already installed.
    pub fn add_install(
        &mut self,
        pkg: &PackageHeader,
        path: &Path,
        upgrade: bool,
    ) -> Result<(), Error> {
        let header = pkg.header_ptr();
        let c_path = CString::new(path.as_os_str().as_bytes()).unwrap();
        let key = c_path.as_ptr() as librpm_sys::fnpyKey;
        self.paths.push(c_path);
        let upgrade_flag = if upgrade { 1 } else { 0 };
        // rpmtsAddInstallElement -> addPackage internally opens the database
        // (rpmtsOpenDB) and creates match iterators to resolve upgrades/obsoletes,
        // mutating the RPM <= 4.18 global tracking lists. See docs/locking.md.
        let _lock = rpm_global_lock();
        let rc = unsafe {
            librpm_sys::rpmtsAddInstallElement(self.ts, header, key, upgrade_flag, ptr::null_mut())
        };
        if rc != 0 {
            fail!(
                crate::error::ErrorKind::Transaction,
                "failed to add install element"
            );
        }
        Ok(())
    }

    /// Add a package to be reinstalled (same version).
    ///
    /// `path` must point to the `.rpm` file on disk — it is used to open the package payload
    /// during [`run()`](Transaction::run).
    pub fn add_reinstall(&mut self, pkg: &PackageHeader, path: &Path) -> Result<(), Error> {
        let header = pkg.header_ptr();
        let c_path = CString::new(path.as_os_str().as_bytes()).unwrap();
        let key = c_path.as_ptr() as librpm_sys::fnpyKey;
        self.paths.push(c_path);
        // See add_install: addPackage opens the DB and creates iterators internally.
        let _lock = rpm_global_lock();
        let rc = unsafe { librpm_sys::rpmtsAddReinstallElement(self.ts, header, key) };
        if rc != 0 {
            fail!(
                crate::error::ErrorKind::Transaction,
                "failed to add reinstall element"
            );
        }
        Ok(())
    }

    /// Add a package to be erased (uninstalled).
    ///
    /// The package must be identified by its database offset (record number),
    /// which can be obtained from a database query via [`Iter::offset()`](crate::db::Iter::offset).
    pub fn add_erase(&mut self, pkg: &PackageHeader) -> Result<(), Error> {
        let header = pkg.header_ptr();
        // rpmtsAddEraseElement -> removePackage internally creates match iterators
        // over the database, mutating the RPM <= 4.18 global tracking lists.
        let _lock = rpm_global_lock();
        let rc = unsafe { librpm_sys::rpmtsAddEraseElement(self.ts, header, -1) };
        if rc != 0 {
            fail!(
                crate::error::ErrorKind::Transaction,
                "failed to add erase element"
            );
        }
        Ok(())
    }

    /// Add a package to be restored from the database.
    #[cfg(has_rpmelementtype_tr_restored)]
    pub fn add_restore(&mut self, pkg: &PackageHeader) -> Result<(), Error> {
        let header = pkg.header_ptr();
        // See add_install: element addition opens the DB / creates iterators internally.
        let _lock = rpm_global_lock();
        let rc = unsafe { librpm_sys::rpmtsAddRestoreElement(self.ts, header) };
        if rc != 0 {
            fail!(
                crate::error::ErrorKind::Transaction,
                "failed to add restore element"
            );
        }
        Ok(())
    }

    /// Set transaction flags controlling execution behavior.
    pub fn set_flags(&mut self, flags: TransactionFlags) {
        unsafe { librpm_sys::rpmtsSetFlags(self.ts, flags.bits()) };
    }

    /// Get the current transaction flags.
    pub fn flags(&self) -> TransactionFlags {
        TransactionFlags(unsafe { librpm_sys::rpmtsFlags(self.ts) })
    }

    /// Set the signature and digest verification flags used while processing
    /// package files during this transaction.
    pub fn set_verification_flags(&mut self, flags: VerificationFlags) {
        unsafe { librpm_sys::rpmtsSetVSFlags(self.ts, flags.bits()) };
    }

    /// Get the signature and digest verification flags for this transaction.
    pub fn verification_flags(&self) -> VerificationFlags {
        VerificationFlags::from_bits(unsafe { librpm_sys::rpmtsVSFlags(self.ts) })
    }

    /// Configure package verification for this transaction.
    ///
    /// This applies both the signature/digest verification flags and the
    /// optional keyring from `options`. If no keyring is configured, RPM will
    /// load the system keyring when needed.
    pub fn set_verify_options(&mut self, options: &VerifyOptions) {
        unsafe {
            librpm_sys::rpmtsSetVSFlags(self.ts, options.flags.bits());
            let keyring = options
                .keyring
                .as_ref()
                .map_or(ptr::null_mut(), |keyring| keyring.as_ptr());
            librpm_sys::rpmtsSetKeyring(self.ts, keyring);
        }
    }

    /// Set the problem filter for [`run`](Transaction::run).
    pub fn set_problem_filter(&mut self, filter: ProblemFilter) {
        self.problem_filter = filter;
    }

    /// Set a progress callback for transaction execution.
    ///
    /// The callback receives [`CallbackEvent`] values as the transaction progresses through
    /// install, erase, verify, and script phases.
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: FnMut(CallbackEvent) + 'db,
    {
        self.callback_state.user_callback = Some(Box::new(callback));
    }

    /// Remove the progress callback, if any.
    ///
    /// The internal OPEN_FILE/CLOSE_FILE handler remains active.
    pub fn clear_callback(&mut self) {
        self.callback_state.user_callback = None;
    }

    /// Check dependency and conflict problems without executing.
    ///
    /// Returns `Ok(())` if no problems are found, or a
    /// [`TransactionError`] with the problem set.
    pub fn check(&mut self) -> Result<(), TransactionError> {
        // rpmtsCheck opens the database (rpmtsOpenDB) and creates many match
        // iterators to resolve dependencies, mutating the RPM <= 4.18 global
        // tracking lists (rpmdbRock, rpmmiRock). See docs/locking.md.
        let lock = rpm_global_lock();
        let rc = unsafe { librpm_sys::rpmtsCheck(self.ts) };
        drop(lock);
        if rc != 0 {
            let ps = unsafe { librpm_sys::rpmtsProblems(self.ts) };
            let problems = unsafe { Problems::from_ptr(ps) };
            return Err(TransactionError { problems });
        }
        // rpmtsCheck on RPM 6.0+ always returns 0; check the problem set directly.
        let ps = unsafe { librpm_sys::rpmtsProblems(self.ts) };
        let problems = unsafe { Problems::from_ptr(ps) };
        if !problems.is_empty() {
            return Err(TransactionError { problems });
        }
        Ok(())
    }

    /// Compute the installation/erasure ordering.
    ///
    /// Returns the number of unorderable elements (0 means fully ordered).
    pub fn order(&mut self) -> Result<usize, Error> {
        let rc = unsafe { librpm_sys::rpmtsOrder(self.ts) };
        if rc < 0 {
            fail!(
                crate::error::ErrorKind::Transaction,
                "failed to order transaction elements"
            );
        }
        Ok(rc as usize)
    }

    /// Execute the transaction.
    ///
    /// Acquires the process-wide `mutation_lock` and RPM's cross-process `.rpm.lock` before
    /// executing. On failure, returns a [`TransactionError`] containing the problem set.
    /// The user callback, if configured, runs while these locks are held and must not call back
    /// into librpm operations that acquire them. If the callback panics, the panic is caught while
    /// inside librpm and resumed after RPM has returned and its locks have been released.
    ///
    /// **Warning:** `rpmtsRun` mutates the transaction flags internally (expanding `NOSCRIPTS`
    /// into individual sub-flags). This wrapper saves and restores the flags around each call.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::path::Path;
    /// # use librpm::{Db, PackageHeader, VerifyOptions};
    /// # use librpm::transaction::TransactionFlags;
    /// # librpm::init().unwrap();
    /// # let mut db = Db::open().unwrap();
    /// # let path = Path::new("package.rpm");
    /// # let package = PackageHeader::from_file(
    /// #     path,
    /// #     Some(&VerifyOptions::skip_verification()),
    /// # ).unwrap();
    /// let mut transaction = db.transaction();
    /// transaction.add_install(&package, path, false).unwrap();
    /// transaction.set_flags(TransactionFlags::TEST);
    /// transaction.run().unwrap();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn run(&mut self) -> Result<(), TransactionError> {
        let _mutation = mutation_lock();
        // rpmtsRun -> rpmtsSetup/rpmtsPrepare opens the database and creates
        // match iterators (fingerprinting, conflict checks) throughout its
        // execution, mutating the RPM <= 4.18 global tracking lists. Hold
        // rpm_global_lock for the whole run. Lock ordering is always
        // mutation_lock first, then rpm_global_lock. See docs/locking.md.
        let _global = rpm_global_lock();

        // Acquire cross-process lock
        let txn =
            unsafe { librpm_sys::rpmtxnBegin(self.ts, librpm_sys::rpmtxnFlags_e_RPMTXN_WRITE) };
        if txn.is_null() {
            return Err(TransactionError {
                problems: unsafe { Problems::from_ptr(ptr::null_mut()) },
            });
        }

        // Save flags — rpmtsRun mutates them (expands NOSCRIPTS into sub-flags)
        let pre_run_flags = unsafe { librpm_sys::rpmtsFlags(self.ts) };

        let rc =
            unsafe { librpm_sys::rpmtsRun(self.ts, ptr::null_mut(), self.problem_filter.bits()) };

        // Restore flags
        unsafe { librpm_sys::rpmtsSetFlags(self.ts, pre_run_flags) };

        let callback_panic = self.callback_state.callback_panic.take();

        // Release cross-process lock
        unsafe { librpm_sys::rpmtxnEnd(txn) };

        if let Some(panic) = callback_panic {
            std::panic::resume_unwind(panic);
        }

        if rc != 0 {
            let ps = unsafe { librpm_sys::rpmtsProblems(self.ts) };
            let problems = unsafe { Problems::from_ptr(ps) };
            return Err(TransactionError { problems });
        }

        Ok(())
    }

    /// Get the current problem set (may be non-empty after `check` or `run`).
    pub fn problems(&self) -> Problems {
        let ps = unsafe { librpm_sys::rpmtsProblems(self.ts) };
        unsafe { Problems::from_ptr(ps) }
    }

    /// Number of elements in this transaction.
    pub fn len(&self) -> usize {
        unsafe { librpm_sys::rpmtsNElements(self.ts) as usize }
    }

    /// Whether the transaction has no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over the transaction elements.
    pub fn elements(&self) -> ElementIter<'_> {
        let tsi = unsafe { librpm_sys::rpmtsiInit(self.ts) };
        ElementIter {
            tsi,
            _marker: PhantomData,
        }
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        // Restoring the transaction set and emptying its elements can release
        // RPM objects tracked in process-global state. See docs/locking.md.
        let _lock = rpm_global_lock();
        unsafe {
            librpm_sys::rpmtsSetNotifyCallback(self.ts, None, ptr::null_mut());
        }
        if let Some(fd) = self.callback_state.open_fd.take() {
            unsafe { librpm_sys::Fclose(fd) };
        }
        self.callback_state.user_callback = None;
        unsafe {
            librpm_sys::rpmtsSetFlags(self.ts, self.saved_flags);
            librpm_sys::rpmtsSetVSFlags(self.ts, self.saved_verification_flags);
            librpm_sys::rpmtsSetKeyring(self.ts, self.saved_keyring);
            librpm_sys::rpmKeyringFree(self.saved_keyring);
            librpm_sys::rpmtsEmpty(self.ts);
        }
    }
}
