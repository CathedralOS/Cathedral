# Storage layout lab

This host-only Rust experiment measures a possible DBFS content representation.
Start at [main.rs](main.rs): choose chunk/inline policies, execute
[workloads](workloads.rs), report their costs. [chunks.rs](chunks.rs) owns
immutable chunk interning, flat manifests, range replacement and explicit
retained-version reachability. There are no guest dependencies or storage-service
protocol changes. The eventual engine belongs to the platform; this tool is a
disposable measurement baseline for the [storage proposal](../../wiki/proposals/0002_dbfs_storage_foundation.md).

From the repository root:

```powershell
python tools/storage-lab/run.py
```

The runner uses this folder's pinned Rust toolchain and locked dependencies,
checks that all 48 cases completed, then saves `measurements.csv` and
`context.json` under `build/storage-lab/`. Every workload checks reconstructed
bytes; retained-history cases check every historical version. Context records
the compiler, host, revision, dirty paths and exact input source hashes.

For policy checks and lint, run inside this folder:

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

## What is measured

The matrix compares 4/16/64 KiB fixed storage chunks and inline limits 0/128 bytes.
BLAKE3 hashes chunk and manifest bytes using distinct fixed domain prefixes.
The official crate selects available CPU implementations; this test does not
enable Rayon or measure a forced-portable/SIMD comparison. Hashing's internal
1 KiB chunks are unrelated to these storage boundaries.

The manifest encoding is exactly 17 header bytes (mode, logical length, chunk
size, count), followed by inline bytes or 32-byte chunk digests. It is experimental,
packed and entirely in memory. Full flat manifests are rebuilt on each edit.
Empty contents use an inline empty manifest. This is not a selected durable codec.

| Column | Meaning |
| --- | --- |
| `logical_bytes` | Input bytes, or bytes changed for edits; copied logical contents for the copy row |
| `hashed_payload_bytes`, `hashed_manifest_bytes` | Bytes submitted to hashing during the measured operation, excluding fixed domain prefixes |
| `new_payload_bytes`, `new_manifest_bytes` | Unique packed bytes interned during the operation, before collection |
| `live_payload_bytes`, `live_manifest_bytes` | Reachable packed bytes after the row's explicit retention policy |
| `live_chunks`, `live_versions` | Unique reachable chunks/manifests; not object/identity counts |
| `elapsed_us` | One host elapsed sample, including hashing, allocations, map lookup and manifest/reference copies; excludes input generation, readback and collection |

For 64 edits, allocation/hash counts exclude the initial object and include all
edits. Collection occurs after the batch for both retention policies, so latest-only
does not model incremental GC or peak-RAM savings. The two single-patch cases and
prefix insertion retain both old and new versions; they each start with only the
original retained. The copy row clones 100 version descriptors: it measures zero
new content, but excludes the new object/head records a real engine must write.
Descriptor cloning still consumes RAM and time proportional to the flat manifest.

Fixtures are deterministic nonzero-seeded xorshift bytes, with distinct seeds for
tiny objects and edits. They intentionally avoid repeated zero-filled chunks that
would make deduplication look unrealistically effective. They are synthetic,
uncompressed, single-threaded workloads, not a representative user-data corpus.

## Observed baseline

The initial run completed all 48 cases. These deterministic packed-byte results
come from the checked-in fixtures and encoding; regenerate CSV/context with the
runner rather than treating host timing as a stable performance claim.

For a 100-byte aligned replacement inside the 10 MiB fixture:

| Chunk size | New payload | New flat manifest |
| --- | --- | --- |
| 4 KiB | 4,096 bytes | 81,937 bytes |
| 16 KiB | 16,384 bytes | 20,497 bytes |
| 64 KiB | 65,536 bytes | 5,137 bytes |

The boundary-crossing variant doubles the payload in each row. With 4 KiB chunks,
64 small edits create 266,240 payload bytes but 5,243,968 manifest bytes. This
justifies comparing a chunk-reference tree before choosing chunk size; the flat
representation can dominate precisely where small chunks save payload work.

For 4,096 distinct 32-byte objects, packed contents plus manifests occupy 331,776
bytes without inlining and 200,704 with the 128-byte threshold, before any metadata
engine or allocation overhead. The copy case interns no new content or manifests,
but has no object/head-table accounting. Inserting one byte at the beginning of
the large fixture creates 10,485,761 new payload bytes for every fixed chunk size.
That exposes a boundary-shift cost, without establishing that all workloads need
content-defined chunking.

## Limits and next evidence

Packed byte counts exclude hash-to-location index keys/values, object heads,
namespace entries, allocator metadata, device alignment, logs, compression,
encryption, replication and fragmentation. Inline payload appears in manifest
bytes; add the two live-byte columns when comparing total representation size.
These counts are not disk-space or process-memory measurements.

No file/device IO, durable publication, transactions, authorization, decoder,
fault recovery or concurrent readers are implemented here. `retain` is a bounded
host sweep over explicitly provided versions, not a proposed production GC. A
version belongs to the store/policy that created it; passing versions between
stores or using collected versions is outside this internal experiment.

Next compare chunk-reference trees and actual metadata engine candidates on these
traces, then add metadata scans, allocation/recovery faults and near-full-device
workloads. The existing [catalog recovery tests](../../source-rs/platform/libraries/object-store/CHARTER.md)
remain the source of durability evidence for the current guest implementation.
