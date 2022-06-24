//! Transaction sets: librpm's transaction API

use librpm_sys::rpmtsiNext;

use crate::db::Iter;

use super::GlobalState;
use super::te::{TransactionElement, ElementType};
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

    pub(crate) fn iter(&mut self, flags: Vec<ElementType>) -> TransactionSetIterator {
        let iterator = unsafe { librpm_sys::rpmtsiInit(*self.0.get_mut()) };

        unsafe { TransactionSetIterator::from_ptr(iterator, flags) }
    }
}

pub(crate) struct TransactionSetIterator{
    ptr: AtomicPtr<librpm_sys::rpmtsi_s>,
    flags: Vec<ElementType>,
    exhausted: bool
}

impl TransactionSetIterator {
    pub(crate) unsafe fn from_ptr(ffi_tsi: librpm_sys::rpmtsi, flags: Vec<ElementType>) -> Self {
        assert!(!ffi_tsi.is_null());

        TransactionSetIterator {
            ptr: AtomicPtr::from(ffi_tsi),
            flags,
            exhausted: false
        }
    }

    fn get_bit_flags(&mut self) -> u32 {
        let mut bitflags = 0u32;

        for &flag in &self.flags {
            bitflags |= flag as u32;
        }

        bitflags
    }
}

impl Iterator for TransactionSetIterator {
    type Item = TransactionElement;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted { return None };

        let element = unsafe { librpm_sys::rpmtsiNext(*self.ptr.get_mut(), self.get_bit_flags()) };
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
