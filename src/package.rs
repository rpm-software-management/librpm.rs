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
use std::convert::TryFrom;
use std::{fmt, time};

/// RPM packages
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Package {
    pub(crate) name: String,
    pub(crate) epoch: Option<i32>,
    pub(crate) version: String,
    pub(crate) release: String,
    pub(crate) arch: Option<String>,
    pub(crate) license: String,
    pub(crate) summary: String,
    pub(crate) description: String,
    pub(crate) buildtime: i32,
}

impl Package {
    /// Name of the package
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Epoch of the package
    pub fn epoch(&self) -> Option<i32> {
        self.epoch
    }

    /// Version of the package
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Release of the package
    pub fn release(&self) -> &str {
        &self.release
    }

    /// Arch of the package
    pub fn arch(&self) -> Option<&str> {
        self.arch.as_deref()
    }

    /// EVR (epoch, version, release) of the package
    pub fn evr(&self) -> String {
        if let Some(epoch) = &self.epoch {
            format!("{}:{}-{}", epoch, self.version, self.release)
        } else {
            format!("{}-{}", self.version, self.release)
        }
    }

    /// NEVRA (name, epoch, version, release, arch) of the package
    pub fn nevra(&self) -> String {
        if let Some(arch) = &self.arch {
            format!("{}-{}.{}", self.name, self.evr(), arch)
        } else {
            format!("{}-{}", self.name, self.evr())
        }
    }

    /// License of the package
    pub fn license(&self) -> &str {
        &self.license
    }

    /// Succinct description of the package
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Longer description of the package
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Buildtime of the package
    pub fn buildtime(&self) -> time::SystemTime {
        let buildtime = u64::try_from(self.buildtime).expect("negative build time");
        time::SystemTime::UNIX_EPOCH + time::Duration::new(buildtime, 0)
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nevra())
    }
}
