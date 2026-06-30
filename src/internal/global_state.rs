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

//! Thread-safe tracking of librpm's process-global configuration state.
//!
//! librpm's `rpmReadConfigFiles` / `rpmInitCrypto` must only be called once
//! per process. This module tracks whether that has happened and serializes
//! macro operations that mutate the global macro context.

use std::sync::{Mutex, MutexGuard, OnceLock};

static CONFIG_STATE: OnceLock<Mutex<ConfigState>> = OnceLock::new();

/// Tracking struct for process-global configuration in RPM
pub(crate) struct ConfigState {
    /// Have any configuration functions been called? (Specifically any ones
    /// which invoke `rpmInitCrypto`, which it seems should only be called once)
    pub configured: bool,
}

impl ConfigState {
    /// Obtain an exclusive lock to the config state
    pub fn lock() -> MutexGuard<'static, Self> {
        CONFIG_STATE
            .get_or_init(|| Mutex::new(ConfigState { configured: false }))
            .lock()
            .unwrap()
    }
}
