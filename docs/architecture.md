# Architecture

This document describes the high-level design of librpm.rs: how it
decomposes librpm's C API into Rust types, the ownership and lifetime
relationships between those types, and the error handling strategy.

For thread-safety specifics, see [locking.md](locking.md).

## The `rpmts` god object

In librpm's C API, nearly everything flows through the "transaction set"
(`rpmts`). It bundles database configuration, verification flags, the
keyring, root directory, transaction elements, ordering state, and more.
A single `rpmts` is used for both read-only queries and mutating
transactions.

librpm.rs decomposes much of this functionality into purpose-specific types
or functions that de-emphasizes the role of the transaction set:

| Rust type | Wraps | Role |
|-----------|-------|------|
| `Db` | `rpmts` (1:1) | Read-only database queries, persistent system keyring management |
| `Transaction<'db>` | borrows `Db`'s `rpmts` | Install/erase lifecycle (mutating) |
| `PackageHeader::from_file()` | creates ephemeral `rpmts` | Read a `.rpm` file (no DB needed) |
| `Spec::build()` | `rpmSpec` (via ephemeral `rpmts`) | Spec file parsing and building (`build` feature) |

External callers never interact with `TransactionSet` or `rpmts` directly.
The `TransactionSet` wrapper is kept internal and owns the `rpmts` pointer.

librpm.rs also creates additional wrappers and composite types to minimize the
centrality of `TransactionSet`.

| Rust type | Wraps | Role |
|-----------|-------|------|
| `VerifyOptions` | flags + optional `Keyring` | Verification configuration |
| `VerificationFlags` | `rpmVSFlags_e` | Control verification checks |
| `Keyring` | `rpmKeyring` (standalone) | In-memory trusted key management |
| `PubKey` | `rpmPubkey` (standalone) | Individual public key |

## Public API type overview

The main public types are organized around the operations they support:

| Area | Important types | Purpose |
|------|-----------------|---------|
| Database | `Db`, `Iter`, `Index` | Open an RPM database, query package headers, and select database indices |
| Transactions | `Transaction`, `TransactionFlags`, `ProblemFilter`, `Element`, `ElementType` | Build, inspect, check, order, and run package transactions |
| Packages | `PackageHeader`, `Tag`, `TagData`, `Files`, `FileEntry`, `FileAttrs`, `FileState`, `ChangelogEntry` | Read package metadata, file lists, attributes, and changelogs |
| Verification | `VerifyOptions`, `VerificationFlags`, `Keyring`, `PubKey` | Configure signature/digest verification and trusted keys |
| Payloads | `PackageReader`, `ArchiveEntry` | Read files from an RPM payload sequentially or by entry |
| Dependencies | `Dependency`, `Dependencies`, `DepFlags` | Inspect and compare package dependencies |
| Build and signing | `Spec`, `BuildArgs`, `BuildFlags`, `SignArgs` | Parse/build spec files and sign packages when the corresponding features are enabled |
| Configuration and errors | `MacroContext`, `Version`, `Error`, `RpmErrorKind`, `TransactionError` | Expand RPM macros, parse versions, and report failures |
| Logging | `LogBehavior`, `LogLevel` | Control librpm logging and its Rust `log` integration |

Most public wrappers own or reference refcounted RPM objects. The lifetime
relationships and exceptions to ordinary Rust borrowing are described in the
ownership section below.

### Note: the read/write boundary is not clean

The type decomposition above suggests a tidy split - `Db` reads, `Transaction`
writes - but librpm does not honor that boundary internally. Almost any
operation may **lazily open the database** (`rpmtsOpenDB`) and **create match
iterators** on first use, including calls that look purely read-only or purely
in-memory: adding a transaction element (`rpmtsAddInstallElement` resolves
upgrades/obsoletes), checking dependencies (`rpmtsCheck`), loading a keyring
(`rpmtsGetKeyring` scans for gpg-pubkey headers), and running a transaction
(`rpmtsRun` fingerprints and conflict-checks throughout).

On RPM <= 4.18 those lazy opens and iterator churn mutate process-global
tracking lists that are not internally synchronized, so the locking
model cannot simply wrap "the write methods": it must cover every entry point
that may touch the database, regardless of which Rust type exposes it. See
[locking.md](locking.md) for the full call-site inventory and the
`mutation_lock` / `rpm_global_lock` ordering.

The goal with librpm.rs is to provide one library which works across different
versions of librpm, therefore, we perform locking which may be superfluous on
newer versions but is required to keep older versions safe.

## Ownership and lifetimes

```text
Db
 └── TransactionSet (1:1, owns rpmts)
      │
      ├── Iter ── PackageHeader ── Header
      │   (internally refcounted,          (owns refcounted
      │    independent of Db)               header, independent)
      │
      └── Transaction<'db> (&mut Db borrow)
           │
           ├── Element<'txn> (pointer into rpmts state)
           │
           └── Problems ── Problem
               (refcounted rpmps)  (refcounted rpmProblem)
```

Key relationships:

- **`Db` owns `TransactionSet`** (1:1). Each `Db` has its own `rpmts`
  with an independent database connection.

