//! Compare version strings and parse structured EVR values.
//!
//! Run with: cargo run --example version_comparison

use std::cmp::Ordering;

fn main() {
    librpm::init().expect("failed to initialize librpm");

    // Simple string comparison
    let pairs = [
        ("1.0", "2.0"),
        ("1.0.1", "1.0"),
        ("1.0~rc1", "1.0"),
        ("1.0^post1", "1.0"),
        ("2.0", "2.0"),
    ];

    for (a, b) in pairs {
        let ord = librpm::version::vercmp(a, b);
        let symbol = match ord {
            Ordering::Less => "<",
            Ordering::Equal => "=",
            Ordering::Greater => ">",
        };
        println!("  {a} {symbol} {b}");
    }
    println!();

    // Structured version parsing with epoch support
    let v1 = librpm::Version::parse("1:3.2.1-5.fc44").unwrap();
    let v2 = librpm::Version::parse("2.0-1.fc44").unwrap();

    println!(
        "v1 = {v1} (epoch={}, ver={}, rel={})",
        v1.epoch().unwrap_or("(none)"),
        v1.version(),
        v1.release().unwrap_or("(none)")
    );
    println!(
        "v2 = {v2} (epoch={}, ver={}, rel={})",
        v2.epoch().unwrap_or("(none)"),
        v2.version(),
        v2.release().unwrap_or("(none)")
    );
    println!("v1 > v2? {} (epoch wins)", v1 > v2);
    println!();

    // Sorting a list of versions
    let mut versions: Vec<librpm::Version> =
        ["1.0-1.fc44", "1:0.5-1.fc44", "2.0-1.fc44", "1.0-2.fc44"]
            .iter()
            .map(|s| librpm::Version::parse(s).unwrap())
            .collect();

    versions.sort();
    println!("Sorted versions:");
    for v in &versions {
        println!("  {v}");
    }
}
