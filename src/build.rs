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

//! RPM spec file parsing and package building
//!
//! Requires the `build` feature and links against `librpmbuild.so`.
//!
//! # Example
//!
//! ```no_run
//! use librpm::build::{BuildArgs, Spec, SpecFlags};
//!
//! librpm::init();
//!
//! // For inspection only (no build), use parse() directly:
//! let spec = Spec::parse("/path/to/package.spec", SpecFlags::NONE, None).unwrap();
//! println!("source header name: {}", spec.source_header().name());
//!
//! // For building, use parse_for_build() which handles NOFINALIZE automatically:
//! let args = BuildArgs::new();
//! let mut spec = Spec::parse_for_build("pkg.spec", SpecFlags::NONE, &args, None).unwrap();
//! spec.build(&args).unwrap();
//! ```

use crate::internal::header::Header;
use crate::internal::ts::TransactionSet;
use crate::package::PackageHeader;
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

unsafe extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

/// Flags controlling spec file parsing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct SpecFlags(u32);

impl SpecFlags {
    /// No special flags.
    pub const NONE: Self = Self(librpmbuild_sys::rpmSpecFlags_e_RPMSPEC_NONE);
    /// Parse for any architecture.
    pub const ANYARCH: Self = Self(librpmbuild_sys::rpmSpecFlags_e_RPMSPEC_ANYARCH);
    /// Force parse even if errors would normally prevent it.
    pub const FORCE: Self = Self(librpmbuild_sys::rpmSpecFlags_e_RPMSPEC_FORCE);
    /// Do not process %lang tags.
    pub const NOLANG: Self = Self(librpmbuild_sys::rpmSpecFlags_e_RPMSPEC_NOLANG);
    /// Do not check for valid UTF-8 in header strings.
    #[cfg(has_rpmspecflag_noutf8)]
    pub const NOUTF8: Self = Self(librpmbuild_sys::rpmSpecFlags_e_RPMSPEC_NOUTF8);
    /// Skip header finalization during parse (e.g. when building immediately after).
    #[cfg(has_rpmspecflag_nofinalize)]
    pub const NOFINALIZE: Self = Self(librpmbuild_sys::rpmSpecFlags_e_RPMSPEC_NOFINALIZE);

    /// Returns [`NOFINALIZE`](Self::NOFINALIZE) when available, [`NONE`](Self::NONE) otherwise.
    ///
    /// Use this when parsing a spec that will be built immediately after, to
    /// portably avoid the double-finalization issue across RPM versions.
    /// See `quirks.md` for details.
    pub const fn nofinalize_or_none() -> Self {
        #[cfg(has_rpmspecflag_nofinalize)]
        {
            Self::NOFINALIZE
        }
        #[cfg(not(has_rpmspecflag_nofinalize))]
        {
            Self::NONE
        }
    }
}

impl std::ops::BitOr for SpecFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SpecFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Flags controlling which build stages to execute.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct BuildFlags(i32);

impl BuildFlags {
    /// No build stages.
    pub const NONE: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_NONE);
    /// Execute %prep.
    pub const PREP: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_PREP);
    /// Execute %build.
    pub const BUILD: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_BUILD);
    /// Execute %install.
    pub const INSTALL: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_INSTALL);
    /// Execute %check.
    pub const CHECK: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_CHECK);
    /// Execute %clean.
    pub const CLEAN: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_CLEAN);
    /// Check %files manifest.
    pub const FILECHECK: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_FILECHECK);
    /// Create source package.
    pub const PACKAGESOURCE: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_PACKAGESOURCE);
    /// Create binary package(s).
    pub const PACKAGEBINARY: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_PACKAGEBINARY);
    /// Remove source(s) and patch(s).
    pub const RMSOURCE: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_RMSOURCE);
    /// Remove build sub-tree.
    pub const RMBUILD: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_RMBUILD);
    /// Remove spec file.
    pub const RMSPEC: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_RMSPEC);
    /// Execute %conf.
    #[cfg(has_rpmbuildflag_conf)]
    pub const CONF: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_CONF);
    /// Create the build directory tree (needed when using [`PREP`](Self::PREP)).
    #[cfg(has_rpmbuildflag_mkbuilddir)]
    pub const MKBUILDDIR: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_MKBUILDDIR);
    /// Don't execute or package (dry run).
    pub const NOBUILD: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_NOBUILD);
}

