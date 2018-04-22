//! librpm error types

use failure::{Backtrace, Context, Fail};
use std::fmt::{self, Display};

/// Error type
#[derive(Debug)]
pub struct Error {
    /// Contextual information about the error
    inner: Context<ErrorKind>,

    /// Optional description message
    description: Option<String>,
}

impl Error {
    /// Create a new error
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            inner: Context::new(kind),
            description: None,
        }
    }

    /// Create a new error with the given description
    pub fn with_description(kind: ErrorKind, description: String) -> Self {
        Self {
            inner: Context::new(kind),
            description: Some(description),
        }
    }

    /// Obtain the inner `ErrorKind` for this error
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Self {
        Self {
            inner,
            description: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.description {
            Some(ref desc) => write!(f, "{}: {}", &self.inner, desc),
            None => Display::fmt(&self.inner, f),
        }
    }
}

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    /// Configuration errors
    #[fail(display = "configuration error")]
    ConfigError,
}

/// Create a new error (of a given enum variant) with a formatted message
macro_rules! err {
    ($variant:ident, $msg:expr) => {
        ::error::Error::with_description(::error::ErrorKind::$variant, $msg.to_owned())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)+) => {
        ::error::Error::with_description(::error::ErrorKind::$variant, format!($fmt, $($arg)+))
    };
}

/// Create and return an error enum variant with a formatted message
macro_rules! fail {
    ($variant:ident, $msg:expr) => {
        return Err(err!($variant, $msg));
    };
    ($variant:ident, $fmt:expr, $($arg:tt)+) => {
        return Err(err!($variant, $fmt, $($arg)+));
    };
}
