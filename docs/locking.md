# Threading and librpm

This document describes the thread-safety properties of librpm (the C library)
and how librpm.rs accounts for them. It is intended for contributors working on
the binding internals.

## librpm's threading model

librpm was not originally designed with thread safety as a primary concern. Some
internal synchronization exists (e.g. the macro context has a recursive mutex
since RPM 4.12), but coverage across the entire library is incomplete and
inconsistent, especially across older versions. Some forms of process-global state
is also unprotected across any version of librpm. As librpm.rs is designed to
compile against multiple versions of librpm, we add some locking to guard against
problems present in older versions of librpm even if newer versions has its own
locking. The following sections describe the specific areas that affect librpm.rs.

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

`rpmReadConfigFiles` reads the system rpmrc and macro configuration. It
vicariously calls `rpmInitCrypto`, which initializes the cryptographic backend
(NSS or OpenSSL depending on the RPM build). `rpmReadConfigFiles` and
`rpmInitCrypto` must be called exactly once per process. librpm.rs enforces
this through the `ConfigState::configured` flag, guarded by the same
`ConfigState` mutex.

librpm.rs enforces the once-only requirement through `ConfigState`, a struct
behind a `OnceLock<Mutex<ConfigState>>` in `src/internal/global_state.rs`.
`ConfigState` contains a single `configured: bool` flag. The public entry
points `librpm::init()` and `librpm::init_with()` both call
`config::read_file()`, which:

1. Acquires `ConfigState::lock()`.
2. Checks `configured` — if already `true`, returns an error immediately
   without calling into C.
3. Calls `rpmReadConfigFiles` (which internally calls `rpmInitCrypto`).
4. Sets `configured = true` and releases the lock.

Other operations that require configuration (`Db::open()`,
`Keyring::from_rpmdb()`, `Db::import_pubkey()`, `Db::delete_pubkey()`)
also acquire the `ConfigState` lock and check the flag, returning an
error if `init()` has not been called. This prevents use-before-init
rather than relying on opaque C-level failures.

### Transaction set lazy initialization

`rpmtsInitIterator` performs lazy-init mutations on the transaction set the
first time it is called: opening the database (`rpmtsOpenDB`) and loading the
keyring (`loadKeyring`). These are check-then-act patterns without locks,
making them unsafe under concurrent `&rpmts` access.

After lazy init, it only reads from the ts.

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

#### Recognizing this race (symptoms)

This bug is easy to misdiagnose because the crash is spatially and temporally
disconnected from the code that caused it. If you see the following pattern,
suspect the tracking-list race:

- **Crash *after* all work succeeds, at process exit.** A test binary reports
  every test as `ok`/passed and *then* the process dies. In CI you see the
  test summary line followed by a signal, e.g. `error: test failed, ...
  (signal: 11, SIGSEGV: invalid memory reference)` with no failing assertion.
  Exit-time `atexit` handlers run after `main`/the test harness returns, so the
  logs look like a clean pass right up to the crash.
- **SIGSEGV (signal 11), not a Rust panic.** The corruption is in C-owned
  memory, so there is no Rust backtrace, just a raw segfault or occasionally
  a `double free`/`free(): invalid pointer` from glibc.
- **The backtrace bottoms out in librpm exit cleanup.** Frames to look for:
  `rpmAtExit`, `rpmdbClose`, `dbiCursorFree`, `rpmdbFreeIterator`,
  `rpmmiFree` — all reached from `__run_exit_handlers` / `exit`, *not* from
  your call sites. The offending code has long since returned.
- **Nondeterministic and concurrency-gated.** It reproduces only when multiple
  threads (or multiple `Db`/`rpmts` instances across threads) open the DB or
  create/free iterators concurrently. Serial runs never crash; adding threads
  raises the rate. Expect flakiness (~25% in the reproducer above), not a
  100% failure — a passing run does not mean the bug is absent.
- **Version-specific: only RPM <= 4.18.** Reproduces on e.g. CentOS Stream 9
  (RPM 4.16) but *not* on Fedora 40+/RHEL 10 (RPM 4.19+) or RPM 6.x, where the
  lists were removed. A crash that appears only on the older-RPM CI leg is a
  strong tell.

The practical lesson for maintainers: any newly added FFI call that could
lazily open the database or create a match iterator — even one that looks
read-only or purely in-memory — must acquire `rpm_global_lock`, or it
reopens this exact failure mode. See the call-site tables below.

**RPM 4.19+ removed these global lists entirely**, eliminating the issue.
The `rpmAtExit` function and the `rpmmiRock`/`rpmiiRock`/`rpmdbRock` variables
no longer exist.

#### librpm.rs mitigation

librpm.rs uses a process-wide `Mutex<()>` (`rpm_global_lock()` in
`src/internal/global_state.rs`) to serialize the FFI calls that touch these
global lists.

