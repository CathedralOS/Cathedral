# Storage layout lab

This host-only Rust experiment compares content representations for DBFS. Start
at [main.rs](main.rs): choose policies, execute [workloads](workloads.rs), report
costs. [chunks.rs](chunks.rs) stages payload changes, publishes version manifests,
reads ranges and retains live contents. Descend into
[chunk_tree/chunk_tree.rs](chunk_tree/chunk_tree.rs) for fixed-fanout construction,
path replacement, range traversal and shared-node reachability. `mod.rs` is wiring.
The eventual engine belongs to the platform; this is a measurement tool for the
[storage proposal](../../wiki/proposals/0002_dbfs_storage_foundation.md).

The optional [engine comparison](engines/README.md) persists identical records
through redb and SQLite (DELETE/WAL), measuring file extents and redb backend IO,
with process recovery and capacity-refusal checks. Its named entrance is
[engines/engines.rs](engines/engines.rs). The representation measurements below
remain a separate in-memory experiment.

The [stress lab](engines/stress/README.md) adds competing publication requests,
active snapshots, expected-revision conflicts, simulated crash cuts and bounded
reclamation. It tests a small record protocol, separate from the chunk layouts.

From the repository root:

```powershell
python tools/storage-lab/run.py
```

The runner uses this folder's pinned Rust toolchain and locked dependencies,
verifies all 270 cases and cross-layout payload equivalence, then saves
`measurements.csv` and `context.json` under `build/storage-lab/`. Every workload
checks reconstructed bytes; retained-history cases check every historical
version. Context records the compiler, host, revision, dirty paths and source
hashes, including the tree's named body. Run policy checks inside this folder:

```powershell
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
rustfmt --edition 2024 --check chunk_tree/chunk_tree.rs
```

## Representation and operations

The matrix compares 4/16/64 KiB payload chunks and inline limits 0/128 bytes for
three layouts: a flat manifest, a tree with fanout 32, and a tree with fanout 128.
BLAKE3 uses distinct domain prefixes for chunks, version manifests and tree nodes.
The official crate selects available CPU implementations; no Rayon or
forced-portable/SIMD comparison is enabled. Internal BLAKE3 chunk boundaries do
not determine storage boundaries.

| Record | Experimental packed encoding |
| --- | --- |
| Inline version | 17-byte header (mode, logical length, chunk size, count), then inline bytes |
| Flat version | Same header, then one 32-byte digest per payload chunk |
| Tree version | Same header, then fanout:u32 and root digest:32; 53 bytes total |
| Tree node | level:u8, count:u32, then one 32-byte digest per child; at most the selected fanout |

Level zero points to payload chunks; higher levels point to nodes. Positions
follow fixed fanout, with a short final node at any level. Nodes are immutable
and interned by digest. A same-length replacement rebuilds only affected paths.
A batch validates every range first, applies overlapping edits in input order,
interns each touched payload chunk once, and rebuilds each affected node once.
Insertion still uses a full `put`, preserving the original fixed-boundary baseline.

Version descriptors contain only a digest and logical length for both layouts.
Flat lists now live in the store instead of being cloned into every descriptor;
this removes the old copy-row RAM/time artifact without changing flat packed-byte
counts. Copying contents still does not include creating an actual object/head
record. Empty contents use an inline empty manifest in all layouts.

## Measurements and workloads

| Column | Meaning |
| --- | --- |
| `layout`, `chunk_bytes`, `inline_limit` | Representation policy; fanout is in the tree layout name |
| `logical_bytes` | Input bytes, bytes edited, bytes requested by a range read, or logical contents copied |
| `hashed_payload_bytes`, `hashed_manifest_bytes` | Bytes submitted to hashing, excluding fixed domain prefixes; manifest bytes include tree nodes |
| `new_payload_bytes`, `new_manifest_bytes` | Unique packed bytes interned during the operation, before collection |
| `live_payload_bytes`, `live_manifest_bytes` | Reachable packed bytes after explicit retention; metadata includes retained version manifests and tree nodes |
| `live_chunks`, `live_versions`, `live_tree_nodes` | Distinct retained payloads, version manifests and tree nodes; not object IDs |
| `read_metadata_bytes`, `read_metadata_records` | Unique whole packed metadata records needed for the range read under the cold authentication model |
| `read_payload_bytes`, `read_payload_records` | Unique whole payload chunks covering that range; inline contents are counted as metadata |
| `elapsed_us` | One host elapsed sample, including resident-map access and allocations; excludes verification and collection |

There are fifteen cases for each of eighteen policies:

- 4,096 distinct 32-byte objects and a sequential 10 MiB object.
- One hundred references to the same content version.
- Aligned 1-byte and 100-byte edits, and a boundary-crossing 100-byte edit.
- Sixty-four scattered 100-byte edits, separately published with all history,
  separately published retaining only the final version, and published in one batch.
