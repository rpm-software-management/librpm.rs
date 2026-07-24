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

//! Tags are identifiers for RPM headers

#![allow(
    dead_code,
    missing_docs,
    non_camel_case_types,
    clippy::upper_case_acronyms
)]

use std::ffi::{CStr, CString};
use std::fmt;
use std::str::FromStr;

use num_traits::FromPrimitive;

use crate::Index;

/// Identifiers for data in RPM headers (`rpmTag_e` in librpm)
#[repr(isize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, num_derive::FromPrimitive)]
pub enum Tag {
    /// Unknown tag
    NOT_FOUND = librpm_sys::Workarounds_W_RPMTAG_NOT_FOUND as isize,

    // -----------------------------------------------------------------------
    // Header private tags (61–100)
    // -----------------------------------------------------------------------
    /// Current image
    HEADERIMAGE = librpm_sys::rpmTag_e_RPMTAG_HEADERIMAGE as isize,

    /// Signatures
    HEADERSIGNATURES = librpm_sys::rpmTag_e_RPMTAG_HEADERSIGNATURES as isize,

    /// Original image
    HEADERIMMUTABLE = librpm_sys::rpmTag_e_RPMTAG_HEADERIMMUTABLE as isize,

    /// Regions
    HEADERREGIONS = librpm_sys::rpmTag_e_RPMTAG_HEADERREGIONS as isize,

    /// I18N string locales
    HEADERI18NTABLE = librpm_sys::rpmTag_e_RPMTAG_HEADERI18NTABLE as isize,

    // -----------------------------------------------------------------------
    // Signature tags (SIG_BASE = 256, SIG_TOP = 999)
    // -----------------------------------------------------------------------
    /// Sentinel for beginning of signature tag range (256)
    SIG_BASE = librpm_sys::rpmTag_e_RPMTAG_SIG_BASE as isize,

    /// Header + payload size, 32-bit (int)
    SIGSIZE = librpm_sys::rpmTag_e_RPMTAG_SIGSIZE as isize,

    /// PGP 2.6.3 signature (binary)
    SIGPGP = librpm_sys::rpmTag_e_RPMTAG_SIGPGP as isize,

    /// MD5 checksum of header + payload (binary)
    SIGMD5 = librpm_sys::rpmTag_e_RPMTAG_SIGMD5 as isize,

    /// GnuPG signature (binary)
    SIGGPG = librpm_sys::rpmTag_e_RPMTAG_SIGGPG as isize,

    /// Embedded public keys (string array)
    PUBKEYS = librpm_sys::rpmTag_e_RPMTAG_PUBKEYS as isize,

    /// DSA header signature (binary)
    DSAHEADER = librpm_sys::rpmTag_e_RPMTAG_DSAHEADER as isize,

    /// RSA header signature (binary)
    RSAHEADER = librpm_sys::rpmTag_e_RPMTAG_RSAHEADER as isize,

    /// SHA1 digest of the immutable header region (string)
    SHA1HEADER = librpm_sys::rpmTag_e_RPMTAG_SHA1HEADER as isize,

    /// Header + payload size, 64-bit (int64)
    LONGSIGSIZE = librpm_sys::rpmTag_e_RPMTAG_LONGSIGSIZE as isize,

    /// Uncompressed payload size, 64-bit (int64)
    LONGARCHIVESIZE = librpm_sys::rpmTag_e_RPMTAG_LONGARCHIVESIZE as isize,

    /// SHA-256 digest of the immutable header region (string)
    SHA256HEADER = librpm_sys::rpmTag_e_RPMTAG_SHA256HEADER as isize,

    /// fsverity signatures (string array)
    #[cfg(has_rpmtag_veritysignatures)]
    VERITYSIGNATURES = librpm_sys::rpmTag_e_RPMTAG_VERITYSIGNATURES as isize,

    /// fsverity signature algorithm (int)
    #[cfg(has_rpmtag_veritysignaturealgo)]
    VERITYSIGNATUREALGO = librpm_sys::rpmTag_e_RPMTAG_VERITYSIGNATUREALGO as isize,

    /// OpenPGP header-only signatures (string array)
    #[cfg(has_rpmtag_openpgp)]
    OPENPGP = librpm_sys::rpmTag_e_RPMTAG_OPENPGP as isize,

    /// SHA3-256 digest of the immutable header region (string)
    #[cfg(has_rpmtag_sha3_256header)]
    SHA3_256HEADER = librpm_sys::rpmTag_e_RPMTAG_SHA3_256HEADER as isize,

    /// Sentinel for end of signature tag range (999)
    #[cfg(has_rpmtag_sig_top)]
    SIG_TOP = librpm_sys::rpmTag_e_RPMTAG_SIG_TOP as isize,

    // -----------------------------------------------------------------------
    // Standard package tags (1000+)
    // -----------------------------------------------------------------------
    /// Package name (string)
    NAME = librpm_sys::rpmTag_e_RPMTAG_NAME as isize,

    /// Package version (string)
    VERSION = librpm_sys::rpmTag_e_RPMTAG_VERSION as isize,

    /// Package release (string)
    RELEASE = librpm_sys::rpmTag_e_RPMTAG_RELEASE as isize,

    /// Package epoch (int)
    EPOCH = librpm_sys::rpmTag_e_RPMTAG_EPOCH as isize,

    /// One-line package summary (I18N string)
    SUMMARY = librpm_sys::rpmTag_e_RPMTAG_SUMMARY as isize,

    /// Multi-line package description (I18N string)
    DESCRIPTION = librpm_sys::rpmTag_e_RPMTAG_DESCRIPTION as isize,

    /// Unix timestamp of when the package was built (int)
    BUILDTIME = librpm_sys::rpmTag_e_RPMTAG_BUILDTIME as isize,

    /// Hostname of the build machine (string)
    BUILDHOST = librpm_sys::rpmTag_e_RPMTAG_BUILDHOST as isize,

    /// Unix timestamp of when the package was installed (int)
    INSTALLTIME = librpm_sys::rpmTag_e_RPMTAG_INSTALLTIME as isize,

    /// Installed package size in bytes (int)
    SIZE = librpm_sys::rpmTag_e_RPMTAG_SIZE as isize,

    /// Distribution the package was built for (string)
    DISTRIBUTION = librpm_sys::rpmTag_e_RPMTAG_DISTRIBUTION as isize,

    /// Vendor who provided the package (string)
    VENDOR = librpm_sys::rpmTag_e_RPMTAG_VENDOR as isize,

    /// GIF icon (binary)
    GIF = librpm_sys::rpmTag_e_RPMTAG_GIF as isize,

    /// XPM icon (binary)
    XPM = librpm_sys::rpmTag_e_RPMTAG_XPM as isize,

    /// License or copyright of the package (string)
    LICENSE = librpm_sys::rpmTag_e_RPMTAG_LICENSE as isize,

    /// Name/email of the package maintainer (string)
    PACKAGER = librpm_sys::rpmTag_e_RPMTAG_PACKAGER as isize,

    /// Package group/category (I18N string)
    GROUP = librpm_sys::rpmTag_e_RPMTAG_GROUP as isize,

    /// Changelog entries, stored internally as parallel arrays; see CHANGELOGTIME/CHANGELOGNAME/CHANGELOGTEXT (string array, internal)
    CHANGELOG = librpm_sys::rpmTag_e_RPMTAG_CHANGELOG as isize,

    /// Source filenames (string array)
    SOURCE = librpm_sys::rpmTag_e_RPMTAG_SOURCE as isize,

    /// Patch filenames (string array)
    PATCH = librpm_sys::rpmTag_e_RPMTAG_PATCH as isize,

    /// Upstream project URL (string)
    URL = librpm_sys::rpmTag_e_RPMTAG_URL as isize,

    /// Target OS (string; historically stored as int)
    OS = librpm_sys::rpmTag_e_RPMTAG_OS as isize,

    /// Target architecture (string; historically stored as int)
    ARCH = librpm_sys::rpmTag_e_RPMTAG_ARCH as isize,

    /// Pre-install scriptlet (string)
    PREIN = librpm_sys::rpmTag_e_RPMTAG_PREIN as isize,

    /// Post-install scriptlet (string)
    POSTIN = librpm_sys::rpmTag_e_RPMTAG_POSTIN as isize,

    /// Pre-uninstall scriptlet (string)
    PREUN = librpm_sys::rpmTag_e_RPMTAG_PREUN as isize,

