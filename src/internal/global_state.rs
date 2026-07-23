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

//! Thread-safe tracking of librpm's process-global state.
//!
//! This module provides three synchronization primitives:
//!
//! - **`ConfigState`** — tracks whether `rpmReadConfigFiles` / `rpmInitCrypto`
//!   have been called (must happen exactly once per process).
//!
//! - **`rpm_global_lock()`** — serializes FFI calls that touch the
//!   process-global iterator/database tracking lists (`rpmmiRock`,
//!   `rpmdbRock`, `rpmiiRock`) in RPM <= 4.18. Held briefly during
//!   iterator/ts create and destroy. Harmless no-op on RPM 4.19+.
//!
//! - **`mutation_lock()`** — serializes write operations (`rpmtsRun`,
//!   `rpmtsInitDB`, `rpmtsRebuildDB`) that have process-global side
//!   effects: POSIX `fcntl` lock semantics, chroot state, signal masks,
//!   and SIGPIPE handler. See `docs/threading.md` for details.

use std::sync::{Mutex, MutexGuard, OnceLock};

static CONFIG_STATE: OnceLock<Mutex<ConfigState>> = OnceLock::new();

// Serializes FFI calls that mutate librpm's process-global state: database/
// iterator tracking lists (rpmmiRock, rpmdbRock, rpmiiRock in RPM <= 4.18),
// spec parsing, and package building. The tracking lists are removed in
// RPM 4.19+, but the lock remains necessary for spec/build serialization.
static RPM_GLOBAL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

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

/// Acquire the process-wide lock that serializes librpm FFI calls touching
/// global state (database iterators, spec parsing, package building).
pub fn rpm_global_lock() -> MutexGuard<'static, ()> {
    RPM_GLOBAL_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap()
}

// Serializes write operations that have process-global side effects.
// See the module doc and docs/threading.md for details.
static MUTATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Acquire the process-wide lock that serializes database write operations.
pub fn mutation_lock() -> MutexGuard<'static, ()> {
    MUTATION_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}
