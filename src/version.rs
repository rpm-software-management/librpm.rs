//! RPM versions

use std::fmt::{self, Display};

/// Package versions
// TODO: `Ord` (i.e. dependency sets)
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Version(String);

impl Version {
    /// Create a new version
    pub(crate) fn new<S: AsRef<str>>(string: S) -> Version {
        Version(string.as_ref().to_owned())
    }

    /// Get a string representation of this version
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for Version {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
