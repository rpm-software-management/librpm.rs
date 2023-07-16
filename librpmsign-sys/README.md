# librpmsign-sys: bindgen wrapper for rpmsign C library

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![MPL-2.0 Licensed][license-image]][license-link]

This crate uses bindgen to generate an unsafe FFI wrapper for the
[rpmsign C library], which provides a low-level API for signing
**.rpm** files.

This crate isn't intended to be used directly, but instead provides an unsafe,
low-level binding used by the higher level **librpm** crate, which aims to
provide a safe, idiomatic, high-level binding to the C library:

https://rustrpm.org/

## License

Copyright (C) RustRPM Developers

This library is free software.
For more information on free software, see <https://www.gnu.org/philosophy/free-sw.en.html>.

This Source Code Form is subject to the terms of the [Mozilla Public License, v. 2.0].
If a copy of the MPL was not distributed with this file, You can obtain one at <https://mozilla.org/MPL/2.0/>.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/librpmsign-sys.svg
[crate-link]: https://crates.io/crates/librpmsign-sys
[build-image]: https://github.com/rpm-software-management/librpm.rs/actions/workflows/ci.yml/badge.svg?branch=main
[build-link]: https://github.com/rpm-software-management/librpm.rs/actions
[license-image]: https://img.shields.io/badge/license-MPLv2.0-blue.svg
[license-link]: https://github.com/rpm-software-management/librpm.rs/blob/main/LICENSE

[//]: # (general links)

[rpmsign C library]: http://ftp.rpm.org/api/4.14.0/group__rpmsign.html
[Mozilla Public License, v. 2.0]: https://github.com/rpm-software-management/librpm.rs/blob/main/LICENSE
