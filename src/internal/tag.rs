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

use crate::Index;

/// Identifiers for data in RPM headers (`rpmTag_e` in librpm)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tag {
    /// Unknown tag
    NOT_FOUND = librpm_sys::rpmTag_e_RPMTAG_NOT_FOUND as isize,

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
    SIG_BASE = librpm_sys::rpmTag_e_RPMTAG_SIG_BASE as isize,
    SIGSIZE = librpm_sys::rpmTag_e_RPMTAG_SIGSIZE as isize,
    SIGPGP = librpm_sys::rpmTag_e_RPMTAG_SIGPGP as isize,
    SIGMD5 = librpm_sys::rpmTag_e_RPMTAG_SIGMD5 as isize,
    SIGGPG = librpm_sys::rpmTag_e_RPMTAG_SIGGPG as isize,
    PUBKEYS = librpm_sys::rpmTag_e_RPMTAG_PUBKEYS as isize,
    DSAHEADER = librpm_sys::rpmTag_e_RPMTAG_DSAHEADER as isize,
    RSAHEADER = librpm_sys::rpmTag_e_RPMTAG_RSAHEADER as isize,
    SHA1HEADER = librpm_sys::rpmTag_e_RPMTAG_SHA1HEADER as isize,
    LONGSIGSIZE = librpm_sys::rpmTag_e_RPMTAG_LONGSIGSIZE as isize,
    LONGARCHIVESIZE = librpm_sys::rpmTag_e_RPMTAG_LONGARCHIVESIZE as isize,
    NAME = librpm_sys::rpmTag_e_RPMTAG_NAME as isize,
    VERSION = librpm_sys::rpmTag_e_RPMTAG_VERSION as isize,
    RELEASE = librpm_sys::rpmTag_e_RPMTAG_RELEASE as isize,
    EPOCH = librpm_sys::rpmTag_e_RPMTAG_EPOCH as isize,
    SUMMARY = librpm_sys::rpmTag_e_RPMTAG_SUMMARY as isize,
    DESCRIPTION = librpm_sys::rpmTag_e_RPMTAG_DESCRIPTION as isize,
    BUILDTIME = librpm_sys::rpmTag_e_RPMTAG_BUILDTIME as isize,
    BUILDHOST = librpm_sys::rpmTag_e_RPMTAG_BUILDHOST as isize,
    INSTALLTIME = librpm_sys::rpmTag_e_RPMTAG_INSTALLTIME as isize,
    SIZE = librpm_sys::rpmTag_e_RPMTAG_SIZE as isize,
    DISTRIBUTION = librpm_sys::rpmTag_e_RPMTAG_DISTRIBUTION as isize,
    VENDOR = librpm_sys::rpmTag_e_RPMTAG_VENDOR as isize,
    GIF = librpm_sys::rpmTag_e_RPMTAG_GIF as isize,
    XPM = librpm_sys::rpmTag_e_RPMTAG_XPM as isize,
    LICENSE = librpm_sys::rpmTag_e_RPMTAG_LICENSE as isize,
    PACKAGER = librpm_sys::rpmTag_e_RPMTAG_PACKAGER as isize,
    GROUP = librpm_sys::rpmTag_e_RPMTAG_GROUP as isize,
    CHANGELOG = librpm_sys::rpmTag_e_RPMTAG_CHANGELOG as isize,
    SOURCE = librpm_sys::rpmTag_e_RPMTAG_SOURCE as isize,
    PATCH = librpm_sys::rpmTag_e_RPMTAG_PATCH as isize,
    URL = librpm_sys::rpmTag_e_RPMTAG_URL as isize,
    OS = librpm_sys::rpmTag_e_RPMTAG_OS as isize,
    ARCH = librpm_sys::rpmTag_e_RPMTAG_ARCH as isize,
    PREIN = librpm_sys::rpmTag_e_RPMTAG_PREIN as isize,
    POSTIN = librpm_sys::rpmTag_e_RPMTAG_POSTIN as isize,
    PREUN = librpm_sys::rpmTag_e_RPMTAG_PREUN as isize,
    POSTUN = librpm_sys::rpmTag_e_RPMTAG_POSTUN as isize,
    FILESIZES = librpm_sys::rpmTag_e_RPMTAG_FILESIZES as isize,
    FILESTATES = librpm_sys::rpmTag_e_RPMTAG_FILESTATES as isize,
    FILEMODES = librpm_sys::rpmTag_e_RPMTAG_FILEMODES as isize,
    FILERDEVS = librpm_sys::rpmTag_e_RPMTAG_FILERDEVS as isize,
    FILEMTIMES = librpm_sys::rpmTag_e_RPMTAG_FILEMTIMES as isize,
    FILEDIGESTS = librpm_sys::rpmTag_e_RPMTAG_FILEDIGESTS as isize,
    FILELINKTOS = librpm_sys::rpmTag_e_RPMTAG_FILELINKTOS as isize,
    FILEFLAGS = librpm_sys::rpmTag_e_RPMTAG_FILEFLAGS as isize,
    ROOT = librpm_sys::rpmTag_e_RPMTAG_ROOT as isize,
    FILEUSERNAME = librpm_sys::rpmTag_e_RPMTAG_FILEUSERNAME as isize,
    FILEGROUPNAME = librpm_sys::rpmTag_e_RPMTAG_FILEGROUPNAME as isize,
    ICON = librpm_sys::rpmTag_e_RPMTAG_ICON as isize,
    SOURCERPM = librpm_sys::rpmTag_e_RPMTAG_SOURCERPM as isize,
    FILEVERIFYFLAGS = librpm_sys::rpmTag_e_RPMTAG_FILEVERIFYFLAGS as isize,
    ARCHIVESIZE = librpm_sys::rpmTag_e_RPMTAG_ARCHIVESIZE as isize,
    PROVIDENAME = librpm_sys::rpmTag_e_RPMTAG_PROVIDENAME as isize,
    REQUIREFLAGS = librpm_sys::rpmTag_e_RPMTAG_REQUIREFLAGS as isize,
    REQUIRENAME = librpm_sys::rpmTag_e_RPMTAG_REQUIRENAME as isize,
    REQUIREVERSION = librpm_sys::rpmTag_e_RPMTAG_REQUIREVERSION as isize,
    NOSOURCE = librpm_sys::rpmTag_e_RPMTAG_NOSOURCE as isize,
    NOPATCH = librpm_sys::rpmTag_e_RPMTAG_NOPATCH as isize,
    CONFLICTFLAGS = librpm_sys::rpmTag_e_RPMTAG_CONFLICTFLAGS as isize,
    CONFLICTNAME = librpm_sys::rpmTag_e_RPMTAG_CONFLICTNAME as isize,
    CONFLICTVERSION = librpm_sys::rpmTag_e_RPMTAG_CONFLICTVERSION as isize,
    DEFAULTPREFIX = librpm_sys::rpmTag_e_RPMTAG_DEFAULTPREFIX as isize,
    BUILDROOT = librpm_sys::rpmTag_e_RPMTAG_BUILDROOT as isize,
    INSTALLPREFIX = librpm_sys::rpmTag_e_RPMTAG_INSTALLPREFIX as isize,
    EXCLUDEARCH = librpm_sys::rpmTag_e_RPMTAG_EXCLUDEARCH as isize,
    EXCLUDEOS = librpm_sys::rpmTag_e_RPMTAG_EXCLUDEOS as isize,
    EXCLUSIVEARCH = librpm_sys::rpmTag_e_RPMTAG_EXCLUSIVEARCH as isize,
    EXCLUSIVEOS = librpm_sys::rpmTag_e_RPMTAG_EXCLUSIVEOS as isize,
    AUTOREQPROV = librpm_sys::rpmTag_e_RPMTAG_AUTOREQPROV as isize,
    RPMVERSION = librpm_sys::rpmTag_e_RPMTAG_RPMVERSION as isize,
    TRIGGERSCRIPTS = librpm_sys::rpmTag_e_RPMTAG_TRIGGERSCRIPTS as isize,
    TRIGGERNAME = librpm_sys::rpmTag_e_RPMTAG_TRIGGERNAME as isize,
    TRIGGERVERSION = librpm_sys::rpmTag_e_RPMTAG_TRIGGERVERSION as isize,
    TRIGGERFLAGS = librpm_sys::rpmTag_e_RPMTAG_TRIGGERFLAGS as isize,
    TRIGGERINDEX = librpm_sys::rpmTag_e_RPMTAG_TRIGGERINDEX as isize,
    VERIFYSCRIPT = librpm_sys::rpmTag_e_RPMTAG_VERIFYSCRIPT as isize,
    CHANGELOGTIME = librpm_sys::rpmTag_e_RPMTAG_CHANGELOGTIME as isize,
    CHANGELOGNAME = librpm_sys::rpmTag_e_RPMTAG_CHANGELOGNAME as isize,
    CHANGELOGTEXT = librpm_sys::rpmTag_e_RPMTAG_CHANGELOGTEXT as isize,
    PREREQ = librpm_sys::rpmTag_e_RPMTAG_PREREQ as isize,
    PREINPROG = librpm_sys::rpmTag_e_RPMTAG_PREINPROG as isize,
    POSTINPROG = librpm_sys::rpmTag_e_RPMTAG_POSTINPROG as isize,
    PREUNPROG = librpm_sys::rpmTag_e_RPMTAG_PREUNPROG as isize,
    POSTUNPROG = librpm_sys::rpmTag_e_RPMTAG_POSTUNPROG as isize,
    BUILDARCHS = librpm_sys::rpmTag_e_RPMTAG_BUILDARCHS as isize,
    OBSOLETENAME = librpm_sys::rpmTag_e_RPMTAG_OBSOLETENAME as isize,
    VERIFYSCRIPTPROG = librpm_sys::rpmTag_e_RPMTAG_VERIFYSCRIPTPROG as isize,
    TRIGGERSCRIPTPROG = librpm_sys::rpmTag_e_RPMTAG_TRIGGERSCRIPTPROG as isize,
    DOCDIR = librpm_sys::rpmTag_e_RPMTAG_DOCDIR as isize,
    COOKIE = librpm_sys::rpmTag_e_RPMTAG_COOKIE as isize,
    FILEDEVICES = librpm_sys::rpmTag_e_RPMTAG_FILEDEVICES as isize,
    FILEINODES = librpm_sys::rpmTag_e_RPMTAG_FILEINODES as isize,
    FILELANGS = librpm_sys::rpmTag_e_RPMTAG_FILELANGS as isize,
    PREFIXES = librpm_sys::rpmTag_e_RPMTAG_PREFIXES as isize,
    INSTPREFIXES = librpm_sys::rpmTag_e_RPMTAG_INSTPREFIXES as isize,
    TRIGGERIN = librpm_sys::rpmTag_e_RPMTAG_TRIGGERIN as isize,
    TRIGGERUN = librpm_sys::rpmTag_e_RPMTAG_TRIGGERUN as isize,
    TRIGGERPOSTUN = librpm_sys::rpmTag_e_RPMTAG_TRIGGERPOSTUN as isize,
    AUTOREQ = librpm_sys::rpmTag_e_RPMTAG_AUTOREQ as isize,
    AUTOPROV = librpm_sys::rpmTag_e_RPMTAG_AUTOPROV as isize,
    CAPABILITY = librpm_sys::rpmTag_e_RPMTAG_CAPABILITY as isize,
    SOURCEPACKAGE = librpm_sys::rpmTag_e_RPMTAG_SOURCEPACKAGE as isize,
    BUILDPREREQ = librpm_sys::rpmTag_e_RPMTAG_BUILDPREREQ as isize,
    BUILDREQUIRES = librpm_sys::rpmTag_e_RPMTAG_BUILDREQUIRES as isize,
    BUILDCONFLICTS = librpm_sys::rpmTag_e_RPMTAG_BUILDCONFLICTS as isize,
    PROVIDEFLAGS = librpm_sys::rpmTag_e_RPMTAG_PROVIDEFLAGS as isize,
    PROVIDEVERSION = librpm_sys::rpmTag_e_RPMTAG_PROVIDEVERSION as isize,
    DIRINDEXES = librpm_sys::rpmTag_e_RPMTAG_DIRINDEXES as isize,
    BASENAMES = librpm_sys::rpmTag_e_RPMTAG_BASENAMES as isize,
    DIRNAMES = librpm_sys::rpmTag_e_RPMTAG_DIRNAMES as isize,
    ORIGDIRINDEXES = librpm_sys::rpmTag_e_RPMTAG_ORIGDIRINDEXES as isize,
    ORIGBASENAMES = librpm_sys::rpmTag_e_RPMTAG_ORIGBASENAMES as isize,
    ORIGDIRNAMES = librpm_sys::rpmTag_e_RPMTAG_ORIGDIRNAMES as isize,
    OPTFLAGS = librpm_sys::rpmTag_e_RPMTAG_OPTFLAGS as isize,
    DISTURL = librpm_sys::rpmTag_e_RPMTAG_DISTURL as isize,
    PAYLOADFORMAT = librpm_sys::rpmTag_e_RPMTAG_PAYLOADFORMAT as isize,
    PAYLOADCOMPRESSOR = librpm_sys::rpmTag_e_RPMTAG_PAYLOADCOMPRESSOR as isize,
    PAYLOADFLAGS = librpm_sys::rpmTag_e_RPMTAG_PAYLOADFLAGS as isize,
    INSTALLCOLOR = librpm_sys::rpmTag_e_RPMTAG_INSTALLCOLOR as isize,
    INSTALLTID = librpm_sys::rpmTag_e_RPMTAG_INSTALLTID as isize,
    REMOVETID = librpm_sys::rpmTag_e_RPMTAG_REMOVETID as isize,
    PLATFORM = librpm_sys::rpmTag_e_RPMTAG_PLATFORM as isize,
    PATCHESNAME = librpm_sys::rpmTag_e_RPMTAG_PATCHESNAME as isize,
    PATCHESFLAGS = librpm_sys::rpmTag_e_RPMTAG_PATCHESFLAGS as isize,
    PATCHESVERSION = librpm_sys::rpmTag_e_RPMTAG_PATCHESVERSION as isize,
    FILECOLORS = librpm_sys::rpmTag_e_RPMTAG_FILECOLORS as isize,
    FILECLASS = librpm_sys::rpmTag_e_RPMTAG_FILECLASS as isize,
    CLASSDICT = librpm_sys::rpmTag_e_RPMTAG_CLASSDICT as isize,
    FILEDEPENDSX = librpm_sys::rpmTag_e_RPMTAG_FILEDEPENDSX as isize,
    FILEDEPENDSN = librpm_sys::rpmTag_e_RPMTAG_FILEDEPENDSN as isize,
    DEPENDSDICT = librpm_sys::rpmTag_e_RPMTAG_DEPENDSDICT as isize,
    SOURCEPKGID = librpm_sys::rpmTag_e_RPMTAG_SOURCEPKGID as isize,
    FSCONTEXTS = librpm_sys::rpmTag_e_RPMTAG_FSCONTEXTS as isize,
    RECONTEXTS = librpm_sys::rpmTag_e_RPMTAG_RECONTEXTS as isize,
    POLICIES = librpm_sys::rpmTag_e_RPMTAG_POLICIES as isize,
    PRETRANS = librpm_sys::rpmTag_e_RPMTAG_PRETRANS as isize,
    POSTTRANS = librpm_sys::rpmTag_e_RPMTAG_POSTTRANS as isize,
    PRETRANSPROG = librpm_sys::rpmTag_e_RPMTAG_PRETRANSPROG as isize,
    POSTTRANSPROG = librpm_sys::rpmTag_e_RPMTAG_POSTTRANSPROG as isize,
    DISTTAG = librpm_sys::rpmTag_e_RPMTAG_DISTTAG as isize,
    //SUGGESTNAME = librpm_sys::rpmTag_e_RPMTAG_SUGGESTNAME as isize,
    //SUGGESTVERSION = librpm_sys::rpmTag_e_RPMTAG_SUGGESTVERSION as isize,
    //SUGGESTFLAGS = librpm_sys::rpmTag_e_RPMTAG_SUGGESTFLAGS as isize,
    //ENHANCENAME = librpm_sys::rpmTag_e_RPMTAG_ENHANCENAME as isize,
    //ENHANCEVERSION = librpm_sys::rpmTag_e_RPMTAG_ENHANCEVERSION as isize,
    //ENHANCEFLAGS = librpm_sys::rpmTag_e_RPMTAG_ENHANCEFLAGS as isize,
    PRIORITY = librpm_sys::rpmTag_e_RPMTAG_PRIORITY as isize,
    CVSID = librpm_sys::rpmTag_e_RPMTAG_CVSID as isize,
    BLINKPKGID = librpm_sys::rpmTag_e_RPMTAG_BLINKPKGID as isize,
    BLINKHDRID = librpm_sys::rpmTag_e_RPMTAG_BLINKHDRID as isize,
    BLINKNEVRA = librpm_sys::rpmTag_e_RPMTAG_BLINKNEVRA as isize,
    FLINKPKGID = librpm_sys::rpmTag_e_RPMTAG_FLINKPKGID as isize,
    FLINKHDRID = librpm_sys::rpmTag_e_RPMTAG_FLINKHDRID as isize,
    FLINKNEVRA = librpm_sys::rpmTag_e_RPMTAG_FLINKNEVRA as isize,
    PACKAGEORIGIN = librpm_sys::rpmTag_e_RPMTAG_PACKAGEORIGIN as isize,
    TRIGGERPREIN = librpm_sys::rpmTag_e_RPMTAG_TRIGGERPREIN as isize,
    BUILDSUGGESTS = librpm_sys::rpmTag_e_RPMTAG_BUILDSUGGESTS as isize,
    BUILDENHANCES = librpm_sys::rpmTag_e_RPMTAG_BUILDENHANCES as isize,
    SCRIPTSTATES = librpm_sys::rpmTag_e_RPMTAG_SCRIPTSTATES as isize,
    SCRIPTMETRICS = librpm_sys::rpmTag_e_RPMTAG_SCRIPTMETRICS as isize,
    BUILDCPUCLOCK = librpm_sys::rpmTag_e_RPMTAG_BUILDCPUCLOCK as isize,
    FILEDIGESTALGOS = librpm_sys::rpmTag_e_RPMTAG_FILEDIGESTALGOS as isize,
    VARIANTS = librpm_sys::rpmTag_e_RPMTAG_VARIANTS as isize,
    XMAJOR = librpm_sys::rpmTag_e_RPMTAG_XMAJOR as isize,
    XMINOR = librpm_sys::rpmTag_e_RPMTAG_XMINOR as isize,
    REPOTAG = librpm_sys::rpmTag_e_RPMTAG_REPOTAG as isize,
    KEYWORDS = librpm_sys::rpmTag_e_RPMTAG_KEYWORDS as isize,
    BUILDPLATFORMS = librpm_sys::rpmTag_e_RPMTAG_BUILDPLATFORMS as isize,
    PACKAGECOLOR = librpm_sys::rpmTag_e_RPMTAG_PACKAGECOLOR as isize,
    PACKAGEPREFCOLOR = librpm_sys::rpmTag_e_RPMTAG_PACKAGEPREFCOLOR as isize,
    XATTRSDICT = librpm_sys::rpmTag_e_RPMTAG_XATTRSDICT as isize,
    FILEXATTRSX = librpm_sys::rpmTag_e_RPMTAG_FILEXATTRSX as isize,
    DEPATTRSDICT = librpm_sys::rpmTag_e_RPMTAG_DEPATTRSDICT as isize,
    CONFLICTATTRSX = librpm_sys::rpmTag_e_RPMTAG_CONFLICTATTRSX as isize,
    OBSOLETEATTRSX = librpm_sys::rpmTag_e_RPMTAG_OBSOLETEATTRSX as isize,
    PROVIDEATTRSX = librpm_sys::rpmTag_e_RPMTAG_PROVIDEATTRSX as isize,
    REQUIREATTRSX = librpm_sys::rpmTag_e_RPMTAG_REQUIREATTRSX as isize,
    BUILDPROVIDES = librpm_sys::rpmTag_e_RPMTAG_BUILDPROVIDES as isize,
    BUILDOBSOLETES = librpm_sys::rpmTag_e_RPMTAG_BUILDOBSOLETES as isize,
    DBINSTANCE = librpm_sys::rpmTag_e_RPMTAG_DBINSTANCE as isize,
    NVRA = librpm_sys::rpmTag_e_RPMTAG_NVRA as isize,
    FILENAMES = librpm_sys::rpmTag_e_RPMTAG_FILENAMES as isize,
    FILEPROVIDE = librpm_sys::rpmTag_e_RPMTAG_FILEPROVIDE as isize,
    FILEREQUIRE = librpm_sys::rpmTag_e_RPMTAG_FILEREQUIRE as isize,
    FSNAMES = librpm_sys::rpmTag_e_RPMTAG_FSNAMES as isize,
    FSSIZES = librpm_sys::rpmTag_e_RPMTAG_FSSIZES as isize,
    TRIGGERCONDS = librpm_sys::rpmTag_e_RPMTAG_TRIGGERCONDS as isize,
    TRIGGERTYPE = librpm_sys::rpmTag_e_RPMTAG_TRIGGERTYPE as isize,
    ORIGFILENAMES = librpm_sys::rpmTag_e_RPMTAG_ORIGFILENAMES as isize,
    LONGFILESIZES = librpm_sys::rpmTag_e_RPMTAG_LONGFILESIZES as isize,
    LONGSIZE = librpm_sys::rpmTag_e_RPMTAG_LONGSIZE as isize,
    FILECAPS = librpm_sys::rpmTag_e_RPMTAG_FILECAPS as isize,
    FILEDIGESTALGO = librpm_sys::rpmTag_e_RPMTAG_FILEDIGESTALGO as isize,
    BUGURL = librpm_sys::rpmTag_e_RPMTAG_BUGURL as isize,
    EVR = librpm_sys::rpmTag_e_RPMTAG_EVR as isize,
    NVR = librpm_sys::rpmTag_e_RPMTAG_NVR as isize,
    NEVR = librpm_sys::rpmTag_e_RPMTAG_NEVR as isize,
    NEVRA = librpm_sys::rpmTag_e_RPMTAG_NEVRA as isize,
    HEADERCOLOR = librpm_sys::rpmTag_e_RPMTAG_HEADERCOLOR as isize,
    VERBOSE = librpm_sys::rpmTag_e_RPMTAG_VERBOSE as isize,
    EPOCHNUM = librpm_sys::rpmTag_e_RPMTAG_EPOCHNUM as isize,
    PREINFLAGS = librpm_sys::rpmTag_e_RPMTAG_PREINFLAGS as isize,
    POSTINFLAGS = librpm_sys::rpmTag_e_RPMTAG_POSTINFLAGS as isize,
    PREUNFLAGS = librpm_sys::rpmTag_e_RPMTAG_PREUNFLAGS as isize,
    POSTUNFLAGS = librpm_sys::rpmTag_e_RPMTAG_POSTUNFLAGS as isize,
    PRETRANSFLAGS = librpm_sys::rpmTag_e_RPMTAG_PRETRANSFLAGS as isize,
    POSTTRANSFLAGS = librpm_sys::rpmTag_e_RPMTAG_POSTTRANSFLAGS as isize,
    VERIFYSCRIPTFLAGS = librpm_sys::rpmTag_e_RPMTAG_VERIFYSCRIPTFLAGS as isize,
    TRIGGERSCRIPTFLAGS = librpm_sys::rpmTag_e_RPMTAG_TRIGGERSCRIPTFLAGS as isize,
    COLLECTIONS = librpm_sys::rpmTag_e_RPMTAG_COLLECTIONS as isize,
    POLICYNAMES = librpm_sys::rpmTag_e_RPMTAG_POLICYNAMES as isize,
    POLICYTYPES = librpm_sys::rpmTag_e_RPMTAG_POLICYTYPES as isize,
    POLICYTYPESINDEXES = librpm_sys::rpmTag_e_RPMTAG_POLICYTYPESINDEXES as isize,
    POLICYFLAGS = librpm_sys::rpmTag_e_RPMTAG_POLICYFLAGS as isize,
    VCS = librpm_sys::rpmTag_e_RPMTAG_VCS as isize,
    ORDERNAME = librpm_sys::rpmTag_e_RPMTAG_ORDERNAME as isize,
    ORDERVERSION = librpm_sys::rpmTag_e_RPMTAG_ORDERVERSION as isize,
    ORDERFLAGS = librpm_sys::rpmTag_e_RPMTAG_ORDERFLAGS as isize,
    MSSFMANIFEST = librpm_sys::rpmTag_e_RPMTAG_MSSFMANIFEST as isize,
    MSSFDOMAIN = librpm_sys::rpmTag_e_RPMTAG_MSSFDOMAIN as isize,
    INSTFILENAMES = librpm_sys::rpmTag_e_RPMTAG_INSTFILENAMES as isize,
    REQUIRENEVRS = librpm_sys::rpmTag_e_RPMTAG_REQUIRENEVRS as isize,
    PROVIDENEVRS = librpm_sys::rpmTag_e_RPMTAG_PROVIDENEVRS as isize,
    OBSOLETENEVRS = librpm_sys::rpmTag_e_RPMTAG_OBSOLETENEVRS as isize,
    CONFLICTNEVRS = librpm_sys::rpmTag_e_RPMTAG_CONFLICTNEVRS as isize,
    FILENLINKS = librpm_sys::rpmTag_e_RPMTAG_FILENLINKS as isize,
}

