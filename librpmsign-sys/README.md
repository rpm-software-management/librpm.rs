# librpmsign-sys: bindgen wrapper for rpmsign C library

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![LGPL v2.1+ Licensed][license-image]][license-link]

[crate-image]: https://img.shields.io/crates/v/librpmsign-sys.svg
[crate-link]: https://crates.io/crates/librpmsign-sys
[build-image]: https://circleci.com/gh/iqlusion-io/crates.svg?style=shield
[build-link]: https://circleci.com/gh/iqlusion-io/crates
[license-image]: https://img.shields.io/badge/license-LGPLv2.1+-blue.svg
[license-link]: https://github.com/iqlusion-io/crates/blob/master/LICENSE

This crate uses bindgen to generate an unsafe FFI wrapper for the
[rpmsign C library], which provides a low-level API for signing
**.rpm** files.

This crate isn't intended to be used directly, but instead provides an unsafe,
low-level binding used by the higher level **librpm** crate, which aims to
provide a safe, idiomatic, high-level binding to the C library:

https://librpm.rs/

[rpmsign C library]: http://ftp.rpm.org/api/4.14.0/group__rpmsign.html

## License

Copyright (C) 2018 librpm.rs developers

This library is free software; you can redistribute it and/or modify it under
the terms of the GNU Lesser General Public License as published by the Free
Software Foundation; either version 2.1 of the License, or (at your option) any
later version.

This library is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU Lesser General Public License for more details.
