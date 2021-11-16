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

//! Iterators for matches in the RPM database

use super::{header::Header, tag::DBIndexTag, ts::GlobalTS};
use std::{os::raw::c_void, ptr};
use streaming_iterator::StreamingIterator;

/// Iterator over the matches from a database query.
///
/// # Locking
///
/// The global state lock is held only during construction (to safely access
/// the global `rpmts` for `rpmtsInitIterator`). It is *not* held for the
/// iterator's lifetime. This is safe because:
///
/// - `rpmtsInitIterator` takes its own refcounted link (`rpmtsLink`) to the
///   transaction set, keeping it alive independently of our lock.
/// - Index query results are snapshotted into a `dbiIndexSet` at creation
///   time; iteration walks this snapshot, not a live cursor on the index.
/// - Each iterator has its own `rpmdbMatchIterator` / database cursor.
/// - During iteration, the only access back to the `rpmts` is read-only
///   header signature verification (`mi_hdrchk`).
///
/// This design allows multiple iterators to coexist on the same thread
/// without deadlocking (see issue #15).
pub(crate) struct MatchIterator {
    /// Pointer to librpm's match iterator.
    ptr: *mut librpm_sys::rpmdbMatchIterator_s,

    /// Next item in the iterator
    next_item: Option<Header>,

    /// Have we already finished iterating?
    finished: bool,
}

impl MatchIterator {
    /// Create a new `MatchIterator` for the current RPM database, searching
    /// by the (optionally) given search key.
    pub(crate) fn new(tag: DBIndexTag, key_opt: Option<&str>) -> Self {
        let mut txn = GlobalTS::create();
        let next_item = None;
        let finished = false;

        if let Some(key) = key_opt
            && !key.is_empty()
        {
            let ptr = unsafe {
                librpm_sys::rpmtsInitIterator(
                    txn.as_mut_ptr(),
                    tag as librpm_sys::rpm_tag_t,
                    key.as_ptr() as *const c_void,
                    key.len(),
                )
            };

            return Self {
                ptr,
                next_item,
                finished,
            };
        }

        let ptr = unsafe {
            librpm_sys::rpmtsInitIterator(
                txn.as_mut_ptr(),
                tag as librpm_sys::rpm_tag_t,
                ptr::null(),
                0,
            )
        };

        Self {
            ptr,
            next_item,
            finished,
        }
    }
}

/// Use a StreamingIterator to ensure that headers do not outlive `rpmdbNextIterator` calls.
impl StreamingIterator for MatchIterator {
    type Item = Header;

    fn advance(&mut self) {
        // Underlying rpmdb iterator has been consumed
        if self.finished {
            return;
        }

        let header_ptr = unsafe { librpm_sys::rpmdbNextIterator(self.ptr) };

        if header_ptr.is_null() {
            self.finished = true;
            self.next_item = None;
        } else {
            self.next_item = Some(unsafe { Header::from_ptr(header_ptr) })
        }
    }

    fn get(&self) -> Option<&Header> {
        self.next_item.as_ref()
    }
}

impl Drop for MatchIterator {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::rpmdbFreeIterator(self.ptr);
        }
    }
}