    /// Post-uninstall scriptlet (string)
    POSTUN = librpm_sys::rpmTag_e_RPMTAG_POSTUN as isize,

    /// Old-style flat file list before DIRNAMES/BASENAMES split (string array, obsolete)
    OLDFILENAMES = librpm_sys::rpmTag_e_RPMTAG_OLDFILENAMES as isize,

    /// Per-file sizes in bytes (int array)
    FILESIZES = librpm_sys::rpmTag_e_RPMTAG_FILESIZES as isize,

    /// Per-file install states (char array)
    FILESTATES = librpm_sys::rpmTag_e_RPMTAG_FILESTATES as isize,

    /// Per-file Unix mode bits (int16 array)
    FILEMODES = librpm_sys::rpmTag_e_RPMTAG_FILEMODES as isize,

    /// Per-file device numbers (int16 array)
    FILERDEVS = librpm_sys::rpmTag_e_RPMTAG_FILERDEVS as isize,

    /// Per-file modification timestamps (int array)
    FILEMTIMES = librpm_sys::rpmTag_e_RPMTAG_FILEMTIMES as isize,

    /// Per-file digests (string array; algorithm given by FILEDIGESTALGO)
    FILEDIGESTS = librpm_sys::rpmTag_e_RPMTAG_FILEDIGESTS as isize,

    /// Per-file symlink targets (string array)
    FILELINKTOS = librpm_sys::rpmTag_e_RPMTAG_FILELINKTOS as isize,

    /// Per-file flags (RPMFILE_* bitmask, int array)
    FILEFLAGS = librpm_sys::rpmTag_e_RPMTAG_FILEFLAGS as isize,

    /// Root directory (internal, obsolete)
    ROOT = librpm_sys::rpmTag_e_RPMTAG_ROOT as isize,

    /// Per-file owning user names (string array)
    FILEUSERNAME = librpm_sys::rpmTag_e_RPMTAG_FILEUSERNAME as isize,

    /// Per-file owning group names (string array)
    FILEGROUPNAME = librpm_sys::rpmTag_e_RPMTAG_FILEGROUPNAME as isize,

    /// Package icon (binary)
    ICON = librpm_sys::rpmTag_e_RPMTAG_ICON as isize,

    /// NEVRA of the source RPM this binary was built from (string)
    SOURCERPM = librpm_sys::rpmTag_e_RPMTAG_SOURCERPM as isize,

    /// Per-file verification flags (RPMVERIFY_* bitmask, int array)
    FILEVERIFYFLAGS = librpm_sys::rpmTag_e_RPMTAG_FILEVERIFYFLAGS as isize,

    /// Compressed archive size in bytes, 32-bit (int)
    ARCHIVESIZE = librpm_sys::rpmTag_e_RPMTAG_ARCHIVESIZE as isize,

    /// Provided capability names (string array)
    PROVIDENAME = librpm_sys::rpmTag_e_RPMTAG_PROVIDENAME as isize,

    /// Requirement dependency flags (RPMSENSE_* bitmask, int array)
    REQUIREFLAGS = librpm_sys::rpmTag_e_RPMTAG_REQUIREFLAGS as isize,

    /// Required capability names (string array)
    REQUIRENAME = librpm_sys::rpmTag_e_RPMTAG_REQUIRENAME as isize,

    /// Required capability version strings (string array)
    REQUIREVERSION = librpm_sys::rpmTag_e_RPMTAG_REQUIREVERSION as isize,

    /// Source numbers excluded from binary package (int array)
    NOSOURCE = librpm_sys::rpmTag_e_RPMTAG_NOSOURCE as isize,

    /// Patch numbers excluded from binary package (int array)
    NOPATCH = librpm_sys::rpmTag_e_RPMTAG_NOPATCH as isize,

    /// Conflict dependency flags (RPMSENSE_* bitmask, int array)
    CONFLICTFLAGS = librpm_sys::rpmTag_e_RPMTAG_CONFLICTFLAGS as isize,

    /// Conflicting capability names (string array)
    CONFLICTNAME = librpm_sys::rpmTag_e_RPMTAG_CONFLICTNAME as isize,

    /// Conflicting capability version strings (string array)
    CONFLICTVERSION = librpm_sys::rpmTag_e_RPMTAG_CONFLICTVERSION as isize,

    /// Default installation prefix (string, internal, deprecated)
    DEFAULTPREFIX = librpm_sys::rpmTag_e_RPMTAG_DEFAULTPREFIX as isize,

    /// Build root directory (string, internal, obsolete)
    BUILDROOT = librpm_sys::rpmTag_e_RPMTAG_BUILDROOT as isize,

    /// Installation prefix (string, internal, deprecated)
    INSTALLPREFIX = librpm_sys::rpmTag_e_RPMTAG_INSTALLPREFIX as isize,

    /// Architectures this package must not be built for (string array)
    EXCLUDEARCH = librpm_sys::rpmTag_e_RPMTAG_EXCLUDEARCH as isize,

    /// Operating systems this package must not be built for (string array)
    EXCLUDEOS = librpm_sys::rpmTag_e_RPMTAG_EXCLUDEOS as isize,

    /// Architectures this package may be built for (string array)
    EXCLUSIVEARCH = librpm_sys::rpmTag_e_RPMTAG_EXCLUSIVEARCH as isize,

    /// Operating systems this package may be built for (string array)
    EXCLUSIVEOS = librpm_sys::rpmTag_e_RPMTAG_EXCLUSIVEOS as isize,

    /// AutoReq/Prov spec file directive (string, internal)
    AUTOREQPROV = librpm_sys::rpmTag_e_RPMTAG_AUTOREQPROV as isize,

    /// Version of RPM used to build the package (string)
    RPMVERSION = librpm_sys::rpmTag_e_RPMTAG_RPMVERSION as isize,

    /// Trigger scriptlets (string array)
    TRIGGERSCRIPTS = librpm_sys::rpmTag_e_RPMTAG_TRIGGERSCRIPTS as isize,

    /// Trigger dependency names (string array)
    TRIGGERNAME = librpm_sys::rpmTag_e_RPMTAG_TRIGGERNAME as isize,

    /// Trigger dependency version strings (string array)
    TRIGGERVERSION = librpm_sys::rpmTag_e_RPMTAG_TRIGGERVERSION as isize,

    /// Trigger dependency flags (RPMSENSE_* bitmask, int array)
    TRIGGERFLAGS = librpm_sys::rpmTag_e_RPMTAG_TRIGGERFLAGS as isize,

    /// Per-trigger index into the TRIGGERSCRIPTS array (int array)
    TRIGGERINDEX = librpm_sys::rpmTag_e_RPMTAG_TRIGGERINDEX as isize,

    /// %verifyscript scriptlet (string)
    VERIFYSCRIPT = librpm_sys::rpmTag_e_RPMTAG_VERIFYSCRIPT as isize,

    /// Unix timestamps for changelog entries (int array)
    CHANGELOGTIME = librpm_sys::rpmTag_e_RPMTAG_CHANGELOGTIME as isize,

    /// Author strings for changelog entries (string array)
    CHANGELOGNAME = librpm_sys::rpmTag_e_RPMTAG_CHANGELOGNAME as isize,

    /// Text bodies of changelog entries (string array)
    CHANGELOGTEXT = librpm_sys::rpmTag_e_RPMTAG_CHANGELOGTEXT as isize,

    /// Prerequisite dependencies (internal; use REQUIRENAME with RPMSENSE_PREREQ)
    PREREQ = librpm_sys::rpmTag_e_RPMTAG_PREREQ as isize,

    /// Interpreter for the pre-install scriptlet (string array)
    PREINPROG = librpm_sys::rpmTag_e_RPMTAG_PREINPROG as isize,

    /// Interpreter for the post-install scriptlet (string array)
    POSTINPROG = librpm_sys::rpmTag_e_RPMTAG_POSTINPROG as isize,

    /// Interpreter for the pre-uninstall scriptlet (string array)
    PREUNPROG = librpm_sys::rpmTag_e_RPMTAG_PREUNPROG as isize,

    /// Interpreter for the post-uninstall scriptlet (string array)
    POSTUNPROG = librpm_sys::rpmTag_e_RPMTAG_POSTUNPROG as isize,

    /// Architectures to build for (string array)
    BUILDARCHS = librpm_sys::rpmTag_e_RPMTAG_BUILDARCHS as isize,

    /// Obsoleted capability names (string array)
    OBSOLETENAME = librpm_sys::rpmTag_e_RPMTAG_OBSOLETENAME as isize,

