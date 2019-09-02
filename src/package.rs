//! RPM package type: represents `.rpm` files or entries in the RPM database

use crate::{license::License, version::Version};

/// RPM packages
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Package {
    /// Name of the package
    pub name: String,

    /// Version of the package
    pub version: Version,

    /// License of the package
    pub license: License,

    /// Succinct description of the package
    pub summary: String,

    /// Longer description of the package
    pub description: String,
}
