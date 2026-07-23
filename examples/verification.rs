//! Demonstrate package signature verification with custom keyrings.
//!
//! Run with: cargo run --example verification
//!
//! This example loads a GPG key from a file, builds a keyring, and
//! verifies a signed RPM against it. It also shows how to share
//! verification options across multiple package reads.

use std::path::Path;

use librpm::PackageHeader;
use librpm::keyring::{Keyring, PubKey};
use librpm::verify::VerifyOptions;

fn main() {
    librpm::init().expect("failed to initialize librpm");

    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
    let key_path = base.join("keys/rpm-testkey-v4-rsa4096.asc");
    let rpm_path = base.join("rpms/rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm");

    // --- Load a key and build a keyring ---

    let key = PubKey::from_file(&key_path).expect("failed to read key");
    println!("Loaded key: {key:?}");

    let mut keyring = Keyring::new();
    keyring.add_key(&key).expect("failed to add key");

    // --- Verify a package against the custom keyring ---

    let opts = VerifyOptions::new().keyring(keyring.clone());
    match PackageHeader::from_file(&rpm_path, Some(&opts)) {
        Ok(pkg) => println!("Verified OK: {}", pkg.nevra()),
        Err(e) => println!("Verification failed: {e}"),
    }

    // --- Digests only (skip signature checks) ---

    let digest_opts = VerifyOptions::skip_signatures();
    match PackageHeader::from_file(&rpm_path, Some(&digest_opts)) {
        Ok(pkg) => println!("Digest-only OK: {}", pkg.nevra()),
        Err(e) => println!("Digest check failed: {e}"),
    }

    // --- Skip all verification (metadata-only read) ---

    let skip = VerifyOptions::skip_verification();
    let pkg = PackageHeader::from_file(&rpm_path, Some(&skip)).expect("failed to read RPM");
    println!("Metadata-only: {} v{}", pkg.name(), pkg.version());

    // --- Batch verification with shared options ---

    println!("\n=== Batch verification ===");
    let batch_opts = VerifyOptions::new().keyring(keyring);
    for name in ["rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm"] {
        let path = base.join("rpms").join(name);
        match PackageHeader::from_file(&path, Some(&batch_opts)) {
            Ok(pkg) => println!("  OK: {}", pkg.nevra()),
            Err(e) => println!("  FAIL ({}): {e}", name),
        }
    }

    println!("\nDone.");
}
