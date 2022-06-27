//! Idiomatic Rust interface for commiting RPM transactions
//! This uses the internal `TransactionSet` type to manage RPM transactions.

use crate::Package;


pub struct Transaction(crate::internal::ts::TransactionSet);


impl Transaction {
    pub fn new() -> Self {
        Transaction(crate::internal::ts::TransactionSet::create())
    }

    /// Adds a package to be installed in the transation set.
    /// Accepts RPM headers
    pub fn add(&mut self, package: Package) -> Self {
        unimplemented!()
    }

    /// Reinstalls a given package in the transaction set.
    pub fn reinstall(&mut self, package: Package) -> Self {
        unimplemented!()
    }

    /// Removes a given package from the transaction set.
    pub fn remove(&mut self, package: Package) -> Self {
        unimplemented!()
    }

    /// Loads the database from the given file path.
    /// This is useful for testing or installing packages into a separate chroot.
    /// It will check if the database already exists, if so, it will load it. If not, it will create it.
    pub fn use_db(&mut self, db: String) -> Self {
        unimplemented!()
    }

}

