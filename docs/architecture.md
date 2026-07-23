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

librpm.rs decomposes this into purpose-specific types:

| Rust type | Wraps | Role |
|-----------|-------|------|
| `Db` | `rpmts` (1:1) | Read-only database queries |
| `Transaction<'db>` | borrows `Db`'s `rpmts` | Install/erase lifecycle (mutating) |
| `PackageHeader::from_file()` | ephemeral `rpmts` | Read a `.rpm` file (no DB needed) |
| `Keyring` | `rpmKeyring` (standalone) | Trusted key management |
| `PubKey` | `rpmPubkey` (standalone) | Individual public key |
| `VerifyOptions` | flags + optional `Keyring` | Verification configuration |
| `VerificationFlags` | `rpmVSFlags_e` | Control verification checks |
| `Spec` | `rpmSpec` (via `rpmts`) | Spec file parsing and building (`build` feature) |
| `sign::sign_package()` | no `rpmts` | Signing via `librpmsign` (`sign` feature) |

The internal `TransactionSet` wrapper owns the `rpmts` pointer and is
always held inside a `Db`. External callers never interact with
`TransactionSet` directly.

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
  This prevents queries while a transaction is active, which is necessary
  because `rpmtsRun` mutates the `rpmts` state (flags, ordering, problem
  set). Users must collect query results before creating a transaction.

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
  `rpmts` is configured with the given flags and keyring. Cloning is
  cheap (keyring clone is a refcount increment).

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

`TransactionError` wraps a `Problems` set rather than a string, so
callers can programmatically inspect individual problems (type, affected
package, disk need, etc.).

## Configuration model

1. **Process-global initialization**: `librpm::init()` (or `init_with()`)
   calls `rpmReadConfigFiles` and `rpmInitCrypto`. Must happen exactly
   once per process, enforced by `ConfigState`.

2. **Macro context**: RPM's key-value configuration system. Internally
   locked since RPM 4.12 (per-context recursive mutex). librpm.rs does
   not add additional synchronization.

3. **Per-`Db` transaction set**: Each `Db` owns an independent `rpmts`
   with lazy database open and keyring load. Multiple `Db` instances
   can coexist on different threads for concurrent read-only queries.
