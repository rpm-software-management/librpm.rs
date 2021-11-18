//! RPM package type: represents `.rpm` files or entries in the RPM database
use std::fmt;

/// RPM packages
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Package {
    pub(crate) name: String,
    pub(crate) epoch: Option<String>,
    pub(crate) version: String,
    pub(crate) release: String,
    pub(crate) arch: Option<String>,
    pub(crate) license: String,
    pub(crate) summary: String,
    pub(crate) description: String,
}

impl Package {
    /// Name of the package
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Epoch of the package
    pub fn epoch(&self) -> Option<&str> {
        self.epoch.as_ref().map(|s| s.as_str())
    }

    /// Version of the package
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Release of the package
    pub fn release(&self) -> &str {
        &self.release
    }

    /// Arch of the package
    pub fn arch(&self) -> Option<&str> {
        self.arch.as_ref().map(|s| s.as_str())
    }

    /// EVR (epoch, version, release) of the package
    pub fn evr(&self) -> String {
        if let Some(epoch) = &self.epoch {
            format!("{}:{}-{}", epoch, self.version, self.release)
        } else {
            format!("{}-{}", self.version, self.release)
        }
    }

    /// NEVRA (name, epoch, version, release, arch) of the package
    pub fn nevra(&self) -> String {
        if let Some(arch) = &self.arch {
            format!("{}-{}.{}", self.name, self.evr(), arch)
        } else {
            format!("{}-{}", self.name, self.evr())
        }
    }

    /// License of the package
    pub fn license(&self) -> &str {
        &self.license
    }

    /// Succinct description of the package
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Longer description of the package
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nevra())
    }
}
