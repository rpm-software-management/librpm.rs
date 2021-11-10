//! Transaction sets: librpm's transaction API

use super::GlobalState;
use std::sync::atomic::AtomicPtr;
use std::sync::MutexGuard;

/// librpm transactions, a.k.a. "transaction sets" (or `rpmts` librpm type)
///
/// Nearly all access to librpm, including actions which don't necessarily
/// involve operations on the RPM database, require a transaction set.
///
/// This library opens a single global transaction set on command, and all
/// operations which require one acquire it, use it, and then release it.
/// This allows us to keep them out of the public API.
pub(crate) struct TransactionSet(AtomicPtr<librpm_sys::rpmts_s>);

impl TransactionSet {
    /// Create a transaction set (i.e. begin a transaction)
    ///
    /// This is not intended to be invoked directly, but instead obtained
    /// from `GlobalState`.
    pub(crate) fn create() -> Self {
        TransactionSet(AtomicPtr::new(unsafe { librpm_sys::rpmtsCreate() }))
    }
}

impl Drop for TransactionSet {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::rpmtsFree(*self.0.get_mut());
        }
    }
}

impl TransactionSet {
    pub(crate) fn as_mut_ptr(&mut self) -> &mut *mut librpm_sys::rpmts_s {
        self.0.get_mut()
    }
}

/// Crate-public wrapper for acquiring and releasing the global transaction set
/// which also cleans it prior to unlocking it.
pub(crate) struct GlobalTS(MutexGuard<'static, GlobalState>);

impl GlobalTS {
    /// Acquire the global state mutex, giving the current thread exclusive
    /// access to the global transaction set.
    pub fn create() -> Self {
        GlobalTS(GlobalState::lock())
    }

    /// Obtain the internal pointer to the transaction set
    pub(crate) fn as_mut_ptr(&mut self) -> *mut librpm_sys::rpmts_s {
        // Since we're guaranteed to be holding the GlobalState mutex here,
        // we're free to deref the pointer.
        *self.0.ts.as_mut_ptr()
    }
}

/// Tidy up the shared global transaction set between uses
impl Drop for GlobalTS {
    fn drop(&mut self) {
        unsafe {
            librpm_sys::rpmtsClean(self.as_mut_ptr());
        }
    }
}

// pub(crate) enum RpmVsFlags {
//     Default = librpm_sys::rpmVSFlags_e_RPMVSF_DEFAULT as isize,
//     NoHeaderCheck = librpm_sys::rpmVSFlags_e_RPMVSF_NOHDRCHK as isize,
//     NeedPayload = librpm_sys::rpmVSFlags_e_RPMVSF_NEEDPAYLOAD as isize,

//     NoSha1Header = librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA1HEADER as isize,
//     NoSha256Header = librpm_sys::rpmVSFlags_e_RPMVSF_NOSHA256HEADER as isize,
//     NoDsaHeader = librpm_sys::rpmVSFlags_e_RPMVSF_NODSAHEADER as isize,
//     NoRsaHeader = librpm_sys::rpmVSFlags_e_RPMVSF_NORSAHEADER as isize,

//     NoPayload = librpm_sys::rpmVSFlags_e_RPMVSF_NOPAYLOAD as isize,
//     NoMd5 = librpm_sys::rpmVSFlags_e_RPMVSF_NOMD5 as isize,
//     NoDsa = librpm_sys::rpmVSFlags_e_RPMVSF_NODSA as isize,
//     NoRsa = librpm_sys::rpmVSFlags_e_RPMVSF_NORSA as isize,
// }

// const NoDigestMask: isize = RpmVsFlags::NoSha1Header | RpmVsFlags::NoSha256Header | RpmVsFlags::NoPayload | RpmVsFlags::NoMd5;
// const NoSignaturesMask: isize = RpmVsFlags::NoDsaHeader | RpmVsFlags::NoRsaHeader | RpmVsFlags::NoDsa | RpmVsFlags::NoRsa;
// const NoHeaderMask: isize = RpmVsFlags::NoSha1Header | RpmVsFlags::NoSha256Header | RpmVsFlags::NoDsaHeader | RpmVsFlags::NoRsaHeader;
// const NoPayloadMask: isize = RpmVsFlags::NoMd5 | RpmVsFlags::NoPayload | RpmVsFlags::NoDsa | RpmVsFlags::NoRsa;
