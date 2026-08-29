//! Demonstrate operating on a database in an alternate root directory,
//! the library equivalent of `rpm --root` / `dnf --installroot`.
//!
//! Run with: cargo run --example alternate_root
//!
//! This creates a throwaway root in a temporary directory, initializes a
//! fresh RPM database inside it, installs a package into that database, and
//! lists the result — all without touching the host's `/`. The install uses
//! `TransactionFlags::JUSTDB`, so only the database is updated (no files are
//! laid down and no chroot is performed), keeping the example self-contained
//! and runnable without elevated privileges.

use std::path::Path;

use librpm::transaction::{ProblemFilter, TransactionFlags};
use librpm::{Db, PackageHeader};

fn main() {
    librpm::init().expect("failed to initialize librpm");

    // A throwaway root so we never touch the host database.
    let root = tempfile::tempdir().expect("failed to create temp root");
    println!("Using alternate root: {}", root.path().display());

    // Initialize a fresh, empty database under the alternate root.
    let db = Db::open_with_root(root.path()).expect("failed to open database in alternate root");
    db.init_db(0o644).expect("failed to initialize database");
    println!(
        "Initialized empty database ({} packages)",
        db.installed_packages().count()
    );

    // Install a package into the alternate root.
    let rpm_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/rpms/rpm-empty-0-0.x86_64.rpm");
    let pkg = PackageHeader::from_file(&rpm_path, None).expect("failed to read RPM");
    println!("\nInstalling {} into the alternate root...", pkg.nevra());

    let mut db = Db::open_with_root(root.path()).expect("failed to reopen database");
    let mut txn = db.transaction();
    txn.add_install(&pkg, &rpm_path, false)
        .expect("failed to add install element");
    // JUSTDB: record the package in the database without writing files.
    txn.set_flags(TransactionFlags::JUSTDB);
    txn.set_problem_filter(ProblemFilter::IGNORE_OS | ProblemFilter::IGNORE_ARCH);
    txn.check().expect("dependency check failed");
    txn.order().expect("ordering failed");
    txn.run().expect("install failed");

    // Confirm the package landed in the alternate root's database.
    let db = Db::open_with_root(root.path()).expect("failed to reopen database");
    println!("\n=== Installed packages in the alternate root ===");
    for pkg in db.installed_packages() {
        println!("  {}", pkg.nevra());
    }
}
