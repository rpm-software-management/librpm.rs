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

//! Dependency information for RPM packages

use std::ffi::CStr;
use std::fmt;

use crate::Tag;
use crate::internal::header::Header;

/// Dependency set for an RPM package.
///
/// Wraps librpm's `rpmds` handle, collecting all entries into owned data.
/// Packages with no dependencies of a given type produce an empty set.
pub struct Dependencies {
    entries: Vec<Dependency>,
}

impl Dependencies {
    pub(crate) fn from_header(header: &Header, tag: Tag) -> Self {
        let ds = unsafe { librpm_sys::rpmdsNew(header.as_ptr(), tag as librpm_sys::rpmTagVal, 0) };

        if ds.is_null() {
            return Dependencies {
                entries: Vec::new(),
            };
        }

        let count = unsafe { librpm_sys::rpmdsCount(ds) };
        let mut entries = Vec::with_capacity(count.max(0) as usize);

        unsafe { librpm_sys::rpmdsInit(ds) };

        while unsafe { librpm_sys::rpmdsNext(ds) } >= 0 {
            let name = unsafe {
                let p = librpm_sys::rpmdsN(ds);
                assert!(!p.is_null());
                CStr::from_ptr(p)
                    .to_str()
                    .expect("dependency name is not UTF-8")
                    .to_owned()
            };

            let evr_raw = unsafe {
                let p = librpm_sys::rpmdsEVR(ds);
                assert!(!p.is_null());
                CStr::from_ptr(p)
                    .to_str()
                    .expect("dependency EVR is not UTF-8")
            };
            let evr = if evr_raw.is_empty() {
                None
            } else {
                Some(evr_raw.to_owned())
            };

            let flags = DepFlags(unsafe { librpm_sys::rpmdsFlags(ds) });

            entries.push(Dependency { name, evr, flags });
        }

        unsafe { librpm_sys::rpmdsFree(ds) };

        Dependencies { entries }
    }

    /// Number of dependencies in the set.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the set contains no dependencies.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over the dependencies in this set.
    pub fn iter(&self) -> std::slice::Iter<'_, Dependency> {
        self.entries.iter()
    }
}

impl fmt::Debug for Dependencies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dependencies")
            .field("len", &self.len())
            .finish()
    }
}

impl<'a> IntoIterator for &'a Dependencies {
    type Item = &'a Dependency;
    type IntoIter = std::slice::Iter<'a, Dependency>;

    fn into_iter(self) -> std::slice::Iter<'a, Dependency> {
        self.iter()
    }
}

impl IntoIterator for Dependencies {
    type Item = Dependency;
    type IntoIter = std::vec::IntoIter<Dependency>;

    fn into_iter(self) -> std::vec::IntoIter<Dependency> {
        self.entries.into_iter()
    }
}

/// A single dependency entry within an RPM package.
#[derive(Debug, Clone)]
pub struct Dependency {
    name: String,
    evr: Option<String>,
    flags: DepFlags,
}

impl Dependency {
    /// Name of the dependency (e.g. `"python3"`, `"rpmlib(CompressedFileNames)"`).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Version constraint, if any (e.g. `"3.6"`, `"1:2.3.4-5.el9"`).
    pub fn evr(&self) -> Option<&str> {
        self.evr.as_deref()
    }

    /// Sense flags describing the version comparison and dependency context.
    pub fn flags(&self) -> DepFlags {
        self.flags
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(evr) = &self.evr {
            let op = self.flags.version_cmp_str();
            if !op.is_empty() {
                write!(f, " {op} {evr}")?;
            }
        }
        Ok(())
    }
}

/// Dependency sense flags from the RPM header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepFlags(u32);

impl DepFlags {
    /// Raw flag bits.
    pub fn bits(self) -> u32 {
        self.0
    }

    /// Version comparison operator as a string (`"<"`, `"<="`, `"="`, `">="`, `">"`, or `""`).
    pub fn version_cmp_str(self) -> &'static str {
        let cmp = self.0
            & (librpm_sys::rpmsenseFlags_e_RPMSENSE_LESS
                | librpm_sys::rpmsenseFlags_e_RPMSENSE_GREATER
                | librpm_sys::rpmsenseFlags_e_RPMSENSE_EQUAL);
        match cmp {
            0 => "",
            x if x == librpm_sys::rpmsenseFlags_e_RPMSENSE_LESS => "<",
            x if x == librpm_sys::rpmsenseFlags_e_RPMSENSE_GREATER => ">",
            x if x == librpm_sys::rpmsenseFlags_e_RPMSENSE_EQUAL => "=",
            x if x
                == (librpm_sys::rpmsenseFlags_e_RPMSENSE_LESS
                    | librpm_sys::rpmsenseFlags_e_RPMSENSE_EQUAL) =>
            {
                "<="
            }
            x if x
                == (librpm_sys::rpmsenseFlags_e_RPMSENSE_GREATER
                    | librpm_sys::rpmsenseFlags_e_RPMSENSE_EQUAL) =>
            {
                ">="
            }
            _ => "?",
        }
    }

    /// Dependency requires `<` comparison.
    pub fn is_less(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_LESS != 0
    }

    /// Dependency requires `>` comparison.
    pub fn is_greater(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_GREATER != 0
    }

    /// Dependency requires `=` comparison.
    pub fn is_equal(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_EQUAL != 0
    }

    /// Dependency is a pre-install prerequisite.
    pub fn is_prereq(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_PREREQ != 0
    }

    /// Dependency is on an `rpmlib(...)` feature.
    pub fn is_rpmlib(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_RPMLIB != 0
    }

    /// Dependency is needed by a `%pre` scriptlet.
    pub fn is_script_pre(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_SCRIPT_PRE != 0
    }

    /// Dependency is needed by a `%post` scriptlet.
    pub fn is_script_post(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_SCRIPT_POST != 0
    }

    /// Dependency is needed by a `%preun` scriptlet.
    pub fn is_script_preun(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_SCRIPT_PREUN != 0
    }

    /// Dependency is needed by a `%postun` scriptlet.
    pub fn is_script_postun(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_SCRIPT_POSTUN != 0
    }

    /// Dependency is needed by a `%pretrans` scriptlet.
    pub fn is_pretrans(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_PRETRANS != 0
    }

    /// Dependency is needed by a `%posttrans` scriptlet.
    pub fn is_posttrans(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_POSTTRANS != 0
    }

    /// Dependency is a `config(...)` dependency.
    pub fn is_config(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_CONFIG != 0
    }

    /// Dependency is a meta-dependency (ordering hint only, not enforced).
    pub fn is_meta(self) -> bool {
        self.0 & librpm_sys::rpmsenseFlags_e_RPMSENSE_META != 0
    }
}
