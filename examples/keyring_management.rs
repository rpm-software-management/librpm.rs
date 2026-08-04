//! Demonstrate keyring management operations.
//!
//! Run with: cargo run --example keyring_management
//!
//! This example shows how to:
//! - Load the system keyring from the RPM database
//! - Import a public key into the RPM database
//! - Iterate over keys in a keyring with Keyring::keys()
//! - Look up a specific key with Keyring::lookup()
//! - Delete a key from the RPM database
//!
//! Note: Import and delete operations require root privileges and will
//! fail with a permission error if run as a non-root user.

use std::path::Path;

use librpm::keyring::{Keyring, PubKey};

fn main() {
    librpm::init().expect("failed to initialize librpm");

    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
    let key_path = base.join("keys/rpm-testkey-v4-rsa4096.asc");

    println!("=== Keyring Management Demo ===\n");

    // --- Load the system keyring from RPM database ---

    println!("1. Loading system keyring from RPM database...");
    match Keyring::from_rpmdb() {
        Ok(keyring) => {
            #[cfg(has_rpmkeyring_rpmkeyringinititerator)]
            {
                let count = keyring.keys().count();
                println!("   Loaded system keyring with {} key(s)", count);
            }
            #[cfg(not(has_rpmkeyring_rpmkeyringinititerator))]
            {
                println!("   Loaded system keyring (key iteration not available)");
            }
        }
        Err(e) => println!("   Failed to load system keyring: {e}"),
    }

    // --- Iterate over keys with Keyring::keys() ---

    #[cfg(has_rpmkeyring_rpmkeyringinititerator)]
    {
        println!("\n2. Iterating over system keyring keys...");
        match Keyring::from_rpmdb() {
            Ok(keyring) => {
                for (idx, key) in keyring.keys().enumerate() {
                    #[cfg(has_rpmkeyring_rpmpubkeykeyidashex)]
                    if let Some(key_id) = key.key_id_hex() {
                        println!("   Key {}: ID={}", idx + 1, key_id);
                    }
                    #[cfg(not(has_rpmkeyring_rpmpubkeykeyidashex))]
                    {
                        println!("   Key {}: {:?}", idx + 1, key);
                    }
                }
            }
            Err(e) => println!("   Failed to load keyring: {e}"),
        }
    }

    // --- Import a key to the RPM database ---

    #[cfg(any(has_rpmkeyring_rpmtxnimportpubkey, has_rpmkeyring_rpmtsimportpubkey))]
    {
        println!("\n3. Importing a test key into the RPM database...");
        println!("   (requires root privileges)");
        println!("   Note: import_to_rpmdb() requires binary PGP packet data.");
        println!("   For ASCII-armored keys, you need to dearmor them first.");

        match std::fs::read(&key_path) {
            Ok(key_data) => {
                // Check if the key is ASCII-armored (starts with "-----BEGIN")
                let is_armored = key_data.starts_with(b"-----BEGIN");

                if is_armored {
                    println!("   Key is ASCII-armored - dearmoring required for import");
                    println!("   Skipping import (would need base64 decoding)");
                    println!("   Tip: Use `gpg --dearmor < key.asc > key.gpg` to convert");
                } else {
                    match Keyring::import_to_rpmdb(&key_data) {
                        Ok(()) => {
                            println!("   Successfully imported binary key");

                            // Verify the import by looking it up
                            #[cfg(has_rpmkeyring_rpmkeyringlookupkey)]
                            if let Ok(key) = PubKey::new(&key_data) {
                                if let Ok(keyring) = Keyring::from_rpmdb() {
                                    match keyring.lookup(&key) {
                                        Some(found_key) => {
                                            println!("   Verified: key is now in system keyring");
                                            #[cfg(has_rpmkeyring_rpmpubkeykeyidashex)]
                                            if let Some(key_id) = found_key.key_id_hex() {
                                                println!("   Key ID: {}", key_id);
                                            }
                                        }
                                        None => println!("   Warning: key not found after import"),
                                    }
                                }
                            }
                        }
                        Err(e) => println!("   Import failed (may need root): {e}"),
                    }
                }
            }
            Err(e) => println!("   Failed to read key file: {e}"),
        }
    }

    // --- Look up a key with Keyring::lookup() ---

    #[cfg(has_rpmkeyring_rpmkeyringlookupkey)]
    {
        println!("\n4. Looking up keys in a custom keyring...");

        // Build a custom keyring with our test key
        match PubKey::from_file(&key_path) {
            Ok(test_key) => {
                let mut custom_keyring = Keyring::new();
                custom_keyring
                    .add_key(&test_key)
                    .expect("failed to add key");

                #[cfg(has_rpmkeyring_rpmpubkeykeyidashex)]
                if let Some(key_id) = test_key.key_id_hex() {
                    println!("   Added test key with ID: {}", key_id);
                }

                // Look up the key we just added
                match custom_keyring.lookup(&test_key) {
                    Some(found_key) => {
                        println!("   Successfully looked up the key in custom keyring");
                        #[cfg(has_rpmkeyring_rpmpubkeykeyidashex)]
                        if let Some(id) = found_key.key_id_hex() {
                            println!("   Found key ID: {}", id);
                        }
                    }
                    None => println!("   Key not found (unexpected!)"),
                }

                // Try to look up a key that doesn't exist
                let other_key_path = base.join("keys/rpm-testkey-v4-rsa2048.asc");
                if let Ok(other_key) = PubKey::from_file(&other_key_path) {
                    match custom_keyring.lookup(&other_key) {
                        Some(_) => println!("   Found unexpected key"),
                        None => println!("   Correctly returned None for missing key"),
                    }
                }
            }
            Err(e) => println!("   Failed to read test key: {e}"),
        }
    }

    // --- Delete a key from the RPM database ---

    #[cfg(has_rpmkeyring_rpmtxndeletepubkey)]
    {
        println!("\n5. Deleting the test key from the RPM database...");
        println!("   (requires root privileges)");

        match std::fs::read(&key_path) {
            Ok(key_data) => {
                if let Ok(key) = PubKey::new(&key_data) {
                    match Keyring::delete_from_rpmdb(&key) {
                        Ok(()) => {
                            println!("   Successfully deleted key");

                            // Verify deletion
                            #[cfg(has_rpmkeyring_rpmkeyringlookupkey)]
                            if let Ok(keyring) = Keyring::from_rpmdb() {
                                match keyring.lookup(&key) {
                                    Some(_) => {
                                        println!("   Warning: key still present after delete")
                                    }
                                    None => {
                                        println!("   Verified: key removed from system keyring")
                                    }
                                }
                            }
                        }
                        Err(e) => println!("   Delete failed (may need root): {e}"),
                    }
                }
            }
            Err(e) => println!("   Failed to read key file: {e}"),
        }
    }

    println!("\n=== Demo Complete ===");
}