    /// Interpreter for the %verifyscript (string array)
    VERIFYSCRIPTPROG = librpm_sys::rpmTag_e_RPMTAG_VERIFYSCRIPTPROG as isize,

    /// Interpreter for trigger scriptlets (string array)
    TRIGGERSCRIPTPROG = librpm_sys::rpmTag_e_RPMTAG_TRIGGERSCRIPTPROG as isize,

    /// Documentation directory (internal)
    DOCDIR = librpm_sys::rpmTag_e_RPMTAG_DOCDIR as isize,

    /// Build cookie for reproducibility checking (string)
    COOKIE = librpm_sys::rpmTag_e_RPMTAG_COOKIE as isize,

    /// Per-file device numbers (int array)
    FILEDEVICES = librpm_sys::rpmTag_e_RPMTAG_FILEDEVICES as isize,

    /// Per-file inode numbers (int array)
    FILEINODES = librpm_sys::rpmTag_e_RPMTAG_FILEINODES as isize,

    /// Per-file locale/language tags (string array)
    FILELANGS = librpm_sys::rpmTag_e_RPMTAG_FILELANGS as isize,

    /// Relocatable prefixes (string array)
    PREFIXES = librpm_sys::rpmTag_e_RPMTAG_PREFIXES as isize,

    /// Prefixes at which the package was installed (string array)
    INSTPREFIXES = librpm_sys::rpmTag_e_RPMTAG_INSTPREFIXES as isize,

    /// Pre-install trigger (internal)
    TRIGGERIN = librpm_sys::rpmTag_e_RPMTAG_TRIGGERIN as isize,

    /// Pre-uninstall trigger (internal)
    TRIGGERUN = librpm_sys::rpmTag_e_RPMTAG_TRIGGERUN as isize,

    /// Post-uninstall trigger (internal)
    TRIGGERPOSTUN = librpm_sys::rpmTag_e_RPMTAG_TRIGGERPOSTUN as isize,

    /// AutoReq directive (internal)
    AUTOREQ = librpm_sys::rpmTag_e_RPMTAG_AUTOREQ as isize,

    /// AutoProv directive (internal)
    AUTOPROV = librpm_sys::rpmTag_e_RPMTAG_AUTOPROV as isize,

    /// Package capability mask (int, internal, obsolete)
    CAPABILITY = librpm_sys::rpmTag_e_RPMTAG_CAPABILITY as isize,

    /// Set to 1 if this header is for a source RPM (int)
    SOURCEPACKAGE = librpm_sys::rpmTag_e_RPMTAG_SOURCEPACKAGE as isize,

    /// Build prerequisites (internal; use REQUIRENAME)
    BUILDPREREQ = librpm_sys::rpmTag_e_RPMTAG_BUILDPREREQ as isize,

    /// Build-time requirements (internal; use REQUIRENAME)
    BUILDREQUIRES = librpm_sys::rpmTag_e_RPMTAG_BUILDREQUIRES as isize,

    /// Build-time conflicts (internal; use CONFLICTNAME)
    BUILDCONFLICTS = librpm_sys::rpmTag_e_RPMTAG_BUILDCONFLICTS as isize,

    /// Provide dependency flags (RPMSENSE_* bitmask, int array)
    PROVIDEFLAGS = librpm_sys::rpmTag_e_RPMTAG_PROVIDEFLAGS as isize,

    /// Provided capability version strings (string array)
    PROVIDEVERSION = librpm_sys::rpmTag_e_RPMTAG_PROVIDEVERSION as isize,

    /// Obsolete dependency flags (RPMSENSE_* bitmask, int array)
    OBSOLETEFLAGS = librpm_sys::rpmTag_e_RPMTAG_OBSOLETEFLAGS as isize,

    /// Obsoleted capability version strings (string array)
    OBSOLETEVERSION = librpm_sys::rpmTag_e_RPMTAG_OBSOLETEVERSION as isize,

    /// Directory indices into DIRNAMES for each file in BASENAMES (int array)
    DIRINDEXES = librpm_sys::rpmTag_e_RPMTAG_DIRINDEXES as isize,

    /// File basenames (string array; pair with DIRINDEXES + DIRNAMES)
    BASENAMES = librpm_sys::rpmTag_e_RPMTAG_BASENAMES as isize,

    /// Unique directory components referenced by files (string array)
    DIRNAMES = librpm_sys::rpmTag_e_RPMTAG_DIRNAMES as isize,

    /// Original directory indices before relocation (int array)
    ORIGDIRINDEXES = librpm_sys::rpmTag_e_RPMTAG_ORIGDIRINDEXES as isize,

    /// Original file basenames before relocation (string array)
    ORIGBASENAMES = librpm_sys::rpmTag_e_RPMTAG_ORIGBASENAMES as isize,

    /// Original directory names before relocation (string array)
    ORIGDIRNAMES = librpm_sys::rpmTag_e_RPMTAG_ORIGDIRNAMES as isize,

    /// Compiler optimization flags used during build (string)
    OPTFLAGS = librpm_sys::rpmTag_e_RPMTAG_OPTFLAGS as isize,

    /// URL to the package in a distribution (string)
    DISTURL = librpm_sys::rpmTag_e_RPMTAG_DISTURL as isize,

    /// Payload format, e.g. "cpio" (string)
    /// Note: Misleading - even "stripped" / RPMv6 payloads declare "cpio" despite not being cpio
    PAYLOADFORMAT = librpm_sys::rpmTag_e_RPMTAG_PAYLOADFORMAT as isize,

    /// Payload compression algorithm, e.g. "gzip", "xz" (string)
    PAYLOADCOMPRESSOR = librpm_sys::rpmTag_e_RPMTAG_PAYLOADCOMPRESSOR as isize,

    /// Payload compression level flags (string)
    PAYLOADFLAGS = librpm_sys::rpmTag_e_RPMTAG_PAYLOADFLAGS as isize,

    /// Transaction color assigned at install time (int)
    INSTALLCOLOR = librpm_sys::rpmTag_e_RPMTAG_INSTALLCOLOR as isize,

    /// Transaction ID of the installing transaction (int)
    INSTALLTID = librpm_sys::rpmTag_e_RPMTAG_INSTALLTID as isize,

    /// Transaction ID of the removing transaction (int)
    REMOVETID = librpm_sys::rpmTag_e_RPMTAG_REMOVETID as isize,

    /// Build platform string, e.g. "x86_64-redhat-linux" (string)
    PLATFORM = librpm_sys::rpmTag_e_RPMTAG_PLATFORM as isize,

    /// Patches dependency names — deprecated SuSE placeholder (string array)
    PATCHESNAME = librpm_sys::rpmTag_e_RPMTAG_PATCHESNAME as isize,

    /// Patches dependency flags — deprecated SuSE placeholder (int array)
    PATCHESFLAGS = librpm_sys::rpmTag_e_RPMTAG_PATCHESFLAGS as isize,

    /// Patches dependency versions — deprecated SuSE placeholder (string array)
    PATCHESVERSION = librpm_sys::rpmTag_e_RPMTAG_PATCHESVERSION as isize,

    /// Per-file ELF color flags for multilib (int array)
    FILECOLORS = librpm_sys::rpmTag_e_RPMTAG_FILECOLORS as isize,

    /// Per-file index into CLASSDICT (int array)
    FILECLASS = librpm_sys::rpmTag_e_RPMTAG_FILECLASS as isize,

    /// File class strings from `file(1)` (string array)
    CLASSDICT = librpm_sys::rpmTag_e_RPMTAG_CLASSDICT as isize,

    /// Per-file start index into DEPENDSDICT (int array)
    FILEDEPENDSX = librpm_sys::rpmTag_e_RPMTAG_FILEDEPENDSX as isize,

    /// Per-file count of entries in DEPENDSDICT (int array)
    FILEDEPENDSN = librpm_sys::rpmTag_e_RPMTAG_FILEDEPENDSN as isize,

    /// Flattened (tag, index) pairs for per-file dependencies (int array)
    DEPENDSDICT = librpm_sys::rpmTag_e_RPMTAG_DEPENDSDICT as isize,

    /// MD5 digest of the source RPM (binary)
    #[cfg(has_rpmtag_sourcesigmd5)]
    SOURCESIGMD5 = librpm_sys::rpmTag_e_RPMTAG_SOURCESIGMD5 as isize,

