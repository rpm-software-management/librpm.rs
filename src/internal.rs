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


//! Internal functionality not exposed outside of this crate
//!
//! We hide the guts of how we interact with librpm until we're sure it's safe to expose

pub(crate) mod global_state;
pub(crate) mod header;
pub(crate) mod iterator;
pub(crate) mod tag;
pub(crate) mod td;
pub(crate) mod ts;

pub(crate) use self::global_state::GlobalState;
