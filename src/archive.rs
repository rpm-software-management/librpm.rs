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

//! Archive (payload) extraction for RPM packages
//!
//! Provides sequential, streaming access to the file contents inside an
//! `.rpm` package. Each entry in the archive can be read via [`std::io::Read`].
//!
//! ```no_run
//! use librpm::archive::PackageReader;
//! use std::io::Read;
//! use std::path::Path;
//!
//! let mut pkg = PackageReader::open(Path::new("package.rpm")).unwrap();
//!
//! while let Some(mut entry) = pkg.next_entry().unwrap() {
//!     println!("{}: {} bytes", entry.path(), entry.size());
//!     if entry.has_content() {
//!         let mut buf = Vec::new();
//!         entry.read_to_end(&mut buf).unwrap();
//!     }
//! }
//! ```

use std::ffi::CStr;
use std::io;
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::NonNull;

use crate::error::ErrorKind;
use crate::files::{FileAttrs, FileEntry, FileState, Files};
use crate::internal::header::Header;
use crate::package::PackageHeader;

unsafe extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

/// RAII wrapper around librpm's `FD_t` file descriptor.
struct Fd(librpm_sys::FD_t);

impl Drop for Fd {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::Fclose(self.0);
        }
    }
}

/// An RPM package archive reader.
///
/// Provides sequential access to the files inside an RPM package's payload.
/// Entries are visited in the order they appear in the archive; random access
/// is not supported.
///
/// Use [`next_entry`](PackageArchive::next_entry) to advance through the archive.
/// Each [`ArchiveEntry`] borrows the archive mutably, so the compiler enforces
/// the constraint that you must finish with one entry before advancing to the
/// next.
pub struct PackageReader {
    fi: NonNull<librpm_sys::rpmfi_s>,
    _fd: Fd,
    files: Files,
    header: PackageHeader,
    exhausted: bool,
}

impl PackageReader {
    /// Open an `.rpm` file for archive extraction.
    ///
    /// Reads the package header and prepares the payload for sequential
    /// iteration. The file remains open until the `PackageReader` is dropped.
    pub fn open(path: &Path) -> Result<Self, crate::error::Error> {
        let (hdr, raw_fd) = Header::read_package_file(path)
            .map_err(|e| crate::error::Error::new(ErrorKind::Archive, Some(format!("{e:?}"))))?;

        let fd = Fd(raw_fd);
        let files = Files::from_header(&hdr);
        let header = PackageHeader::from_header(&hdr);

        let fi = unsafe {
            librpm_sys::rpmfiNewArchiveReader(
                fd.0,
                files.as_ptr(),
                librpm_sys::rpmFileIter_e_RPMFI_ITER_READ_ARCHIVE as i32,
            )
        };

        let fi = NonNull::new(fi).ok_or_else(|| {
            crate::error::Error::new(
                ErrorKind::Archive,
                Some("failed to create archive reader".to_string()),
            )
        })?;

        Ok(PackageReader {
            fi,
            _fd: fd,
            files,
            header,
            exhausted: false,
        })
    }

    /// Advance to the next entry in the archive.
    ///
    /// Returns `Ok(None)` when all entries have been visited. The returned
    /// [`ArchiveEntry`] borrows this archive mutably, so you must drop it
    /// (or let it go out of scope) before calling `next_entry` again.
    pub fn next_entry(&mut self) -> Result<Option<ArchiveEntry<'_>>, crate::error::Error> {
        if self.exhausted {
            return Ok(None);
        }

        let ix = unsafe { librpm_sys::rpmfiNext(self.fi.as_ptr()) };

        if ix < 0 {
            self.exhausted = true;
            return Ok(None);
        }

        let files_ptr = self.files.ptr().expect("archive has files");
        Ok(Some(ArchiveEntry {
            fi: self.fi,
            entry: FileEntry {
                ptr: files_ptr,
                index: ix,
                _marker: PhantomData,
            },
            _marker: PhantomData,
        }))
    }

    /// Access the package metadata (name, version, dependencies, etc.).
    pub fn package(&self) -> &PackageHeader {
        &self.header
    }
}

impl Drop for PackageReader {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::rpmfiArchiveClose(self.fi.as_ptr());
            librpm_sys::rpmfiFree(self.fi.as_ptr());
        }
    }
}

/// A single entry in an RPM archive.
///
/// Provides metadata about the current file and, for regular files,
/// streaming access to its content via [`std::io::Read`].
///
/// This type borrows the parent [`PackageReader`] mutably, which enforces
/// sequential access at compile time.
pub struct ArchiveEntry<'a> {
    fi: NonNull<librpm_sys::rpmfi_s>,
    entry: FileEntry<'a>,
    _marker: PhantomData<&'a mut PackageReader>,
}

impl ArchiveEntry<'_> {
    /// Full path of the file.
    pub fn path(&self) -> String {
        self.entry.path()
    }

    /// Base name of the file (filename without directory).
    pub fn basename(&self) -> &str {
        self.entry.basename()
    }

    /// Directory name of the file (including trailing slash).
    pub fn dirname(&self) -> &str {
        self.entry.dirname()
    }

    /// File size in bytes.
    pub fn size(&self) -> u64 {
        self.entry.size()
    }

    /// File mode (Unix permission bits and file type).
    pub fn mode(&self) -> u16 {
        self.entry.mode()
    }

    /// Owner (user name) of the file.
    pub fn user(&self) -> &str {
        self.entry.user()
    }

    /// Group of the file.
    pub fn group(&self) -> &str {
        self.entry.group()
    }

    /// File attribute flags (config, doc, ghost, license, etc.).
    pub fn flags(&self) -> FileAttrs {
        self.entry.flags()
    }

    /// Symlink target, or `None` if this is not a symbolic link.
    pub fn link_target(&self) -> Option<&str> {
        self.entry.link_target()
    }

    /// File capabilities string, or `None` if no capabilities are set.
    pub fn caps(&self) -> Option<&str> {
        self.entry.caps()
    }

    /// File modification time.
    pub fn mtime(&self) -> u64 {
        self.entry.mtime()
    }

    /// File install state.
    ///
    /// For packages read from `.rpm` files (as opposed to the RPM database),
    /// the state is always `Missing` (unavailable).
    pub fn state(&self) -> FileState {
        self.entry.state()
    }

    /// Binary digest of the file, or `None` if no digest is available.
    pub fn digest(&self) -> Option<&[u8]> {
        self.entry.digest()
    }

    /// Whether this entry has file content stored in the archive.
    ///
    /// Returns `true` for regular files and `false` for hardlinks whose
    /// content appears under a different entry, as well as directories,
    /// symlinks, and other non-regular file types.
    pub fn has_content(&self) -> bool {
        unsafe { librpm_sys::rpmfiArchiveHasContent(self.fi.as_ptr()) == 1 }
    }
}

impl io::Read for ArchiveEntry<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = unsafe {
            librpm_sys::rpmfiArchiveRead(self.fi.as_ptr(), buf.as_mut_ptr().cast(), buf.len())
        };
        if n < 0 {
            let errmsg = unsafe { librpm_sys::rpmfileStrerror(n as i32) };
            let msg = if errmsg.is_null() {
                "archive read error".to_string()
            } else {
                let s = unsafe { CStr::from_ptr(errmsg) }
                    .to_string_lossy()
                    .into_owned();
                unsafe { free(errmsg.cast()) };
                s
            };
            Err(io::Error::other(msg))
        } else {
            Ok(n as usize)
        }
    }
}
