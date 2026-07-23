//! Tests for VerificationFlags, VerifyOptions, and package verification.

use std::path::Path;

use librpm::keyring::{Keyring, PubKey};
use librpm::verify::{VerificationFlags, VerifyOptions};
use librpm::{PackageHeader, RpmErrorKind};

mod common;

fn testdata_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

fn signed_rpm_path() -> std::path::PathBuf {
    testdata_path().join("rpms/rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm")
}

fn unsigned_rpm_path() -> std::path::PathBuf {
    testdata_path().join("rpms/rpm-empty-0-0.x86_64.rpm")
}

fn load_test_key() -> PubKey {
    PubKey::from_file(&testdata_path().join("keys/rpm-testkey-v4-rsa4096.asc")).unwrap()
}

// --- VerificationFlags ---

mod verification_flags {
    use super::*;

    #[test]
    fn test_flags_default_is_zero() {
        assert_eq!(VerificationFlags::DEFAULT.bits(), 0);
    }

    #[test]
    fn test_flags_bitor() {
        let combined = VerificationFlags::NODSA | VerificationFlags::NORSA;
        assert_ne!(combined.bits(), 0);
        assert_eq!(
            combined.bits(),
            VerificationFlags::NODSA.bits() | VerificationFlags::NORSA.bits()
        );
    }

    #[test]
    fn test_flags_bitor_assign() {
        let mut flags = VerificationFlags::DEFAULT;
        flags |= VerificationFlags::NOMD5;
        assert_eq!(flags.bits(), VerificationFlags::NOMD5.bits());
    }

    #[test]
    fn test_flags_bitand() {
        let all = VerificationFlags::all_disabled();
        let masked = all & VerificationFlags::mask_nosignatures();
        assert_eq!(masked.bits(), VerificationFlags::mask_nosignatures().bits());
    }

    #[test]
    fn test_flags_not() {
        let inverted = !VerificationFlags::DEFAULT;
        assert_ne!(inverted.bits(), 0);
    }

    #[test]
    fn test_mask_nosignatures() {
        let sigs = VerificationFlags::mask_nosignatures();
        assert_ne!(sigs.bits(), 0);
        #[allow(unused_mut)]
        let mut actual = VerificationFlags::NODSAHEADER.bits()
            | VerificationFlags::NORSAHEADER.bits()
            | VerificationFlags::NODSA.bits()
            | VerificationFlags::NORSA.bits();
        #[cfg(has_rpmvsflag_noopenpgp)]
        {
            actual |= VerificationFlags::NOOPENPGP.bits();
        }
        assert_eq!(sigs.bits(), actual);
    }

    #[test]
    fn test_mask_nodigests() {
        let digests = VerificationFlags::mask_nodigests();
        assert_ne!(digests.bits(), 0);
        assert_ne!(
            digests.bits() & VerificationFlags::NOMD5.bits(),
            0,
            "mask_nodigests should include NOMD5"
        );
        assert_ne!(
            digests.bits() & VerificationFlags::NOSHA1HEADER.bits(),
            0,
            "mask_nodigests should include NOSHA1HEADER"
        );
        assert_ne!(
            digests.bits() & VerificationFlags::NOSHA256HEADER.bits(),
            0,
            "mask_nodigests should include NOSHA256HEADER"
        );
    }

    #[test]
    fn test_all_disabled() {
        let all = VerificationFlags::all_disabled();
        assert_ne!(all.bits(), 0);
        assert_eq!(
            all.bits() & VerificationFlags::mask_nosignatures().bits(),
            VerificationFlags::mask_nosignatures().bits(),
            "all_disabled should include all signature flags"
        );
        assert_eq!(
            all.bits() & VerificationFlags::mask_nodigests().bits(),
            VerificationFlags::mask_nodigests().bits(),
            "all_disabled should include all digest flags"
        );
    }

    #[test]
    fn test_flags_clone_copy() {
        let flags = VerificationFlags::NOMD5;
        let cloned = flags;
        assert_eq!(flags.bits(), cloned.bits());
    }
}

// --- Package verification ---

#[test]
fn test_from_file_skip_verification() {
    common::configure();
    let opts = VerifyOptions::skip_verification();
    let pkg = PackageHeader::from_file(&signed_rpm_path(), Some(&opts)).unwrap();
    assert_eq!(pkg.name(), "rpm-basic");
}

#[test]
fn test_from_file_none_uses_system_defaults() {
    common::configure();
    // With None, librpm uses system defaults: all checks enabled, system keyring.
    // The test RPM is signed with a test key NOT in the system keyring, so
    // this will return NotTrusted or NoKey depending on the RPM version.
    let result = PackageHeader::from_file(&signed_rpm_path(), None);
    // We just check that it doesn't panic — the result depends on system config.
    // On most test systems the test key is not trusted, so we expect an error.
    let _result = result;
}

#[test]
fn test_from_file_with_correct_keyring() {
    common::configure();
    let mut keyring = Keyring::new();
    let key = load_test_key();
    keyring.add_key(&key).unwrap();

    let opts = VerifyOptions::new().keyring(keyring);
    let pkg = PackageHeader::from_file(&signed_rpm_path(), Some(&opts)).unwrap();
    assert_eq!(pkg.name(), "rpm-basic");
    assert_eq!(pkg.version(), "2.3.4");
}

#[test]
fn test_from_file_with_empty_keyring_rejects_signed() {
    common::configure();
    let keyring = Keyring::new();
    let opts = VerifyOptions::new().keyring(keyring);

    let result = PackageHeader::from_file(&signed_rpm_path(), Some(&opts));
    assert!(
        result.is_err(),
        "empty keyring should reject signed package"
    );
    let err = result.unwrap_err();
    assert!(
        err == RpmErrorKind::NoKey || err == RpmErrorKind::NotTrusted,
        "expected NoKey or NotTrusted, got: {err}"
    );
}

#[test]
fn test_from_file_digests_only() {
    common::configure();
    let opts = VerifyOptions::skip_signatures();
    let pkg = PackageHeader::from_file(&signed_rpm_path(), Some(&opts)).unwrap();
    assert_eq!(pkg.name(), "rpm-basic");
}

#[test]
fn test_from_file_unsigned_skip_verify() {
    common::configure();
    let opts = VerifyOptions::skip_verification();
    let pkg = PackageHeader::from_file(&unsigned_rpm_path(), Some(&opts)).unwrap();
    assert_eq!(pkg.name(), "rpm-empty");
}

#[test]
fn test_shared_options_across_multiple_reads() {
    common::configure();
    let mut keyring = Keyring::new();
    let key = load_test_key();
    keyring.add_key(&key).unwrap();

    let opts = VerifyOptions::new().keyring(keyring);

    let pkg1 = PackageHeader::from_file(&signed_rpm_path(), Some(&opts)).unwrap();
    assert_eq!(pkg1.name(), "rpm-basic");

    let pkg2 = PackageHeader::from_file(&signed_rpm_path(), Some(&opts)).unwrap();
    assert_eq!(pkg2.name(), "rpm-basic");
    assert_eq!(pkg1, pkg2);
}
