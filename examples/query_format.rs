/// Format installed packages using RPM query format strings.
///
/// This is the Rust equivalent of `rpm -qa --queryformat '...'`.
///
/// Run with: cargo run --example query_format

fn main() {
    librpm::init().expect("failed to initialize librpm");

    let db = librpm::Db::open().expect("failed to open RPM database");

    println!("=== Custom query formats ===\n");

    // NEVRA — the standard package identifier
    println!("--- NEVRA ---");
    for pkg in db.find(librpm::Index::Name, "bash") {
        let nevra = pkg.format("%{NAME}-%{EPOCH}:%{VERSION}-%{RELEASE}.%{ARCH}");
        println!("  {}", nevra.unwrap());
    }

    // Detailed build info
    println!("\n--- Build info ---");
    for pkg in db.find(librpm::Index::Name, "rpm") {
        let info = pkg
            .format("%{NAME} built on %{BUILDHOST} at %{BUILDTIME:date}")
            .unwrap();
        println!("  {}", info);
    }

    // Tabular output with padding
    println!("\n--- Kernel packages (padded columns) ---");
    for pkg in db.find(librpm::Index::Name, "kernel-core") {
        let row = pkg.format("%-30{NAME} %10{SIZE} bytes  %{VENDOR}").unwrap();
        println!("  {}", row);
    }

    // Multi-line format with license and description
    println!("\n--- Package detail ---");
    for pkg in db.find(librpm::Index::Name, "glibc") {
        let detail = pkg
            .format("Name:    %{NAME}\nVersion: %{VERSION}-%{RELEASE}\nLicense: %{LICENSE}\nSummary: %{SUMMARY}")
            .unwrap();
        println!("{detail}\n");
    }
}
