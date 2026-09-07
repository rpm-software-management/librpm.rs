# librpmbuild-sys: bindgen wrapper for librpmbuild C library

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![MPL-2.0 AND GPL-2.0-or-later Licensed][license-image]][license-link]

This crate uses bindgen to generate an unsafe FFI wrapper for the
[rpmbuild C library], which provides a low-level API for building **.rpm**
files for use with the [RPM Package Manager].

This crate isn't intended to be used directly, but instead provides an unsafe,
low-level binding used by the higher level **librpm** crate, which aims to
provide a safe, idiomatic, high-level binding to the C library:

https://rustrpm.org/

## License

This crate is licensed under MPL-2.0 for its authored Rust code and
GPL-2.0-or-later for bindings derived from RPM's `rpmbuild` headers. See
[LICENSES.md](../LICENSES.md).

Copyright (C) RustRPM Developers

This library is free software.
For more information on free software, see <https://www.gnu.org/philosophy/free-sw.en.html>.

Repository-authored source is subject to the Mozilla Public License, v. 2.0.
RPM-derived bindings are subject to the RPM license terms described in
[LICENSES.md](../LICENSES.md).

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/librpmbuild-sys.svg
[crate-link]: https://crates.io/crates/librpmbuild-sys
[build-image]: https://github.com/rpm-software-management/librpm.rs/actions/workflows/ci.yml/badge.svg?branch=main
[build-link]: https://github.com/rpm-software-management/librpm.rs/actions
[license-image]: https://img.shields.io/badge/license-MPL--2.0%20AND%20GPL--2.0--or--later-blue.svg
[license-link]: https://github.com/rpm-software-management/librpm.rs/blob/main/LICENSE

[//]: # (general links)

[rpmbuild C library]: http://ftp.rpm.org/api/4.14.0/group__rpmbuild.html
[RPM Package Manager]: http://rpm.org/
[Mozilla Public License, v. 2.0]: https://github.com/rpm-software-management/librpm.rs/blob/main/LICENSE
