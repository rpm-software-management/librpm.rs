//! Tests for Keyring and PubKey types.

use std::path::Path;

use librpm::keyring::{Keyring, PubKey};

mod common;

fn testdata_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

fn test_key_path() -> std::path::PathBuf {
    testdata_path().join("keys/rpm-testkey-v4-rsa4096.asc")
}

fn load_test_key() -> PubKey {
    PubKey::from_file(&test_key_path()).unwrap()
}

// --- PubKey ---

#[test]
fn test_pubkey_new_invalid_data() {
    common::configure();
    let result = PubKey::new(b"not a valid key");
    assert!(result.is_err());
}

#[test]
fn test_pubkey_from_file() {
    common::configure();
    let key = PubKey::from_file(&test_key_path()).unwrap();
    assert!(key.base64().is_some());
}

#[test]
fn test_pubkey_from_file_nonexistent() {
    common::configure();
    let result = PubKey::from_file(Path::new("/nonexistent/key.asc"));
    assert!(result.is_err());
}

#[test]
fn test_pubkey_base64() {
    common::configure();
    let key = load_test_key();
    let b64 = key.base64().expect("base64 should succeed");
    assert!(b64.len() > 100, "base64 output should be substantial");
}

#[cfg(has_rpmkeyring_rpmpubkeykeyidashex)]
#[test]
fn test_pubkey_key_id_hex() {
    common::configure();
    let key = load_test_key();
    let id = key.key_id_hex().expect("key_id_hex should succeed");
    assert!(!id.is_empty());
    assert!(
        id.to_lowercase().contains("30d073b5"),
        "key ID should contain 30D073B5, got: {id}"
    );
}

#[cfg(has_rpmkeyring_rpmpubkeyfingerprintashex)]
#[test]
fn test_pubkey_fingerprint_hex() {
    common::configure();
    let key = load_test_key();
    let fp = key.fingerprint_hex().expect("fingerprint should succeed");
    assert!(!fp.is_empty());
}

#[test]
fn test_pubkey_clone() {
    common::configure();
    let key = load_test_key();
    let cloned = key.clone();
    assert_eq!(key.base64(), cloned.base64());
}

#[test]
fn test_pubkey_debug() {
    common::configure();
    let key = load_test_key();
    let debug = format!("{key:?}");
    assert!(debug.contains("PubKey"));
}

// --- Keyring ---

#[test]
fn test_keyring_new_empty() {
    common::configure();
    let keyring = Keyring::new();
    let debug = format!("{keyring:?}");
    assert!(debug.contains("Keyring"));
}

#[test]
fn test_keyring_add_key() {
    common::configure();
    let mut keyring = Keyring::new();
    let key = load_test_key();

    keyring.add_key(&key).unwrap();

    // Adding the same key again should not error.
    let key2 = key.clone();
    keyring.add_key(&key2).unwrap();
}

#[test]
fn test_keyring_clone() {
    common::configure();
    let mut keyring = Keyring::new();
    let key = load_test_key();
    keyring.add_key(&key).unwrap();

    let _cloned = keyring.clone();
}

#[cfg(has_rpmkeyring_rpmkeyringmodify)]
#[test]
fn test_keyring_remove_key() {
    common::configure();
    let mut keyring = Keyring::new();
    let key = load_test_key();

    keyring.add_key(&key).unwrap();
    let removed = keyring.remove_key(&key).unwrap();
    assert!(removed, "remove should succeed for existing key");
}

#[cfg(has_rpmkeyring_rpmkeyringlookupkey)]
#[test]
fn test_keyring_lookup() {
    common::configure();
    let mut keyring = Keyring::new();
    let key = load_test_key();

    assert!(keyring.lookup(&key).is_none(), "empty keyring: no match");

    keyring.add_key(&key).unwrap();
    let found = keyring.lookup(&key);
    assert!(found.is_some(), "should find added key");
}

#[cfg(has_rpmkeyring_rpmkeyringinititerator)]
#[test]
fn test_keyring_iter_empty() {
    common::configure();
    let keyring = Keyring::new();
    assert_eq!(keyring.keys().count(), 0);
}

#[cfg(has_rpmkeyring_rpmkeyringinititerator)]
#[test]
fn test_keyring_iter_with_keys() {
    common::configure();
    let mut keyring = Keyring::new();
    let key = load_test_key();
    keyring.add_key(&key).unwrap();

    let keys: Vec<PubKey> = keyring.keys().collect();
    assert_eq!(keys.len(), 1);
    assert!(keys[0].base64().is_some());
}

#[test]
fn test_keyring_from_rpmdb() {
    common::configure();
    let keyring = Keyring::from_rpmdb().unwrap();
    let _debug = format!("{keyring:?}");
}

#[cfg(all(
    has_rpmkeyring_rpmtxnimportpubkey,
    has_rpmkeyring_rpmtxndeletepubkey,
    has_rpmkeyring_rpmtsimportpubkey,
    has_rpmkeyring_rpmkeyringlookupkey,
))]
#[test]
#[ignore = "modifies system rpmdb, requires root"]
fn test_import_to_rpmdb() {
    common::configure();
    let key_data = std::fs::read(test_key_path()).unwrap();
    Keyring::import_to_rpmdb(&key_data).unwrap();

    // Clean up: delete the imported key
    let db = librpm::Db::open().unwrap();
    let keyring = db.keyring();
    let test_key = PubKey::from_file(&test_key_path()).unwrap();
    if keyring.lookup(&test_key).is_some() {
        Keyring::delete_from_rpmdb(&test_key).unwrap();
    }
}

#[cfg(all(
    has_rpmkeyring_rpmtxnimportpubkey,
    has_rpmkeyring_rpmtxndeletepubkey,
    has_rpmkeyring_rpmkeyringlookupkey,
))]
#[test]
#[ignore = "modifies system rpmdb, requires root"]
fn test_delete_from_rpmdb() {
    common::configure();
    let key_data = std::fs::read(test_key_path()).unwrap();

    // Import first so we have something to delete
    Keyring::import_to_rpmdb(&key_data).unwrap();

    let test_key = PubKey::from_file(&test_key_path()).unwrap();
    Keyring::delete_from_rpmdb(&test_key).unwrap();

    // Verify deletion
    let db = librpm::Db::open().unwrap();
    let keyring = db.keyring();
    assert!(
        keyring.lookup(&test_key).is_none(),
        "key should no longer be in the keyring after deletion"
    );
}
