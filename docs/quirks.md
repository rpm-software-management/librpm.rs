# librpm API quirks

This document records non-obvious behavior in the librpm / librpmbuild C API
that librpm.rs works around, with enough context to understand why each
workaround exists.

## NULL transaction root dir crashes plugins (RPM 4.19+)

**Symptom**: `Transaction::run()` segfaults (SIGSEGV) inside an RPM
transaction plugin — e.g. `syslog_tsm_pre` in `/usr/lib64/rpm-plugins/syslog.so`
— reached from `rpmtsRun()`. Deterministic (100%), and version-specific to
RPM 4.19+ / CentOS Stream 10, where the syslog plugin is enabled by default.

**Root cause**: `rpmtsCreate()` initializes `ts->rootDir` to NULL and never
defaults it; `rpmtsRootDir(ts)` returns that NULL verbatim. The root dir only
becomes non-NULL when `rpmtsSetRootDir()` is called — which the `rpm`/`dnf`
CLIs as well as the Python bindings always do (`rpmtsSetRootDir(ts, "/")`).

**Why the CLIs are not affected**: `rpm` and `dnf` always call
`rpmtsSetRootDir(ts, "/")` (or the `--root` value) right after creating the
transaction set, so `rpmtsRootDir()` is never NULL for them.

**Workaround**: `TransactionSet::create()` sets the root dir to `"/"` when it
is unset. This is the single chokepoint through which every `rpmts` in
librpm.rs is created, and it runs before anything can set the root, so it
guarantees no librpm.rs `rpmts` ever has a NULL root dir. There is no RPM
macro or init-time setting for the root directory (unlike the database path,
which is the `_dbpath` macro), so this is the only place the default can be
applied. It is applied conditionally so a future chroot/root API — or an
upstream `rpmtsCreate()` that grows its own default — is not clobbered.

## Double finalization (NOFINALIZE)

**Symptom**: `rpmSpecBuild()` fails with "Duplicate Os entries in package"
(and Platform, Optflags, Sourcerpm) when called on a spec that was parsed
without `RPMSPEC_NOFINALIZE`.

**Root cause**: `finalizeSpec()` adds OS, Platform, Optflags, and Sourcerpm
tags to every package header via `headerPutString()`, which *appends* rather
than replaces.  It is called in two places:

1. At the end of `rpmSpecParse()` (unless `RPMSPEC_NOFINALIZE` is set).
2. Inside `rpmSpecBuild()`, via `parseGeneratedSpecs()`, which processes
   dynamically generated `.specpart` files (e.g. debuginfo subpackages) and
   then calls `finalizeSpec()` to finalize any newly added subpackages.

If the spec was parsed *without* `NOFINALIZE`, path (1) already finalized
all packages.  When path (2) runs during the build, the existing packages
get finalized a second time, producing duplicate header entries that fail
`checkForDuplicates()`.

**Why rpmbuild is not affected**: The rpmbuild CLI defaults to
`RPMSPEC_NOFINALIZE` for all binary builds (`rpmbuild.cc:67`).  It only
clears the flag for source-only builds (`-bs`, `-rs`, `-ts`) where no
binary packaging occurs and `parseGeneratedSpecs()` is never reached.

Note that `NOFINALIZE` must *not* be used for source-only builds (where no
`INSTALL`, `PACKAGEBINARY`, or `FILECHECK` flags are set).  In that case,
`parseGeneratedSpecs()` is never called during `rpmSpecBuild()`, so
`finalizeSpec()` would never run — producing an SRPM with an unfinalized
header (missing Os, Platform, Optflags, and Sourcerpm tags).

**Workaround**: Use `Spec::parse_for_build()`, which inspects the `BuildArgs`
to determine whether `NOFINALIZE` is needed — matching rpmbuild's logic.  For
callers using `Spec::parse()` directly, `SpecFlags::nofinalize_or_none()`
returns `NOFINALIZE` on RPM versions that have it and `NONE` on older versions
(where the flag and the double-finalization issue both do not exist), but the
caller must ensure it is only applied for binary builds.

**Upstream status**: This is a latent issue in the library API, not a bug
in rpmbuild. Ideally `finalizeSpec()` would be made idempotent.

## MKBUILDDIR and RPM 6 build directories

**Symptom**: `%setup` fails during `%prep` with a "cd: no such file or
directory" error when building on RPM 6 without `RPMBUILD_MKBUILDDIR`.

**Root cause**: RPM 4.19 / RPM 6 introduced per-package build directories.
Instead of unpacking directly into `%{_builddir}` (e.g. `BUILD/`), RPM 6
creates a wrapper directory `%{_builddir}/%{name}-%{version}-build/` that
contains `%{specpartsdir}` (for dynamic subpackage `.specpart` files) and
`rpmbuild.env` (build environment variables sourced by later stages).

The `RPMBUILD_MKBUILDDIR` flag triggers this directory setup.  Without it,
`%setup` tries to `cd` into a directory that does not exist.

On older RPM versions (pre-4.19), `%setup` handled directory creation
itself and `RPMBUILD_MKBUILDDIR` does not exist.

**Why rpmbuild is not affected**: rpmbuild always includes `MKBUILDDIR`
when running `%prep` or higher stages (`rpmbuild.cc:691-696`).

**Workaround**: `BuildArgs::new()` conditionally includes `MKBUILDDIR` via
`#[cfg(has_rpmbuildflag_mkbuilddir)]`, so it is used when available and
omitted on older RPM versions where it is not needed.  The raw
`BuildFlags::MKBUILDDIR` constant is also cfg-gated for callers using the
low-level `Spec::build()` method directly.
