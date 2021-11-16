//! Thread-safe tracking struct for RPM's global mutable state
//!
//! librpm has a lot of global mutable state, and depending on what state it
//! is in various calls are safe (or not).
//!
//! This struct tracks changes to librpm's global state based on functions we
//! have (or have not) invoked.

use super::ts::TransactionSet;
use once_cell::sync::Lazy;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::cell::RefCell;

static RPM_GLOBAL_STATE: Lazy<ReentrantMutex<RefCell<GlobalState>>> =
    Lazy::new(|| ReentrantMutex::new(RefCell::new(GlobalState::default())));

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
    pub fn lock() -> ReentrantMutexGuard<'static, RefCell<Self>> {
        RPM_GLOBAL_STATE.lock()
    }
}
