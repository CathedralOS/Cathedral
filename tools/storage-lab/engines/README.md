# Transactional engine comparison

This host experiment puts the layout lab's exact byte records into existing
engines. It tests durable publication, ordered scans, reopen, process termination
and capacity refusal. It does not select Cathedral's production engine or format.
The [storage proposal](../../../wiki/proposals/0002_dbfs_storage_foundation.md)
owns the architectural requirements; the guest catalog remains unchanged.

Start at [engines.rs](engines.rs): dispatch measurements or recovery, then sequence
trace creation, commits, observation, checkpoint and reopen. [trace.rs](trace.rs)
constructs identical records for every engine. [adapters.rs](adapters.rs) owns
transaction and scan operations; [backend.rs](backend.rs) owns redb's observable
host IO. [recovery.rs](recovery.rs) owns child-process gates and capacity failure.
[run.py](run.py) builds, checks the comparison matrix and kills/reopens writers.
`mod.rs` only wires these leaves together.

## Reproduce

From the repository root:

```powershell
python tools/storage-lab/engines/run.py
```

The optional `engines` Cargo feature builds pinned redb 4.4.0 and rusqlite 0.40.2
with bundled libsqlite3-sys 0.38.2 (SQLite 3.53.2 in the recorded run). Building
bundled SQLite needs a host C compiler. The normal layout runner still needs only
BLAKE3. The locked crate dependencies belong to this tool, outside `source-rs/`.

Each run creates a new directory under `build/storage-engines/` and keeps its DBs,
`measurements.csv` and `context.json`. Rust refuses an existing destination. Context
records host/compiler, revision, tracked dirty paths, source hashes, runtime SQLite
version and completed recovery checks. Failed runs can leave DBs without a final
report. No old DB is reused or deleted. Avoid running the two lab runners together:
they build different features of the same executable in the same target directory.

From `tools/storage-lab/`:

```powershell
cargo test --locked --features engines
cargo clippy --locked --features engines --all-targets -- -D warnings
cargo fmt --check
rustfmt --edition 2024 --check engines/engines.rs chunk_tree/chunk_tree.rs
```

## Controlled workloads and engine settings

There are 48 cells: three engine configurations, two layouts (flat and tree-32),
two payload chunk sizes (4/16 KiB), and four workloads:

- `tiny`: 4,096 keyed 32-byte values in one transaction. This raw-record control
  repeats across layout/chunk settings; it does not use content manifests.
- `single`: one byte replaced at offset 512 in a deterministic 10 MiB object.
- `separate`: 64 adjacent one-byte edits, each published in its own transaction.
- `batch`: the same edits published in one transaction.

The latter three initialize the same content representation as the layout lab.
Keys distinguish hashed chunks, manifests and tree nodes. Each publication commits
changed content records, the current head and one sequence-keyed change record
together. All immutable history and change records remain retained. One of the
64 edits already matches its original byte; it still publishes a change record.
Separate and batched publication deliberately expose different intermediate states.

All payloads live inside the same database table as metadata for this baseline.
There is no external blob allocator or cross-store atomicity claim. The host model
verifies reconstructed contents; each engine's complete record set is compared
byte-for-byte with that model before and after reopen. An ordered event-prefix
scan is checked separately. These scans are not an implementation of DBFS queries.

| Configuration | Settings |
| --- | --- |
| redb | One byte-key/value table, `Durability::Immediate`, 8 MiB cache, default 4 KiB pages, private file backend |
| SQLite DELETE | One `WITHOUT ROWID` byte-key/value table, 4 KiB pages, rollback journal, `synchronous=FULL`, 8 MiB cache target, mmap disabled |
| SQLite WAL | Same table/page/cache settings, WAL with `synchronous=FULL`, automatic checkpoints disabled, explicit truncating checkpoint |

