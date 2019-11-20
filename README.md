# librpm.rs: RPM Package Manager binding for Rust

[![Crate][crate-image]][crate-link]
[![Build Status][build-image]][build-link]
[![LGPL v2.1+ Licensed][license-image]][license-link]

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

## How to build for CentOS 7

```
$ docker build -t librpm.rs -f My.Dockerfile . 
$ docker run -it --rm --mount type=bind,src=$PWD,dst=/work --mount type=bind,src=$PWD/.cargo,dst=/root/.cargo  librpm.rs /bin/bash
(docker) $ cd /work
(docker) $ cargo build --target-dir docker-target
```

## License

Copyright (C) 2018-2019 RustRPM Developers

This library is free software; you can redistribute it and/or modify it under
the terms of the GNU Lesser General Public License as published by the Free
Software Foundation; either version 2.1 of the License, or (at your option) any
later version.

This library is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the [GNU Lesser General Public License] for more details.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/librpm.svg
[crate-link]: https://crates.io/crates/librpm
[build-image]: https://travis-ci.com/RustRPM/librpm.rs.svg?branch=master
[build-link]: https://travis-ci.com/RustRPM/librpm.rs/
[license-image]: https://img.shields.io/badge/license-LGPLv2.1+-blue.svg
[license-link]: https://github.com/RustRPM/librpm-rs/blob/master/COPYING

[//]: # (general links)

[GNU Lesser General Public License]: https://github.com/RustRPM/librpm-rs/blob/master/COPYING
