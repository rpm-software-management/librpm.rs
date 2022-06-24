//! Transaction sets: librpm's transaction API

use librpm_sys::rpmtsiNext;

use crate::db::Iter;

use super::GlobalState;
use super::te::{TransactionElement, ElementTypes};
use super::txn::Transaction;
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::sync::atomic::AtomicPtr;
use std::sync::MutexGuard;
use bitflags::bitflags;

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

    pub(crate) fn check(&mut self) -> bool {
        unsafe { librpm_sys::rpmtsCheck(*self.0.get_mut()) == 0 }
    }

    pub(crate) fn root_dir(&mut self) -> String {
        unsafe {
            librpm_sys::rpmtsRootDir(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }

    pub(crate) fn set_root_dir(&mut self, root_dir: &str) -> Result<(), InvalidRootDir> {
        let str = CString::new(root_dir).expect("invalid string passed as root_dir");
        unsafe {
            if librpm_sys::rpmtsSetRootDir(*self.0.get_mut(), str.as_ptr()) == 0 {
                Ok(())
            } else {
                Err(InvalidRootDir)
            }
        }
    }

    pub(crate) fn element_length(&mut self) -> i32 {
        unsafe { librpm_sys::rpmtsNElements(*self.0.get_mut()) }
    }

    pub(crate) fn get_element(&mut self, index: i32) -> TransactionElement {
        if index > self.element_length() - 1 {
            panic!("out of bounds transaction element access")
        }

        let rpmte = unsafe { librpm_sys::rpmtsElement(*self.0.get_mut(), index) };
        unsafe { TransactionElement::from_ptr(rpmte) }
    }

    pub(crate) fn iter(&mut self, flags: ElementTypes) -> TransactionSetIterator {
        let iterator = unsafe { librpm_sys::rpmtsiInit(*self.0.get_mut()) };

        unsafe { TransactionSetIterator::from_ptr(iterator, flags) }
    }

    pub(crate) fn transaction_id(&mut self) -> u32 {
        unsafe { librpm_sys::rpmtsGetTid(*self.0.get_mut()) }
    }

    pub(crate) fn set_transaction_id(&mut self, id: u32) -> u32 {
        unsafe { librpm_sys::rpmtsSetTid(*self.0.get_mut(), id) }
    }

    pub(crate) fn flags(&mut self) -> TransFlags {
        unsafe { TransFlags::from_bits_truncate(librpm_sys::rpmtsFlags(*self.0.get_mut())) }
    }

    pub(crate) fn set_flags(&mut self, flags: TransFlags) -> TransFlags {
        unsafe { TransFlags::from_bits_truncate(librpm_sys::rpmtsSetFlags(*self.0.get_mut(), flags.bits())) }
    }

    pub(crate) fn transaction_begin(&mut self, flags: TransFlags) -> Transaction {
        unsafe { Transaction::from_ptr(librpm_sys::rpmtxnBegin(*self.0.get_mut(), flags.bits())) }
    }
}

#[derive(Debug)]
pub(crate) struct InvalidRootDir;

impl Display for InvalidRootDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the root dir passed is invalid")
    }
}

pub(crate) struct TransactionSetIterator{
    ptr: AtomicPtr<librpm_sys::rpmtsi_s>,
    flags: ElementTypes,
    exhausted: bool
}

impl TransactionSetIterator {
    pub(crate) unsafe fn from_ptr(ffi_tsi: librpm_sys::rpmtsi, flags: ElementTypes) -> Self {
        assert!(!ffi_tsi.is_null());

        TransactionSetIterator {
            ptr: AtomicPtr::from(ffi_tsi),
            flags,
            exhausted: false
        }
    }
}

impl Iterator for TransactionSetIterator {
    type Item = TransactionElement;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted { return None };

        let element = unsafe { librpm_sys::rpmtsiNext(*self.ptr.get_mut(), self.flags.bits()) };
        if element.is_null() {
            self.exhausted = true;
            return None;
        }

        unsafe { Some(TransactionElement::from_ptr(element)) }
    }
}

impl Drop for TransactionSetIterator {
    fn drop(&mut self) {
        unsafe { librpm_sys::rpmtsiFree(*self.ptr.get_mut()) };
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

// TRANS FLAGS!!! 🏳️‍⚧️
bitflags! {
    pub(crate) struct TransFlags: u32 {
        const RPMTRANS_FLAG_NONE = 0;
        const RPMTRANS_FLAG_TEST = (1 << 0);
        const RPMTRANS_FLAG_BUILD_PROBS = (1 << 1);
        const RPMTRANS_FLAG_NOSCRIPTS = (1 << 2);
        
        const RPMTRANS_FLAG_JUSTDB = (1 << 3);
        const RPMTRANS_FLAG_NOTRIGGERS = (1 << 4);
        const RPMTRANS_FLAG_NODOCS = (1 << 5);
        const RPMTRANS_FLAG_ALLFILES = (1 << 6);
        
        const RPMTRANS_FLAG_NOPLUGINS = (1 << 7);
        const RPMTRANS_FLAG_NOCONTEXTS = (1 << 8);
        const RPMTRANS_FLAG_NOCAPS = (1 << 9);
        const RPMTRANS_FLAG_NOTRIGGERPREIN = (1 << 16);
        
        const RPMTRANS_FLAG_NOPRE = (1 << 17);
        const RPMTRANS_FLAG_NOPOST = (1 << 18);
        const RPMTRANS_FLAG_NOTRIGGERIN = (1 << 19);
        const RPMTRANS_FLAG_NOTRIGGERUN = (1 << 20);
        
        const RPMTRANS_FLAG_NOPREUN = (1 << 21);
        const RPMTRANS_FLAG_NOPOSTUN = (1 << 22);
        const RPMTRANS_FLAG_NOTRIGGERPOSTUN = (1 << 23);
        const RPMTRANS_FLAG_NOPRETRANS = (1 << 24);
        
        const RPMTRANS_FLAG_NOPOSTTRANS = (1 << 25);
        const RPMTRANS_FLAG_NOFILEDIGEST = (1 << 27);
        const RPMTRANS_FLAG_NOCONFIGS = (1 << 30);
        
        const RPMTRANS_FLAG_DEPLOOPS = (1 << 31);
    
    }
}