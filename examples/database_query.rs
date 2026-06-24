//! Query the installed RPM package database.
//!
//! Run with: cargo run --example database_query

fn main() {
    librpm::init().expect("failed to initialize librpm");

    let db = librpm::Db::open().expect("failed to open RPM database");

    println!("Looking up 'rpm' in the database...");
    for pkg in db.find(librpm::Index::Name, "rpm") {
        println!("  found: {} ({})", pkg.nevra(), pkg.summary());
    }

    println!(
        "\nTotal installed packages: {}",
        db.installed_packages().count()
    );
}
