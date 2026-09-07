# License information

The Rust code authored in this repository is licensed under the Mozilla
Public License, version 2.0, as stated in `LICENSE`.

RPM is covered by the license notice in its `COPYING` file. RPM's entire code
base is GPL-2.0-or-later, with an alternative LGPL-2.0-or-later license for
code in its `lib` and `rpmio` directories and code derived from that code.
The authoritative text is available in the RPM source distribution:

<https://github.com/rpm-software-management/rpm/blob/master/COPYING>

Accordingly, the package metadata identifies the RPM-derived portions of the
low-level crates as follows:

* `librpm-sys`: MPL-2.0 for repository-authored code; it links against the
  LGPL-2.0-or-later `librpm` and `librpmio` libraries.
* `librpmsign-sys`: MPL-2.0 for repository-authored code; it links against
  the LGPL-2.0-or-later `librpmsign` library.
* `librpmbuild-sys`: MPL-2.0 for repository-authored code and
  GPL-2.0-or-later for the linked `librpmbuild` component.

The RPM `COPYING` file should be included when distributing artifacts that
include these RPM-derived bindings.

Linking against an LGPL library does not relicense a sys crate. The
`librpmbuild-sys` feature is separately documented as GPL-covered because the
resulting combination with `librpmbuild` must comply with GPL-2.0-or-later.