SQLite's effective pragmas are checked on open, and a blocked checkpoint fails the
run. redb's backend serializes IO through a host mutex and calls `File::sync_all`;
it does not implement interprocess file locking. Every DB has one owner, and a
recovery writer is killed and reaped before the verifier opens it. This adapter
is not suitable for arbitrary shared database files.

## What the columns measure

| Columns | Boundary |
| --- | --- |
| `transactions`, `submitted_bytes` | Measured publication count and sum of submitted key/value lengths, including heads/events; initialization excluded |
| `before_*`, `after_*` | Main file and summed WAL/SHM/journal logical lengths around measured publications |
| `peak_observed_bytes` | Maximum sum sampled before and after commits; not the transient within-transaction peak |
| `checkpoint_*` | Logical lengths after explicit checkpoint; redb has no separate checkpoint operation here |
| `commit_us` | Sum of engine transaction times, including requested durability; excludes trace construction and verification |
| `scan_us` | Ordered event-prefix scan and materialization, excluding oracle comparison |
| `checkpoint_us`, `reopen_us` | Explicit checkpoint and open/initialization time; subsequent verification excluded |
| `records` | Final retained table entries |
| `backend_write_bytes`, `backend_writes`, `backend_syncs` | redb successful backend writes and syncs during measured transactions; SQLite reports `NA` |

Backend bytes count requested writes, including rewrites, not physical media IO.
They exclude initialization, verification, checkpoint, close, reads and file-length
changes. File lengths include unused/reserved address space and can be sparse;
they are not allocated disk blocks or live data size. SQLite DELETE journals can
exist and disappear inside a transaction, outside our sampling points. WAL length
cannot stand in for total writes. A SQLite VFS counter would be needed for a fair
write-amplification comparison.

## Recorded evidence

The Windows x86-64/Rust 1.94.0 run `20261016T171014357222Z` passed all 48 cells and
nine recovery/capacity checks. Representative 4 KiB chunk results:

| redb publication | Submitted bytes | Backend write bytes | Write calls | Syncs |
| --- | ---: | ---: | ---: | ---: |
| Flat, one small edit | 86,181 | 172,352 | 11 | 1 |
| Tree-32, one small edit | 6,555 | 61,760 | 15 | 1 |
| Flat, 64 separate edits | 5,429,449 | 11,505,664 | 842 | 64 |
| Tree-32, 64 separate edits | 413,011 | 3,960,832 | 963 | 64 |
| Flat, one batch of 64 edits | 86,181 | 176,448 | 12 | 1 |
| Tree-32, one batch of 64 edits | 6,555 | 65,856 | 16 | 1 |

The tree's packed-record saving survives the engine, but engine overhead reduces
its proportional benefit. Batching saves both backing writes and durable commit
boundaries when the application permits one publication. It does not make a small
write free or establish the best chunk size for general workloads.

After a single tree-32 edit, redb's main file length was 33,689,600 bytes. SQLite's
post-checkpoint main length was 12,386,304 bytes in both modes. Before checkpoint,
SQLite WAL retained an additional 90,480 bytes of WAL/SHM while its main length
was 12,374,016 bytes. These are extent observations, not a disk-utilization ranking.
The 4,096 tiny records occupied a 1,056,768-byte redb extent versus a 249,856-byte
SQLite main extent after checkpoint.

The CSV contains timing samples, but this is one host, synthetic uncompressed
data, warm OS caches, default engine tuning and different SQL/KV adapter work.
No throughput or latency ranking is justified. Caches have similar configured
budgets, not proven equal memory use. Neither peak RAM nor physical read/write
traffic is measured.

## Recovery boundaries

For each configuration, the runner kills a separate writer with no destructor
cleanup at two gates: after staging all records but before commit, and after a
durable commit returns but before application acknowledgement. A fresh process
must recover exactly the old or new complete record set, respectively. This tests
process death and lost acknowledgement; it does not inject torn writes, reordered
device IO, power loss or arbitrary interruption inside commit.

