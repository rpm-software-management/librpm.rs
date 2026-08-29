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

//! Keyring and public key management
//!
//! A [`Keyring`] is a collection of trusted [`PubKey`]s used for verifying RPM package signatures.
//! Keyrings can be created empty, loaded from the system RPM database, or populated manually.
//!
//! # Example
//!
//! ```no_run
//! use librpm::keyring::{Keyring, PubKey};
//!
//! librpm::init().unwrap();
//!
//! // Load the system keyring
//! let keyring = Keyring::from_rpmdb().unwrap();
//!
//! // Or build one manually
//! let mut keyring = Keyring::new();
//! let key = PubKey::new(&std::fs::read("key.gpg").unwrap()).unwrap();
//! keyring.add_key(&key).unwrap();
//! ```

use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::error::{Error, ErrorKind};
use crate::internal::ts::TransactionSet;

unsafe extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

/// An RPM public key.
///
/// Wraps librpm's refcounted `rpmPubkey` handle. Cloning increments the refcount;
/// dropping decrements it.
pub struct PubKey {
    ptr: librpm_sys::rpmPubkey, // *mut rpmPubkey_s
}

// Safety: rpmPubkey is a heap-allocated, self-contained object with no thread-local state.
// Moving it between threads is safe.
unsafe impl Send for PubKey {}

impl PubKey {
    /// Create a public key from raw OpenPGP packet data.
    pub fn new(data: &[u8]) -> Result<Self, Error> {
        let ptr = unsafe { librpm_sys::rpmPubkeyNew(data.as_ptr(), data.len()) };
        if ptr.is_null() {
            fail!(ErrorKind::Keyring, "failed to create public key from data");
        }
        Ok(Self { ptr })
    }

    /// Read a public key from an ASCII-armored file.
    #[cfg(has_rpmkeyring_rpmpubkeyread)]
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let c_path = CString::new(path.as_os_str().as_bytes())
            .map_err(|e| format_err!(ErrorKind::InvalidArg, "{}", e))?;
        let ptr = unsafe { librpm_sys::rpmPubkeyRead(c_path.as_ptr()) };
        if ptr.is_null() {
            fail!(ErrorKind::Keyring, "failed to read public key from file");
        }
        Ok(Self { ptr })
    }

    /// Get the base64 representation of this key.
    ///
    /// Returns `None` if the key data cannot be encoded.
    pub fn base64(&self) -> Option<String> {
        let c_str = unsafe { librpm_sys::rpmPubkeyBase64(self.ptr) };
        if c_str.is_null() {
            return None;
        }
        let s = unsafe { CStr::from_ptr(c_str) }
            .to_string_lossy()
            .into_owned();
        unsafe { free(c_str as *mut std::ffi::c_void) };
        Some(s)
    }

    /// Get the key's fingerprint as a hex string.
    #[cfg(has_rpmkeyring_rpmpubkeyfingerprintashex)]
    pub fn fingerprint_hex(&self) -> Option<&str> {
        let c_str = unsafe { librpm_sys::rpmPubkeyFingerprintAsHex(self.ptr) };
        if c_str.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().ok()
    }

    /// Get the key ID as a hex string.
    #[cfg(has_rpmkeyring_rpmpubkeykeyidashex)]
    pub fn key_id_hex(&self) -> Option<&str> {
        let c_str = unsafe { librpm_sys::rpmPubkeyKeyIDAsHex(self.ptr) };
        if c_str.is_null() {
            return None;
        }
        unsafe { CStr::from_ptr(c_str) }.to_str().ok()
    }

    pub(crate) fn as_ptr(&self) -> librpm_sys::rpmPubkey {
        self.ptr
    }
}

impl Clone for PubKey {
    fn clone(&self) -> Self {
        #[cfg(has_rpmkeyring_rpmpubkeylink)]
        unsafe {
            librpm_sys::rpmPubkeyLink(self.ptr);
        }
        #[cfg(not(has_rpmkeyring_rpmpubkeylink))]
        {
            compile_error!("rpmPubkeyLink is required for PubKey::clone");
        }
        Self { ptr: self.ptr }
    }
}

impl Drop for PubKey {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::rpmPubkeyFree(self.ptr);
        }
    }
}

impl std::fmt::Debug for PubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("PubKey");
        #[cfg(has_rpmkeyring_rpmpubkeykeyidashex)]
        if let Some(id) = self.key_id_hex() {
            s.field("key_id", &id);
        }
        s.finish()
    }
}

/// A collection of RPM public keys for signature verification.
///
/// Wraps librpm's refcounted `rpmKeyring` handle. A keyring can be created empty,
/// loaded from the system RPM database, or populated manually by adding [`PubKey`]s.
///
/// # Thread safety
///
/// `Keyring` is `Send` but `!Sync`: it can be moved between threads but cannot be
/// shared by reference across threads.
pub struct Keyring {
    ptr: librpm_sys::rpmKeyring, // *mut rpmKeyring_s
}

