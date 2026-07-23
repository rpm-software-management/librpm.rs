/// Errors returned by librpm operations such as reading package files
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpmErrorKind {
    /// Generic not found code
    NotFound,
    /// Generic failure code
    Fail,
    /// Signature is OK but key is not trusted
    NotTrusted,
    /// No public key available to verify the signature
    NoKey,
    /// I/O error (e.g. file not found, permission denied)
    Io(String),
}

impl std::fmt::Display for RpmErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "package not found"),
            Self::Fail => write!(f, "operation failed"),
            Self::NotTrusted => write!(f, "signature is present but key is not trusted"),
            Self::NoKey => write!(f, "no public key available to verify signature"),
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for RpmErrorKind {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RpmReturnCode {
    Ok = librpm_sys::rpmRC_e_RPMRC_OK,
    NotFound = librpm_sys::rpmRC_e_RPMRC_NOTFOUND,
    Fail = librpm_sys::rpmRC_e_RPMRC_FAIL,
    NotTrusted = librpm_sys::rpmRC_e_RPMRC_NOTTRUSTED,
    NoKey = librpm_sys::rpmRC_e_RPMRC_NOKEY,
}

impl RpmReturnCode {
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            librpm_sys::rpmRC_e_RPMRC_OK => Some(Self::Ok),
            librpm_sys::rpmRC_e_RPMRC_NOTFOUND => Some(Self::NotFound),
            librpm_sys::rpmRC_e_RPMRC_FAIL => Some(Self::Fail),
            librpm_sys::rpmRC_e_RPMRC_NOTTRUSTED => Some(Self::NotTrusted),
            librpm_sys::rpmRC_e_RPMRC_NOKEY => Some(Self::NoKey),
            _ => None,
        }
    }
}