The capacity case commits a baseline, caps further growth (redb backend length
limit or SQLite `max_page_count`), and attempts a transaction containing changed
head/event/content plus a 16 MiB value. That transaction must fail. Reopening must
recover the baseline and accept a subsequent small transaction. The injected
redb limit is removed at reopen; SQLite's page cap is connection-local too. This
checks refusal/rollback and later usability after a limit is lifted, not useful
progress while a real disk remains full. WAL/journal space is not capped by the
SQLite page limit.

Active engine read transactions, concurrent writers, long-lived reader pressure,
GC, allocator fragmentation, recovery-time bounds and sustained near-full-store
progress remain unmeasured. Retaining immutable model history is not a substitute
for testing the engine's active snapshots.

## Reuse and portability assessment

| Candidate | Why retain it in the study | Main integration boundary |
| --- | --- | --- |
| redb 4.4.0 | Reusable Rust transactional B-tree; measured here; experimental `no_std` backend entrance | Allocator, synchronization/progress, bounded IO/space, backend durability and experimental API stability |
| SQLite | Established journal/WAL machinery; measured in both modes | C runtime/build plus a Cathedral VFS implementing IO, locking and WAL coordination |
| LMDB | Useful mapped copy-on-write reference; not benchmarked here | File mapping, locking and reader lifetime need an OS adaptation |
| RocksDB | Useful LSM/compaction reference; not benchmarked here | Broader filesystem/threading integration and maintenance/resource budgeting |

The [compile probe](portability/lib.rs) successfully checks redb's backend entrance
on both bare-metal targets. From `tools/storage-lab/engines/portability/`:

```powershell
cargo check --locked --target x86_64-unknown-none
cargo check --locked --target aarch64-unknown-none
```

The probe disables default features and enables `experimental-api-5`. Upstream
requires allocation, atomic compare-and-swap support and abort-on-panic. Its
`no_std` synchronization uses spinning locks: scheduler progress and contention
need explicit review before integration with Cathedral's CPU budgets. Compilation
does not supply an allocator, link a runnable guest, prove synchronization or
implement a block/capability adapter. It also does not make rusqlite portable.

The next useful experiment is a bounded redb backend with a simulated volatile/
durable device image: inject failures at writes/flushes, recover old-or-new state,
and measure space under retained readers and repeated reclamation. This is a
candidate for deeper testing, not a production commitment. If blobs later move
outside the engine, publish-before-reference ordering and orphan recovery must
be tested again across that new boundary. A guest storage-provider integration
should follow this evidence; the kernel need not acquire database policy.

Primary sources and dependency provenance:

- [redb 4.4.0 backend contract](https://docs.rs/redb/4.4.0/redb/trait.StorageBackend.html)
  and [builder](https://docs.rs/redb/4.4.0/redb/struct.Builder.html).
- redb's published source commit: [design](https://github.com/cberner/redb/blob/7061f7e60d4573311c0636e161d417b93e9fd2b1/docs/design.md),
  [`no_std` requirements](https://github.com/cberner/redb/blob/7061f7e60d4573311c0636e161d417b93e9fd2b1/README.md#no_std),
  [synchronization](https://github.com/cberner/redb/blob/7061f7e60d4573311c0636e161d417b93e9fd2b1/src/sync.rs).
- [SQLite atomic commit](https://www.sqlite.org/atomiccommit.html),
  [WAL](https://www.sqlite.org/wal.html) and [VFS](https://www.sqlite.org/vfs.html).
- [LMDB API](https://github.com/LMDB/lmdb/blob/mdb.master/libraries/liblmdb/lmdb.h)
  and [RocksDB overview](https://github.com/facebook/rocksdb/wiki/RocksDB-Overview).

The tool links upstream crates; it contains no translated engine implementation.
redb is MIT OR Apache-2.0; rusqlite is MIT. Exact resolved packages and checksums
are in the two lockfiles. Existing owner questions about isolation, retention,
authority and deduplication still apply; this experiment resolves none of those
contracts and introduces no new owner-level choice.
