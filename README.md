# librpm.rs: RPM Package Manager binding for Rust

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![MPL-2.0 Licensed][license-image]][license-link]

The [librpm] C library (available in the `rpm-devel` RPM package) exposes a
programmatic interface to the [RPM Package Manager], and this crate aims to
provide a safe, idiomatic Rust wrapper.

[Documentation](https://rustrpm.org/librpm/)

[librpm]: http://ftp.rpm.org/api/4.14.0/
[RPM Package Manager]: http://rpm.org/

## Status

- [X] Search and query RPM database by tag with exact match, glob, and regex
- [ ] RPM database management: create database, delete database
- [ ] Install and upgrade packages
- [ ] Version comparison support (i.e. dependency sets)
- [ ] RPM reader API (i.e. for `.rpm` files)
- [ ] RPM builder API (i.e. `librpmbuild`)
- [ ] RPM signing API (i.e. `librpmsign`)

## License

Copyright (C) RustRPM Developers

This library is free software.
For more information on free software, see <https://www.gnu.org/philosophy/free-sw.en.html>.

This Source Code Form is subject to the terms of the [Mozilla Public License, v. 2.0].
If a copy of the MPL was not distributed with this file, You can obtain one at <https://mozilla.org/MPL/2.0/>.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/librpm.svg
[crate-link]: https://crates.io/crates/librpm
[build-image]: https://travis-ci.org/rpm-software-management/librpm.rs.svg?branch=master
[build-link]: https://travis-ci.org/rpm-software-management/librpm.rs/
[license-image]: https://img.shields.io/badge/license-MPLv2.0-blue.svg
[license-link]: https://github.com/rpm-software-management/librpm.rs/blob/main/LICENSE

[//]: # (general links)

[Mozilla Public License, v. 2.0]: https://github.com/rpm-software-management/librpm.rs/blob/main/LICENSE
