# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### 0.4.0 -- August 30, 2026

### Changed

* **Breaking**: `MatchMode` enum removed
* **Breaking**: `Db::find_re` removed, replaced by `Db::find_regex` and `Db::find_glob`
  functions which do not take a `MatchMode` parameter.

### Added

* `Db::init_db()`, `Db::rebuild()`, and `Db::verify()` for RPM database
  management — equivalent to `rpm --initdb`, `rpm --rebuilddb`, and
  `rpmdb --verifydb` respectively.
* Transaction support for installing, upgrading, and erasing packages: see
  `Db::transaction()` and the `Transaction` struct.
* Keyring and public key management: `Keyring` and `PubKey` types for loading,
  inspecting, and managing trusted GPG keys. Keyrings can be created empty,
  loaded from the system RPM database (`Keyring::from_rpmdb()`), or populated
  manually. `PubKey` supports loading from armored files, base64 encoding,
  fingerprint and key ID introspection.
* Signature and digest verification: `VerifyOptions` and `VerificationFlags`
  control which checks are performed when reading `.rpm` files.
  `Package::from_file()` now accepts an optional `&VerifyOptions` parameter —
  `None` uses system defaults (verify everything with the system keyring),
  `Some(&VerifyOptions::skip_verification())` disables all checks.
* `Db::keyring()` returns the keyring loaded from the RPM database.
* New cfg-gated APIs for newer RPM versions: `Keyring::remove_key()`,
  `Keyring::lookup()`, `Keyring::keys()` iterator, `PubKey::from_file()`,
  `PubKey::fingerprint_hex()`, `PubKey::key_id_hex()`.
* `Db::import_pubkey()` and `Db::delete_pubkey()` for persistent key management.
* `Db::open_with_root()` opens the database rooted at an alternate directory,
  the equivalent of `rpm --root` / `dnf --installroot`. The database is
  resolved at `<root>/<_dbpath>` and transactions treat `<root>` as the
  filesystem root, enabling operations on an image, chroot, or isolated
  temporary database without touching the host's `/`.
* `Db::import_pubkey()` and `Db::delete_pubkey()` import and delete public keys
  honoring the database's root directory.
* `Display` and `FromStr` for `Tag`, enabling round-trip conversion between
  tag names (e.g. "Name", "Version") and `Tag` enum variants. Wraps librpm's
  `rpmTagGetName` and `rpmTagGetValue`. Also implements `TryFrom<i32>` for
  converting raw tag numbers back to `Tag` variants.

## 0.3.0 -- August 3, 2026

### Changed

* **Breaking**: `Package` renamed to `PackageHeader.
* **Breaking:** `PackageHeader::from_file()` now takes a second parameter
  `options: Option<&VerifyOptions>`. Pass `None` to verify with system
  defaults, or `Some(&VerifyOptions::skip_verification())` to preserve
  the previous behavior of skipping all verification.

### Added

* Spec file parsing and package building via `librpmbuild` - `Spec::parse()`,
  source/package iteration, `Spec::build()` for building packages, etc.
* `SignArgs::resign()` convenience method for re-signing packages
* `PackageHeader::format()` applies an RPM query format string to a package header using
  `%{TAG}` syntax, equivalent to `rpm --queryformat` or librpm's `headerFormat()`.
* `Index` enum expanded with `Basenames`, `Dirnames`, `Instfilenames`,
  `Providename`, `Requirename`, `Conflictname`, `Obsoletename`, `Group`,
  `Triggername`, `Recommendname`, `Suggestname`, `Supplementname`,
  `Enhancename`, `Filetriggername`, and `Transfiletriggername` variants
* `Db::find_re()` searches by glob or regex pattern via `rpmdbSetIteratorRE`,
  with a new `MatchMode` enum (`Glob`, `Regex`)
* `Iter::match_count()` and `Iter::offset()` expose the iterator's index
  snapshot count and current record offset (`rpmdbGetIteratorCount`,
  `rpmdbGetIteratorOffset`)
* `Db::init_db()`, `Db::rebuild()`, and `Db::verify()` for RPM database
  management — equivalent to `rpm --initdb`, `rpm --rebuilddb`, and
  `rpmdb --verifydb` respectively.
* `archive::PackageReader` provides sequential, streaming access to the file
  contents inside an `.rpm` package's payload.
* `PackageHeader::changelogs()` returns changelog entries as `Vec<ChangelogEntry>`.
  Each `ChangelogEntry` provides `time()`, `timestamp()`, `name()`, and `text()` accessors.
* `logging::set_verbosity()` controls librpm's log verbosity at the C level
  and `logging::last_message()` retrieves the most recent librpm log message.
  A `LogLevel` enum provides the available verbosity levels.
* `logging::set_behavior()` switches between routing log messages through
  Rust's `log` crate (`LogBehavior::LogCrate`, the default) or librpm's
  native stderr output (`LogBehavior::Default`).
* When `LogBehavior::LogCrate` mode is set, you can install any
  `log`-compatible backend (e.g. `env_logger`) and messages appear
  automatically with the target `"librpm"`.
* `PackageHeader::is_source()` distinguishes SRPMs from binary RPMs
  via `headerIsSource`.
* `PackageHeader::has_tag()` checks for the presence of a tag without
  decoding it, via `headerIsEntry`.
* `Dependency::is_rich()`, `is_weak()`, `is_reverse()` predicates expose
  the corresponding librpm `rpmdsIs*` functions.
* `Dependency::satisfies()` compares two dependencies using `rpmdsCompare`.
* `FileEntry::mtime()` returns the file modification time as `SystemTime`.
* `FileEntry::state()` returns the file install state as a new `FileState`
  enum (also available on `ArchiveEntry`).
* `Files::find()` looks up a file by path using `rpmfilesFindFN`.
* `librpm::arch()` and `librpm::os()` return the configured architecture
  and OS name via `rpmGetArchInfo` / `rpmGetOsInfo`.

### Fixed

* Fixed a crash (SIGSEGV in `rpmAtExit`) when using multiple `Db` instances
  from concurrent threads on RPM <= 4.18 (e.g. CentOS Stream 9). The crash
  was caused by unsynchronized global linked lists in `rpmdb.c` that track
  live iterators and databases. Iterator creation, destruction, and transaction
  set cleanup are now serialized through a process-wide lock.

## 0.2.1 -- July 2, 2026

### Fixed

* Fixed documentation links
* Fixed builds against Ubuntu 24.04's build of RPM, which is slightly different from CentOS Stream.

## 0.2.0 -- July 2, 2026

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
