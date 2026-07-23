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
use crate::internal::iterator::{MatchIterator, MireMode};
use crate::internal::mutation_lock;
use crate::internal::tag::DBIndexTag;
use crate::internal::ts::TransactionSet;
use crate::package::PackageHeader;
use crate::transaction::Transaction;
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

    /// Find packages where `index` matches `pattern` using glob or regex.
    ///
    /// The pattern is applied as a secondary filter on an initial full-index
    /// scan: librpm's `rpmdbSetIteratorRE` narrows the result set after the
    /// iterator is created over all entries for the given tag.
    pub fn find_re<S: AsRef<str>>(&self, index: Index, pattern: S, mode: MatchMode) -> Iter {
        let mire = match mode {
            MatchMode::Glob => MireMode::Glob,
            MatchMode::Regex => MireMode::Regex,
        };
        Iter(MatchIterator::new_re(
            self.ts.as_ptr(),
            index.into(),
            pattern.as_ref(),
            mire,
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

    /// Initialize a new, empty RPM database at the configured `_dbpath`.
    ///
    /// `perms` is the Unix file permission mode (e.g. `0o644`) for the
    /// newly created database files.
    ///
    /// This is the equivalent of `rpm --initdb`.
    pub fn init_db(&self, perms: i32) -> Result<(), Error> {
        let _lock = mutation_lock();
        let rc = unsafe { librpm_sys::rpmtsInitDB(self.ts.as_ptr(), perms) };
        if rc != 0 {
            fail!(
                crate::error::ErrorKind::Database,
                "failed to initialize RPM database"
            );
        }
        Ok(())
    }

    /// Rebuild the RPM database from installed package headers.
    ///
    /// This is the equivalent of `rpm --rebuilddb`. It recreates the
    /// database indices from the installed package headers.
    pub fn rebuild(&self) -> Result<(), Error> {
        let _lock = mutation_lock();
        let rc = unsafe { librpm_sys::rpmtsRebuildDB(self.ts.as_ptr()) };
        if rc != 0 {
            fail!(
                crate::error::ErrorKind::Database,
                "failed to rebuild RPM database"
            );
        }
        Ok(())
    }

    /// Create a transaction for installing, upgrading, or erasing packages.
    ///
    /// The transaction borrows the `Db` exclusively — complete any queries
    /// before calling this method. Drop the transaction when done to
    /// release the borrow.
    pub fn transaction(&mut self) -> Transaction<'_> {
        Transaction::new(self)
    }

    /// Get the keyring associated with this database.
    ///
    /// Returns a [`Keyring`](crate::keyring::Keyring) loaded from the
    /// RPM database's trusted keys. The returned keyring is an
    /// independent refcounted object — it can outlive the `Db`.
    pub fn keyring(&self) -> crate::keyring::Keyring {
        unsafe {
            let raw = librpm_sys::rpmtsGetKeyring(self.ts.as_ptr(), 1);
            crate::keyring::Keyring::from_raw(raw)
        }
    }

    pub(crate) fn ts_ptr(&self) -> *mut librpm_sys::rpmts_s {
        self.ts.as_ptr()
    }

    /// Verify the integrity of the RPM database.
    ///
    /// This is the equivalent of `rpmdb --verifydb`. Returns an error
    /// if the database has integrity problems.
    pub fn verify(&self) -> Result<(), Error> {
        let _lock = mutation_lock();
        let rc = unsafe { librpm_sys::rpmtsVerifyDB(self.ts.as_ptr()) };
        if rc != 0 {
            fail!(
                crate::error::ErrorKind::Database,
                "RPM database verification failed"
            );
        }
        Ok(())
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

impl Iter {
    /// Return the number of packages matched by this iterator's index query.
    ///
    /// This is the snapshot count established when the iterator was created,
    /// not the number of remaining items.
    pub fn match_count(&self) -> usize {
        self.0.match_count()
    }

    /// Return the database offset (record number) of the most recently
    /// returned package header, or `0` before the first call to `next()`.
    pub fn offset(&self) -> u32 {
        self.0.offset()
    }
}

impl Iterator for Iter {
    type Item = PackageHeader;

    fn next(&mut self) -> Option<PackageHeader> {
        self.0.next().map(PackageHeader::from_header)
    }
}

/// Searchable fields in the RPM package headers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Index {
    /// Search by package name.
    Name,
    /// Search by file basename.
    Basenames,
    /// Search by directory name.
    Dirnames,
    /// Search by installed file path.
    Instfilenames,
    /// Search by provided capability name.
    Providename,
    /// Search by required dependency name.
    Requirename,
    /// Search by conflict name.
    Conflictname,
    /// Search by obsoleted package name.
    Obsoletename,
    /// Search by package group.
    Group,
    /// Search by trigger dependency name.
    Triggername,
    /// Search by recommend dependency name.
    Recommendname,
    /// Search by suggest dependency name.
    Suggestname,
    /// Search by supplement dependency name.
    Supplementname,
    /// Search by enhance dependency name.
    Enhancename,
    /// Search by file trigger dependency name.
    Filetriggername,
    /// Search by transactional file trigger dependency name.
    Transfiletriggername,
}

/// Pattern matching mode for [`Db::find_re`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MatchMode {
    /// POSIX glob pattern (fnmatch-style).
    Glob,
    /// POSIX extended regular expression.
    Regex,
}
