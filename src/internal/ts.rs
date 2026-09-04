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

/// librpm transactions, a.k.a. "transaction sets" (or `rpmts` librpm type)
///
/// Nearly all access to librpm, including actions which don't necessarily
/// involve operations on the RPM database, require a transaction set.
///
/// # Ownership
///
/// Each `TransactionSet` owns exactly one refcounted reference to an
/// `rpmts`. When dropped, `rpmtsFree` decrements the refcount. The
/// underlying `rpmts` may outlive this wrapper if C-level consumers
/// (e.g. `rpmdbMatchIterator` via `rpmtsLink`) hold additional references.
///
/// # Thread safety
///
/// `TransactionSet` is `Send` but intentionally `!Sync`:
///
/// - **`Send`**: the `rpmts` is a self-contained heap allocation with no
///   thread-local state. Moving it between threads is safe.
/// - **`!Sync`**: `rpmtsInitIterator` performs lazy-init mutations (DB
///   open, keyring load) that are not synchronized internally. Concurrent
///   `&TransactionSet` access from multiple threads would be a data race.
pub(crate) struct TransactionSet(*mut librpm_sys::rpmts_s);

impl std::fmt::Debug for TransactionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TransactionSet").field(&self.0).finish()
    }
}

// Safety: see doc comment on TransactionSet above.
unsafe impl Send for TransactionSet {}

impl TransactionSet {
    /// Create a transaction set (i.e. begin a transaction)
    pub(crate) fn create() -> Self {
        // Safety: rpmtsCreate returns a valid, non-null transaction set pointer.
        let ts = unsafe { librpm_sys::rpmtsCreate() };
        // rpmtsCreate() leaves ts->rootDir NULL; setting it is the caller's job.
        // Python bindings always call rpmtsSetRootDir(ts, "/")) during initialization,
        // so we will do the same.

        // Without this step (which is documented to be required), segfaults
        // can be encountered in during `rpmtsRun()`.
        unsafe {
            if librpm_sys::rpmtsRootDir(ts).is_null() {
                librpm_sys::rpmtsSetRootDir(ts, c"/".as_ptr());
            }
        }
        TransactionSet(ts)
    }

    /// Obtain the raw pointer to the underlying `rpmts`.
    ///
    /// Takes `&self` because `rpmtsInitIterator` only performs lazy-init
    /// mutations (DB open, keyring load) that are safe in single-threaded
    /// use, and `TransactionSet` is `!Sync`.
    pub(crate) fn as_ptr(&self) -> *mut librpm_sys::rpmts_s {
        self.0
    }
}

impl Drop for TransactionSet {
    fn drop(&mut self) {
        // Safety: self.0 was created by rpmtsCreate and has not been freed.
        // The rpmts may still be alive after this call if iterators hold
        // additional references via rpmtsLink.
        // rpmtsFree -> rpmtsCloseDB -> rpmdbClose removes from the
        // process-global rpmdbRock linked list (RPM <= 4.18) without
        // synchronization. See docs/threading.md.
        let _lock = super::rpm_global_lock();
        unsafe {
            librpm_sys::rpmtsFree(self.0);
        }
    }
}
