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

//! RPM database access
//!
//! The database used is whichever one is configured as the `_dbpath` in the
//! in the global macro context. By default this is unset: you will need to
//! call [`librpm::init()`](crate::init) to read the default "rpmrc"
//! configuration, then [`Db::open()`] to obtain a handle for querying.
//!
//! # Example
//!
//! Finding the "rpm-devel" RPM in the database:
//!
//! ```no_run
//! # fn main() -> Result<(), librpm::error::Error> {
//! use librpm::{Db, Index};
//!
//! librpm::init()?;
//! let db = Db::open()?;
//! let mut matches = db.find(Index::Name, "rpm-devel");
//! if let Some(package) = matches.next() {
//!     println!("package name: {}", package.name());
//!     println!("package summary: {}", package.summary());
//!     println!("package version: {}", package.version());
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::Error;
use crate::internal::iterator::MatchIterator;
use crate::internal::tag::DBIndexTag;
use crate::internal::ts::TransactionSet;
use crate::package::Package;
use streaming_iterator::StreamingIterator;

/// Handle to the RPM database.
///
/// Each `Db` owns its own librpm transaction set (`rpmts`). When dropped,
/// the transaction set and any associated database connection are freed.
///
/// Call [`librpm::init`](crate::init) first, then [`Db::open`] to obtain
/// a handle.
///
/// # Thread safety
///
/// `Db` is `Send` (can be moved to another thread) but `!Sync` (cannot be
/// shared by reference across threads). This matches the underlying
/// `rpmts`, which performs unsynchronized lazy-init mutations on first
/// database access. Multiple `Db` instances on different threads are safe
/// because each has an independent transaction set and database connection.
///
/// # Iterator lifetime
///
/// Iterators returned by [`find`](Db::find) and
/// [`installed_packages`](Db::installed_packages) do not borrow the `Db`.
/// The `Db` can be dropped while iterators are still alive because
/// `rpmtsInitIterator` takes its own refcounted links to the `rpmts` and
/// `rpmdb` internally. Collected [`Package`] values are also independent
/// — each owns a refcounted header.
#[derive(Debug)]
pub struct Db {
    ts: TransactionSet,
}

impl Db {
    /// Open the default RPM database.
    ///
    /// Returns an error if configuration has not been loaded yet via
    /// [`librpm::init`](crate::init) or [`librpm::init_with`](crate::init_with).
    pub fn open() -> Result<Self, Error> {
        let global_state = crate::internal::ConfigState::lock();
        if !global_state.configured {
            fail!(
                crate::error::ErrorKind::Config,
                "RPM has not been configured; call librpm::init() first"
            );
        }
        Ok(Db {
            ts: TransactionSet::create(),
        })
    }

    /// Find an exact match for `key` in the given `index`.
    pub fn find<S: AsRef<str>>(&self, index: Index, key: S) -> Iter {
        Iter(MatchIterator::new(
            self.ts.as_ptr(),
            index.into(),
            Some(key.as_ref()),
        ))
    }

    /// Find all packages installed on the local system.
    pub fn installed_packages(&self) -> Iter {
        Iter(MatchIterator::new(
            self.ts.as_ptr(),
            DBIndexTag::PACKAGES,
            None,
        ))
    }
}

/// Iterator over the RPM database which returns `Package` structs.
///
/// Wraps an internal `StreamingIterator` (whose items are borrowed from
/// the C-level cursor) and clones each header into an owned `Package`
/// before the cursor advances. This makes `Package` values safe to
/// collect and use after the iterator — and even after the `Db` — is
/// dropped.
pub struct Iter(MatchIterator);

impl Iterator for Iter {
    type Item = Package;

    fn next(&mut self) -> Option<Package> {
        self.0.next().map(Package::from_header)
    }
}

/// Searchable fields in the RPM package headers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Index {
    /// Search by package name.
    Name,
}
