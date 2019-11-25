//! RPM database access
//!
//! The database used is whichever one is configured as the `_dbpath` in the
//! in the global macro context. By default this is unset: you will need to
//! call `librpm::config::read_file(None)` to read the default "rpmrc"
//! configuration.
//!
//! # Example
//!
//! Finding the "rpm-devel" RPM in the database:
//!
//! ```
//! use librpm::{Db, Index};
//!
//! use std::path::Path;
//!
//! let db = Db::open::<&Path>().unwrap();
//! let mut matches = Index::Name.find(&db, "rpm-devel");
//! let package = matches.next().unwrap();
//!
//! println!("package name: {}", package.name);
//! println!("package summary: {}", package.summary);
//! println!("package version: {}", package.version);
//! ```

use crate::error::{Error, ErrorKind};
use crate::internal::{iterator::MatchIterator, tag::Tag};
use crate::package::Package;
use streaming_iterator::StreamingIterator;

use std::ffi::CString;
use std::fmt;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

static mut LIB_RPM_CONFIGURED: bool = false;

#[derive(Debug, PartialEq, Eq)]
pub struct Db {}

pub struct DbBuilder<P>
where
    P: AsRef<Path> + fmt::Debug,
{
    config: Option<P>,
}

impl<P> Default for DbBuilder<P>
where
    P: AsRef<Path> + fmt::Debug,
{
    fn default() -> Self {
        Self { config: None }
    }
}

impl Db {
    pub fn open<P>() -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        DbBuilder::<&Path>::new().open()
    }

    pub fn open_with<P>() -> DbBuilder<P>
    where
        P: AsRef<Path> + fmt::Debug,
    {
        DbBuilder::default()
    }

    /// Find installed packages with a search key that exactly matches the given tag.
    ///
    /// Panics if the glob contains null bytes.
    pub fn find<S: AsRef<str>>(&self, index: Index, key: S) -> Iter {
        index.find(self, key)
    }
}

impl<P> DbBuilder<P>
where
    P: AsRef<Path> + fmt::Debug,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config(mut self, config: P) -> Self {
        self.config = Some(config);
        self
    }

    pub fn open(self) -> Result<Db, Error> {
        let rc = {
            let cstr;
            let p = match self.config {
                Some(ref path) => {
                    if !path.as_ref().exists() {
                        fail!(
                            ErrorKind::Config,
                            "no such file: {}",
                            path.as_ref().display()
                        )
                    }
                    cstr = CString::new(path.as_ref().as_os_str().as_bytes()).map_err(|e| {
                        format_err!(
                            ErrorKind::Config,
                            "invalid path: {} ({})",
                            path.as_ref().display(),
                            e
                        )
                    })?;
                    cstr.as_ptr()
                }
                None => ptr::null(),
            };
            unsafe {
                if LIB_RPM_CONFIGURED {
                    fail!(
                        ErrorKind::AlreadyConfigured,
                        "librpm is already configured, global state can't be configured again"
                    )
                }
                LIB_RPM_CONFIGURED = true;
            }
            unsafe { librpm_sys::rpmReadConfigFiles(p, ptr::null()) }
        };
        if rc != 0 {
            match self.config {
                Some(path) => fail!(
                    ErrorKind::Config,
                    "error reading RPM config from: {}",
                    path.as_ref().display()
                ),
                None => fail!(
                    ErrorKind::Config,
                    "error reading RPM config from default location"
                ),
            }
        }
        Ok(Db {})
    }
}

/// Iterator over the RPM database which returns `Package` structs.
pub struct Iter(MatchIterator);

impl Iterator for Iter {
    type Item = Package;

    /// Obtain the next header from the iterator.
    fn next(&mut self) -> Option<Package> {
        self.0.next().map(|h| h.to_package())
    }
}

/// Searchable fields in the RPM package headers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Index {
    /// Search by package name.
    Name,

    /// Search by package version.
    Version,

    /// Search by package license.
    License,

    /// Search by package summary.
    Summary,

    /// Search by package description.
    Description,
}

impl Index {
    /// Find an exact match in the given index
    pub fn find<S: AsRef<str>>(self, _db: &Db, key: S) -> Iter {
        Iter(MatchIterator::new(self.into(), Some(key.as_ref())))
    }
}

impl Into<Tag> for Index {
    fn into(self) -> Tag {
        match self {
            Index::Name => Tag::NAME,
            Index::Version => Tag::VERSION,
            Index::License => Tag::LICENSE,
            Index::Summary => Tag::SUMMARY,
            Index::Description => Tag::DESCRIPTION,
        }
    }
}

/// Find all packages installed on the local system.
pub fn installed_packages() -> Iter {
    Iter(MatchIterator::new(Tag::NAME, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn db_opens() {
        Db::open::<&Path>().unwrap();
    }
}