**Many higher-level librpm calls touch these lists internally**, without any
obvious iterator or database API in the signature. Any call that may lazily
open the database (`rpmtsOpenDB` -> `rpmdbRock`) or create/free a match iterator
(`rpmtsInitIterator`/`rpmdbFreeIterator` -> `rpmmiRock`) must hold the lock,
even if it looks like a pure transaction or keyring operation. The direct calls
are the obvious cases; the indirect ones below are where the original locking
model had a gap that caused SIGSEGV at `rpmAtExit`.

Direct list-touching calls:

| Call site | FFI function | List affected |
|-----------|-------------|---------------|
| `MatchIterator::new()` / `new_re()` | `rpmtsInitIterator` | `rpmmiRock` (+ `rpmdbRock` via lazy DB open) |
| `MatchIterator::drop()` | `rpmdbFreeIterator` | `rpmmiRock` |
| `TransactionSet::drop()` | `rpmtsFree` | `rpmdbRock` (via `rpmtsCloseDB` -> `rpmdbClose`) |
| `Spec::parse()` | `rpmSpecParse` | Macro context, global rpmts |
| `Spec::build()` | `rpmSpecBuild` | Macro context, global rpmts |

Indirect list-touching calls (open the DB and/or create iterators internally):

| Call site | FFI function | Internal path |
|-----------|-------------|---------------|
| `Transaction::add_install()` / `add_reinstall()` | `rpmtsAddInstallElement` / `rpmtsAddReinstallElement` | `addPackage` -> `rpmtsOpenDB` + upgrade/obsolete iterators |
| `Transaction::add_erase()` / `add_restore()` | `rpmtsAddEraseElement` / `rpmtsAddRestoreElement` | `removePackage` -> match iterators over the DB |
| `Transaction::check()` | `rpmtsCheck` | `rpmtsOpenDB` + many dependency-resolution iterators |
| `Transaction::run()` | `rpmtsRun` | `rpmtsSetup`/`rpmtsPrepare` open the DB and create iterators throughout execution |
| `Db::keyring()` / `Keyring::from_rpmdb()` | `rpmtsGetKeyring(ts, 1)` | `loadKeyringFromDB` -> `rpmtsInitIterator` (gpg-pubkey lookup) |
| `Db::import_pubkey()` / `delete_pubkey()` | `rpmtxnImportPubkey` / `rpmtxnDeletePubkey` | writes a gpg-pubkey "package", opening the DB |
| `Db::init_db()` / `rebuild()` / `verify()` | `rpmtsInitDB` / `rpmtsRebuildDB` / `rpmtsVerifyDB` | open/create the database |
| `PackageHeader::from_file()` / `archive::PackageReader` | `rpmReadPackageFile` | with signature checking (`RPMVSF_DEFAULT`) and no explicit keyring, calls `rpmtsGetKeyring(ts, 1)` -> `loadKeyringFromDB` -> opens the DB + gpg-pubkey iterator |

For the brief direct calls the lock is held only for the duration of the FFI
call. For calls that do substantial work under the lock (`rpmtsCheck`,
`rpmtsRun`), the lock is held for the entire call because librpm creates and
frees iterators at points we do not control. Database iteration
(`rpmdbNextIterator`) and all read-only header operations still run entirely
without the lock.

On RPM 4.19+, the tracking lists no longer exist, so the only remaining
justification for `rpm_global_lock` is spec/build serialization (see below).
The indirect call sites above still acquire it uniformly rather than
version-gating each site; keeping the lock a single, always-on primitive is
simpler and the contention it adds on 4.19+ is negligible. It could be made a
version-gated no-op in the future without affecting `mutation_lock` — see
"Are the two locks redundant?" below.

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

### Write-operation serialization (`mutation_lock`)

**This lock serializes all RPM write operations within the process.**

`rpmtsRun`, `rpmtsInitDB`, `rpmtsRebuildDB`, and `rpmtsVerifyDB` have
process-global side effects that are unsafe under concurrent execution:

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

#### librpm.rs mitigation

librpm.rs uses a second process-wide `Mutex<()>` (`mutation_lock()` in
`src/internal/global_state.rs`) to serialize these write operations:

| Call site | FFI function(s) | Side effects |
|-----------|----------------|--------------|
| `Transaction::run()` | `rpmtxnBegin`, `rpmtsRun`, `rpmtxnEnd` | All of the above |
| `Db::import_pubkey()` | `rpmtxnImportPubkey` (or `rpmtsImportPubkey` on older RPM) | Keystore write |
| `Db::delete_pubkey()` | `rpmtxnBegin`, `rpmtxnDeletePubkey`, `rpmtxnEnd` | Keystore write |
| `Db::init_db()` | `rpmtsInitDB` | Database file creation |
| `Db::rebuild()` | `rpmtsRebuildDB` | Database rewrite |
| `Db::verify()` | `rpmtsVerifyDB` | Database read (but shares chroot path) |

#### Interaction with `rpm_global_lock` (lock ordering)

