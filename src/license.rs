//! RPM licenses

use std::fmt::{self, Display};

/// License types
// TODO: support AND/OR, enum for license types
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct License(String);

impl License {
    /// Create a new license
    pub(crate) fn new<S: AsRef<str>>(string: S) -> License {
        License(string.as_ref().to_owned())
    }

    /// Get a string representation of this license
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for License {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for License {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
