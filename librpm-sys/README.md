# librpm-sys: bindgen wrapper for RPM Package Manager C library

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![LGPL v2.1+ Licensed][license-image]][license-link]

This crate uses bindgen to generate an unsafe FFI wrapper for the
[librpm C library], which provides a low-level API for interacting with the
[RPM Package Manager] and **.rpm** files.

This crate isn't intended to be used directly, but instead provides an unsafe,
low-level binding used by the higher level **librpm** crate, which aims to
provide a safe, idiomatic, high-level binding to the C library:

https://rustrpm.org/

If you're intending to add a feature to the **librpm** crate however, you have
come to the right place. You can find documentation here:

[Documentation]

## License

Copyright (C) 2018-2019 RustRPM Developers

This library is free software; you can redistribute it and/or modify it under
the terms of the GNU Lesser General Public License as published by the Free
Software Foundation; either version 2.1 of the License, or (at your option) any
later version.

This library is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU Lesser General Public License for more details.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/librpm-sys.svg
[crate-link]: https://crates.io/crates/librpm-sys
[build-image]: https://travis-ci.org/rpm-software-management/librpm.rs.svg?branch=master
[build-link]: https://travis-ci.org/rpm-software-management/librpm.rs/
[license-image]: https://img.shields.io/badge/license-LGPLv2.1+-blue.svg
[license-link]: https://github.com/iqlusion-io/crates/blob/master/LICENSE

[//]: # (general links)

[Documentation]: https://rustrpm.org/librpm-sys/
[librpm C library]: http://ftp.rpm.org/api/4.14.0/
[RPM Package Manager]: http://rpm.org/
