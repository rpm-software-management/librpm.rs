/*
 * Copyright (C) RustRPM Developers
 *
 * Licensed under the Mozilla Public License Version 2.0
 * Fedora-License-Identifier: MPLv2.0
 * SPDX-2.0-License-Identifier: MPL-2.0
 * SPDX-3.0-License-Identifier: MPL-2.0
 *
 * This is free software.
 * For more information on the license, see LICENSE.
 * For more information on free software, see <https://www.gnu.org/philosophy/free-sw.en.html>.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at <https://mozilla.org/MPL/2.0/>.
 */

//! RPM macros are configuration parameters that have largely replaced the
//! previous rpmrc system.

use crate::error::{Error, ErrorKind};
use crate::internal::GlobalState;
use librpm_sys;
use std::ffi::{CStr, CString};

/// Scopes in which macros are defined
pub struct MacroContext(librpm_sys::rpmMacroContext);

/// Obtain the default global context
impl Default for MacroContext {
    fn default() -> MacroContext {
        // Safety: rpmGlobalMacroContext is a process-wide static pointer
        // provided by librpm. It is valid for the lifetime of the process.
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
            CString::new(macro_string).map_err(|e| format_err!(ErrorKind::InvalidArg, "{}", e))?;

        // Aggressively synchronize all macro operations through the global
        // lock. This serializes even non-global context operations, but these
        // are not hot paths and correctness is more important here.
        let _lock = GlobalState::lock();
        // Safety: cstr is a valid null-terminated C string, and self.0 is a
        // valid rpmMacroContext obtained from librpm. The global lock ensures
        // exclusive access to librpm's macro state.
        unsafe {
            librpm_sys::rpmDefineMacro(self.0, cstr.as_ptr(), level as i32);
        }

        Ok(())
    }

    /// Delete a macro from this context.
    pub fn pop(&self, name: &str) -> Result<(), Error> {
        let cstr = CString::new(name).map_err(|e| format_err!(ErrorKind::InvalidArg, "{}", e))?;

        let _lock = GlobalState::lock();
        // Safety: cstr is a valid null-terminated C string, and self.0 is a
        // valid rpmMacroContext. The global lock ensures exclusive access.
        unsafe {
            librpm_sys::rpmPopMacro(self.0, cstr.as_ptr());
        }

        Ok(())
    }

    /// Expand a macro expression and return the result as a string.
    ///
    /// The expression is expanded using this context's macro definitions.
    /// Macro references use `%{name}` syntax.
    ///
    /// ```no_run
    /// use librpm::MacroContext;
    ///
    /// librpm::init();
    /// let ctx = MacroContext::default();
    /// let arch = ctx.expand("%{_arch}").unwrap();
    /// println!("build arch: {arch}");
    /// ```
    pub fn expand(&self, expr: &str) -> Result<String, Error> {
        let cstr = CString::new(expr).map_err(|e| format_err!(ErrorKind::InvalidArg, "{}", e))?;

        let _lock = GlobalState::lock();
        let mut obuf: *mut std::os::raw::c_char = std::ptr::null_mut();
        let rc = unsafe { librpm_sys::rpmExpandMacros(self.0, cstr.as_ptr(), &mut obuf, 0) };

        if rc < 0 || obuf.is_null() {
            if !obuf.is_null() {
                unsafe { free(obuf.cast()) };
            }
            fail!(ErrorKind::Macro, "macro expansion failed: {}", expr);
        }

        let result = unsafe { CStr::from_ptr(obuf) }
            .to_string_lossy()
            .into_owned();
        unsafe { free(obuf.cast()) };
        Ok(result)
    }

    /// Test whether a macro is defined in this context.
    ///
    /// ```no_run
    /// use librpm::MacroContext;
    ///
    /// librpm::init();
    /// let ctx = MacroContext::default();
    /// assert!(ctx.is_defined("_arch"));
    /// ```
    pub fn is_defined(&self, name: &str) -> bool {
        let Ok(cstr) = CString::new(name) else {
            return false;
        };

        let _lock = GlobalState::lock();
        unsafe { librpm_sys::rpmMacroIsDefined(self.0, cstr.as_ptr()) != 0 }
    }
}

/// Expand a macro expression and return its numeric value.
///
/// Uses the global macro context. Boolean values (`Y`/`y` return 1,
/// `N`/`n` return 0) are accepted. An undefined or non-numeric macro
/// returns 0.
///
/// ```no_run
/// librpm::init();
/// let uid = librpm::macro_context::expand_numeric("%{_root_uid}");
/// ```
pub fn expand_numeric(expr: &str) -> i32 {
    let Ok(cstr) = CString::new(expr) else {
        return 0;
    };

    let _lock = GlobalState::lock();
    unsafe { librpm_sys::rpmExpandNumeric(cstr.as_ptr()) }
}

unsafe extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}
