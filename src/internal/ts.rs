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
use std::sync::atomic::AtomicPtr;

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
            librpm_sys::rpmtsFree(*self.as_mut_ptr());
        }
    }
}

impl TransactionSet {
    pub(crate) fn as_mut_ptr(&mut self) -> &mut *mut librpm_sys::rpmts_s {
        self.0.get_mut()
    }
}

/// Crate-public wrapper for the global transaction set pointer.
///
/// Briefly acquires the global state lock to obtain the raw `rpmts` pointer,
/// then releases it. The pointer remains valid because the global
/// `TransactionSet` lives for the process lifetime, and librpm's
/// `rpmtsInitIterator` takes its own refcounted link to the `rpmts`.
pub(crate) struct GlobalTS(*mut librpm_sys::rpmts_s);

impl GlobalTS {
    /// Briefly acquire the global state lock and snapshot the transaction set pointer.
    pub fn create() -> Self {
        let mut state = GlobalState::lock();
        GlobalTS(*state.ts.as_mut_ptr())
    }

    /// Obtain the internal pointer to the transaction set
    pub(crate) fn as_mut_ptr(&mut self) -> *mut librpm_sys::rpmts_s {
        self.0
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