    /// Per-file SELinux file contexts (string array, obsolete)
    FILECONTEXTS = librpm_sys::rpmTag_e_RPMTAG_FILECONTEXTS as isize,

    /// Filesystem-level SELinux contexts (string array, extension)
    FSCONTEXTS = librpm_sys::rpmTag_e_RPMTAG_FSCONTEXTS as isize,

    /// RE-context policy entries (string array, extension)
    RECONTEXTS = librpm_sys::rpmTag_e_RPMTAG_RECONTEXTS as isize,

    /// SELinux *.te policy file contents (string array)
    POLICIES = librpm_sys::rpmTag_e_RPMTAG_POLICIES as isize,

    /// Pre-transaction scriptlet (string)
    PRETRANS = librpm_sys::rpmTag_e_RPMTAG_PRETRANS as isize,

    /// Post-transaction scriptlet (string)
    POSTTRANS = librpm_sys::rpmTag_e_RPMTAG_POSTTRANS as isize,

    /// Interpreter for the pre-transaction scriptlet (string array)
    PRETRANSPROG = librpm_sys::rpmTag_e_RPMTAG_PRETRANSPROG as isize,

    /// Interpreter for the post-transaction scriptlet (string array)
    POSTTRANSPROG = librpm_sys::rpmTag_e_RPMTAG_POSTTRANSPROG as isize,

    /// Distribution tag string, e.g. ".fc40" (string)
    DISTTAG = librpm_sys::rpmTag_e_RPMTAG_DISTTAG as isize,

    /// Old-style suggests dependency names (string array, obsolete)
    OLDSUGGESTSNAME = librpm_sys::rpmTag_e_RPMTAG_OLDSUGGESTSNAME as isize,

    /// Old-style suggests dependency versions (string array, obsolete)
    OLDSUGGESTSVERSION = librpm_sys::rpmTag_e_RPMTAG_OLDSUGGESTSVERSION as isize,

    /// Old-style suggests dependency flags (int array, obsolete)
    OLDSUGGESTSFLAGS = librpm_sys::rpmTag_e_RPMTAG_OLDSUGGESTSFLAGS as isize,

    /// Old-style enhances dependency names (string array, obsolete)
    OLDENHANCESNAME = librpm_sys::rpmTag_e_RPMTAG_OLDENHANCESNAME as isize,

    /// Old-style enhances dependency versions (string array, obsolete)
    OLDENHANCESVERSION = librpm_sys::rpmTag_e_RPMTAG_OLDENHANCESVERSION as isize,

    /// Old-style enhances dependency flags (int array, obsolete)
    OLDENHANCESFLAGS = librpm_sys::rpmTag_e_RPMTAG_OLDENHANCESFLAGS as isize,

    /// Priority placeholder — unimplemented (int array)
    PRIORITY = librpm_sys::rpmTag_e_RPMTAG_PRIORITY as isize,

    /// CVS/SVN ID — unimplemented (string)
    CVSID = librpm_sys::rpmTag_e_RPMTAG_CVSID as isize,

    /// Backward link package IDs — unimplemented (string array)
    BLINKPKGID = librpm_sys::rpmTag_e_RPMTAG_BLINKPKGID as isize,

    /// Backward link header IDs — unimplemented (string array)
    BLINKHDRID = librpm_sys::rpmTag_e_RPMTAG_BLINKHDRID as isize,

    /// Backward link NEVRAs — unimplemented (string array)
    BLINKNEVRA = librpm_sys::rpmTag_e_RPMTAG_BLINKNEVRA as isize,

    /// Forward link package IDs — unimplemented (string array)
    FLINKPKGID = librpm_sys::rpmTag_e_RPMTAG_FLINKPKGID as isize,

    /// Forward link header IDs — unimplemented (string array)
    FLINKHDRID = librpm_sys::rpmTag_e_RPMTAG_FLINKHDRID as isize,

    /// Forward link NEVRAs — unimplemented (string array)
    FLINKNEVRA = librpm_sys::rpmTag_e_RPMTAG_FLINKNEVRA as isize,

    /// Package origin — unimplemented (string)
    PACKAGEORIGIN = librpm_sys::rpmTag_e_RPMTAG_PACKAGEORIGIN as isize,

    /// Pre-install trigger (internal)
    TRIGGERPREIN = librpm_sys::rpmTag_e_RPMTAG_TRIGGERPREIN as isize,

    /// Build suggests — unimplemented, internal
    BUILDSUGGESTS = librpm_sys::rpmTag_e_RPMTAG_BUILDSUGGESTS as isize,

    /// Build enhances — unimplemented, internal
    BUILDENHANCES = librpm_sys::rpmTag_e_RPMTAG_BUILDENHANCES as isize,

    /// Scriptlet exit codes — unimplemented (int array)
    SCRIPTSTATES = librpm_sys::rpmTag_e_RPMTAG_SCRIPTSTATES as isize,

    /// Scriptlet execution times — unimplemented (int array)
    SCRIPTMETRICS = librpm_sys::rpmTag_e_RPMTAG_SCRIPTMETRICS as isize,

    /// Build CPU clock — unimplemented (int)
    BUILDCPUCLOCK = librpm_sys::rpmTag_e_RPMTAG_BUILDCPUCLOCK as isize,

    /// Per-file digest algorithms — unimplemented (int array)
    FILEDIGESTALGOS = librpm_sys::rpmTag_e_RPMTAG_FILEDIGESTALGOS as isize,

    /// Package variants — unimplemented (string array)
    VARIANTS = librpm_sys::rpmTag_e_RPMTAG_VARIANTS as isize,

    /// X major version — unimplemented (int)
    XMAJOR = librpm_sys::rpmTag_e_RPMTAG_XMAJOR as isize,

    /// X minor version — unimplemented (int)
    XMINOR = librpm_sys::rpmTag_e_RPMTAG_XMINOR as isize,

    /// Repository tag — unimplemented (string)
    REPOTAG = librpm_sys::rpmTag_e_RPMTAG_REPOTAG as isize,

    /// Package keywords — unimplemented (string array)
    KEYWORDS = librpm_sys::rpmTag_e_RPMTAG_KEYWORDS as isize,

    /// Build platforms — unimplemented (string array)
    BUILDPLATFORMS = librpm_sys::rpmTag_e_RPMTAG_BUILDPLATFORMS as isize,

    /// Package color — unimplemented (int)
    PACKAGECOLOR = librpm_sys::rpmTag_e_RPMTAG_PACKAGECOLOR as isize,

    /// Package preferred color — unimplemented (int)
    PACKAGEPREFCOLOR = librpm_sys::rpmTag_e_RPMTAG_PACKAGEPREFCOLOR as isize,

    /// Extended attributes dictionary — unimplemented (string array)
    XATTRSDICT = librpm_sys::rpmTag_e_RPMTAG_XATTRSDICT as isize,

    /// Per-file extended attributes indices — unimplemented (int array)
    FILEXATTRSX = librpm_sys::rpmTag_e_RPMTAG_FILEXATTRSX as isize,

    /// Dependency attributes dictionary — unimplemented (string array)
    DEPATTRSDICT = librpm_sys::rpmTag_e_RPMTAG_DEPATTRSDICT as isize,

    /// Per-conflict extended attribute indices — unimplemented (int array)
    CONFLICTATTRSX = librpm_sys::rpmTag_e_RPMTAG_CONFLICTATTRSX as isize,

    /// Per-obsolete extended attribute indices — unimplemented (int array)
    OBSOLETEATTRSX = librpm_sys::rpmTag_e_RPMTAG_OBSOLETEATTRSX as isize,

    /// Per-provide extended attribute indices — unimplemented (int array)
    PROVIDEATTRSX = librpm_sys::rpmTag_e_RPMTAG_PROVIDEATTRSX as isize,

    /// Per-require extended attribute indices — unimplemented (int array)
    REQUIREATTRSX = librpm_sys::rpmTag_e_RPMTAG_REQUIREATTRSX as isize,

    /// Build provides — unimplemented, internal
    BUILDPROVIDES = librpm_sys::rpmTag_e_RPMTAG_BUILDPROVIDES as isize,

    /// Build obsoletes — unimplemented, internal
    BUILDOBSOLETES = librpm_sys::rpmTag_e_RPMTAG_BUILDOBSOLETES as isize,

    /// Installed package database instance number (int, extension)
    DBINSTANCE = librpm_sys::rpmTag_e_RPMTAG_DBINSTANCE as isize,

