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

//! Bridge between librpm's C logging system and Rust's `log` crate.
//!
//! By default, librpm logs to stderr using its native logging. Call
//! [`set_behavior`] with [`LogBehavior::LogCrate`] to route messages
//! through Rust's [`log`] facade instead.
//!
//! This module also provides [`set_verbosity`] and [`last_message`]
//! for direct access to librpm's log state.

use std::ffi::CStr;
use std::ptr;

/// librpm log verbosity levels.
///
/// These correspond to the `rpmlogLvl_e` values in `rpmlog.h` and control
/// which messages librpm will emit. Use with [`set_verbosity`] to set the
/// maximum verbosity level.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u32)]
pub enum LogLevel {
    /// System is unusable
    Emergency = librpm_sys::rpmlogLvl_e_RPMLOG_EMERG,
    /// Action must be taken immediately
    Alert = librpm_sys::rpmlogLvl_e_RPMLOG_ALERT,
    /// Critical conditions
    Critical = librpm_sys::rpmlogLvl_e_RPMLOG_CRIT,
    /// Error conditions
    Error = librpm_sys::rpmlogLvl_e_RPMLOG_ERR,
    /// Warning conditions
    Warning = librpm_sys::rpmlogLvl_e_RPMLOG_WARNING,
    /// Normal but significant condition
    Notice = librpm_sys::rpmlogLvl_e_RPMLOG_NOTICE,
    /// Informational
    Info = librpm_sys::rpmlogLvl_e_RPMLOG_INFO,
    /// Debug-level messages
    Debug = librpm_sys::rpmlogLvl_e_RPMLOG_DEBUG,
}

/// Controls how librpm log messages are dispatched.
///
/// Use with [`set_behavior`] to change the logging behavior.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum LogBehavior {
    /// Route messages through Rust's `log` crate.
    /// librpm's default stderr output is suppressed.
    LogCrate,

    /// Use librpm's native logging (stderr). This is the default.
    Default,
}

#[allow(non_upper_case_globals)]
fn to_log_level(pri: librpm_sys::rpmlogLvl_e) -> log::Level {
    match pri {
        librpm_sys::rpmlogLvl_e_RPMLOG_EMERG
        | librpm_sys::rpmlogLvl_e_RPMLOG_ALERT
        | librpm_sys::rpmlogLvl_e_RPMLOG_CRIT
        | librpm_sys::rpmlogLvl_e_RPMLOG_ERR => log::Level::Error,
        librpm_sys::rpmlogLvl_e_RPMLOG_WARNING => log::Level::Warn,
        librpm_sys::rpmlogLvl_e_RPMLOG_NOTICE | librpm_sys::rpmlogLvl_e_RPMLOG_INFO => {
            log::Level::Info
        }
        librpm_sys::rpmlogLvl_e_RPMLOG_DEBUG => log::Level::Debug,
        _ => log::Level::Trace,
    }
}

unsafe extern "C" fn rpm_log_callback(
    rec: librpm_sys::rpmlogRec,
    _data: librpm_sys::rpmlogCallbackData,
) -> std::os::raw::c_int {
    let pri = unsafe { librpm_sys::rpmlogRecPriority(rec) };
    let msg_ptr = unsafe { librpm_sys::rpmlogRecMessage(rec) };

    if !msg_ptr.is_null() {
        let level = to_log_level(pri);
        let msg = unsafe { CStr::from_ptr(msg_ptr) }.to_string_lossy();
        let msg = msg.trim_end();
        log::log!(target: "librpm", level, "{}", msg);
    }

    0
}

/// Set the maximum verbosity level for librpm's internal logging.
///
/// Messages at `level` and above (more severe) will be emitted; messages
/// below `level` are suppressed at the C level before reaching either
/// the `log` callback or librpm's default output.
///
/// # Example
///
/// ```no_run
/// librpm::init().unwrap();
/// // Only emit warnings and errors from librpm
/// librpm::logging::set_verbosity(librpm::logging::LogLevel::Warning);
/// ```
pub fn set_verbosity(level: LogLevel) {
    // RPMLOG_UPTO(pri) = (1 << (pri + 1)) - 1
    let mask = (1i32 << (level as u32 + 1)) - 1;
    unsafe {
        librpm_sys::rpmlogSetMask(mask);
    }
}

/// Set the logging behavior.
///
/// Controls whether librpm log messages are forwarded through Rust's `log`
/// crate or sent to librpm's default output (typically stderr).
///
/// This also adjusts the log mask: [`LogBehavior::LogCrate`] passes all
/// levels through (letting the Rust `log` framework filter), while
/// [`LogBehavior::Default`] restores librpm's default mask (notice and
/// above). A subsequent call to [`set_verbosity`] overrides either default.
///
/// The initial behavior is [`LogBehavior::Default`].
///
/// # Example
///
/// ```no_run
/// librpm::init().unwrap();
/// // Route librpm messages through Rust's log crate
/// librpm::logging::set_behavior(librpm::logging::LogBehavior::LogCrate);
/// ```
pub fn set_behavior(behavior: LogBehavior) {
    unsafe {
        match behavior {
            LogBehavior::LogCrate => {
                // Pass all levels through; let the Rust log framework filter
                librpm_sys::rpmlogSetMask(
                    ((1u32 << (librpm_sys::rpmlogLvl_e_RPMLOG_DEBUG + 1)) - 1) as i32,
                );
                librpm_sys::rpmlogSetCallback(Some(rpm_log_callback), ptr::null_mut());
            }
            LogBehavior::Default => {
                librpm_sys::rpmlogSetCallback(None, ptr::null_mut());
                // Restore librpm's default: RPMLOG_UPTO(RPMLOG_NOTICE)
                librpm_sys::rpmlogSetMask(
                    ((1u32 << (librpm_sys::rpmlogLvl_e_RPMLOG_NOTICE + 1)) - 1) as i32,
                );
            }
        }
    }
}

/// Return the text of the last librpm log message, if any.
///
/// This is useful for retrieving details after an operation fails, since
/// some librpm errors only report specifics through the log system rather
/// than through return codes.
///
/// # Example
///
/// ```no_run
/// librpm::init().unwrap();
/// if let Some(msg) = librpm::logging::last_message() {
///     eprintln!("last librpm message: {msg}");
/// }
/// ```
pub fn last_message() -> Option<String> {
    let ptr = unsafe { librpm_sys::rpmlogMessage() };
    if ptr.is_null() {
        return None;
    }
    let msg = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .trim_end()
        .to_string();
    if msg.is_empty() { None } else { Some(msg) }
}