- **`Transaction<'db>` borrows `&mut Db`** exclusively via `PhantomData`.
  This prevents new queries from being started through `Db` while a
  transaction is active, which is necessary because `rpmtsRun` mutates the
  `rpmts` state (flags, ordering, problem set). Existing `Iter` values do not
  borrow `Db`; their C-level references keep the database and transaction set
  alive across transaction creation and cleanup. The exclusive borrow also
  prevents `Db::keyring()` and other `Db` methods from changing the shared
  transaction-set configuration before `Transaction` restores it.

- **`Iter` / `MatchIterator`** hold internal refcounted links to the
  `rpmts` and `rpmdb` (via `rpmtsLink`/`rpmdbLink` inside
  `rpmtsInitIterator`). They are independent of the `Db` that created
  them and can outlive it.

- **`PackageHeader`** owns a refcounted `Header` (via `headerLink`). Fully
  independent of `Db`, iterators, and transactions.

- **`Keyring`** wraps a refcounted `rpmKeyring`. Clone increments the
  refcount (`rpmKeyringLink`); Drop decrements it (`rpmKeyringFree`).
  Independent of any `Db` or transaction set.

- **`PubKey`** wraps a refcounted `rpmPubkey`. Same Link/Free pattern.
  Independent of the `Keyring` that may contain it.

- **`VerifyOptions`** bundles `VerificationFlags` with an optional
  `Keyring`. When passed to `PackageHeader::from_file()`, the ephemeral
  `rpmts` is configured with the given flags and keyring. It can also be
  applied to a `Transaction` with `set_verify_options()`. Cloning is cheap
  (keyring clone is a refcount increment).

```text
Keyring (standalone, refcounted rpmKeyring)
 ├── PubKey (refcounted rpmPubkey, independent)
 └── KeyringIter<'kr> (borrows Keyring)

VerifyOptions
 └── Keyring (optional, cloned/refcounted)
```

- **`Transaction`** manages file I/O for install elements. When
  `add_install` or `add_reinstall` is called, the file path is stored as
  a `CString` in the transaction's `paths` vec. During `run()`, the
  internal callback trampoline handles `RPMCALLBACK_INST_OPEN_FILE` by
  calling `Fopen` on the stored path and `RPMCALLBACK_INST_CLOSE_FILE`
  by calling `Fclose`. This is transparent to callers.

  The callback trampoline also adapts to the RPM version at compile time.
  RPM >= 4.17 supports notify style 1, where the callback receives an
  `rpmte` (transaction element) as its first argument; older versions use
  style 0, where it receives a `Header`. The `nevra_from_callback()`
  function is cfg-gated to extract the NEVRA string from whichever type
  is provided.

- **`Element<'txn>`** borrows the `Transaction` via `PhantomData`. The
  underlying `rpmte` pointer is owned by the `rpmts` and is invalidated
  when the transaction is dropped (`rpmtsEmpty`).

- **`Problem`** and **`Problems`** are refcounted (`rpmProblemLink`/
  `rpmProblemFree` and `rpmpsFree`). They can outlive the transaction
  that produced them.

## Transaction configuration lifetime

`Transaction` temporarily configures the `rpmts` owned by its `Db`. On
creation it snapshots the transaction flags, verification flags, and current
keyring. `set_verify_options()` can then replace both the verification flags
and keyring for that transaction. When the transaction is dropped, all three
settings are restored; the keyring snapshot uses RPM's own reference-counted
`rpmtsGetKeyring(ts, 0)` handle. This prevents verification configuration from
bleeding into later transactions using the same `Db`. The RPM-specific reasons
for restoring these settings are detailed in [API quirks][transaction quirks].

[transaction quirks]: quirks.md#transaction-flags-are-temporarily-expanded

## Error handling strategy

librpm.rs uses several error types, each scoped to its domain:

| Type | Module | Used by |
|------|--------|---------|
| `Error` / `ErrorKind` | `error` | Most API operations (config, database, macro, transaction) |
| `RpmErrorKind` | `internal::rc` | `Package::from_file()` — maps C return codes |
| `TransactionError` | `transaction` | `Transaction::run()` and `Transaction::check()` — wraps `Problems` |
| `SignError` | `sign` | Signing operations |

`ErrorKind` is `#[non_exhaustive]` to allow adding new variants without
breaking downstream.

## Configuration model

1. **Process-global initialization**: `librpm::init()` (or `init_with()`)
   calls `rpmReadConfigFiles` and `rpmInitCrypto`. Must happen exactly
   once per process, enforced by `ConfigState`.

2. **Macro context**: RPM's key-value configuration system. Internally
   locked since RPM 4.12 (with a per-context recursive mutex). librpm.rs does
   not add additional synchronization as we presume a more recent version
   of librpm.

3. **Per-`Db` transaction set**: Each `Db` owns an independent `rpmts`
   with lazy database open and keyring load. Multiple `Db` instances
   can coexist on different threads for concurrent read-only queries. The
   configured `_dbpath` remains process-global; see [API quirks][dbpath quirk].

[dbpath quirk]: quirks.md#rpm-database-path-is-process-global