    /// Name-Epoch-Version-Release.Arch string (string, extension)
    NVRA = librpm_sys::rpmTag_e_RPMTAG_NVRA as isize,

    // -----------------------------------------------------------------------
    // Extension / virtual tags (5000+)
    // -----------------------------------------------------------------------
    /// Full path for each file, assembled from DIRNAMES + DIRINDEXES + BASENAMES (string array, extension)
    FILENAMES = librpm_sys::rpmTag_e_RPMTAG_FILENAMES as isize,

    /// Per-file provided capabilities (string array, extension)
    FILEPROVIDE = librpm_sys::rpmTag_e_RPMTAG_FILEPROVIDE as isize,

    /// Per-file required capabilities (string array, extension)
    FILEREQUIRE = librpm_sys::rpmTag_e_RPMTAG_FILEREQUIRE as isize,

    /// Filesystem names — unimplemented (string array)
    FSNAMES = librpm_sys::rpmTag_e_RPMTAG_FSNAMES as isize,

    /// Filesystem sizes — unimplemented (int64 array)
    FSSIZES = librpm_sys::rpmTag_e_RPMTAG_FSSIZES as isize,

    /// Human-readable trigger condition strings (string array, extension)
    TRIGGERCONDS = librpm_sys::rpmTag_e_RPMTAG_TRIGGERCONDS as isize,

    /// Human-readable trigger type strings (string array, extension)
    TRIGGERTYPE = librpm_sys::rpmTag_e_RPMTAG_TRIGGERTYPE as isize,

    /// Original full file paths before relocation (string array, extension)
    ORIGFILENAMES = librpm_sys::rpmTag_e_RPMTAG_ORIGFILENAMES as isize,

    /// Per-file sizes in bytes, 64-bit (int64 array)
    LONGFILESIZES = librpm_sys::rpmTag_e_RPMTAG_LONGFILESIZES as isize,

    /// Installed package size in bytes, 64-bit (int64)
    LONGSIZE = librpm_sys::rpmTag_e_RPMTAG_LONGSIZE as isize,

    /// Per-file POSIX capabilities strings (string array)
    FILECAPS = librpm_sys::rpmTag_e_RPMTAG_FILECAPS as isize,

    /// Digest algorithm used for FILEDIGESTS (PGPHASHALGO_* int)
    FILEDIGESTALGO = librpm_sys::rpmTag_e_RPMTAG_FILEDIGESTALGO as isize,

    /// Bug tracker URL (string)
    BUGURL = librpm_sys::rpmTag_e_RPMTAG_BUGURL as isize,

    /// Epoch:Version-Release string (string, extension)
    EVR = librpm_sys::rpmTag_e_RPMTAG_EVR as isize,

    /// Name-Version-Release string (string, extension)
    NVR = librpm_sys::rpmTag_e_RPMTAG_NVR as isize,

    /// Name-Epoch:Version-Release string (string, extension)
    NEVR = librpm_sys::rpmTag_e_RPMTAG_NEVR as isize,

    /// Name-Epoch:Version-Release.Arch string (string, extension)
    NEVRA = librpm_sys::rpmTag_e_RPMTAG_NEVRA as isize,

    /// Header color for multilib compatibility (int, extension)
    HEADERCOLOR = librpm_sys::rpmTag_e_RPMTAG_HEADERCOLOR as isize,

    /// Verbose query flag (int, extension)
    VERBOSE = librpm_sys::rpmTag_e_RPMTAG_VERBOSE as isize,

    /// Epoch as a plain integer (int, extension; 0 when EPOCH is absent)
    EPOCHNUM = librpm_sys::rpmTag_e_RPMTAG_EPOCHNUM as isize,

    /// Pre-install scriptlet flags (RPMSCRIPT_* bitmask, int)
    PREINFLAGS = librpm_sys::rpmTag_e_RPMTAG_PREINFLAGS as isize,

    /// Post-install scriptlet flags (RPMSCRIPT_* bitmask, int)
    POSTINFLAGS = librpm_sys::rpmTag_e_RPMTAG_POSTINFLAGS as isize,

    /// Pre-uninstall scriptlet flags (RPMSCRIPT_* bitmask, int)
    PREUNFLAGS = librpm_sys::rpmTag_e_RPMTAG_PREUNFLAGS as isize,

    /// Post-uninstall scriptlet flags (RPMSCRIPT_* bitmask, int)
    POSTUNFLAGS = librpm_sys::rpmTag_e_RPMTAG_POSTUNFLAGS as isize,

    /// Pre-transaction scriptlet flags (RPMSCRIPT_* bitmask, int)
    PRETRANSFLAGS = librpm_sys::rpmTag_e_RPMTAG_PRETRANSFLAGS as isize,

    /// Post-transaction scriptlet flags (RPMSCRIPT_* bitmask, int)
    POSTTRANSFLAGS = librpm_sys::rpmTag_e_RPMTAG_POSTTRANSFLAGS as isize,

    /// %verifyscript flags (RPMSCRIPT_* bitmask, int)
    VERIFYSCRIPTFLAGS = librpm_sys::rpmTag_e_RPMTAG_VERIFYSCRIPTFLAGS as isize,

    /// Per-trigger scriptlet flags (RPMSCRIPT_* bitmask, int array)
    TRIGGERSCRIPTFLAGS = librpm_sys::rpmTag_e_RPMTAG_TRIGGERSCRIPTFLAGS as isize,

    /// List of collections — unimplemented (string array)
    COLLECTIONS = librpm_sys::rpmTag_e_RPMTAG_COLLECTIONS as isize,

    /// SELinux policy module names (string array)
    POLICYNAMES = librpm_sys::rpmTag_e_RPMTAG_POLICYNAMES as isize,

    /// SELinux policy module types (string array)
    POLICYTYPES = librpm_sys::rpmTag_e_RPMTAG_POLICYTYPES as isize,

    /// Per-policy-type indices into POLICYNAMES (int array)
    POLICYTYPESINDEXES = librpm_sys::rpmTag_e_RPMTAG_POLICYTYPESINDEXES as isize,

    /// SELinux policy flags (int array)
    POLICYFLAGS = librpm_sys::rpmTag_e_RPMTAG_POLICYFLAGS as isize,

    /// Version control system URL (string)
    VCS = librpm_sys::rpmTag_e_RPMTAG_VCS as isize,

    /// Ordering dependency names (string array)
    ORDERNAME = librpm_sys::rpmTag_e_RPMTAG_ORDERNAME as isize,

    /// Ordering dependency version strings (string array)
    ORDERVERSION = librpm_sys::rpmTag_e_RPMTAG_ORDERVERSION as isize,

    /// Ordering dependency flags (int array)
    ORDERFLAGS = librpm_sys::rpmTag_e_RPMTAG_ORDERFLAGS as isize,

    /// MSSF manifest — unimplemented reservation (string array)
    MSSFMANIFEST = librpm_sys::rpmTag_e_RPMTAG_MSSFMANIFEST as isize,

    /// MSSF domain — unimplemented reservation (string array)
    MSSFDOMAIN = librpm_sys::rpmTag_e_RPMTAG_MSSFDOMAIN as isize,

    /// Full installed file paths (string array, extension)
    INSTFILENAMES = librpm_sys::rpmTag_e_RPMTAG_INSTFILENAMES as isize,

    /// NEVR strings for each requirement (string array, extension)
    REQUIRENEVRS = librpm_sys::rpmTag_e_RPMTAG_REQUIRENEVRS as isize,

    /// NEVR strings for each provided capability (string array, extension)
    PROVIDENEVRS = librpm_sys::rpmTag_e_RPMTAG_PROVIDENEVRS as isize,

    /// NEVR strings for each obsoleted capability (string array, extension)
    OBSOLETENEVRS = librpm_sys::rpmTag_e_RPMTAG_OBSOLETENEVRS as isize,

    /// NEVR strings for each conflicting capability (string array, extension)
    CONFLICTNEVRS = librpm_sys::rpmTag_e_RPMTAG_CONFLICTNEVRS as isize,

    /// Per-file hard link counts (int array, extension)
    FILENLINKS = librpm_sys::rpmTag_e_RPMTAG_FILENLINKS as isize,

    // -----------------------------------------------------------------------
    // Weak dependency tags
    // -----------------------------------------------------------------------
    /// Recommends dependency names (string array)
    RECOMMENDNAME = librpm_sys::rpmTag_e_RPMTAG_RECOMMENDNAME as isize,