// Safety: rpmKeyring is a heap-allocated, self-contained object with
// no thread-local state.
unsafe impl Send for Keyring {}

impl Keyring {
    /// Create an empty keyring.
    pub fn new() -> Self {
        let ptr = unsafe { librpm_sys::rpmKeyringNew() };
        assert!(!ptr.is_null());
        Self { ptr }
    }

    /// Load the system keyring from the RPM database.
    ///
    /// Requires [`librpm::init()`](crate::init) to have been called.
    pub fn from_rpmdb() -> Result<Self, Error> {
        let global_state = crate::internal::ConfigState::lock();
        if !global_state.configured {
            fail!(
                ErrorKind::Config,
                "RPM has not been configured; call librpm::init() first"
            );
        }

        let ts = TransactionSet::create();
        // autoload=1: load the keyring from the database if not already loaded
        let kr_ptr = unsafe { librpm_sys::rpmtsGetKeyring(ts.as_ptr(), 1) };
        if kr_ptr.is_null() {
            fail!(ErrorKind::Keyring, "failed to load system keyring");
        }
        Ok(Self { ptr: kr_ptr })
    }

    /// Import a public key into the system RPM keystore.
    ///
    /// This persistently stores the key in the RPM database, making it
    /// available for signature verification across all future operations.
    /// This is the programmatic equivalent of `rpm --import`.
    ///
    /// **Note:** This function expects binary PGP packet data, not ASCII-armored keys.
    /// If you have an ASCII-armored key file (starting with `-----BEGIN PGP PUBLIC KEY BLOCK-----`),
    /// use [`import_pubkey_file_to_rpmdb()`](Self::import_pubkey_file_to_rpmdb) instead.
    ///
    /// Requires [`librpm::init()`](crate::init) to have been called.
    /// Typically requires root privileges.
    #[cfg(any(has_rpmkeyring_rpmtxnimportpubkey, has_rpmkeyring_rpmtsimportpubkey))]
    pub fn import_to_rpmdb(key_data: &[u8]) -> Result<(), Error> {
        let global_state = crate::internal::ConfigState::lock();
        if !global_state.configured {
            fail!(
                ErrorKind::Config,
                "RPM has not been configured; call librpm::init() first"
            );
        }
        drop(global_state);

        let ts = TransactionSet::create();
        let _lock = crate::internal::mutation_lock();

        #[cfg(has_rpmkeyring_rpmtxnimportpubkey)]
        let rc = {
            let txn = unsafe {
                librpm_sys::rpmtxnBegin(ts.as_ptr(), librpm_sys::rpmtxnFlags_e_RPMTXN_WRITE)
            };
            if txn.is_null() {
                fail!(ErrorKind::Keyring, "failed to acquire RPM lock");
            }
            let rc =
                unsafe { librpm_sys::rpmtxnImportPubkey(txn, key_data.as_ptr(), key_data.len()) };
            unsafe { librpm_sys::rpmtxnEnd(txn) };
            rc
        };

        #[cfg(not(has_rpmkeyring_rpmtxnimportpubkey))]
        let rc = unsafe {
            librpm_sys::rpmtsImportPubkey(ts.as_ptr(), key_data.as_ptr(), key_data.len())
        };

        if rc != librpm_sys::rpmRC_e_RPMRC_OK {
            fail!(
                ErrorKind::Keyring,
                "failed to import key into RPM database (rc={})",
                rc
            );
        }
        Ok(())
    }

    /// Delete a public key from the system RPM keystore.
    ///
    /// Removes the key from the RPM database so it will no longer be
    /// used for signature verification. This is the programmatic
    /// equivalent of `rpm -e gpg-pubkey-...`.
    ///
    /// Requires [`librpm::init()`](crate::init) to have been called.
    /// Typically requires root privileges.
    #[cfg(has_rpmkeyring_rpmtxndeletepubkey)]
    pub fn delete_from_rpmdb(key: &PubKey) -> Result<(), Error> {
        let global_state = crate::internal::ConfigState::lock();
        if !global_state.configured {
            fail!(
                ErrorKind::Config,
                "RPM has not been configured; call librpm::init() first"
            );
        }
        drop(global_state);

        let ts = TransactionSet::create();
        let _lock = crate::internal::mutation_lock();

        let txn =
            unsafe { librpm_sys::rpmtxnBegin(ts.as_ptr(), librpm_sys::rpmtxnFlags_e_RPMTXN_WRITE) };
        if txn.is_null() {
            fail!(ErrorKind::Keyring, "failed to acquire RPM lock");
        }

        let rc = unsafe { librpm_sys::rpmtxnDeletePubkey(txn, key.as_ptr()) };
        unsafe { librpm_sys::rpmtxnEnd(txn) };

        if rc != librpm_sys::rpmRC_e_RPMRC_OK {
            fail!(
                ErrorKind::Keyring,
                "failed to delete key from RPM database (rc={})",
                rc
            );
        }
        Ok(())
    }

