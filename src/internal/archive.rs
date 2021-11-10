#[derive(Debug)]
pub enum RpmErrorKind {
    /// Generic not found code
    NotFound,
    /// Generic failure code
    Fail,
    /// Signature is OK but key is not trusted
    NotTrusted,
    /// No public key available to verify the signature
    NoKey,
}

pub enum RpmReturnCode {
    Ok = librpm_sys::rpmRC_e_RPMRC_OK as isize,
    NotFound = librpm_sys::rpmRC_e_RPMRC_NOTFOUND as isize,
    Fail = librpm_sys::rpmRC_e_RPMRC_FAIL as isize,
    NotTrusted = librpm_sys::rpmRC_e_RPMRC_NOTTRUSTED as isize,
    NoKey = librpm_sys::rpmRC_e_RPMRC_NOKEY as isize,
}