    /// Recommends dependency version strings (string array)
    RECOMMENDVERSION = librpm_sys::rpmTag_e_RPMTAG_RECOMMENDVERSION as isize,

    /// Recommends dependency flags (RPMSENSE_* bitmask, int array)
    RECOMMENDFLAGS = librpm_sys::rpmTag_e_RPMTAG_RECOMMENDFLAGS as isize,

    /// Suggests dependency names (string array)
    SUGGESTNAME = librpm_sys::rpmTag_e_RPMTAG_SUGGESTNAME as isize,

    /// Suggests dependency version strings (string array)
    SUGGESTVERSION = librpm_sys::rpmTag_e_RPMTAG_SUGGESTVERSION as isize,

    /// Suggests dependency flags (RPMSENSE_* bitmask, int array)
    SUGGESTFLAGS = librpm_sys::rpmTag_e_RPMTAG_SUGGESTFLAGS as isize,

    /// Supplements dependency names (string array)
    SUPPLEMENTNAME = librpm_sys::rpmTag_e_RPMTAG_SUPPLEMENTNAME as isize,

    /// Supplements dependency version strings (string array)
    SUPPLEMENTVERSION = librpm_sys::rpmTag_e_RPMTAG_SUPPLEMENTVERSION as isize,

    /// Supplements dependency flags (RPMSENSE_* bitmask, int array)
    SUPPLEMENTFLAGS = librpm_sys::rpmTag_e_RPMTAG_SUPPLEMENTFLAGS as isize,

    /// Enhances dependency names (string array)
    ENHANCENAME = librpm_sys::rpmTag_e_RPMTAG_ENHANCENAME as isize,

    /// Enhances dependency version strings (string array)
    ENHANCEVERSION = librpm_sys::rpmTag_e_RPMTAG_ENHANCEVERSION as isize,

    /// Enhances dependency flags (RPMSENSE_* bitmask, int array)
    ENHANCEFLAGS = librpm_sys::rpmTag_e_RPMTAG_ENHANCEFLAGS as isize,

    /// NEVR strings for each recommend dependency (string array, extension)
    RECOMMENDNEVRS = librpm_sys::rpmTag_e_RPMTAG_RECOMMENDNEVRS as isize,

    /// NEVR strings for each suggest dependency (string array, extension)
    SUGGESTNEVRS = librpm_sys::rpmTag_e_RPMTAG_SUGGESTNEVRS as isize,

    /// NEVR strings for each supplement dependency (string array, extension)
    SUPPLEMENTNEVRS = librpm_sys::rpmTag_e_RPMTAG_SUPPLEMENTNEVRS as isize,

    /// NEVR strings for each enhance dependency (string array, extension)
    ENHANCENEVRS = librpm_sys::rpmTag_e_RPMTAG_ENHANCENEVRS as isize,

    /// Payload content encoding, e.g. "utf-8" (string)
    ENCODING = librpm_sys::rpmTag_e_RPMTAG_ENCODING as isize,

    // -----------------------------------------------------------------------
    // File trigger tags
    // -----------------------------------------------------------------------
    /// File trigger scriptlets (string array)
    FILETRIGGERSCRIPTS = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERSCRIPTS as isize,

    /// Interpreters for file trigger scriptlets (string array)
    FILETRIGGERSCRIPTPROG = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERSCRIPTPROG as isize,

    /// File trigger scriptlet flags (RPMSCRIPT_* bitmask, int array)
    FILETRIGGERSCRIPTFLAGS = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERSCRIPTFLAGS as isize,

    /// File trigger dependency names/globs (string array)
    FILETRIGGERNAME = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERNAME as isize,

    /// Per-file-trigger index into FILETRIGGERSCRIPTS (int array)
    FILETRIGGERINDEX = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERINDEX as isize,

    /// File trigger dependency version strings (string array)
    FILETRIGGERVERSION = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERVERSION as isize,

    /// File trigger dependency flags (RPMSENSE_* bitmask, int array)
    FILETRIGGERFLAGS = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERFLAGS as isize,

    /// Transactional file trigger scriptlets (string array)
    TRANSFILETRIGGERSCRIPTS = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERSCRIPTS as isize,

    /// Interpreters for transactional file trigger scriptlets (string array)
    TRANSFILETRIGGERSCRIPTPROG = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERSCRIPTPROG as isize,

    /// Transactional file trigger scriptlet flags (RPMSCRIPT_* bitmask, int array)
    TRANSFILETRIGGERSCRIPTFLAGS = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERSCRIPTFLAGS as isize,

    /// Transactional file trigger dependency names/globs (string array)
    TRANSFILETRIGGERNAME = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERNAME as isize,

    /// Per-transactional-file-trigger index into TRANSFILETRIGGERSCRIPTS (int array)
    TRANSFILETRIGGERINDEX = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERINDEX as isize,

    /// Transactional file trigger dependency version strings (string array)
    TRANSFILETRIGGERVERSION = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERVERSION as isize,

    /// Transactional file trigger dependency flags (RPMSENSE_* bitmask, int array)
    TRANSFILETRIGGERFLAGS = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERFLAGS as isize,

    /// Per-file-trigger scriptlet priorities (int array)
    FILETRIGGERPRIORITIES = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERPRIORITIES as isize,

    /// Per-transactional-file-trigger scriptlet priorities (int array)
    TRANSFILETRIGGERPRIORITIES = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERPRIORITIES as isize,

    /// Human-readable file trigger condition strings (string array, extension)
    FILETRIGGERCONDS = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERCONDS as isize,

    /// Human-readable file trigger type strings (string array, extension)
    FILETRIGGERTYPE = librpm_sys::rpmTag_e_RPMTAG_FILETRIGGERTYPE as isize,

    /// Human-readable transactional file trigger condition strings (string array, extension)
    TRANSFILETRIGGERCONDS = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERCONDS as isize,

    /// Human-readable transactional file trigger type strings (string array, extension)
    TRANSFILETRIGGERTYPE = librpm_sys::rpmTag_e_RPMTAG_TRANSFILETRIGGERTYPE as isize,

    // -----------------------------------------------------------------------
    // IMA / fsverity file signature tags
    // -----------------------------------------------------------------------
    /// Per-file IMA signatures (string array)
    FILESIGNATURES = librpm_sys::rpmTag_e_RPMTAG_FILESIGNATURES as isize,

    /// Length of each IMA signature in FILESIGNATURES (int)
    FILESIGNATURELENGTH = librpm_sys::rpmTag_e_RPMTAG_FILESIGNATURELENGTH as isize,

    // -----------------------------------------------------------------------
    // Payload digest tags
    // -----------------------------------------------------------------------
    /// SHA-256 digest of the compressed payload (string array)
    #[cfg(has_rpmtag_payloadsha256)]
    PAYLOADSHA256 = librpm_sys::rpmTag_e_RPMTAG_PAYLOADSHA256 as isize,

    /// Algorithm used for PAYLOADSHA256 (int, obsolete)
    #[cfg(has_rpmtag_payloadsha256algo)]
    PAYLOADSHA256ALGO = librpm_sys::rpmTag_e_RPMTAG_PAYLOADSHA256ALGO as isize,

    /// Auto-installed flag — unimplemented reservation (int)
    AUTOINSTALLED = librpm_sys::rpmTag_e_RPMTAG_AUTOINSTALLED as isize,

    /// Package identity — unimplemented reservation (string)
    IDENTITY = librpm_sys::rpmTag_e_RPMTAG_IDENTITY as isize,

    /// Modularity label, e.g. "nodejs:12:..." (string)
    MODULARITYLABEL = librpm_sys::rpmTag_e_RPMTAG_MODULARITYLABEL as isize,

    /// Alternate SHA-256 digest of the compressed payload (string array)
    #[cfg(has_rpmtag_payloadsha256alt)]
    PAYLOADSHA256ALT = librpm_sys::rpmTag_e_RPMTAG_PAYLOADSHA256ALT as isize,

    /// Architecture suffix string (string, extension)
    #[cfg(has_rpmtag_archsuffix)]
    ARCHSUFFIX = librpm_sys::rpmTag_e_RPMTAG_ARCHSUFFIX as isize,

    /// Embedded spec file (string)
    #[cfg(has_rpmtag_spec)]
    SPEC = librpm_sys::rpmTag_e_RPMTAG_SPEC as isize,

