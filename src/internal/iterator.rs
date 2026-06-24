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

use super::{header::Header, rpm_global_lock, tag::DBIndexTag};
use std::{ffi::CString, os::raw::c_void, ptr};
use streaming_iterator::StreamingIterator;

/// Match mode for `rpmdbSetIteratorRE`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum MireMode {
    Glob,
    Regex,
}

/// Iterator over the matches from a database query.
///
/// # Lifetime independence from `Db`
///
/// A `MatchIterator` does not borrow the `Db` or `TransactionSet` that
/// created it. This is safe because `rpmtsInitIterator` internally calls
/// `rpmtsLink` and `rpmdbLink`, giving the C-level iterator its own
/// reference-counted links to both the transaction set and the database.
/// The iterator walks a snapshot of the index taken at creation time.
/// When dropped, `rpmdbFreeIterator` releases those links.
///
/// # Header ownership
///
/// `rpmdbNextIterator` returns a pointer to an internal header that is
/// only valid until the next call. We use `StreamingIterator` to enforce
/// this, then the public `Iter` adapter clones each header (via
/// `Header::from_ptr` → `headerLink`) before it can be invalidated.
pub(crate) struct MatchIterator {
    /// Pointer to librpm's match iterator.
    ptr: *mut librpm_sys::rpmdbMatchIterator_s,

    /// Next item in the iterator
    next_item: Option<Header>,

    /// Have we already finished iterating?
    finished: bool,
}

impl MatchIterator {
    /// Create a new `MatchIterator` for the given transaction set's database,
    /// searching by the (optionally) given search key.
    pub(crate) fn new(
        ts: *mut librpm_sys::rpmts_s,
        tag: DBIndexTag,
        key_opt: Option<&str>,
    ) -> Self {
        // rpmtsInitIterator inserts into the process-global rpmmiRock linked
        // list (RPM <= 4.18) without synchronization. See docs/threading.md.
        let _lock = rpm_global_lock();

        if let Some(key) = key_opt
            && !key.is_empty()
        {
            // SAFETY: `ts` is a valid rpmts pointer owned by the caller's
            // `Db` (each `Db` owns its own `TransactionSet`).  `c_key` is a
            // NUL-terminated CString kept alive for the duration of this call;
            // rpmtsInitIterator copies the key internally.  We pass keylen = 0
            // so librpm uses strlen(), matching the convention of all upstream
            // callers.  The returned pointer is either a valid
            // rpmdbMatchIterator or NULL (no match); both are safe to store —
            // all librpm iterator functions accept NULL.
            let c_key = CString::new(key).expect("search key must not contain NUL bytes");
            let ptr = unsafe {
                librpm_sys::rpmtsInitIterator(
                    ts,
                    tag as librpm_sys::rpm_tag_t,
                    c_key.as_ptr() as *const c_void,
                    0,
                )
            };

            return Self {
                ptr,
                next_item: None,
                finished: false,
            };
        }

        // SAFETY: NULL keyp with keylen 0 requests all entries from the
        // given index.  Same pointer-validity argument as above.
        let ptr = unsafe {
            librpm_sys::rpmtsInitIterator(ts, tag as librpm_sys::rpm_tag_t, ptr::null(), 0)
        };

        Self {
            ptr,
            next_item: None,
            finished: false,
        }
    }

    /// Create a `MatchIterator` that filters results using a glob or regex
    /// pattern via `rpmdbSetIteratorRE`.
    pub(crate) fn new_re(
        ts: *mut librpm_sys::rpmts_s,
        tag: DBIndexTag,
        pattern: &str,
        mode: MireMode,
    ) -> Self {
        // rpmtsInitIterator inserts into the process-global rpmmiRock linked
        // list (RPM <= 4.18) without synchronization. See docs/threading.md.
        let _lock = rpm_global_lock();

        let ptr = unsafe {
            librpm_sys::rpmtsInitIterator(ts, tag as librpm_sys::rpm_tag_t, ptr::null(), 0)
        };

        if !ptr.is_null() {
            let c_pattern = CString::new(pattern).expect("pattern must not contain NUL bytes");
            let mire_mode = match mode {
                MireMode::Glob => librpm_sys::rpmMireMode_e_RPMMIRE_GLOB,
                MireMode::Regex => librpm_sys::rpmMireMode_e_RPMMIRE_REGEX,
            };
            // SAFETY: `ptr` is non-null and was just created above.
            // `c_pattern` is a NUL-terminated CString that outlives this
            // call; rpmdbSetIteratorRE copies the pattern internally (into
            // mi->mi_re via mireDup) and compiles it (regcomp for regex,
            // stored fnmatch flags for glob), so `c_pattern` need not
            // outlive the iterator.
            unsafe {
                librpm_sys::rpmdbSetIteratorRE(
                    ptr,
                    tag as librpm_sys::rpm_tag_t,
                    mire_mode,
                    c_pattern.as_ptr(),
                );
            }
        }

        Self {
            ptr,
            next_item: None,
            finished: false,
        }
    }

    /// Return the total match count from the index snapshot.
    pub(crate) fn match_count(&self) -> usize {
        if self.ptr.is_null() {
            return 0;
        }
        // SAFETY: `self.ptr` is non-null (checked above) and valid for the
        // lifetime of this `MatchIterator` — it is only freed in `Drop`.
        // rpmdbGetIteratorCount is a read-only accessor on the iterator.
        unsafe { librpm_sys::rpmdbGetIteratorCount(self.ptr) as usize }
    }

    /// Return the database offset of the most recently returned header.
    pub(crate) fn offset(&self) -> u32 {
        if self.ptr.is_null() {
            return 0;
        }
        // SAFETY: Same as `match_count` — read-only accessor, `self.ptr`
        // is non-null and valid.
        unsafe { librpm_sys::rpmdbGetIteratorOffset(self.ptr) }
    }
}

/// Use a StreamingIterator to ensure that headers do not outlive `rpmdbNextIterator` calls.
impl StreamingIterator for MatchIterator {
    type Item = Header;

    fn advance(&mut self) {
        if self.finished {
            return;
        }

        // SAFETY: `self.ptr` is valid (or NULL, which rpmdbNextIterator
        // handles by returning NULL).  The returned Header pointer is
        // borrowed from the iterator's internal buffer and is only valid
        // until the next call to rpmdbNextIterator — this is why we use
        // StreamingIterator rather than Iterator, preventing the caller
        // from holding a reference across advance() calls.
        let header_ptr = unsafe { librpm_sys::rpmdbNextIterator(self.ptr) };

        if header_ptr.is_null() {
            self.finished = true;
            self.next_item = None;
        } else {
            // SAFETY: `header_ptr` is non-null and points to a valid
            // header owned by the iterator.  `Header::from_ptr` calls
            // `headerLink` to take its own refcounted reference, so the
            // Header remains valid even after the next rpmdbNextIterator
            // call invalidates the iterator's internal pointer.
            self.next_item = Some(unsafe { Header::from_ptr(header_ptr) })
        }
    }

    fn get(&self) -> Option<&Header> {
        self.next_item.as_ref()
    }
}

impl Drop for MatchIterator {
    fn drop(&mut self) {
        // rpmdbFreeIterator removes from the process-global rpmmiRock linked
        // list (RPM <= 4.18) without synchronization. See docs/threading.md.
        let _lock = rpm_global_lock();
        unsafe {
            librpm_sys::rpmdbFreeIterator(self.ptr);
        }
    }
}
