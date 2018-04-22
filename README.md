# librpm.rs: RPM Package Manager binding for Rust

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![LGPL v2.1+ Licensed][license-image]][license-link]

[crate-image]: https://img.shields.io/crates/v/librpm.svg
[crate-link]: https://crates.io/crates/librpm
[build-image]: https://circleci.com/gh/tarcieri/librpm-rs.svg?style=shield
[build-link]: https://circleci.com/gh/tarcieri/librpm-rs
[license-image]: https://img.shields.io/badge/license-LGPLv2.1+-blue.svg
[license-link]: https://github.com/tarcieri/librpm-rs/blob/master/LICENSE

The [librpm] C library (available in the `rpm-devel` RPM package) exposes a
programmatic interface to the [RPM Package Manager], and this crate aims to
provide a safe, idiomatic Rust wrapper.

[Documentation](https://librpm.rs/librpm/)

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

Copyright (C) 2018 librpm.rs developers

This library is free software; you can redistribute it and/or modify it under
the terms of the GNU Lesser General Public License as published by the Free
Software Foundation; either version 2.1 of the License, or (at your option) any
later version.

This library is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the [GNU Lesser General Public License] for more details.

[GNU Lesser General Public License][COPYING]