impl std::ops::BitOr for BuildFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for BuildFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BuildFlags {
    /// Set or clear specific flags.
    pub const fn set(self, flags: Self, on: bool) -> Self {
        if on {
            Self(self.0 | flags.0)
        } else {
            Self(self.0 & !flags.0)
        }
    }
}

/// Section identifiers for [`Spec::get_section`] and [`SpecPkg::get_section`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Section(i32);

impl Section {
    /// Return the entire spec in preprocessed (macro-expanded) format.
    pub const NONE: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_NONE);
    /// %prep section.
    pub const PREP: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_PREP);
    /// %build section.
    pub const BUILD: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_BUILD);
    /// %install section.
    pub const INSTALL: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_INSTALL);
    /// %check section.
    pub const CHECK: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_CHECK);
    /// %clean section.
    pub const CLEAN: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_CLEAN);
    /// %files list (for package sections).
    pub const FILE_LIST: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_FILE_LIST);
    /// %files -f entries (for package sections).
    pub const FILE_FILE: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_FILE_FILE);
    /// %policy section (for package sections).
    pub const POLICY: Self = Self(librpmbuild_sys::rpmBuildFlags_e_RPMBUILD_POLICY);
}

/// Flags describing the kind of source entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct SourceFlags(u32);

impl SourceFlags {
    /// Entry is a source file.
    pub const SOURCE: Self = Self(librpmbuild_sys::rpmSourceFlags_e_RPMBUILD_ISSOURCE);
    /// Entry is a patch file.
    pub const PATCH: Self = Self(librpmbuild_sys::rpmSourceFlags_e_RPMBUILD_ISPATCH);
    /// Entry is an icon.
    pub const ICON: Self = Self(librpmbuild_sys::rpmSourceFlags_e_RPMBUILD_ISICON);
    /// `NoSource` / `NoPatch` marker.
    pub const NO: Self = Self(librpmbuild_sys::rpmSourceFlags_e_RPMBUILD_ISNO);

    /// Test whether a specific flag is set.
    pub fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

/// Builder-style arguments for [`Spec::build_packages`].
///
/// By default, both binary and source packages are produced (equivalent to
/// `rpmbuild -ba`).  Use the builder methods to customize.
///
/// # Example
///
/// ```no_run
/// use librpm::build::{BuildArgs, Spec, SpecFlags};
///
/// librpm::init();
/// let args = BuildArgs::new().source(false); // binary only
/// let mut spec = Spec::parse_for_build("pkg.spec", SpecFlags::NONE, &args, None).unwrap();
/// spec.build(&args).unwrap();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct BuildArgs {
    binary: bool,
    source: bool,
    check: bool,
    clean: bool,
    extra_flags: BuildFlags,
}

impl BuildArgs {
    /// Create default build arguments (binary and source packages enabled).
    ///
    /// Equivalent to `rpmbuild -ba`.  Build stages (`PREP`, `BUILD`,
    /// `INSTALL`, `CONF`, `MKBUILDDIR`) are included automatically when
    /// binary packaging is enabled.  Use [`binary(false)`](Self::binary) for
    /// a source-only build (equivalent to `rpmbuild -bs`).
    pub fn new() -> Self {
        BuildArgs {
            binary: true,
            source: true,
            check: false,
            clean: false,
            extra_flags: BuildFlags::NONE,
        }
    }

    /// Create build arguments from raw [`BuildFlags`].
    ///
    /// Unlike [`new()`](Self::new), this does not include any default stages.
    pub fn from_flags(flags: BuildFlags) -> Self {
        BuildArgs {
            binary: false,
            source: false,
            check: false,
            clean: false,
            extra_flags: flags,
        }
    }

    /// Whether to produce binary package(s). Default: `true`.
    ///
    /// When enabled, the build stages needed for binary packaging (`PREP`,
    /// `BUILD`, `INSTALL`, and version-specific stages like `CONF` and
    /// `MKBUILDDIR`) are included automatically.
    pub fn binary(mut self, yes: bool) -> Self {
        self.binary = yes;
        self
    }

