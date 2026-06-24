# Threading and librpm

This document describes the thread-safety properties of librpm (the C library)
and how librpm.rs accounts for them. It is intended for contributors working on
the binding internals.

## librpm's threading model

librpm was not designed with thread safety as a primary concern. Some
internal synchronization exists (e.g. the macro context has a recursive mutex
since RPM 4.12), but coverage is incomplete and inconsistent — other
process-global state has no protection at all, and the protection that exists
varies across versions. The following sections describe the specific areas that
affect librpm.rs.

### Macro context

The global macro context (`rpmGlobalMacroContext`) is a process-wide table of
key-value pairs used for configuration. Since RPM 4.12, the macro context has
an internal recursive mutex. All public macro API functions (`rpmDefineMacro`,
`rpmPopMacro`, `rpmExpandMacros`, `rpmMacroIsDefined`, etc.) acquire this lock
before reading or mutating the table. The lock is per-context, so every
context instance — global or independently created — is independently
synchronized. Concurrent calls from multiple threads will not corrupt state,
though of course the ordering of concurrent define/pop operations is
nondeterministic.

librpm.rs does not add any additional synchronization for macro operations,
relying on the C-level locking.

### Configuration initialization

`rpmReadConfigFiles` and `rpmInitCrypto` must be called exactly once per
process. librpm.rs enforces this through the `ConfigState::configured` flag,
guarded by the same `ConfigState` mutex.

### Transaction set lazy initialization

`rpmtsInitIterator` performs lazy-init mutations on the transaction set the
first time it is called: opening the database (`rpmtsOpenDB`) and loading the
keyring (`loadKeyring`). These are check-then-act patterns without locks,
making them unsafe under concurrent `&rpmts` access.

librpm.rs marks `TransactionSet` as `Send` but `!Sync`, preventing concurrent
`&self` access to a single instance. Each `Db` owns its own `TransactionSet`,
so lazy-init races within a single `rpmts` cannot occur.

### Global iterator/database tracking lists (RPM <= 4.18)

**This is the most critical threading concern for librpm.rs.**

RPM versions up to and including 4.18 maintain three process-global linked
lists in `rpmdb.c`:

| Variable | Tracks |
|----------|--------|
| `rpmmiRock` | All live `rpmdbMatchIterator` instances |
| `rpmiiRock` | All live `rpmdbIndexIterator` instances |
| `rpmdbRock` | All open `rpmdb` instances |

These lists exist solely so that `rpmAtExit()` (registered via `atexit(3)`)
can clean up any resources that were not explicitly freed before process exit.

The list operations — insert-at-head on creation, walk-and-unlink on
destruction — are completely unsynchronized. When multiple threads
concurrently create or destroy iterators (via `rpmtsInitIterator` /
`rpmdbFreeIterator`) or transaction sets (via `rpmtsFree` -> `rpmdbClose`),
they race on these list pointers. This corrupts the linked list structure.
At process exit, `rpmAtExit()` walks the corrupted list and crashes
(typically a SIGSEGV in `dbiCursorFree`).

This was verified with a C reproducer: 4 threads each creating an `rpmts`,
iterating the database, and freeing it. Crash rate was ~25% over 200 runs on
CentOS Stream 9 (RPM 4.16). Sequential-only runs: 0 crashes in 200 runs.

**RPM 4.19+ removed these global lists entirely**, eliminating the issue.
The `rpmAtExit` function and the `rpmmiRock`/`rpmiiRock`/`rpmdbRock` variables
no longer exist.

#### librpm.rs mitigation

librpm.rs uses a process-wide `Mutex<()>` (`rpm_global_lock()` in
`src/internal/global_state.rs`) to serialize the specific FFI calls that
touch these global lists:

| Call site | FFI function | List affected |
|-----------|-------------|---------------|
| `MatchIterator::new()` | `rpmtsInitIterator` | `rpmmiRock` (+ `rpmdbRock` via lazy DB open) |
| `MatchIterator::drop()` | `rpmdbFreeIterator` | `rpmmiRock` |
| `TransactionSet::drop()` | `rpmtsFree` | `rpmdbRock` (via `rpmtsCloseDB` -> `rpmdbClose`) |
| `Spec::parse()` | `rpmSpecParse` | Macro context, global rpmts |
| `Spec::build()` | `rpmSpecBuild` | Macro context, global rpmts |