    /// URL of upstream translation repository (string)
    #[cfg(has_rpmtag_translationurl)]
    TRANSLATIONURL = librpm_sys::rpmTag_e_RPMTAG_TRANSLATIONURL as isize,

    /// Upstream release monitoring URL or identifier (string)
    #[cfg(has_rpmtag_upstreamreleases)]
    UPSTREAMRELEASES = librpm_sys::rpmTag_e_RPMTAG_UPSTREAMRELEASES as isize,

    /// Pre-uninstall-transaction scriptlet (string)
    #[cfg(has_rpmtag_preuntrans)]
    PREUNTRANS = librpm_sys::rpmTag_e_RPMTAG_PREUNTRANS as isize,

    /// Post-uninstall-transaction scriptlet (string)
    #[cfg(has_rpmtag_postuntrans)]
    POSTUNTRANS = librpm_sys::rpmTag_e_RPMTAG_POSTUNTRANS as isize,

    /// Interpreter for the pre-uninstall-transaction scriptlet (string array)
    #[cfg(has_rpmtag_preuntransprog)]
    PREUNTRANSPROG = librpm_sys::rpmTag_e_RPMTAG_PREUNTRANSPROG as isize,

    /// Interpreter for the post-uninstall-transaction scriptlet (string array)
    #[cfg(has_rpmtag_postuntransprog)]
    POSTUNTRANSPROG = librpm_sys::rpmTag_e_RPMTAG_POSTUNTRANSPROG as isize,

    /// Pre-uninstall-transaction scriptlet flags (RPMSCRIPT_* bitmask, int)
    #[cfg(has_rpmtag_preuntransflags)]
    PREUNTRANSFLAGS = librpm_sys::rpmTag_e_RPMTAG_PREUNTRANSFLAGS as isize,

    /// Post-uninstall-transaction scriptlet flags (RPMSCRIPT_* bitmask, int)
    #[cfg(has_rpmtag_postuntransflags)]
    POSTUNTRANSFLAGS = librpm_sys::rpmTag_e_RPMTAG_POSTUNTRANSFLAGS as isize,

    /// Systemd-sysusers declarations (string array, extension)
    #[cfg(has_rpmtag_sysusers)]
    SYSUSERS = librpm_sys::rpmTag_e_RPMTAG_SYSUSERS as isize,

    /// Uncompressed payload size (int64)
    #[cfg(has_rpmtag_payloadsize)]
    PAYLOADSIZE = librpm_sys::rpmTag_e_RPMTAG_PAYLOADSIZE as isize,

    /// Alternate uncompressed payload size (int64)
    #[cfg(has_rpmtag_payloadsizealt)]
    PAYLOADSIZEALT = librpm_sys::rpmTag_e_RPMTAG_PAYLOADSIZEALT as isize,

    /// RPM format version (int)
    #[cfg(has_rpmtag_rpmformat)]
    RPMFORMAT = librpm_sys::rpmTag_e_RPMTAG_RPMFORMAT as isize,

    /// Per-file index into MIMEDICT (int array)
    #[cfg(has_rpmtag_filemimeindex)]
    FILEMIMEINDEX = librpm_sys::rpmTag_e_RPMTAG_FILEMIMEINDEX as isize,

    /// MIME type strings referenced by FILEMIMEINDEX (string array)
    #[cfg(has_rpmtag_mimedict)]
    MIMEDICT = librpm_sys::rpmTag_e_RPMTAG_MIMEDICT as isize,

    /// Per-file MIME types, assembled from FILEMIMEINDEX + MIMEDICT (string array, extension)
    #[cfg(has_rpmtag_filemimes)]
    FILEMIMES = librpm_sys::rpmTag_e_RPMTAG_FILEMIMES as isize,

    /// Package header digests (string array)
    #[cfg(has_rpmtag_packagedigests)]
    PACKAGEDIGESTS = librpm_sys::rpmTag_e_RPMTAG_PACKAGEDIGESTS as isize,

    /// Algorithms used for PACKAGEDIGESTS (int array)
    #[cfg(has_rpmtag_packagedigestalgos)]
    PACKAGEDIGESTALGOS = librpm_sys::rpmTag_e_RPMTAG_PACKAGEDIGESTALGOS as isize,

    /// NEVR of the source RPM (string)
    #[cfg(has_rpmtag_sourcenevr)]
    SOURCENEVR = librpm_sys::rpmTag_e_RPMTAG_SOURCENEVR as isize,

    /// SHA-512 digest of the compressed payload (string)
    #[cfg(has_rpmtag_payloadsha512)]
    PAYLOADSHA512 = librpm_sys::rpmTag_e_RPMTAG_PAYLOADSHA512 as isize,

    /// Alternate SHA-512 digest of the compressed payload (string)
    #[cfg(has_rpmtag_payloadsha512alt)]
    PAYLOADSHA512ALT = librpm_sys::rpmTag_e_RPMTAG_PAYLOADSHA512ALT as isize,

    /// SHA3-256 digest of the compressed payload (string)
    #[cfg(has_rpmtag_payloadsha3_256)]
    PAYLOADSHA3_256 = librpm_sys::rpmTag_e_RPMTAG_PAYLOADSHA3_256 as isize,

    /// Alternate SHA3-256 digest of the compressed payload (string)
    #[cfg(has_rpmtag_payloadsha3_256alt)]
    PAYLOADSHA3_256ALT = librpm_sys::rpmTag_e_RPMTAG_PAYLOADSHA3_256ALT as isize,
}

impl From<Tag> for i32 {
    fn from(val: Tag) -> Self {
        val as i32
    }
}

impl From<Tag> for u32 {
    fn from(val: Tag) -> Self {
        val as u32
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Safety: rpmTagGetName operates on librpm's static tag table,
        // is thread-safe, and does not require initialization.
        let name = unsafe { librpm_sys::rpmTagGetName(*self as librpm_sys::rpmTagVal) };
        if name.is_null() {
            return write!(f, "(unknown:{})", u32::from(*self));
        }
        let cstr = unsafe { CStr::from_ptr(name) };
        f.write_str(cstr.to_str().expect("tag name is not UTF-8"))
    }
}

/// Error returned when a string doesn't match any known RPM tag name.
#[derive(Debug, Clone)]
pub struct ParseTagError(());

impl fmt::Display for ParseTagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown RPM tag name")
    }
}

impl std::error::Error for ParseTagError {}

impl FromStr for Tag {
    type Err = ParseTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cstr = CString::new(s).map_err(|_| ParseTagError(()))?;
        // Safety: rpmTagGetValue operates on librpm's static tag table,
        // is thread-safe, and does not require initialization.
        let val = unsafe { librpm_sys::rpmTagGetValue(cstr.as_ptr()) };
        if val == Tag::NOT_FOUND.into() {
            return Err(ParseTagError(()));
        }
        Tag::from_isize(val as isize).ok_or(ParseTagError(()))
    }
}

impl From<Index> for DBIndexTag {
    fn from(i: Index) -> Self {
        match i {
            Index::Name => DBIndexTag::NAME,
            Index::Basenames => DBIndexTag::BASENAMES,
            Index::Dirnames => DBIndexTag::DIRNAMES,
            Index::Instfilenames => DBIndexTag::INSTFILENAMES,
            Index::Providename => DBIndexTag::PROVIDENAME,
            Index::Requirename => DBIndexTag::REQUIRENAME,
            Index::Conflictname => DBIndexTag::CONFLICTNAME,
            Index::Obsoletename => DBIndexTag::OBSOLETENAME,
            Index::Group => DBIndexTag::GROUP,
            Index::Triggername => DBIndexTag::TRIGGERNAME,
            Index::Recommendname => DBIndexTag::RECOMMENDNAME,
            Index::Suggestname => DBIndexTag::SUGGESTNAME,
            Index::Supplementname => DBIndexTag::SUPPLEMENTNAME,
            Index::Enhancename => DBIndexTag::ENHANCENAME,
            Index::Filetriggername => DBIndexTag::FILETRIGGERNAME,
            Index::Transfiletriggername => DBIndexTag::TRANSFILETRIGGERNAME,
        }
    }
}