- Sixty-four adjacent one-byte edits within one chunk, separately published or batched.
- Three 100-byte range reads: aligned, crossing a payload boundary, and at the tail.
- One-byte prefix insertion.

Edit counters exclude initial object creation. Collection occurs after each case,
so latest-only does not model incremental GC or peak-RAM savings. Single-edit and
insertion cases retain the old and new versions. Batch and separate/latest-only
cases have identical final packed representations; the runner checks this.
Separate publication exposes intermediate versions; batching exposes only the end
state, so applications must allow that difference in commit granularity.

Range costs assume no metadata/payload cache across calls, and reuse each record
within a call. Because the flat manifest is one hashed record, authenticating it
requires the whole list. A tree requires its version record and the selected paths.
These are structural byte requirements: the host reads decoded resident structures
and does not fetch or rehash encoded records during reads. Cached metadata, partial
verification proofs, read-ahead, device alignment and IO latency are not modeled.
Non-read rows leave read columns zero; this is not a claim that mutation needs no reads.

Fixtures use deterministic nonzero-seeded xorshift bytes to avoid artificial
zero-filled deduplication wins. They are synthetic, uncompressed workloads.
Some edits can already equal the old byte; hashing work and new stored bytes
therefore need not match. The 64 local edits change contents on 63 publications.

## Observed comparison

For a 1-byte or aligned 100-byte edit in the 10 MiB fixture:

| Chunk size | New payload, every layout | Flat metadata | Tree, fanout 32 | Tree, fanout 128 |
| --- | --- | --- | --- | --- |
| 4 KiB | 4,096 bytes | 81,937 bytes | 2,212 bytes | 4,799 bytes |
| 16 KiB | 16,384 bytes | 20,497 bytes | 1,727 bytes | 4,319 bytes |
| 64 KiB | 65,536 bytes | 5,137 bytes | 1,247 bytes | 4,223 bytes |

The tree reduces metadata amplification, not payload chunk rounding. At 4 KiB,
64 scattered edits create 266,240 new payload bytes in every layout. Their
separately published metadata totals are 5,243,968 / 135,424 / 307,136 bytes for
flat / tree-32 / tree-128. Retaining all 65 versions uses 5,325,905 / 220,473 /
389,854 metadata bytes; retaining only the latest uses 81,937 / 85,049 / 82,718.
The tree pays a modest baseline overhead to share metadata between versions.

For 64 adjacent one-byte edits with 4 KiB chunks, separate publication creates
258,048 new payload bytes (63 different chunks); batching creates 4,096. Tree-32
metadata falls from 139,356 bytes to 2,212. Scattered edits do not gain the same
payload saving: this trace touches different chunks, but batching still shares
ancestor rebuilds. A fanout-128 scattered batch touches every leaf and creates
82,718 metadata bytes, slightly more than the 81,937-byte flat manifest.

An aligned 100-byte cold range read requires 81,937 metadata bytes in one flat
record, 2,212 bytes in four tree-32 records, or 4,799 bytes in three tree-128
records, plus one 4 KiB payload. Smaller fanout saves packed bytes here but adds a
level of dependent lookup. This is not enough evidence to select a device layout.
A full fanout-128 node is 4,101 packed bytes, already larger than a 4 KiB block;
physical packing and allocation units can materially change these comparisons.

Inline tiny contents still avoid extra records. Fixed chunks in every layout
still lose reuse after the one-byte prefix insertion: 10,485,761 new payload bytes.
The tree addresses reference-update cost; it does not address shifted chunk boundaries.

## Limits and next evidence

Packed counts exclude placement-index entries, object heads, namespaces,
allocator overhead, device alignment, journals, compression, encryption,
replication and fragmentation. They are neither disk-space nor process-memory
measurements. No file/device IO, durable publication, transaction isolation,
authorization, decoder, recovery or concurrent readers are implemented. `retain`
is a bounded host sweep, not a production GC. Versions belong to their originating
store/policy and must remain retained while used.

The thirteen tests cover exhaustive small edits, inline/tail boundaries, tree
height changes, range reads, overlapping batches, invalid late edits, canonical
rebuild equivalence, shared paths, and collection with historical/shared nodes.
No new owner semantics are selected by this experiment.

The [engine comparison](engines/README.md) now exercises durable record publication,
ordered scans, reopen, process death and capacity denial. Physical allocation,
arbitrary IO faults, active-reader pressure and near-full-store progress remain
next evidence. This positional content tree is not a general B-tree database engine. The existing
[catalog recovery tests](../../source-rs/platform/libraries/object-store/CHARTER.md)
remain the durability evidence for the current guest implementation.