The lock is held only for the duration of each FFI call, not for the lifetime
of iterators or transaction sets. Database iteration (`rpmdbNextIterator`) and
all read-only header operations run entirely without the lock.

On RPM 4.19+, the lock remains necessary for spec/build serialization (see
below) even though the global tracking lists no longer exist.

### Spec parsing and package building

`rpmSpecParse` and `rpmSpecBuild` interact with process-global state in several
ways that make concurrent calls unsafe:

- **Macro context side effects**: Both functions read and modify the global
  macro table. `rpmSpecParse` expands macros like `%{_topdir}`, `%{_sourcedir}`,
  and `%{_builddir}` to resolve paths, and spec directives like `%define` /
  `%global` add or modify entries. `rpmSpecBuild` expands macros during script
  generation and may define build-time macros. While the macro table itself has
  an internal lock (since RPM 4.12), the *semantic* consistency of macro values
  across a parse-then-build sequence is not guaranteed if another thread
  redefines `%{_topdir}` between the two calls.

- **Internal transaction set**: `rpmSpecBuild` creates and uses its own
  `rpmts` internally. On RPM <= 4.18, this touches the global `rpmdbRock`
  tracking list (see above).

- **Filesystem side effects**: Both functions create directories, write
  temporary scripts to `/var/tmp`, and execute shell commands. Concurrent
  builds that share a `%{_topdir}` would collide on these paths.

`Spec::parse()` and `Spec::build()` both acquire `rpm_global_lock()`,
serializing them against each other and against database operations. This
prevents the internal state corruption described above.

However, callers that set up macros (`%{_topdir}`, `%{_sourcedir}`) before
parsing must ensure those macros remain consistent through the build. Since
`MacroContext::define()` acquires and releases the lock for each call, there
is a window between macro setup and `Spec::parse()` where another thread
could overwrite the values. Callers performing concurrent builds should use
their own higher-level synchronization to cover the entire
define → parse → build sequence.

### Process-global state affecting write operations (future)

The following global state is relevant when librpm.rs eventually supports
`rpmtsRun` (install/erase/upgrade). It does not affect the current read-only
API.

| State | Location | Impact |
|-------|----------|--------|
| Chroot (`rootState`) | `rpmchroot.cc` | Only one rootDir active at a time; concurrent `rpmtsRun` with different roots will corrupt |
| Signal blocking (`blocked`, `oldMask`) | `rpmsq.cc` | Shared refcount; concurrent write transactions will nest incorrectly |
| SIGPIPE handler | `transaction.cc` | `sigaction` save/restore races if two `rpmtsRun` overlap |
| `.rpm.lock` via `fcntl` | `rpmlock.cc` | POSIX `fcntl` locks are per-(process,inode), not per-fd: closing any fd to the lock file releases ALL locks the process holds on that inode |

The `fcntl` issue is the most subtle: if transaction set A holds a write lock
and transaction set B is dropped (closing B's fd to `.rpm.lock`), A's lock
silently disappears. This means at most one `Db` should perform write
operations at a time.

## Summary of Rust type markers

| Type | `Send` | `Sync` | Rationale |
|------|--------|--------|-----------|
| `TransactionSet` | Yes | No | Heap-allocated, self-contained; but lazy-init mutations in `rpmtsInitIterator` are not thread-safe for concurrent `&self` |
| `Db` | Yes (inherited) | No (inherited) | Contains `TransactionSet` |
| `MatchIterator` | No (default) | No (default) | Contains raw pointer |
| `Header` | No (default) | No (default) | Contains raw pointer |
| `Package` | No (inherited) | No (inherited) | Contains `Header` |
| `Spec` | Yes | No (default) | Heap-allocated, refcounted handle; global-state FFI calls serialized by `rpm_global_lock()` |

Multiple `Db` instances on separate threads are safe for read-only queries.
The `rpm_global_lock` serializes the global-state-touching FFI calls; iteration
and header access are fully concurrent.
