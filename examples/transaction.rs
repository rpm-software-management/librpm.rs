//! Demonstrate transaction support: add packages, check dependencies,
//! inspect elements and problems.
//!
//! Run with: cargo run --example transaction
//!
//! This example uses `TransactionFlags::TEST` (dry run) and does not
//! modify the system. The bundled test RPM has intentionally unresolvable
//! dependencies, so `check()` will report problems — this is expected
//! and demonstrates the problem-reporting API.

use std::path::Path;

use librpm::transaction::{CallbackEvent, TransactionFlags};
use librpm::{Db, PackageHeader, VerifyOptions};

fn main() {
    librpm::init().expect("failed to initialize librpm");

    let rpm_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/rpms/rpm-basic-with-rsa4096-2.3.4-5.el9.noarch.rpm");
    let rpm_empty_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/rpms/rpm-empty-0-0.x86_64.rpm");

    let pkg = PackageHeader::from_file(&rpm_path, Some(&VerifyOptions::skip_verification()))
        .expect("failed to read RPM");
    let empty_pkg =
        PackageHeader::from_file(&rpm_empty_path, Some(&VerifyOptions::skip_verification()))
            .expect("failed to read RPM");

    println!("Loaded package: {} ({})", pkg.nevra(), pkg.summary());
    println!(
        "Loaded package: {} ({})",
        empty_pkg.nevra(),
        empty_pkg.summary()
    );

    let mut db = Db::open().expect("failed to open RPM database");

    // Create a transaction and add the package as an upgrade
    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_path, true)
        .expect("failed to add install element");
    txn.add_install(&empty_pkg, &rpm_empty_path, true)
        .expect("failed to add install element");

    // Set the TEST flag to avoid actually making any system changes - comment this line to actually attempt installation
    txn.set_flags(TransactionFlags::TEST);

    // Register a progress callback (events only fire during run())
    txn.set_callback(|event| match event {
        CallbackEvent::InstOpenFile { nevra } => println!("  Opening: {nevra}"),
        CallbackEvent::InstStart { nevra } => println!("  Installing: {nevra}"),
        CallbackEvent::InstProgress { amount, total } => {
            println!("  Progress: {amount}/{total} bytes")
        }
        CallbackEvent::InstStop { nevra } => println!("  Completed: {nevra}"),
        CallbackEvent::InstCloseFile { nevra } => println!("  Closed: {nevra}"),
        CallbackEvent::ScriptStart { nevra, tag } => {
            println!("  Running scriptlet (tag {tag}): {nevra}")
        }
        _ => {}
    });

    println!("\n=== Transaction Elements ({}) ===", txn.len());
    for elem in txn.elements() {
        println!(
            "  {:?}  {}  (size: {} bytes)",
            elem.element_type(),
            elem.nevra(),
            elem.pkg_file_size(),
        );
    }

    // Check dependencies — this will fail because rpm-basic has
    // unresolvable deps like "methylamine >= 1.0.0-1"
    println!("\n=== Dependency Check ===");
    match txn.check() {
        Ok(()) => println!("  No dependency problems found."),
        Err(e) => {
            let problems = e.problems();
            println!("  Found {} problem(s):", problems.len());
            for problem in problems.iter() {
                println!(
                    "    [{:?}] {} — {}",
                    problem.problem_type(),
                    problem.package_nevr(),
                    problem,
                );
            }
        }
    }

    // Order is still possible even with unsatisfied deps
    println!("\n=== Ordering ===");
    match txn.order() {
        Ok(unordered) => println!("  Ordered ({unordered} unorderable elements)"),
        Err(e) => println!("  Ordering failed: {e}"),
    }

    // uncomment this to actually attempt transaction execution
    // txn.run().expect("Could not execute transaction");

    println!("\nDone. No changes were made to the system (TEST mode).");
}
