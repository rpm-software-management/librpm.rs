//! librpm error types

use std::fmt::{self, Display};

/// Error type
#[derive(Debug)]
pub struct Error {
    /// Kind of error
    kind: ErrorKind,

    /// Optional description message
    msg: Option<String>,
}

impl Error {
    /// Create a new error with the given description
    pub fn new(kind: ErrorKind, msg: Option<String>) -> Self {
        Self { kind, msg }
    }

    /// Obtain the inner `ErrorKind` for this error
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind, msg: None }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;

        if let Some(msg) = &self.msg {
            write!(f, ": {}", msg)?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorKind {
    /// Configuration errors
    Config,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Config => write!(f, "configuration error"),
        }
    }
}

/// Create a new error (of a given enum variant) with a formatted message
macro_rules! format_err {
    ($kind:path, $msg:expr) => {
        $crate::error::Error::new($kind, Some($msg.to_string()))
    };
    ($kind:path, $fmt:expr, $($arg:tt)+) => {
        format_err!($kind, &format!($fmt, $($arg)+))
    };
}

/// Create and return an error enum variant with a formatted message
macro_rules! fail {
    ($kind:path, $msg:expr) => {
        return Err(format_err!($kind, $msg))
    };
    ($kind:path, $fmt:expr, $($arg:tt)+) => {
        return Err(format_err!($kind, $fmt, $($arg)+))
    };
}