    /// Add a public key to the keyring.
    ///
    /// Returns `Ok(true)` if the key was added, `Ok(false)` if it was already present.
    pub fn add_key(&mut self, key: &PubKey) -> Result<bool, Error> {
        let rc = unsafe { librpm_sys::rpmKeyringAddKey(self.ptr, key.as_ptr()) };
        match rc {
            0 => Ok(true),
            1 => Ok(false),
            _ => {
                fail!(ErrorKind::Keyring, "failed to add key to keyring");
            }
        }
    }

    /// Remove a public key from the keyring.
    #[cfg(has_rpmkeyring_rpmkeyringmodify)]
    pub fn remove_key(&mut self, key: &PubKey) -> Result<bool, Error> {
        let rc = unsafe {
            librpm_sys::rpmKeyringModify(
                self.ptr,
                key.as_ptr(),
                librpm_sys::rpmKeyringModifyMode_e_RPMKEYRING_DELETE,
            )
        };
        match rc {
            0 => Ok(true),
            1 => Ok(false),
            _ => {
                fail!(ErrorKind::Keyring, "failed to remove key from keyring");
            }
        }
    }

    /// Look up a key in the keyring.
    ///
    /// Returns a new refcounted reference to the matching key, or `None` if not found.
    #[cfg(has_rpmkeyring_rpmkeyringlookupkey)]
    pub fn lookup(&self, key: &PubKey) -> Option<PubKey> {
        let found = unsafe { librpm_sys::rpmKeyringLookupKey(self.ptr, key.as_ptr()) };
        if found.is_null() {
            return None;
        }
        Some(PubKey { ptr: found })
    }

    /// Iterate over the keys in this keyring.
    #[cfg(has_rpmkeyring_rpmkeyringinititerator)]
    pub fn keys(&self) -> KeyringIter<'_> {
        let iter_ptr = unsafe { librpm_sys::rpmKeyringInitIterator(self.ptr, 0) };
        KeyringIter {
            ptr: iter_ptr,
            _marker: std::marker::PhantomData,
        }
    }

    pub(crate) fn as_ptr(&self) -> librpm_sys::rpmKeyring {
        self.ptr
    }

    /// Wrap a raw `rpmKeyring` pointer that already has its own refcount.
    pub(crate) fn from_raw(ptr: librpm_sys::rpmKeyring) -> Self {
        assert!(!ptr.is_null());
        Self { ptr }
    }
}

impl Default for Keyring {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Keyring {
    fn clone(&self) -> Self {
        #[cfg(has_rpmkeyring_rpmkeyringlink)]
        unsafe {
            librpm_sys::rpmKeyringLink(self.ptr);
        }
        #[cfg(not(has_rpmkeyring_rpmkeyringlink))]
        {
            compile_error!("rpmKeyringLink is required for Keyring::clone");
        }
        Self { ptr: self.ptr }
    }
}

impl Drop for Keyring {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::rpmKeyringFree(self.ptr);
        }
    }
}

impl std::fmt::Debug for Keyring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keyring").field("ptr", &self.ptr).finish()
    }
}

/// Iterator over the keys in a [`Keyring`].
///
/// Created by [`Keyring::keys`].
#[cfg(has_rpmkeyring_rpmkeyringinititerator)]
pub struct KeyringIter<'kr> {
    ptr: librpm_sys::rpmKeyringIterator,
    _marker: std::marker::PhantomData<&'kr Keyring>,
}

#[cfg(has_rpmkeyring_rpmkeyringinititerator)]
impl<'kr> Iterator for KeyringIter<'kr> {
    type Item = PubKey;

    fn next(&mut self) -> Option<PubKey> {
        if self.ptr.is_null() {
            return None;
        }
        let key_ptr = unsafe { librpm_sys::rpmKeyringIteratorNext(self.ptr) };
        if key_ptr.is_null() {
            return None;
        }
        // rpmKeyringIteratorNext returns a weak reference; we must
        // increment the refcount before wrapping as an owned PubKey.
        #[cfg(has_rpmkeyring_rpmpubkeylink)]
        unsafe {
            librpm_sys::rpmPubkeyLink(key_ptr);
        }
        Some(PubKey { ptr: key_ptr })
    }
}

#[cfg(has_rpmkeyring_rpmkeyringinititerator)]
impl Drop for KeyringIter<'_> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { librpm_sys::rpmKeyringIteratorFree(self.ptr) };
        }
    }
}
