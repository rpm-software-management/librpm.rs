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

//! bindgen configuration for librpm-sys
//!
//! For more on using librpm, see "Chapter 15. Programming RPM with C" from the
//! Fedora RPM Guide (Draft 0.1):
//!
//! https://docs.fedoraproject.org/en-US/Fedora_Draft_Documentation/0.1/html/RPM_Guide/ch-programming-c.html

use bindgen::Builder;
use std::{env, path::PathBuf};

/// Find all `pub const {prefix}{NAME} :` patterns in the bindgen output,
/// returning the NAME suffix for each match.
///
/// Bindgen may or may not insert newlines between items depending on the
/// libclang version and formatting settings, so we search the raw text
/// instead of iterating by line.
fn find_consts<'a>(src: &'a str, prefix: &'a str) -> impl Iterator<Item = &'a str> {
    let needle = format!("pub const {prefix}");
    let needle_len = needle.len();
    let mut start = 0;
    std::iter::from_fn(move || {
        let pos = src[start..].find(&needle)?;
        let abs = start + pos + needle_len;
        start = abs;
        let rest = &src[abs..];
        let end = rest.find(|c: char| !c.is_ascii_alphanumeric() && c != '_')?;
        Some(&rest[..end])
    })
}

fn main() {
    // Link with librpm.so + librpmio.so
    //
    // See "Table 16-3: Required rpm libraries" from the "Compiling and Linking
    // RPM Programs" section of "Programming RPM with C" (see above).
    //
    // We don't yet link against librpmbuild.so or librpmsign.so because bindgen
    // is having trouble generating bindings for these libraries. See
    // `librpm.hpp` for more information.
    println!("cargo:rustc-link-lib=rpm");
    println!("cargo:rustc-link-lib=rpmio");

    let builder = Builder::default()
        .header("include/librpm.hpp")
        // ----------------------------------------------------------
        // Functions: only uncommented entries are generated.
        // Commented entries are the API surface exposed by librpm's
        // Python bindings (plus a few Rust-relevant extras), ready
        // to uncomment as safe Rust wrappers are added.
        // ----------------------------------------------------------
        // header.h — package header access
        .allowlist_function("headerNew")
        .allowlist_function("headerFree")
        .allowlist_function("headerLink")
        // .allowlist_function("headerExport")
        // .allowlist_function("headerCopy")
        // .allowlist_function("headerImport")
        // .allowlist_function("headerRead")
        // .allowlist_function("headerWrite")
        .allowlist_function("headerIsEntry")
        .allowlist_function("headerGet")
        // .allowlist_function("headerPutBin")
        // .allowlist_function("headerPutString")
        // .allowlist_function("headerPutUint8")
        // .allowlist_function("headerPutUint16")
        // .allowlist_function("headerPutUint32")
        // .allowlist_function("headerPutUint64")
        // .allowlist_function("headerDel")
        .allowlist_function("headerFormat")
        // .allowlist_function("headerFreeIterator")
        // .allowlist_function("headerInitIterator")
        // .allowlist_function("headerNextTag")
        // .allowlist_function("headerGetString")
        .allowlist_function("headerIsSource")
        // .allowlist_function("headerConvert")
        // .allowlist_function("headerCheck")
        // rpmdb.h — database access
        .allowlist_function("rpmdbGetIteratorOffset")
        .allowlist_function("rpmdbGetIteratorCount")
        .allowlist_function("rpmdbSetIteratorRE")
        .allowlist_function("rpmdbNextIterator")
        .allowlist_function("rpmdbFreeIterator")
        // .allowlist_function("rpmdbIndexIteratorInit")
        // .allowlist_function("rpmdbIndexIteratorNextTd")
        // .allowlist_function("rpmdbIndexIteratorNumPkgs")
        // .allowlist_function("rpmdbIndexIteratorPkgOffset")
        // .allowlist_function("rpmdbIndexIteratorTagNum")
        // .allowlist_function("rpmdbIndexIteratorFree")
        // .allowlist_function("rpmdbCookie")
        // rpmds.h — dependency sets
        .allowlist_function("rpmdsNew")
        .allowlist_function("rpmdsFree")
        // .allowlist_function("rpmdsLink")
        // .allowlist_function("rpmdsCurrent")
        .allowlist_function("rpmdsCount")
        // .allowlist_function("rpmdsIx")
        // .allowlist_function("rpmdsSetIx")
        // .allowlist_function("rpmdsDNEVR")
        .allowlist_function("rpmdsN")
        .allowlist_function("rpmdsEVR")
        // .allowlist_function("rpmdsTi")
        .allowlist_function("rpmdsFlags")
        // .allowlist_function("rpmdsTagN")
        // .allowlist_function("rpmdsInstance")
        .allowlist_function("rpmdsIsWeak")
        .allowlist_function("rpmdsIsReverse")
        // .allowlist_function("rpmdsIsSysuser")
        // .allowlist_function("rpmdsColor")
        .allowlist_function("rpmdsNext")
        .allowlist_function("rpmdsInit")
        // .allowlist_function("rpmdsFind")
        // .allowlist_function("rpmdsMerge")
        // .allowlist_function("rpmdsSearch")
        .allowlist_function("rpmdsCompare")
        // .allowlist_function("rpmdsNewPool")
        // .allowlist_function("rpmdsThisPool")
        .allowlist_function("rpmdsSinglePool")
        // .allowlist_function("rpmdsRpmlibPool")
        .allowlist_function("rpmdsIsRich")
        // rpmfi.h — file info (legacy iterator-style API; Python
        // only uses rpmfiFree/rpmfiNext for archive iteration)
        .allowlist_function("rpmfiFree")
        .allowlist_function("rpmfiNext")
        // rpmio.h — I/O routines
        .allowlist_function("Fopen")
        .allowlist_function("Fclose")
        // .allowlist_function("Fdopen")
        // .allowlist_function("Fclose")
        // .allowlist_function("Fflush")
        // .allowlist_function("Fread")
        // .allowlist_function("Fwrite")
        // .allowlist_function("Fseek")
        // .allowlist_function("Ftell")
        .allowlist_function("Ferror")
        .allowlist_function("Fstrerror")
        // .allowlist_function("Fdescr")
        // rpmlib.h — configuration and package reading
        .allowlist_function("rpmReadConfigFiles")
        .allowlist_function("rpmGetArchInfo")
        .allowlist_function("rpmGetOsInfo")
        // .allowlist_function("rpmMachineScore")
        // .allowlist_function("rpmFreeRpmrc")
        // .allowlist_function("rpmVersionCompare")
        // .allowlist_function("headerCheck")
        .allowlist_function("rpmReadPackageFile")
        // rpmmacro.h — macro system
        .allowlist_function("rpmDefineMacro")
        .allowlist_function("rpmPopMacro")
        .allowlist_function("delMacro")
        // .allowlist_function("rpmPushMacro")
        .allowlist_function("rpmExpandMacros")
        .allowlist_function("rpmMacroIsDefined")
        // .allowlist_function("rpmDumpMacroTable")
        // .allowlist_function("rpmFreeMacros")
        // .allowlist_function("rpmExpand")
        .allowlist_function("rpmExpandNumeric")
        // rpmtag.h — tag metadata
        // .allowlist_function("rpmTagGetTagType")
        // .allowlist_function("rpmTagGetReturnType")
        // .allowlist_function("rpmTagGetClass")
        .allowlist_function("rpmTagGetName")
        .allowlist_function("rpmTagGetValue")
        // .allowlist_function("rpmTagGetNames")
        // rpmtd.h — tag data container
        // .allowlist_function("rpmtdNew")
        // .allowlist_function("rpmtdFree")
        .allowlist_function("rpmtdReset")
        .allowlist_function("rpmtdFreeData")
        // .allowlist_function("rpmtdCount")
        // .allowlist_function("rpmtdClass")
        // .allowlist_function("rpmtdGetFlags")
        // .allowlist_function("rpmtdGetIndex")
        // .allowlist_function("rpmtdSetIndex")
        // .allowlist_function("rpmtdInit")
        // .allowlist_function("rpmtdNext")
        .allowlist_function("rpmtdNextString")
        // .allowlist_function("rpmtdGetChar")
        // .allowlist_function("rpmtdGetUint16")
        // .allowlist_function("rpmtdGetUint32")
        // .allowlist_function("rpmtdGetUint64")
        // .allowlist_function("rpmtdGetString")
        // .allowlist_function("rpmtdGetNumber")
        // rpmte.h — transaction elements
        .allowlist_function("rpmteType")
        .allowlist_function("rpmteN")
        .allowlist_function("rpmteE")
        .allowlist_function("rpmteV")
        .allowlist_function("rpmteR")
        .allowlist_function("rpmteA")
        // .allowlist_function("rpmteO")
        // .allowlist_function("rpmteColor")
        .allowlist_function("rpmtePkgFileSize")
        // .allowlist_function("rpmteParent")
        .allowlist_function("rpmteProblems")
        .allowlist_function("rpmteDBOffset")
        .allowlist_function("rpmteNEVR")
        .allowlist_function("rpmteNEVRA")
        .allowlist_function("rpmteKey")
        // .allowlist_function("rpmteSetUserdata")
        // .allowlist_function("rpmteUserdata")
        .allowlist_function("rpmteFailed")
        .allowlist_function("rpmteDS")
        .allowlist_function("rpmteFiles")
        // .allowlist_function("rpmteVerified")
        // .allowlist_function("rpmteVfyLevel")
        // .allowlist_function("rpmteSetVfyLevel")
        // rpmts.h — transaction sets
        .allowlist_function("rpmtsCreate")
        .allowlist_function("rpmtsFree")
        .allowlist_function("rpmtsClean")
        .allowlist_function("rpmtsInitIterator")
        .allowlist_function("rpmtsCheck")
        .allowlist_function("rpmtsOrder")
        .allowlist_function("rpmtsRun")
        // .allowlist_function("rpmtsCloseDB")
        // .allowlist_function("rpmtsOpenDB")
        .allowlist_function("rpmtsInitDB")
        // .allowlist_function("rpmtsGetDBMode")
        // .allowlist_function("rpmtsSetDBMode")
        .allowlist_function("rpmtsRebuildDB")
        .allowlist_function("rpmtsVerifyDB")
        .allowlist_function("rpmtsImportPubkey")
        .allowlist_function("rpmtxnImportPubkey")
        .allowlist_function("rpmtxnDeletePubkey")
        // .allowlist_function("rpmtxnRebuildKeystore")
        .allowlist_function("rpmtxnBegin")
        .allowlist_function("rpmtxnEnd")
        .allowlist_function("rpmtsGetKeyring")
        .allowlist_function("rpmtsSetKeyring")
        // .allowlist_function("rpmtsSetSolveCallback")
        .allowlist_function("rpmtsProblems")
        .allowlist_function("rpmtsEmpty")
        .allowlist_function("rpmtsVSFlags")
        .allowlist_function("rpmtsSetVSFlags")
        // .allowlist_function("rpmtsVfyFlags")
        // .allowlist_function("rpmtsSetVfyFlags")
        // .allowlist_function("rpmtsVfyLevel")
        // .allowlist_function("rpmtsSetVfyLevel")
        .allowlist_function("rpmtsRootDir")
        .allowlist_function("rpmtsSetRootDir")
        // .allowlist_function("rpmtsSetScriptFd")
        // .allowlist_function("rpmtsGetTid")
        // .allowlist_function("rpmtsGetRdb")
        .allowlist_function("rpmtsFlags")
        .allowlist_function("rpmtsSetFlags")
        // .allowlist_function("rpmtsColor")
        // .allowlist_function("rpmtsPrefColor")
        // .allowlist_function("rpmtsSetColor")
        // .allowlist_function("rpmtsSetPrefColor")
        .allowlist_function("rpmtsSetNotifyCallback")
        .allowlist_function("rpmtsSetNotifyStyle")
        .allowlist_function("rpmtsGetNotifyStyle")
        .allowlist_function("rpmtsAddInstallElement")
        .allowlist_function("rpmtsAddReinstallElement")
        .allowlist_function("rpmtsAddRestoreElement")
        .allowlist_function("rpmtsAddEraseElement")
        .allowlist_function("rpmtsNElements")
        .allowlist_function("rpmtsElement")
        .allowlist_function("rpmtsiFree")
        .allowlist_function("rpmtsiInit")
        .allowlist_function("rpmtsiNext")
        // rpmkeyring.h — keyring and public key management
        .allowlist_function("rpmKeyringNew")
        .allowlist_function("rpmKeyringFree")
        .allowlist_function("rpmKeyringLink")
        .allowlist_function("rpmKeyringAddKey")
        .allowlist_function("rpmKeyringInitIterator")
        .allowlist_function("rpmKeyringIteratorNext")
        .allowlist_function("rpmKeyringIteratorFree")
        // .allowlist_function("rpmKeyringVerifySig")
        // .allowlist_function("rpmKeyringVerifySig2")
        .allowlist_function("rpmKeyringLookupKey")
        .allowlist_function("rpmKeyringModify")
        .allowlist_function("rpmPubkeyNew")
        .allowlist_function("rpmPubkeyFree")
        .allowlist_function("rpmPubkeyLink")
        .allowlist_function("rpmPubkeyBase64")
        .allowlist_function("rpmPubkeyRead")
        .allowlist_function("rpmPubkeyFingerprintAsHex")
        .allowlist_function("rpmPubkeyKeyIDAsHex")
        .allowlist_function("rpmPubkeyArmorWrap")
        // rpmfiles.h — file info (index-style, modern)
        .allowlist_function("rpmfilesNew")
        .allowlist_function("rpmfilesFree")
        // .allowlist_function("rpmfilesLink")
        .allowlist_function("rpmfilesFC")
        .allowlist_function("rpmfilesFindFN")
        // .allowlist_function("rpmfilesFindOFN")
        // .allowlist_function("rpmfilesIter")
        .allowlist_function("rpmfilesDigestAlgo")
        // .allowlist_function("rpmfilesCompare")
        .allowlist_function("rpmfilesBN")
        .allowlist_function("rpmfilesDN")
        .allowlist_function("rpmfilesDI")
        .allowlist_function("rpmfilesFN")
        // .allowlist_function("rpmfilesOBN")
        // .allowlist_function("rpmfilesODN")
        // .allowlist_function("rpmfilesODI")
        // .allowlist_function("rpmfilesOFN")
        .allowlist_function("rpmfilesFFlags")
        .allowlist_function("rpmfilesFMode")
        // .allowlist_function("rpmfilesVFlags")
        .allowlist_function("rpmfilesFState")
        .allowlist_function("rpmfilesFLink")
        .allowlist_function("rpmfilesFSize")
        // .allowlist_function("rpmfilesFColor")
        // .allowlist_function("rpmfilesFClass")
        // .allowlist_function("rpmfilesFNlink")
        // .allowlist_function("rpmfilesFLinks")
        // .allowlist_function("rpmfilesFLangs")
        .allowlist_function("rpmfilesFDigest")
        // .allowlist_function("rpmfilesFSignature")
        // .allowlist_function("rpmfilesVSignature")
        // .allowlist_function("rpmfilesFRdev")
        // .allowlist_function("rpmfilesFInode")
        .allowlist_function("rpmfilesFMtime")
        .allowlist_function("rpmfilesFUser")
        .allowlist_function("rpmfilesFGroup")
        .allowlist_function("rpmfilesFCaps")
        // .allowlist_function("rpmfilesVerify")
        // rpmver.h — version parsing and comparison
        .allowlist_function("rpmvercmp")
        .allowlist_function("rpmverParse")
        .allowlist_function("rpmverNew")
        .allowlist_function("rpmverFree")
        .allowlist_function("rpmverE")
        .allowlist_function("rpmverV")
        .allowlist_function("rpmverR")
        .allowlist_function("rpmverEVR")
        .allowlist_function("rpmverCmp")
        // rpmprob.h — problem reporting
        .allowlist_function("rpmProblemFree")
        .allowlist_function("rpmProblemLink")
        .allowlist_function("rpmProblemGetPkgNEVR")
        .allowlist_function("rpmProblemGetAltNEVR")
        .allowlist_function("rpmProblemGetType")
        // .allowlist_function("rpmProblemGetKey")
        .allowlist_function("rpmProblemGetStr")
        .allowlist_function("rpmProblemGetDiskNeed")
        .allowlist_function("rpmProblemString")
        // rpmps.h — problem sets
        .allowlist_function("rpmpsFree")
        .allowlist_function("rpmpsNumProblems")
        .allowlist_function("rpmpsInitIterator")
        .allowlist_function("rpmpsFreeIterator")
        .allowlist_function("rpmpsiNext")
        // rpmlog.h — logging
        .allowlist_function("rpmlogSetCallback")
        .allowlist_function("rpmlogRecMessage")
        .allowlist_function("rpmlogRecPriority")
        .allowlist_function("rpmlogSetMask")
        .allowlist_function("rpmlogMessage")
        // rpmarchive.h — cpio archive read/write
        // .allowlist_function("rpmfiNewArchiveWriter")
        .allowlist_function("rpmfiNewArchiveReader")
        .allowlist_function("rpmfiArchiveClose")
        // .allowlist_function("rpmfiArchiveTell")
        // .allowlist_function("rpmfiArchiveWrite")
        // .allowlist_function("rpmfiArchiveWriteFile")
        .allowlist_function("rpmfiArchiveRead")
        .allowlist_function("rpmfiArchiveHasContent")
        .allowlist_function("rpmfiArchiveReadToFile")
        .allowlist_function("rpmfileStrerror")
        // rpmstrpool.h — string interning pool
        // .allowlist_function("rpmstrPoolCreate")
        // .allowlist_function("rpmstrPoolFree")
        // .allowlist_function("rpmstrPoolLink")
        // .allowlist_function("rpmstrPoolFreeze")
        // .allowlist_function("rpmstrPoolUnfreeze")
        // .allowlist_function("rpmstrPoolId")
        // .allowlist_function("rpmstrPoolStr")
        // .allowlist_function("rpmstrPoolNumStr")
        // ----------------------------------------------------------
        // Variables
        // ----------------------------------------------------------
        .allowlist_var("rpmGlobalMacroContext")
        // ----------------------------------------------------------
        // Types: enums and flags used by the public API.
        // Opaque pointer types (Header, rpmts, rpmds, …) are pulled
        // in automatically by their functions.
        // ----------------------------------------------------------
        // rpmtag.h — tag identifiers and metadata
        .allowlist_type("rpmTag_e")
        .allowlist_type("rpmDbiTag_e")
        .allowlist_type("rpmSigTag_e")
        .allowlist_type("rpmTagType_e")
        .allowlist_type("rpmTagClass_e")
        // rpmtypes.h — return codes
        .allowlist_type("rpmRC_e")
        // header.h — header access flags
        .allowlist_type("headerGetFlags_e")
        // .allowlist_type("headerPutFlags_e")
        // .allowlist_type("headerImportFlags_e")
        // .allowlist_type("hMagic")
        // rpmts.h — transaction control flags
        .allowlist_type("rpmtransFlags_e")
        .allowlist_type("rpmVSFlags_e")
        .allowlist_type("rpmtxnFlags_e")
        .allowlist_type("rpmprobFilterFlags_e")
        // rpmte.h — transaction element type
        .allowlist_type("rpmElementType_e")
        // rpmds.h — dependency flags
        .allowlist_type("rpmsenseFlags_e")
        // rpmdb.h — iterator match mode
        .allowlist_type("rpmMireMode_e")
        // ----------------------------------------------------------
        // Headers not yet included in librpm.hpp
        // ----------------------------------------------------------
        // rpmfiles.h — file type, state, and iteration mode
        .allowlist_type("rpmfileAttrs_e")
        .allowlist_type("rpmFileIter_e")
        .allowlist_type("rpmfileState_e")
        // rpmkeyring.h
        .allowlist_type("rpmKeyringModifyMode_e")
        // rpmprob.h — problem type
        .allowlist_type("rpmProblemType_e")
        // rpmcallback.h — callback event type
        .allowlist_type("rpmCallbackType_e")
        // rpmlog.h — log level
        .allowlist_type("rpmlogLvl_e")
        // rpmcrypto.h — digest algorithm
        // .allowlist_type("rpmHashAlgo_e")
        // librpm.hpp (local workaround)
        .allowlist_type("Workarounds");

    // Write generated bindings to OUT_DIR (to be included in the crate)
    let output_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("binding.rs");

    builder
        .generate()
        .unwrap()
        .write_to_file(&output_path)
        .unwrap();

    let bindings_src = std::fs::read_to_string(&output_path).unwrap();
    // rpmtsSetNotifyStyle (RPM >= 4.17) switches callbacks from legacy
    // Header-based payloads (style 0) to rpmte-based payloads (style 1).
    // When absent, the trampoline must extract NEVRA from a Header instead.
    if bindings_src.contains("rpmtsSetNotifyStyle") {
        println!("cargo:rpmts_set_notify_style=1");
    }

    // Detect function availability for keyring APIs that may not exist on older RPM.
    for name in [
        "rpmKeyringInitIterator",
        "rpmKeyringIteratorNext",
        "rpmKeyringIteratorFree",
        "rpmKeyringModify",
        "rpmKeyringLookupKey",
        "rpmPubkeyRead",
        "rpmPubkeyFingerprintAsHex",
        "rpmPubkeyKeyIDAsHex",
        "rpmPubkeyArmorWrap",
        "rpmPubkeyLink",
        "rpmKeyringLink",
        "rpmtsImportPubkey",
        "rpmtxnImportPubkey",
        "rpmtxnDeletePubkey",
    ] {
        if bindings_src.contains(name) {
            println!("cargo:rpmkeyring_{name}=1");
        }
    }

    for cap in find_consts(&bindings_src, "rpmTag_e_RPMTAG_") {
        println!("cargo:rpmtag_{}=1", cap.to_lowercase());
    }
    for cap in find_consts(&bindings_src, "rpmSigTag_e_RPMSIGTAG_") {
        println!("cargo:rpmsigtag_{}=1", cap.to_lowercase());
    }
    for cap in find_consts(&bindings_src, "rpmVSFlags_e_RPMVSF_") {
        println!("cargo:rpmvsflag_{}=1", cap.to_lowercase());
    }
    for cap in find_consts(&bindings_src, "rpmElementType_e_") {
        println!("cargo:rpmelementtype_{}=1", cap.to_lowercase());
    }
    for cap in find_consts(&bindings_src, "rpmKeyringModifyMode_e_") {
        println!("cargo:rpmkeyring_modifymode_{}=1", cap.to_lowercase());
    }
    for cap in find_consts(&bindings_src, "rpmtransFlags_e_RPMTRANS_FLAG_") {
        println!("cargo:rpmtransflag_{}=1", cap.to_lowercase());
    }
}
