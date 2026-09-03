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

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::error::Error;
use crate::internal::iterator::{MatchIterator, MireMode};
use crate::internal::tag::DBIndexTag;
use crate::internal::ts::TransactionSet;
use crate::internal::{mutation_lock, rpm_global_lock};
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
    /// Open the default RPM database, rooted at `/`.
    ///
    /// Returns an error if configuration has not been loaded yet via
    /// [`librpm::init`](crate::init) or [`librpm::init_with`](crate::init_with).
    pub fn open() -> Result<Self, Error> {
        Ok(Db {
            ts: Self::configured_ts()?,
        })
    }

    /// Open the RPM database rooted at `root`.
    ///
    /// This is the library equivalent of `rpm --root <root>` /
    /// `dnf --installroot <root>`: the database lives at `<root>/<_dbpath>`
    /// and every transaction operation treats `<root>` as the filesystem
    /// root. `root` must be an absolute path.
    ///
    /// Use this to operate on a database in an alternate root — an OS image
    /// or chroot being built, or an isolated database in a temporary
    /// directory — without touching the host's `/`. The database *path*
    /// (`_dbpath`) is still controlled independently via
    /// [`librpm::init_with`](crate::init_with); this only changes the root
    /// it is resolved against.
    ///
    /// Returns an error if configuration has not been loaded yet via
    /// [`librpm::init`](crate::init), or if `root` is not absolute.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # fn main() -> Result<(), librpm::error::Error> {
    /// use std::path::Path;
    /// use librpm::Db;
    ///
    /// librpm::init()?;
    /// // Initialize and query a fresh database under an alternate root.
    /// let db = Db::open_with_root(Path::new("/mnt/sysimage"))?;
    /// db.init_db(0o644)?;
    /// println!("{} packages installed", db.installed_packages().count());
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_root(root: &Path) -> Result<Self, Error> {
        // rpmtsSetRootDir rejects a non-absolute path (returns -1); check up
        // front so callers get a clear message rather than an opaque failure.
        if !root.is_absolute() {
            fail!(
                crate::error::ErrorKind::InvalidArg,
                "root directory must be an absolute path: {}",
                root.display()
            );
        }
        let c_root = CString::new(root.as_os_str().as_bytes()).map_err(|_| {
            format_err!(
                crate::error::ErrorKind::InvalidArg,
                "root path contains an interior NUL byte: {}",
                root.display()
            )
        })?;

        let ts = Self::configured_ts()?;
        // Override the "/" default that TransactionSet::create() applies.
        let rc = unsafe { librpm_sys::rpmtsSetRootDir(ts.as_ptr(), c_root.as_ptr()) };
        if rc != 0 {
            fail!(
                crate::error::ErrorKind::InvalidArg,
                "failed to set root directory: {}",
                root.display()
            );
        }
        Ok(Db { ts })
    }

    /// Create a transaction set after confirming RPM has been configured.
    fn configured_ts() -> Result<TransactionSet, Error> {
        let global_state = crate::internal::ConfigState::lock();
        if !global_state.configured {
            fail!(
                crate::error::ErrorKind::Config,
                "RPM has not been configured; call librpm::init() first"
            );
        }
        Ok(TransactionSet::create())
    }

    /// Find an exact match for `key` in the given `index`.
    pub fn find<S: AsRef<str>>(&self, index: Index, key: S) -> Iter {
        Iter(MatchIterator::new(
            self.ts.as_ptr(),
            index.into(),
            Some(key.as_ref()),
        ))
    }

    /// Find packages where `index` matches `pattern` using regex.
    ///
    /// The pattern is applied as a secondary filter on an initial full-index
    /// scan: librpm's `rpmdbSetIteratorRE` narrows the result set after the
    /// iterator is created over all entries for the given tag.
    pub fn find_regex<S: AsRef<str>>(&self, index: Index, pattern: S) -> Iter {
        Iter(MatchIterator::new_re(
            self.ts.as_ptr(),
            index.into(),
            pattern.as_ref(),
            MireMode::Regex,
        ))
    }

    /// Find packages where `index` matches `pattern` using glob.
    ///
    /// The pattern is applied as a secondary filter on an initial full-index
    /// scan: librpm's `rpmdbSetIteratorRE` narrows the result set after the
    /// iterator is created over all entries for the given tag.
    pub fn find_glob<S: AsRef<str>>(&self, index: Index, pattern: S) -> Iter {
        Iter(MatchIterator::new_re(
            self.ts.as_ptr(),
            index.into(),
            pattern.as_ref(),
            MireMode::Glob,
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
        let _mutation = mutation_lock();
        // rpmtsInitDB opens/creates the database, mutating the RPM <= 4.18
        // global tracking list (rpmdbRock). Lock ordering: mutation_lock first,
        // then rpm_global_lock. See docs/locking.md.
        let _global = rpm_global_lock();
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
        let _mutation = mutation_lock();
        // rpmtsRebuildDB opens the old database and creates a new one, mutating
        // the RPM <= 4.18 global tracking list (rpmdbRock). Lock ordering:
        // mutation_lock first, then rpm_global_lock. See docs/locking.md.
        let _global = rpm_global_lock();
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
        // rpmtsGetKeyring with autoload=1 -> loadKeyringFromDB opens the
        // database and creates a match iterator (gpg-pubkey lookup), mutating
        // the RPM <= 4.18 global tracking lists. See docs/locking.md.
        let _lock = rpm_global_lock();
        unsafe {
            let raw = librpm_sys::rpmtsGetKeyring(self.ts.as_ptr(), 1);
            crate::keyring::Keyring::from_raw(raw)
        }
    }

    /// Import a public key into this database's keystore.
    ///
    /// This persistently stores the key in the RPM database, making it
    /// available for signature verification across all future operations.
    /// This is the programmatic equivalent of `rpm --import`.
    ///
    /// Honors this database's root directory (see [`Db::open_with_root`]), so
    /// the key is written to *this* database. This is the programmatic
    /// equivalent of `rpm --import`;
    /// [`Keyring::import_to_rpmdb`](crate::keyring::Keyring::import_to_rpmdb)
    /// is the static, root-`/` convenience wrapper around it.
    ///
    /// Expects binary PGP packet data, not ASCII-armored keys. Typically
    /// requires root privileges when operating on the system database.
    #[cfg(any(has_rpmkeyring_rpmtxnimportpubkey, has_rpmkeyring_rpmtsimportpubkey))]
    pub fn import_pubkey(&self, key_data: &[u8]) -> Result<(), Error> {
        let _mutation = mutation_lock();
        // Importing a pubkey writes a gpg-pubkey "package" to the database,
        // opening it (rpmdbRock) on RPM <= 4.18. Lock ordering: mutation_lock
        // first, then rpm_global_lock. See docs/locking.md.
        let _global = rpm_global_lock();

        #[cfg(has_rpmkeyring_rpmtxnimportpubkey)]
        let rc = {
            let txn = unsafe {
                librpm_sys::rpmtxnBegin(self.ts.as_ptr(), librpm_sys::rpmtxnFlags_e_RPMTXN_WRITE)
            };
            if txn.is_null() {
                fail!(
                    crate::error::ErrorKind::Keyring,
                    "failed to acquire RPM lock"
                );
            }
            let rc =
                unsafe { librpm_sys::rpmtxnImportPubkey(txn, key_data.as_ptr(), key_data.len()) };
            unsafe { librpm_sys::rpmtxnEnd(txn) };
            rc
        };

        #[cfg(not(has_rpmkeyring_rpmtxnimportpubkey))]
        let rc = unsafe {
            librpm_sys::rpmtsImportPubkey(self.ts.as_ptr(), key_data.as_ptr(), key_data.len())
        };

        if rc != librpm_sys::rpmRC_e_RPMRC_OK {
            fail!(
                crate::error::ErrorKind::Keyring,
                "failed to import key into RPM database (rc={})",
                rc
            );
        }
        Ok(())
    }

    /// Delete a public key from this database's keystore.
    ///
    /// Removes the key from the RPM database so it will no longer be
    /// used for signature verification. This is the programmatic
    /// equivalent of `rpm -e gpg-pubkey-...`.
    ///
    /// Honors this database's root directory. Typically requires root privileges
    /// when operating on the system database.
    #[cfg(has_rpmkeyring_rpmtxndeletepubkey)]
    pub fn delete_pubkey(&self, key: &crate::keyring::PubKey) -> Result<(), Error> {
        let _mutation = mutation_lock();
        // Deleting a pubkey erases the gpg-pubkey "package" from the database,
        // opening it (rpmdbRock) on RPM <= 4.18. Lock ordering: mutation_lock
        // first, then rpm_global_lock. See docs/locking.md.
        let _global = rpm_global_lock();

        let txn = unsafe {
            librpm_sys::rpmtxnBegin(self.ts.as_ptr(), librpm_sys::rpmtxnFlags_e_RPMTXN_WRITE)
        };
        if txn.is_null() {
            fail!(
                crate::error::ErrorKind::Keyring,
                "failed to acquire RPM lock"
            );
        }
        let rc = unsafe { librpm_sys::rpmtxnDeletePubkey(txn, key.as_ptr()) };
        unsafe { librpm_sys::rpmtxnEnd(txn) };

        if rc != librpm_sys::rpmRC_e_RPMRC_OK {
            fail!(
                crate::error::ErrorKind::Keyring,
                "failed to delete key from RPM database (rc={})",
                rc
            );
        }
        Ok(())
    }

    pub(crate) fn ts_ptr(&self) -> *mut librpm_sys::rpmts_s {
        self.ts.as_ptr()
    }

    /// Verify the integrity of the RPM database.
    ///
    /// This is the equivalent of `rpmdb --verifydb`. Returns an error
    /// if the database has integrity problems.
    pub fn verify(&self) -> Result<(), Error> {
        let _mutation = mutation_lock();
        // rpmtsVerifyDB opens the database, mutating the RPM <= 4.18 global
        // tracking list (rpmdbRock). Lock ordering: mutation_lock first, then
        // rpm_global_lock. See docs/locking.md.
        let _global = rpm_global_lock();
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
