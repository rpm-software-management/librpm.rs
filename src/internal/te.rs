//! Transaction Elements

use super::GlobalState;
use std::sync::atomic::AtomicPtr;
use std::sync::MutexGuard;
use bitflags::bitflags;

pub(crate) struct TransactionElement(AtomicPtr<librpm_sys::rpmte_s>);

impl TransactionElement {
    pub(crate) unsafe fn from_ptr(ffi_te: librpm_sys::rpmte) -> Self {
        assert!(!ffi_te.is_null());
        TransactionElement(AtomicPtr::from(ffi_te))
    }

    pub(crate) fn header(&mut self) {
        unsafe {
            librpm_sys::rpmteHeader(*self.0.get_mut());
        }
    }

    pub(crate) fn set_header(&mut self, header: *mut librpm_sys::headerToken_s) {
        unsafe {
            librpm_sys::rpmteSetHeader(*self.0.get_mut(), header);
        }
    }
    pub(crate) fn element_type(&mut self) -> ElementTypes {
        let e = unsafe { librpm_sys::rpmteType(*self.0.get_mut()) };

        ElementTypes::from_bits_truncate(e)
    }
    // NEVRAO of the package
    pub(crate) fn name(&mut self) -> String {
        unsafe {
            librpm_sys::rpmteN(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }
    pub(crate) fn epoch(&mut self) -> String {
        unsafe {
            librpm_sys::rpmteE(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }
    pub(crate) fn version(&mut self) -> String {
        unsafe {
            librpm_sys::rpmteV(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }
    pub(crate) fn release(&mut self) -> String {
        unsafe {
            librpm_sys::rpmteR(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }
    pub(crate) fn arch(&mut self) -> String {
        unsafe {
            librpm_sys::rpmteA(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }
    pub(crate) fn os(&mut self) -> String {
        unsafe {
            librpm_sys::rpmteO(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }

    pub(crate) fn is_source(&mut self) -> i32 {
        unsafe { librpm_sys::rpmteIsSource(*self.0.get_mut()) }
    }

    pub(crate) fn color(&mut self) -> librpm_sys::rpm_color_t {
        unsafe { librpm_sys::rpmteColor(*self.0.get_mut()) }
    }

    pub(crate) fn set_color(&mut self, color: librpm_sys::rpm_color_t) {
        unsafe {
            librpm_sys::rpmteSetColor(*self.0.get_mut(), color);
        }
    }

    pub(crate) fn set_db_instance(&mut self, db_instance: u32) {
        unsafe {
            librpm_sys::rpmteSetDBInstance(*self.0.get_mut(), db_instance);
        }
    }

    pub(crate) fn db_instance(&mut self) -> u32 {
        unsafe { librpm_sys::rpmteDBInstance(*self.0.get_mut()) }
    }

    pub(crate) fn parent(&mut self) -> TransactionElement {
        unsafe { TransactionElement(AtomicPtr::new(librpm_sys::rpmteParent(*self.0.get_mut()))) }
    }
    pub(crate) fn set_parent(&mut self, parent: *mut librpm_sys::rpmte_s) {
        unsafe {
            librpm_sys::rpmteSetParent(*self.0.get_mut(), parent);
        }
    }
    pub(crate) fn problems(&mut self) -> librpm_sys::rpmps_s {
        unsafe {
            librpm_sys::rpmteProblems(*self.0.get_mut())
                .as_ref()
                .unwrap()
                .to_owned()
        }
    }

    pub(crate) fn clean_problems(&mut self) {
        unsafe {
            librpm_sys::rpmteCleanProblems(*self.0.get_mut());
        }
    }

    pub(crate) fn clean_ds(&mut self) {
        unsafe {
            librpm_sys::rpmteCleanDS(*self.0.get_mut());
        }
    }

    pub(crate) fn set_dependson(&mut self, depends: &mut TransactionElement) {
        unsafe {
            librpm_sys::rpmteSetDependsOn(*self.0.get_mut(), *depends.0.get_mut());
        }
    }
    pub(crate) fn dependson(&mut self) -> TransactionElement {
        unsafe {
            TransactionElement(AtomicPtr::new(librpm_sys::rpmteDependsOn(
                *self.0.get_mut(),
            )))
        }
    }
    pub(crate) fn evr(&mut self) -> String {
        unsafe {
            librpm_sys::rpmteEVR(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }
    pub(crate) fn nevra(&mut self) -> String {
        unsafe {
            librpm_sys::rpmteNEVRA(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }
    pub(crate) fn nevr(&mut self) -> String {
        unsafe {
            librpm_sys::rpmteNEVR(*self.0.get_mut())
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
    }
}

bitflags! {
    pub(crate) struct ElementTypes: u32 {
        const ADDED = librpm_sys::rpmElementType_e_TR_ADDED;
        const REMOVED = librpm_sys::rpmElementType_e_TR_REMOVED;
        const RPMDB = librpm_sys::rpmElementType_e_TR_RPMDB;
    }
}
