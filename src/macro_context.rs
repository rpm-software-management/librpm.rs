//! RPM macros are configuration parameters that have largely replaced the
//! previous rpmrc system.

use crate::error::{Error, ErrorKind};
use librpm_sys;
use std::ffi::CString;

/// Scopes in which macros are defined
pub struct MacroContext(librpm_sys::rpmMacroContext);

/// Obtain the default global context
impl Default for MacroContext {
    fn default() -> MacroContext {
        unsafe { MacroContext(librpm_sys::rpmGlobalMacroContext) }
    }
}

impl MacroContext {
    /// Define a macro in this context. Macros take the form:
    ///
    /// `<name>[(opts)] <body>`
    ///
    /// Level defines the macro recursion level (0 is the entry API)
    pub fn define(&self, macro_string: &str, level: isize) -> Result<(), Error> {
        let cstr =
            CString::new(macro_string).map_err(|e| format_err!(ErrorKind::Config, "{}", e))?;

        unsafe {
            librpm_sys::rpmDefineMacro(self.0, cstr.as_ptr(), level as i32);
        }

        Ok(())
    }

    #[cfg(feature = "librpm-4-14")]
    /// Delete a macro from this context.
    pub fn pop(&self, name: &str) -> Result<(), Error> {
        let cstr = CString::new(name).unwrap();

        unsafe {
            librpm_sys::rpmPopMacro(self.0, cstr.as_ptr());
        }

        Ok(())
    }

    #[cfg(not(feature = "librpm-4-14"))]
    /// Delete a macro from this context.
    pub fn delete(&self, name: &str) -> Result<(), Error> {
        let cstr = CString::new(name).unwrap();

        unsafe {
            librpm_sys::delMacro(self.0, cstr.as_ptr());
        }

        Ok(())
    }
}
