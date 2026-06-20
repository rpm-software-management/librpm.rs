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

//! RPM package type: represents `.rpm` files or entries in the RPM database
use crate::internal::header::Header;
use crate::{RpmErrorKind, Tag, TagData};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::{fmt, path::Path, time};

/// RPM packages
///
/// A thin wrapper around a librpm header. Accessors perform tag lookups
/// on demand rather than copying data out of the header up front.
pub struct Package {
    header: Header,
}

impl Package {
    pub(crate) fn from_header(h: &Header) -> Self {
        Package { header: h.clone() }
    }

    /// Create a Package by reading an `.rpm` file
    pub fn from_file(path: &Path) -> Result<Self, RpmErrorKind> {
        let header = Header::from_file(path)?;
        Ok(Package { header })
    }

    /// Look up raw tag data from the underlying RPM header
    pub fn get(&self, tag: Tag) -> Option<TagData<'_>> {
        self.header.get(tag)
    }

    /// Name of the package
    pub fn name(&self) -> &str {
        self.header
            .get(Tag::NAME)
            .expect("NAME tag missing")
            .as_str()
            .expect("NAME tag is not a string")
    }

    /// Epoch of the package
    pub fn epoch(&self) -> Option<i32> {
        self.header
            .get(Tag::EPOCH)
            .map(|d| d.as_int32().expect("EPOCH tag is not an int32"))
    }

    /// Version of the package
    pub fn version(&self) -> &str {
        self.header
            .get(Tag::VERSION)
            .expect("VERSION tag missing")
            .as_str()
            .expect("VERSION tag is not a string")
    }

    /// Release of the package
    pub fn release(&self) -> &str {
        self.header
            .get(Tag::RELEASE)
            .expect("RELEASE tag missing")
            .as_str()
            .expect("RELEASE tag is not a string")
    }

    /// Arch of the package
    pub fn arch(&self) -> Option<&str> {
        self.header
            .get(Tag::ARCH)
            .map(|d| d.as_str().expect("ARCH tag is not a string"))
    }

    /// EVR (epoch, version, release) of the package
    pub fn evr(&self) -> String {
        if let Some(epoch) = self.epoch() {
            format!("{}:{}-{}", epoch, self.version(), self.release())
        } else {
            format!("{}-{}", self.version(), self.release())
        }
    }

    /// NEVRA (name, epoch, version, release, arch) of the package
    pub fn nevra(&self) -> String {
        if let Some(arch) = self.arch() {
            format!("{}-{}.{}", self.name(), self.evr(), arch)
        } else {
            format!("{}-{}", self.name(), self.evr())
        }
    }

    /// License of the package
    pub fn license(&self) -> &str {
        self.header
            .get(Tag::LICENSE)
            .expect("LICENSE tag missing")
            .as_str()
            .expect("LICENSE tag is not a string")
    }

    /// Succinct description of the package
    pub fn summary(&self) -> &str {
        self.header
            .get(Tag::SUMMARY)
            .expect("SUMMARY tag missing")
            .as_str()
            .expect("SUMMARY tag is not a string")
    }

    /// Longer description of the package
    pub fn description(&self) -> &str {
        self.header
            .get(Tag::DESCRIPTION)
            .expect("DESCRIPTION tag missing")
            .as_str()
            .expect("DESCRIPTION tag is not a string")
    }

    /// Buildtime of the package
    pub fn buildtime(&self) -> time::SystemTime {
        let buildtime = self
            .header
            .get(Tag::BUILDTIME)
            .expect("BUILDTIME tag missing")
            .as_int32()
            .expect("BUILDTIME tag is not an int32");
        let buildtime = u64::try_from(buildtime).expect("negative build time");
        time::SystemTime::UNIX_EPOCH + time::Duration::new(buildtime, 0)
    }
}

impl Clone for Package {
    fn clone(&self) -> Self {
        Package {
            header: self.header.clone(),
        }
    }
}

impl fmt::Debug for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Package")
            .field("name", &self.name())
            .field("epoch", &self.epoch())
            .field("version", &self.version())
            .field("release", &self.release())
            .field("arch", &self.arch())
            .finish()
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nevra())
    }
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.epoch() == other.epoch()
            && self.version() == other.version()
            && self.release() == other.release()
            && self.arch() == other.arch()
    }
}

impl Eq for Package {}

impl Hash for Package {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
        self.epoch().hash(state);
        self.version().hash(state);
        self.release().hash(state);
        self.arch().hash(state);
    }
}
