//! RPM package type: represents `.rpm` files or entries in the RPM database
use std::fmt;

/// RPM packages
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Package {
    /// Name of the package
    pub(crate) name: String,

    /// Epoch of the package
    pub(crate) epoch: Option<String>,

    /// Version of the package
    pub(crate) version: String,

    /// Release of the package
    pub(crate) release: String,

    /// Arch of the package
    pub(crate) arch: String,

    /// License of the package
    pub(crate) license: String,

    /// Succinct description of the package
    pub(crate) summary: String,

    /// Longer description of the package
    pub(crate) description: String,
}

impl Package {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn epoch(&self) -> Option<&str> {
        self.epoch.as_ref().map(|s| s.as_str())
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn release(&self) -> &str {
        &self.release
    }

    pub fn arch(&self) -> &str {
        &self.arch
    }

    pub fn evr(&self) -> String {
        if let Some(epoch) = &self.epoch {
            format!("{}:{}-{}", epoch, self.version, self.release)
        } else {
            format!("{}-{}", self.version, self.release)
        }
    }

    pub fn nevra(&self) -> String {
        format!("{}-{}.{}", self.name, self.evr(), self.arch)
    }

    pub fn license(&self) -> &str {
        &self.license
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nevra())
    }
}
