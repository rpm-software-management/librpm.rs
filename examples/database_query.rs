//! Query the installed RPM package database.
//!
//! Run with: cargo run --example database_query

use librpm::{Db, Index};

fn main() {
    librpm::init().expect("failed to initialize librpm");

    let db = Db::open().expect("failed to open RPM database");

    // Look up a package by name (exact match)
    println!("=== Find by Name ===");
    for pkg in db.find(Index::Name, "rpm") {
        println!("  {} — {}", pkg.nevra(), pkg.summary());
    }

    // Find packages that provide a given capability
    println!("\n=== Find by Providename ===");
    for pkg in db.find(Index::Providename, "libc.so.6()(64bit)") {
        println!("  {}", pkg.nevra());
    }

    // Find packages that require a given dependency
    println!("\n=== Find by Requirename ===");
    for pkg in db.find(Index::Requirename, "bash") {
        println!("  {}", pkg.nevra());
    }

    // Find packages that own a specific file path
    println!("\n=== Find by Instfilenames ===");
    for pkg in db.find(Index::Instfilenames, "/usr/bin/python3") {
        println!("  {}", pkg.nevra());
    }

    // Find packages that own files in a specific directory
    println!("\n=== Find by Dirnames ===");
    for pkg in db.find(Index::Dirnames, "/etc/pki/tls/") {
        println!("  {}", pkg.nevra());
    }

    // Glob search by name
    println!("\n=== Glob search: python3* ===");
    for pkg in db.find_glob(Index::Name, "python3*") {
        println!("  {}", pkg.nevra());
    }

    // Regex search by name
    println!("\n=== Regex search: ^lib.*-devel$ ===");
    for pkg in db.find_regex(Index::Name, "^lib.*-devel$") {
        println!("  {}", pkg.nevra());
    }

    // Total installed count
    println!(
        "\nTotal installed packages: {}",
        db.installed_packages().count()
    );
}
