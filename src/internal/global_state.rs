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


//! Thread-safe tracking struct for RPM's global mutable state
//!
//! librpm has a lot of global mutable state, and depending on what state it
//! is in various calls are safe (or not).
//!
//! This struct tracks changes to librpm's global state based on functions we
//! have (or have not) invoked.

use super::ts::TransactionSet;
use once_cell::sync::Lazy;
use std::sync::{Mutex, MutexGuard};

static RPM_GLOBAL_STATE: Lazy<Mutex<GlobalState>> =
    Lazy::new(|| Mutex::new(GlobalState::default()));

/// Tracking struct for mutable global state in RPM
pub(crate) struct GlobalState {
    /// Have any configuration functions been called? (Specifically any ones
    /// which invoke `rpmInitCrypto`, which it seems should only be called once)
    pub configured: bool,

    /// Global shared transaction set created the first time librpm's global
    /// state is accessed.
    pub ts: TransactionSet,
}

impl Default for GlobalState {
    fn default() -> GlobalState {
        GlobalState {
            configured: false,
            ts: TransactionSet::create(),
        }
    }
}

impl GlobalState {
    /// Obtain an exclusive lock to the global state
    pub fn lock() -> MutexGuard<'static, Self> {
        RPM_GLOBAL_STATE.lock().unwrap()
    }
}