    /// Whether to produce a source package. Default: `true`.
    pub fn source(mut self, yes: bool) -> Self {
        self.source = yes;
        self
    }

    /// Whether to run `%check` after building. Default: `false`.
    pub fn check(mut self, yes: bool) -> Self {
        self.check = yes;
        self
    }

    /// Whether to run `%clean` and remove the build tree after packaging.
    /// Default: `false`.
    pub fn clean(mut self, yes: bool) -> Self {
        self.clean = yes;
        self
    }

    /// Add additional [`BuildFlags`].
    ///
    /// This is an escape hatch for flags not covered by the typed methods
    /// (e.g. `BuildFlags::RMSOURCE`, `BuildFlags::RMSPEC`).
    pub fn flags(mut self, flags: BuildFlags) -> Self {
        self.extra_flags |= flags;
        self
    }

    /// Compute the effective [`BuildFlags`] from the current settings.
    fn build_flags(&self) -> BuildFlags {
        let mut flags = BuildFlags::NONE;
        if self.binary {
            flags |= BuildFlags::PREP
                | BuildFlags::BUILD
                | BuildFlags::INSTALL
                | BuildFlags::PACKAGEBINARY;
            #[cfg(has_rpmbuildflag_conf)]
            {
                flags |= BuildFlags::CONF;
            }
            #[cfg(has_rpmbuildflag_mkbuilddir)]
            {
                flags |= BuildFlags::MKBUILDDIR;
            }
        }
        if self.source {
            flags |= BuildFlags::PACKAGESOURCE;
        }
        if self.check {
            flags |= BuildFlags::CHECK;
        }
        if self.clean {
            flags |= BuildFlags::CLEAN | BuildFlags::RMBUILD;
        }
        flags | self.extra_flags
    }

    // rpmbuild only sets NOFINALIZE for builds that include binary stages.
    // `parseGeneratedSpecs()` (which calls `finalizeSpec()` a second time)
    // is only reached when INSTALL, PACKAGEBINARY, or FILECHECK is set.
    // For source-only builds, finalization must happen at parse time.
    fn needs_nofinalize(&self) -> bool {
        let flags = self.build_flags();
        let binary_mask =
            BuildFlags::INSTALL.0 | BuildFlags::PACKAGEBINARY.0 | BuildFlags::FILECHECK.0;
        (flags.0 & binary_mask) != 0
    }
}

impl Default for BuildArgs {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned by [`Spec::build`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildError {
    /// Generic build failure.
    Failed,
    /// Build requirements are missing.
    MissingBuildRequires,
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::Failed => write!(f, "build failed"),
            BuildError::MissingBuildRequires => write!(f, "missing build requirements"),
        }
    }
}

impl std::error::Error for BuildError {}

/// A parsed RPM spec file.
///
/// Wraps librpmbuild's `rpmSpec` handle. Owns the underlying allocation and frees it on drop.
pub struct Spec {
    ptr: librpmbuild_sys::rpmSpec,
}

// Safety: rpmSpec is a heap-allocated, refcounted handle with no thread-local references.
// All FFI calls that touch process-global state (rpmSpecParse, rpmSpecBuild) are serialized
// by rpm_global_lock(). Read-only accessors (source_header, get_section, packages, sources)
// operate on data owned by the handle.
unsafe impl Send for Spec {}

impl Spec {
    /// Parse a spec file for building with the given [`BuildArgs`].
    ///
    /// This automatically applies [`SpecFlags::NOFINALIZE`] when the build args include
    /// binary stages, matching `rpmbuild`'s behavior.  For source-only builds, finalization
    /// happens at parse time (see `docs/quirks.md`).
    ///
    /// Returns `None` if parsing fails (e.g. the file does not exist or contains errors).
    ///
    /// # Panics
    ///
    /// Panics if `spec_file` or `build_root` contain interior NUL bytes.
    pub fn parse_for_build(
        spec_file: &str,
        flags: SpecFlags,
        args: &BuildArgs,
        build_root: Option<&Path>,
    ) -> Option<Self> {
        let mut flags = flags;
        if args.needs_nofinalize() {
            flags |= SpecFlags::nofinalize_or_none();
        }
        Self::parse(spec_file, flags, build_root)
    }

