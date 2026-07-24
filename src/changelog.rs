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

//! Changelog information for RPM packages

use std::fmt;
use std::time;

use crate::Tag;
use crate::internal::header::Header;

/// A single changelog entry within an RPM package.
///
/// Borrows name and text strings from the underlying RPM header;
/// valid as long as the originating [`Package`](crate::Package) is alive.
#[derive(Debug, Clone, Copy)]
pub struct ChangelogEntry<'a> {
    timestamp: u32,
    name: &'a str,
    text: &'a str,
}

impl<'a> ChangelogEntry<'a> {
    /// Unix timestamp of the changelog entry as a `SystemTime`.
    pub fn time(&self) -> time::SystemTime {
        time::SystemTime::UNIX_EPOCH + time::Duration::new(self.timestamp as u64, 0)
    }

    /// Raw Unix timestamp of the changelog entry.
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    /// Author of the changelog entry (e.g. `"John Doe <john@example.com> - 1.0-1"`).
    pub fn name(&self) -> &str {
        self.name
    }

    /// Text body of the changelog entry.
    pub fn text(&self) -> &str {
        self.text
    }
}

impl fmt::Display for ChangelogEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "* {} {}\n{}", self.timestamp, self.name, self.text)
    }
}

pub(crate) fn changelogs_from_header<'a>(header: &'a Header) -> Vec<ChangelogEntry<'a>> {
    let times = header
        .get(Tag::CHANGELOGTIME)
        .and_then(|d| d.as_int32_array().map(|s| s.to_vec()));
    let names = header
        .get(Tag::CHANGELOGNAME)
        .and_then(|d| d.as_str_array().map(|s| s.to_vec()));
    let texts = header
        .get(Tag::CHANGELOGTEXT)
        .and_then(|d| d.as_str_array().map(|s| s.to_vec()));

    match (times, names, texts) {
        (Some(times), Some(names), Some(texts)) => times
            .into_iter()
            .zip(names)
            .zip(texts)
            .map(|((timestamp, name), text)| ChangelogEntry {
                timestamp: timestamp as u32,
                name,
                text,
            })
            .collect(),
        _ => Vec::new(),
    }
}
