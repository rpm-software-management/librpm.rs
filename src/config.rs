//! Support for configuring RPM, i.e. reading configuration files

use crate::error::{Error, ErrorKind};
use crate::internal::GlobalState;
use crate::macro_context::MacroContext;
use librpm_sys;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

/// Name of the macro which defines the path to the database
const DB_PATH_MACRO: &str = "_dbpath";

/// Read RPM configuration (a.k.a. rpmrc)
///
/// If `None` is passed, the default configuration will be used.
///
/// Configuration is global to the process.
pub fn read_file(config_file: Option<&Path>) -> Result<(), Error> {
    let lock = GlobalState::lock();
    let mut global_state = lock.borrow_mut();

    // Avoid invoking `rpmReadConfigFiles` more than once. This vicariously
    // invokes `rpmInitCrypto` which causes segfaults (NULL struct pointer
    // derefs) if invoked more than once.
    if global_state.configured {
        fail!(ErrorKind::Config, "already configured");
    }

    global_state.configured = true;

    let rc = match config_file {
        Some(path) => {
            if !path.exists() {
                fail!(ErrorKind::Config, "no such file: {}", path.display())
            }

            let cstr = CString::new(path.as_os_str().as_bytes()).map_err(|e| {
                format_err!(
                    ErrorKind::Config,
                    "invalid path: {} ({})",
                    path.display(),
                    e
                )
            })?;

            unsafe { librpm_sys::rpmReadConfigFiles(cstr.as_ptr(), ptr::null()) }
        }
        None => unsafe { librpm_sys::rpmReadConfigFiles(ptr::null(), ptr::null()) },
    };

    if rc != 0 {
        match config_file {
            Some(path) => fail!(
                ErrorKind::Config,
                "error reading RPM config from: {}",
                path.display()
            ),
            None => fail!(
                ErrorKind::Config,
                "error reading RPM config from default location"
            ),
        }
    }

    Ok(())
}

/// Set the path to the global RPM database.
pub fn set_db_path(path: &Path) -> Result<(), Error> {
    MacroContext::default().define(&format!("{} {}", DB_PATH_MACRO, path.display()), 0)
}
