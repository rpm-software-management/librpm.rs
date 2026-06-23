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
//! call [`librpm::config::read_file(None)`](crate::config::read_file) to read
//! the default "rpmrc" configuration, which returns a [`Db`] handle for
//! querying.
//!
//! # Example
//!
//! Finding the "rpm-devel" RPM in the database:
//!
//! ```no_run
//! # fn main() -> Result<(), librpm::error::Error> {
//! use librpm::Index;
//!
//! let db = librpm::config::read_file(None)?;
//! let mut matches = db.find(Index::Name, "rpm-devel");
//! if let Some(package) = matches.next() {
//!     println!("package name: {}", package.name());
//!     println!("package summary: {}", package.summary());
//!     println!("package version: {}", package.version());
//! }
//! # Ok(())
//! # }
//! ```

use crate::internal::iterator::MatchIterator;
use crate::internal::tag::DBIndexTag;
use crate::package::Package;
use streaming_iterator::StreamingIterator;

/// Handle to the RPM database, obtained by calling [`crate::config::read_file`].
///
/// All database query methods are on this type, ensuring that configuration
/// has been loaded before any queries are made.
#[derive(Debug)]
pub struct Db {
    _private: (),
}

impl Db {
    pub(crate) fn new() -> Self {
        Db { _private: () }
    }

    /// Find an exact match for `key` in the given `index`.
    pub fn find<S: AsRef<str>>(&self, index: Index, key: S) -> Iter {
        Iter(MatchIterator::new(index.into(), Some(key.as_ref())))
    }

    /// Find all packages installed on the local system.
    pub fn installed_packages(&self) -> Iter {
        Iter(MatchIterator::new(DBIndexTag::PACKAGES, None))
    }
}

/// Iterator over the RPM database which returns `Package` structs.
pub struct Iter(MatchIterator);

impl Iterator for Iter {
    type Item = Package;

    /// Obtain the next header from the iterator.
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