    /// Parse a spec file.
    ///
    /// When parsing a spec that will be built, prefer [`parse_for_build()`](Self::parse_for_build) which handles
    /// [`SpecFlags::NOFINALIZE`] automatically.
    ///
    /// Returns `None` if parsing fails (e.g. the file does not exist or contains errors).
    ///
    /// # Panics
    ///
    /// Panics if `spec_file` or `build_root` contain interior NUL bytes.
    pub fn parse(spec_file: &str, flags: SpecFlags, build_root: Option<&Path>) -> Option<Self> {
        let _lock = crate::internal::rpm_global_lock();

        let spec_c = CString::new(spec_file).expect("spec path contains NUL byte");
        let root_c = build_root
            .map(|p| CString::new(p.as_os_str().as_bytes()).expect("build root contains NUL byte"));
        let root_ptr = root_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr());

        let ptr = unsafe { librpmbuild_sys::rpmSpecParse(spec_c.as_ptr(), flags.0, root_ptr) };
        if ptr.is_null() {
            None
        } else {
            Some(Spec { ptr })
        }
    }

    /// Return the header of the SRPM that would be built from this spec.
    pub fn source_header(&self) -> PackageHeader {
        let hdr = unsafe { librpmbuild_sys::rpmSpecSourceHeader(self.ptr) };
        assert!(!hdr.is_null());
        // rpmSpecSourceHeader returns a borrowed header; Header::from_ptr
        // increments the refcount so our PackageHeader can outlive the Spec.
        let header = unsafe { Header::from_ptr(hdr.cast()) };
        PackageHeader::from_header(&header)
    }

    /// Retrieve a parsed spec script section.
    ///
    /// Use [`Section::PREP`], [`Section::BUILD`], etc. As a special case,
    /// [`Section::NONE`] returns the entire spec in preprocessed format.
    pub fn get_section(&self, section: Section) -> Option<&str> {
        let p = unsafe { librpmbuild_sys::rpmSpecGetSection(self.ptr, section.0) };
        if p.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(p) }
                    .to_str()
                    .expect("spec section is not UTF-8"),
            )
        }
    }

    /// Iterate over the binary (sub)packages defined in this spec.
    pub fn packages(&self) -> SpecPkgIter<'_> {
        let iter = unsafe { librpmbuild_sys::rpmSpecPkgIterInit(self.ptr) };
        SpecPkgIter {
            iter,
            _spec: std::marker::PhantomData,
        }
    }

    /// Iterate over the source and patch entries in this spec.
    pub fn sources(&self) -> SpecSrcIter<'_> {
        let iter = unsafe { librpmbuild_sys::rpmSpecSrcIterInit(self.ptr) };
        SpecSrcIter {
            iter,
            _spec: std::marker::PhantomData,
        }
    }

    /// Build packages according to the given [`BuildArgs`].
    ///
    /// The spec should have been parsed with [`parse_for_build()`](Self::parse_for_build),
    /// which handles `NOFINALIZE` automatically.  See `docs/quirks.md`.
    pub fn build(&mut self, args: &BuildArgs) -> Result<(), BuildError> {
        let build_flags = args.build_flags();
        let txn = TransactionSet::create();

        let mut args: librpmbuild_sys::rpmBuildArguments_s = unsafe { std::mem::zeroed() };
        args.buildAmount = build_flags.0 as u32;

        let rc = {
            let _lock = crate::internal::rpm_global_lock();
            unsafe { librpmbuild_sys::rpmSpecBuild(txn.as_ptr().cast(), self.ptr, &mut args) }
        };

        match rc {
            0 => Ok(()),
            rc if rc == librpmbuild_sys::RPMRC_MISSINGBUILDREQUIRES as i32 => {
                Err(BuildError::MissingBuildRequires)
            }
            _ => Err(BuildError::Failed),
        }
    }
}

impl Drop for Spec {
    fn drop(&mut self) {
        unsafe {
            librpmbuild_sys::rpmSpecFree(self.ptr);
        }
    }
}

impl fmt::Debug for Spec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Spec").finish_non_exhaustive()
    }
}

/// Iterator over the binary (sub)packages in a [`Spec`].
pub struct SpecPkgIter<'spec> {
    iter: librpmbuild_sys::rpmSpecPkgIter,
    _spec: std::marker::PhantomData<&'spec Spec>,
}