impl From<Index> for Tag {
    fn from(i: Index) -> Self {
        match i {
            Index::Name => Tag::NAME,
            Index::Version => Tag::VERSION,
            Index::License => Tag::LICENSE,
            Index::Summary => Tag::SUMMARY,
            Index::Description => Tag::DESCRIPTION,
        }
    }
}

/// RPM database index tags (`rpmDbiTag_e` in librpm)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DBIndexTag {
    PACKAGES = librpm_sys::rpmDbiTag_e_RPMDBI_PACKAGES as isize,
    LABEL = librpm_sys::rpmDbiTag_e_RPMDBI_LABEL as isize,
    NAME = librpm_sys::rpmDbiTag_e_RPMDBI_NAME as isize,
    BASENAMES = librpm_sys::rpmDbiTag_e_RPMDBI_BASENAMES as isize,
    GROUP = librpm_sys::rpmDbiTag_e_RPMDBI_GROUP as isize,
    REQUIRENAME = librpm_sys::rpmDbiTag_e_RPMDBI_REQUIRENAME as isize,
    PROVIDENAME = librpm_sys::rpmDbiTag_e_RPMDBI_PROVIDENAME as isize,
    CONFLICTNAME = librpm_sys::rpmDbiTag_e_RPMDBI_CONFLICTNAME as isize,
    OBSOLETENAME = librpm_sys::rpmDbiTag_e_RPMDBI_OBSOLETENAME as isize,
    TRIGGERNAME = librpm_sys::rpmDbiTag_e_RPMDBI_TRIGGERNAME as isize,
    DIRNAMES = librpm_sys::rpmDbiTag_e_RPMDBI_DIRNAMES as isize,
    INSTALLTID = librpm_sys::rpmDbiTag_e_RPMDBI_INSTALLTID as isize,
    SIGMD5 = librpm_sys::rpmDbiTag_e_RPMDBI_SIGMD5 as isize,
    SHA1HEADER = librpm_sys::rpmDbiTag_e_RPMDBI_SHA1HEADER as isize,
    INSTFILENAMES = librpm_sys::rpmDbiTag_e_RPMDBI_INSTFILENAMES as isize,
}

