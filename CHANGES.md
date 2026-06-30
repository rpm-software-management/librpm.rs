# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

* `Package::from_file()` reads an `.rpm` file directly into a `Package`
* `Package::get()` exposes raw tag data access via `Tag` and `TagData`,
  both of which are now part of the public API
* `Package::files()` returns a `Files` set with per-file metadata (path, size,
  mode, owner, group, digest, flags, link target, capabilities) via the
  `rpmfiles` API. Includes `FileEntry` accessors and `FileAttrs` flag helpers
  (`is_config`, `is_doc`, `is_ghost`, `is_license`, etc.)
* `Package::requires()`, `provides()`, `conflicts()`, `obsoletes()`,
  `recommends()`, `suggests()`, `supplements()`, `enhances()` return a
  `Dependencies` set built from librpm's `rpmds` API. Each `Dependency`
  exposes `name()`, `evr()`, and `flags()` (`DepFlags` with version-comparison
  helpers and sense-flag checks)
* Version comparison utilities - `version::vercmp()` compares two version/release
  strings, `version::Version` parses `[epoch:]version[-release]` strings and
  supports full Rust-native `Ord`/`Eq` comparison via librpm's `rpmverCmp`
* Macro expansion utilities - `MacroContext::expand()` expands `%{name}` expressions,
  `MacroContext::is_defined()` tests for macro existence, and
  `macro_context::expand_numeric()` returns macro values as integers
* `sign` feature: package signing and signature removal via `librpmsign` -
  `sign_package()`, `del_sign()`, `del_file_sign()` with builder-style `SignArgs`

### Changed

* Brought the exposed tag constants up to date with RPM 6.0
* Auto-detect available tag constants at build time for compatibility with older librpm versions
* `Package` is now a thin wrapper around librpm's refcounted header instead
  of eagerly copying all tag values into owned `String` fields. Accessors
  perform tag lookups on demand. `PartialEq` and `Hash` compare by NEVRA.
* **Breaking:** Database query functions (`Index::find()`, `installed_packages()`,
  `db::find()`) are now methods on a new `Db` handle obtained via `Db::open()`.
  Configuration is done via top-level `librpm::init()` / `librpm::init_with()`.
  This ensures at compile time that RPM configuration has been loaded before
  any database queries are made ([#13]).
* Removed `AtomicPtr` in `TransactionSet`, making `TransactionSet` `Send`-only rather than
  `Send + Sync`

### Fixed

* Fixed deadlock when creating multiple database iterators from the same thread ([#15]).
* Fixed undefined behavior in `TagData::char()`: C `char` (1 byte) was
  incorrectly cast to Rust `char` (4 bytes), causing out-of-bounds reads.
  The `Char` variant now holds `u8`.
* Fixed memory leak: call `rpmtdFreeData` in `Header::get()` to free
  container data allocated by `headerGet` (e.g. STRING_ARRAY pointer tables)
* Synchronized `MacroContext::define()`, `pop()`, and `delete()` through
  the global state lock to prevent data races on RPM's macro table
* Fixed build failure on some Fedora/glibc versions caused by `struct timex`
  workaround ([#48]). Switched bindgen to allowlist mode so only the functions
  and types actually used are generated.
* Changed integer `TagData` variants (`Char`, `Int8`, `Int16`, `Int32`, `Int64`)
  from scalars to slices, correctly representing RPM tags that contain arrays
  of values (e.g. `FILESIZES`, `FILEMODES`, `REQUIREFLAGS`). The RPM header format
  stores scalars as arrays of length 1.
* Fixed `config::read_file()` marking state as configured before
  `rpmReadConfigFiles` succeeds, which prevented retry on failure

## 0.1.1 (2018-06-10)

* Update links to new project GitHub page and documentation site

## 0.1.0 (2018-04-23)

* Initial release
