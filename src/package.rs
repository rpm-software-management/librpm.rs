//! RPM package type: represents `.rpm` files or entries in the RPM database
use std::{fmt, path::Path};
use crate::internal::{header::Header, tag::Tag};

/// RPM packages
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Package {
    /// Name of the package
    name: String,

    /// Epoch of the package
    epoch: Option<String>,

    /// Version of the package
    version: String,

    /// Release of the package
    release: String,

    /// Arch of the package
    arch: String,

    /// License of the package
    license: String,

    /// Succinct description of the package
    summary: String,

    /// Longer description of the package
    description: String,
}

impl Package {
    pub(crate) fn from_header(h: &Header) -> Self {
        Package {
            name: h.get(Tag::NAME).unwrap().as_str().unwrap().to_owned(),
            epoch: h.get(Tag::EPOCH).map(|d| d.as_str().unwrap().to_owned()),
            version: h.get(Tag::VERSION).unwrap().as_str().unwrap().to_owned(),
            release: h.get(Tag::RELEASE).unwrap().as_str().unwrap().to_owned(),
            arch: h.get(Tag::ARCH).unwrap().as_str().unwrap().to_owned(),
            license: h.get(Tag::LICENSE).unwrap().as_str().unwrap().to_owned(),
            summary: h.get(Tag::SUMMARY).unwrap().as_str().unwrap().into(),
            description: h.get(Tag::DESCRIPTION).unwrap().as_str().unwrap().into(),
        }
    }

    pub fn from_file(path: &Path) -> Self {
        let header = Header::from_file(path).unwrap();
        Self::from_header(&header)
    }

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