/// RPM package signature tags (`rpmSigTag_e` in librpm)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignatureTag {
    /// Header + payload size (32-bit) in bytes
    SIZE = librpm_sys::rpmSigTag_e_RPMSIGTAG_SIZE as isize,

    /// Broken MD5 (take 1 as isize, deprecated/legacy)
    LEMD5_1 = librpm_sys::rpmSigTag_e_RPMSIGTAG_LEMD5_1 as isize,

    /// Broken MD5 (take 2 as isize, deprecated/legacy)
    LEMD5_2 = librpm_sys::rpmSigTag_e_RPMSIGTAG_LEMD5_2 as isize,

    /// PGP 2.6.3 signature
    PGP = librpm_sys::rpmSigTag_e_RPMSIGTAG_PGP as isize,

    /// MD5 signature
    MD5 = librpm_sys::rpmSigTag_e_RPMSIGTAG_MD5 as isize,

    /// GnuPG signature
    GPG = librpm_sys::rpmSigTag_e_RPMSIGTAG_GPG as isize,

    /// PGP5 signature (deprecated/legacy)
    PGP5 = librpm_sys::rpmSigTag_e_RPMSIGTAG_PGP5 as isize,

    /// Uncompressed payload size in bytes
    PAYLOADSIZE = librpm_sys::rpmSigTag_e_RPMSIGTAG_PAYLOADSIZE as isize,

    /// Broken SHA1 (take 1)
    BADSHA1_1 = librpm_sys::rpmSigTag_e_RPMSIGTAG_BADSHA1_1 as isize,

    /// Broken SHA1 (take 2)
    BADSHA1_2 = librpm_sys::rpmSigTag_e_RPMSIGTAG_BADSHA1_2 as isize,

    /// SHA1 header digest
    SHA1 = librpm_sys::rpmSigTag_e_RPMSIGTAG_SHA1 as isize,

    /// DSA header signature
    DSA = librpm_sys::rpmSigTag_e_RPMSIGTAG_DSA as isize,

    /// RSA header signature
    RSA = librpm_sys::rpmSigTag_e_RPMSIGTAG_RSA as isize,

    /// Header + payload size (64-bit) in bytes
    LONGSIZE = librpm_sys::rpmSigTag_e_RPMSIGTAG_LONGSIZE as isize,

    /// Uncompressed payload size in bytes
    LONGARCHIVESIZE = librpm_sys::rpmSigTag_e_RPMSIGTAG_LONGARCHIVESIZE as isize,
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