/// RPM database index tags (`rpmDbiTag_e` in librpm)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DBIndexTag {
    /// Installed package headers
    PACKAGES = librpm_sys::rpmDbiTag_e_RPMDBI_PACKAGES as isize,

    /// NEVRA label pseudo-index
    LABEL = librpm_sys::rpmDbiTag_e_RPMDBI_LABEL as isize,

    /// Index by package name
    NAME = librpm_sys::rpmDbiTag_e_RPMDBI_NAME as isize,

    /// Index by file basenames
    BASENAMES = librpm_sys::rpmDbiTag_e_RPMDBI_BASENAMES as isize,

    /// Index by package group
    GROUP = librpm_sys::rpmDbiTag_e_RPMDBI_GROUP as isize,

    /// Index by requirement names
    REQUIRENAME = librpm_sys::rpmDbiTag_e_RPMDBI_REQUIRENAME as isize,

    /// Index by provided capability names
    PROVIDENAME = librpm_sys::rpmDbiTag_e_RPMDBI_PROVIDENAME as isize,

    /// Index by conflict names
    CONFLICTNAME = librpm_sys::rpmDbiTag_e_RPMDBI_CONFLICTNAME as isize,

    /// Index by obsolete names
    OBSOLETENAME = librpm_sys::rpmDbiTag_e_RPMDBI_OBSOLETENAME as isize,

    /// Index by trigger dependency names
    TRIGGERNAME = librpm_sys::rpmDbiTag_e_RPMDBI_TRIGGERNAME as isize,

    /// Index by directory names
    DIRNAMES = librpm_sys::rpmDbiTag_e_RPMDBI_DIRNAMES as isize,

    /// Index by install transaction ID
    INSTALLTID = librpm_sys::rpmDbiTag_e_RPMDBI_INSTALLTID as isize,

    /// Index by MD5 signature (obsolete)
    SIGMD5 = librpm_sys::rpmDbiTag_e_RPMDBI_SIGMD5 as isize,

    /// Index by SHA1 header digest (obsolete)
    SHA1HEADER = librpm_sys::rpmDbiTag_e_RPMDBI_SHA1HEADER as isize,

    /// Index by installed file paths
    INSTFILENAMES = librpm_sys::rpmDbiTag_e_RPMDBI_INSTFILENAMES as isize,

    /// Index by file trigger dependency names/globs
    FILETRIGGERNAME = librpm_sys::rpmDbiTag_e_RPMDBI_FILETRIGGERNAME as isize,

    /// Index by transactional file trigger dependency names/globs
    TRANSFILETRIGGERNAME = librpm_sys::rpmDbiTag_e_RPMDBI_TRANSFILETRIGGERNAME as isize,

    /// Index by recommend dependency names
    RECOMMENDNAME = librpm_sys::rpmDbiTag_e_RPMDBI_RECOMMENDNAME as isize,

    /// Index by suggest dependency names
    SUGGESTNAME = librpm_sys::rpmDbiTag_e_RPMDBI_SUGGESTNAME as isize,

    /// Index by supplement dependency names
    SUPPLEMENTNAME = librpm_sys::rpmDbiTag_e_RPMDBI_SUPPLEMENTNAME as isize,

    /// Index by enhance dependency names
    ENHANCENAME = librpm_sys::rpmDbiTag_e_RPMDBI_ENHANCENAME as isize,
}

/// RPM package signature tags (`rpmSigTag_e` in librpm)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignatureTag {
    /// Header + payload size (32-bit) in bytes
    SIZE = librpm_sys::rpmSigTag_e_RPMSIGTAG_SIZE as isize,

    /// Broken MD5, take 1 (deprecated/legacy)
    LEMD5_1 = librpm_sys::rpmSigTag_e_RPMSIGTAG_LEMD5_1 as isize,

    /// Broken MD5, take 2 (deprecated/legacy)
    LEMD5_2 = librpm_sys::rpmSigTag_e_RPMSIGTAG_LEMD5_2 as isize,

    /// PGP 2.6.3 signature
    PGP = librpm_sys::rpmSigTag_e_RPMSIGTAG_PGP as isize,

    /// MD5 signature
    MD5 = librpm_sys::rpmSigTag_e_RPMSIGTAG_MD5 as isize,

    /// GnuPG signature
    GPG = librpm_sys::rpmSigTag_e_RPMSIGTAG_GPG as isize,

    /// PGP5 signature (deprecated/legacy)
    PGP5 = librpm_sys::rpmSigTag_e_RPMSIGTAG_PGP5 as isize,

    /// Uncompressed payload size in bytes (32-bit)
    PAYLOADSIZE = librpm_sys::rpmSigTag_e_RPMSIGTAG_PAYLOADSIZE as isize,

    /// Reserved space in the signature header for in-place signing
    RESERVEDSPACE = librpm_sys::rpmSigTag_e_RPMSIGTAG_RESERVEDSPACE as isize,

    /// Broken SHA1, take 1
    BADSHA1_1 = librpm_sys::rpmSigTag_e_RPMSIGTAG_BADSHA1_1 as isize,

    /// Broken SHA1, take 2
    BADSHA1_2 = librpm_sys::rpmSigTag_e_RPMSIGTAG_BADSHA1_2 as isize,

    /// SHA1 header digest
    SHA1 = librpm_sys::rpmSigTag_e_RPMSIGTAG_SHA1 as isize,

    /// DSA header signature
    DSA = librpm_sys::rpmSigTag_e_RPMSIGTAG_DSA as isize,

    /// RSA header signature
    RSA = librpm_sys::rpmSigTag_e_RPMSIGTAG_RSA as isize,

    /// Header + payload size (64-bit) in bytes
    LONGSIZE = librpm_sys::rpmSigTag_e_RPMSIGTAG_LONGSIZE as isize,

    /// Uncompressed payload size (64-bit) in bytes
    LONGARCHIVESIZE = librpm_sys::rpmSigTag_e_RPMSIGTAG_LONGARCHIVESIZE as isize,

    /// SHA-256 header digest
    SHA256 = librpm_sys::rpmSigTag_e_RPMSIGTAG_SHA256 as isize,

    /// fsverity signatures
    #[cfg(has_rpmsigtag_veritysignatures)]
    VERITYSIGNATURES = librpm_sys::rpmSigTag_e_RPMSIGTAG_VERITYSIGNATURES as isize,

    /// fsverity algorithm
    #[cfg(has_rpmsigtag_veritysignaturealgo)]
    VERITYSIGNATURESALGO = librpm_sys::rpmSigTag_e_RPMSIGTAG_VERITYSIGNATUREALGO as isize,

    /// OpenPGP header-only signatures
    #[cfg(has_rpmsigtag_openpgp)]
    OPENPGP = librpm_sys::rpmSigTag_e_RPMSIGTAG_OPENPGP as isize,

    /// SHA3-256 header digest
    #[cfg(has_rpmsigtag_sha3_256)]
    SHA3_256 = librpm_sys::rpmSigTag_e_RPMSIGTAG_SHA3_256 as isize,

    /// Reserved (sentinel for end of signature tag range)
    #[cfg(has_rpmsigtag_reserved)]
    RESERVED = librpm_sys::rpmSigTag_e_RPMSIGTAG_RESERVED as isize,
}

/// Types of data in tags from headers (`rpmTagType_e` in librpm)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TagType {
    NULL = librpm_sys::rpmTagType_e_RPM_NULL_TYPE as isize,
    CHAR = librpm_sys::rpmTagType_e_RPM_CHAR_TYPE as isize,
    INT8 = librpm_sys::rpmTagType_e_RPM_INT8_TYPE as isize,
    INT16 = librpm_sys::rpmTagType_e_RPM_INT16_TYPE as isize,
    INT32 = librpm_sys::rpmTagType_e_RPM_INT32_TYPE as isize,
    INT64 = librpm_sys::rpmTagType_e_RPM_INT64_TYPE as isize,
    STRING = librpm_sys::rpmTagType_e_RPM_STRING_TYPE as isize,
    BIN = librpm_sys::rpmTagType_e_RPM_BIN_TYPE as isize,
    STRING_ARRAY = librpm_sys::rpmTagType_e_RPM_STRING_ARRAY_TYPE as isize,
    I18NSTRING = librpm_sys::rpmTagType_e_RPM_I18NSTRING_TYPE as isize,
}

/// Classes of data in tags from headers (`rpmTagClass_e` in librpm)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TagClass {
    NULL = librpm_sys::rpmTagClass_e_RPM_NULL_CLASS as isize,
    NUMERIC = librpm_sys::rpmTagClass_e_RPM_NUMERIC_CLASS as isize,
    STRING = librpm_sys::rpmTagClass_e_RPM_STRING_CLASS as isize,
    BINARY = librpm_sys::rpmTagClass_e_RPM_BINARY_CLASS as isize,
}
