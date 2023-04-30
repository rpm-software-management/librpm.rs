use std::sync::atomic::AtomicPtr;


pub(crate) struct Transaction(AtomicPtr<librpm_sys::rpmtxn_s>);

impl Transaction {
    pub(crate) unsafe fn from_ptr(ffi_txn: librpm_sys::rpmtxn) -> Self {
        assert!(!ffi_txn.is_null());

        Transaction(AtomicPtr::from(ffi_txn))
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
      unsafe { librpm_sys::rpmtxnEnd(*self.0.get_mut()) };
    }
}