The two locks protect different invariants (see "Are the two locks
redundant?" below), but a single write operation legitimately needs **both**:
`mutation_lock` for its process-global side effects, and `rpm_global_lock`
because it opens the DB / creates iterators internally.

- **`rpm_global_lock`**: protects RPM <= 4.18 global linked lists (and
  spec/build). For a bare iterator create/destroy it is held only briefly
  (~microseconds); for `rpmtsCheck`/`rpmtsRun` it is held for the whole call.
- **`mutation_lock`**: serializes write operations. Held for the duration of
  `rpmtsRun` (potentially seconds or minutes for large transactions).

**Lock ordering: always acquire `mutation_lock` first, then
`rpm_global_lock`.** Every write path follows this order —
`Transaction::run()`, `Db::init_db()`, `Db::rebuild()`, `Db::verify()`,
`Db::import_pubkey()`, `Db::delete_pubkey()`. Read paths and element-addition
(`add_install` etc.) take only `rpm_global_lock`. Because no path ever acquires
`mutation_lock` while already holding `rpm_global_lock`, this ordering is
deadlock-free.

#### Are the two locks redundant?

Given that "anything that mutates the DB may open the DB," it is reasonable to
ask whether `mutation_lock` is now subsumed by `rpm_global_lock`. It is not,
for two reasons:

1. **Different invariants, different version-applicability.**
   `rpm_global_lock` is fundamentally a RPM <= 4.18 workaround (the tracking
   lists) plus spec/build serialization. `mutation_lock` guards hazards
   present in *every* RPM version, including 4.19+ and 6.x: the `fcntl`
   per-(process,inode) `.rpm.lock` semantics, chroot `rootState`, signal
   masks, and the SIGPIPE handler. These do not disappear when the tracking
   lists do.

2. **Neither caller set is a subset of the other.** Reads (`find`,
   `installed_packages`, keyring load) and spec/build take `rpm_global_lock`
   but never `mutation_lock`. Writes take both. Merging the two would conflate
   independent concerns and would forever prevent making `rpm_global_lock` a
   version-gated no-op on 4.19+ while *keeping* write serialization — a valid
   future optimization, since on 4.19+ the tracking-list rationale is gone but
   the `fcntl`/chroot/signal rationale remains.

#### Interaction with `rpmtxnBegin`/`rpmtxnEnd`

`Transaction::run()` acquires three locks, in this order:

1. `mutation_lock()` — serializes write operations within the process
2. `rpm_global_lock()` — guards the RPM <= 4.18 tracking lists that `rpmtsRun`
   churns internally
3. `rpmtxnBegin(ts, RPMTXN_WRITE)` — acquires the cross-process `.rpm.lock`
   file lock via `fcntl`

The Rust-side lock prevents the `fcntl` hazard (multiple fds to the same
lock file within one process), while the C-side lock prevents concurrent
writes from other processes (e.g. `dnf` or `rpm` CLI running simultaneously).

#### Transaction flags stickiness

`rpmtsRun` internally expands high-level flags like `NOSCRIPTS` into
individual sub-flags (`NOPRE`, `NOPOST`, etc.) and writes them back to
the transaction set. `Transaction::run()` saves and restores the flags
around each call to prevent this mutation from leaking.

## Summary of Rust type markers

| Type | `Send` | `Sync` | Rationale |
|------|--------|--------|-----------|
| `TransactionSet` | Yes | No | Heap-allocated, self-contained; but lazy-init mutations in `rpmtsInitIterator` are not thread-safe for concurrent `&self` |
| `Db` | Yes (inherited) | No (inherited) | Contains `TransactionSet` |
| `Transaction` | No (inherited) | No (inherited) | Borrows `&mut Db`; contains raw `rpmts` pointer |
| `MatchIterator` | No (default) | No (default) | Contains raw pointer |
| `Header` | No (default) | No (default) | Contains raw pointer |
| `PackageHeader` | No (inherited) | No (inherited) | Contains `Header` |
| `Spec` | Yes | No (default) | Heap-allocated, refcounted handle; global-state FFI calls serialized by `rpm_global_lock()` |
| `Element` | No (default) | No (default) | Contains raw `rpmte` pointer |
| `Problem` | No (default) | No (default) | Contains raw `rpmProblem` pointer |
| `Problems` | No (default) | No (default) | Contains raw `rpmps` pointer |

Multiple `Db` instances on separate threads are safe for read-only queries.
The `rpm_global_lock` serializes the global-state-touching FFI calls; iteration
and header access are fully concurrent.

Write operations are serialized by `mutation_lock`. Only one `Transaction::run()`,
`Db::init_db()`, `Db::rebuild()`, or `Db::verify()` call can execute at a time
across all threads in the process.

#### File I/O in the callback trampoline

During `Transaction::run()`, the internal callback trampoline handles
`RPMCALLBACK_INST_OPEN_FILE` and `RPMCALLBACK_INST_CLOSE_FILE` by calling
`Fopen`/`Fclose` on the RPM file path. These calls execute inside the
callback, which fires during `rpmtsRun()` on the same thread. No additional
locking is needed — `mutation_lock` (process-wide) and `.rpm.lock`
(cross-process via `rpmtxnBegin`) are already held.
