//! Sign an RPM package with a GPG key.
//!
//! Usage: cargo run --example signing --features sign -- <package.rpm> <key-id>

use librpm::sign::{self, HashAlgo, SignArgs};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <package.rpm> <key-id>", args[0]);
        std::process::exit(1);
    }
    let rpm_path = &args[1];
    let key_id = &args[2];

    librpm::init().expect("failed to initialize librpm");

    let sign_args = SignArgs::new().key_id(key_id).hash_algo(HashAlgo::SHA256);

    println!("Signing {rpm_path} with key {key_id}...");
    match sign::sign_package(rpm_path, Some(&sign_args)) {
        Ok(()) => println!("Signed successfully."),
        Err(e) => {
            eprintln!("Signing failed: {e}");
            std::process::exit(1);
        }
    }
}
