//! Callbacks for the internal API.
//! This is used to report progress during the transaction.
//! You can use this to report progress during the transaction.

use librpm_sys;

pub(crate) struct CallbackFunction(librpm_sys::rpmCallbackFunction);

impl CallbackFunction {
    pub(crate) unsafe fn from_ptr(ffi_callback: librpm_sys::rpmCallbackFunction) -> Self {
        CallbackFunction(ffi_callback)
    }
}

pub(crate) struct CallbackData(librpm_sys::rpmCallbackData);

impl CallbackData {
    pub(crate) unsafe fn from_ptr(ffi_callback_data: librpm_sys::rpmCallbackData) -> Self {
        CallbackData(ffi_callback_data)
    }
}