/// A binary (sub)package entry from a spec file.
pub struct SpecPkg {
    ptr: librpmbuild_sys::rpmSpecPkg,
}

impl SpecPkg {
    /// Get this sub-package as a [`PackageHeader`] (for accessing name, version, etc.).
    pub fn header(&self) -> PackageHeader {
        let hdr = unsafe { librpmbuild_sys::rpmSpecPkgHeader(self.ptr) };
        assert!(!hdr.is_null());
        let header = unsafe { Header::from_ptr(hdr.cast()) };
        PackageHeader::from_header(&header)
    }

    /// Convenience: the package name.
    pub fn name(&self) -> String {
        self.header().name().to_owned()
    }

    /// Retrieve a package-specific parsed section (FILE_LIST, FILE_FILE, POLICY).
    ///
    /// Returns `None` if the section is empty. The returned string is owned.
    pub fn get_section(&self, section: Section) -> Option<String> {
        let p = unsafe { librpmbuild_sys::rpmSpecPkgGetSection(self.ptr, section.0) };
        if p.is_null() {
            None
        } else {
            let s = unsafe { CStr::from_ptr(p) }
                .to_str()
                .expect("section is not UTF-8")
                .to_owned();
            unsafe { free(p.cast()) };
            Some(s)
        }
    }
}

impl<'spec> Iterator for SpecPkgIter<'spec> {
    type Item = SpecPkg;

    fn next(&mut self) -> Option<SpecPkg> {
        let ptr = unsafe { librpmbuild_sys::rpmSpecPkgIterNext(self.iter) };
        if ptr.is_null() {
            None
        } else {
            Some(SpecPkg { ptr })
        }
    }
}

impl<'spec> Drop for SpecPkgIter<'spec> {
    fn drop(&mut self) {
        unsafe {
            librpmbuild_sys::rpmSpecPkgIterFree(self.iter);
        }
    }
}

/// Iterator over the source/patch entries in a [`Spec`].
pub struct SpecSrcIter<'spec> {
    iter: librpmbuild_sys::rpmSpecSrcIter,
    _spec: std::marker::PhantomData<&'spec Spec>,
}

/// A source or patch entry from a spec file.
pub struct SpecSrc {
    ptr: librpmbuild_sys::rpmSpecSrc,
}

impl SpecSrc {
    /// Base filename of this source or patch.
    pub fn filename(&self) -> &str {
        let p = unsafe { librpmbuild_sys::rpmSpecSrcFilename(self.ptr, 0) };
        assert!(!p.is_null());
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("filename is not UTF-8")
    }

    /// Full path to this source or patch (including `%_sourcedir`).
    pub fn full_path(&self) -> &str {
        let p = unsafe { librpmbuild_sys::rpmSpecSrcFilename(self.ptr, 1) };
        assert!(!p.is_null());
        unsafe { CStr::from_ptr(p) }
            .to_str()
            .expect("full path is not UTF-8")
    }

    /// Flags describing the kind of entry.
    pub fn flags(&self) -> SourceFlags {
        SourceFlags(unsafe { librpmbuild_sys::rpmSpecSrcFlags(self.ptr) })
    }

    /// Source or patch number (e.g. `Source0` => 0, `Patch3` => 3).
    pub fn num(&self) -> i32 {
        unsafe { librpmbuild_sys::rpmSpecSrcNum(self.ptr) }
    }

    /// True if this entry is a source (not a patch).
    pub fn is_source(&self) -> bool {
        self.flags().contains(SourceFlags::SOURCE)
    }

    /// True if this entry is a patch.
    pub fn is_patch(&self) -> bool {
        self.flags().contains(SourceFlags::PATCH)
    }
}

impl<'spec> Iterator for SpecSrcIter<'spec> {
    type Item = SpecSrc;

    fn next(&mut self) -> Option<SpecSrc> {
        let ptr = unsafe { librpmbuild_sys::rpmSpecSrcIterNext(self.iter) };
        if ptr.is_null() {
            None
        } else {
            Some(SpecSrc { ptr })
        }
    }
}

impl<'spec> Drop for SpecSrcIter<'spec> {
    fn drop(&mut self) {
        unsafe {
            librpmbuild_sys::rpmSpecSrcIterFree(self.iter);
        }
    }
}
