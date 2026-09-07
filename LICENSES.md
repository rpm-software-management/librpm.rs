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

* `librpm-sys`: LGPL-2.0-or-later, for bindings derived from `lib` and
  `rpmio`, together with MPL-2.0 for repository-authored code.
* `librpmbuild-sys`: GPL-2.0-or-later, for bindings derived from `rpmbuild`,
  together with MPL-2.0 for repository-authored code.
* `librpmsign-sys`: GPL-2.0-or-later, for bindings derived from `rpmsign`,
  together with MPL-2.0 for repository-authored code.

The `librpm` crate's `macros` feature is opt-in. Enabling it includes bindings
derived from RPM's GPL-2.0-or-later `rpmmacro.h`; distributions using that
feature must account for that GPL-covered component.

The RPM `COPYING` file should be included when distributing artifacts that
include these RPM-derived bindings.
