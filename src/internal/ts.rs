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


//! Transaction sets: librpm's transaction API

use super::GlobalState;
use parking_lot::ReentrantMutexGuard;
use std::{cell::RefCell, sync::atomic::AtomicPtr};

/// librpm transactions, a.k.a. "transaction sets" (or `rpmts` librpm type)
///
/// Nearly all access to librpm, including actions which don't necessarily
/// involve operations on the RPM database, require a transaction set.
///
/// This library opens a single global transaction set on command, and all
/// operations which require one acquire it, use it, and then release it.
/// This allows us to keep them out of the public API.
pub(crate) struct TransactionSet(AtomicPtr<librpm_sys::rpmts_s>);

impl TransactionSet {
    /// Create a transaction set (i.e. begin a transaction)
    ///
    /// This is not intended to be invoked directly, but instead obtained
    /// from `GlobalState`.
    pub(crate) fn create() -> Self {
        TransactionSet(AtomicPtr::new(unsafe { librpm_sys::rpmtsCreate() }))
    }
}

impl Drop for TransactionSet {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::rpmtsFree(*self.0.get_mut());
        }
    }
}

impl TransactionSet {
    pub(crate) fn as_mut_ptr(&mut self) -> &mut *mut librpm_sys::rpmts_s {
        self.0.get_mut()
    }
}

/// Crate-public wrapper for acquiring and releasing the global transaction set
/// which also cleans it prior to unlocking it.
pub(crate) struct GlobalTS(ReentrantMutexGuard<'static, RefCell<GlobalState>>);

impl GlobalTS {
    /// Acquire the global state mutex, giving the current thread exclusive
    /// access to the global transaction set.
    pub fn create() -> Self {
        GlobalTS(GlobalState::lock())
    }

    /// Obtain the internal pointer to the transaction set
    pub(crate) fn as_mut_ptr(&mut self) -> *mut librpm_sys::rpmts_s {
        // Since we're guaranteed to be holding the GlobalState mutex here,
        // we're free to deref the pointer.
        *self.0.borrow_mut().ts.as_mut_ptr()
    }
}

/// Tidy up the shared global transaction set between uses
impl Drop for GlobalTS {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::rpmtsClean(self.as_mut_ptr());
        }
    }
}
