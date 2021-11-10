//! Internal functionality not exposed outside of this crate
//!
//! We hide the guts of how we interact with librpm until we're sure it's safe to expose

pub(crate) mod global_state;
pub(crate) mod header;
pub(crate) mod iterator;
pub(crate) mod tag;
pub(crate) mod td;
pub(crate) mod ts;
pub(crate) mod archive;

pub(crate) use self::global_state::GlobalState;
