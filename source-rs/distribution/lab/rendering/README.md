# Rendering lease investigation

This is a distribution-owned research fixture. It compares the cost of a drawing
hierarchy separately from the authority needed to write its pixels. It does not
choose Cathedral's production surface, compositor or component admission model.

The [rendering and composition proposal](../../../../wiki/proposals/0000_rendering_and_composition.md)
owns the candidate platform architecture informed by this fixture. This page
remains the owner of measured costs, commands and implementation limitations.

Start at [lib.rs](lib.rs) for the three shared pieces: [scene resolution](scene.rs),
[row delegation](rows.rs) and [pixel operations](pixels.rs). The two execution paths
are [bench.rs](bench.rs) for host measurements and [main.rs](main.rs) for guest role
dispatch. [guest.rs](guest.rs) coordinates the real page handoff; its
[bounds probe](guest/bounds.rs) compares checked spans with native stores. The
[boot fixture](../../../kernel/boot/uefi/lab/rendering.rs) supplies grants, checks
kernel counters and verifies reclamation. None of these are public platform APIs.

## Repeat the investigation

From the repository root:

```text
python tools/rendering-lab/run.py
python tools/boot-harness-rs/run.py --smoke --timeout 60
python tools/boot-harness-rs/run.py --smoke --release --timeout 60
```

From `source-rs/`, `cargo test --locked` runs the pure rendering tests alongside
existing policy tests. `cargo test --locked -p cathedral-rendering-lab --features host`
also checks benchmark accounting. The compile-fail example checks that splitting
a row consumes the parent's Rust access. Runtime tests cover neighboring spans,
odd sizes, nested translation/clipping, exclusive overlap rejection, invalid
sources rejected before any output write, and withholding an aborted back buffer.

## Workload and results

Four opaque, non-overlapping leaf rectangles cover a frame, beneath 1, 4 or 8
full-frame ancestors. Every path renders the same scalar RGB pattern and verifies
every final pixel. Sizes are 64x16, 1920x1080 and 1919x1079. Containers describe
translation and clipping; they need no pixel storage in the flattened paths.

| Strategy | Pixel storage | Intermediate copies |
| --- | --- | --- |
| `rows` | One final back buffer, disjoint checked row borrows | None |
| `copy-tree` | Packed leaves and an image at every ancestor | Leaves to parent, then each parent to its parent |
| `flat-packed` | Packed leaves and one final back buffer | Each visible leaf directly to the final target |
| `flat-pages` | Same flattened path, each allocation rounded to 4 KiB | Same one-pass leaf composition |

`copy-tree` is an explicit naive baseline retaining every ancestor image, not a
claim about how an optimized existing compositor works. `rows` runs cooperating
safe Rust in one process. The host's `flat-pages` models allocation sizes only:
its vectors are neither page aligned nor hardware isolated. Guest tests below
exercise actual separate address spaces and mappings at the small fixture size.

Observed on 2026-10-10 UTC, Windows x86-64, Intel family 6/model 151/stepping 2,
Rust 1.95.0-nightly (c7f5f3e0d). Each time is the median of five release samples.
For **1920x1080, depth 8**:

| Strategy | Pixel storage bytes | Copied bytes/frame | Host median microseconds |
| --- | ---: | ---: | ---: |
| `rows` | 8,294,400 | 0 | 1,383 |
| `copy-tree` | 74,649,600 | 66,355,200 | 16,533 |
| `flat-packed` | 16,588,800 | 8,294,400 | 3,349 |
| `flat-pages` | 16,601,088 | 8,294,400 | 3,425 |

All paths additionally render 8,294,400 bytes. Allocation lengths are also the
requested zero-initialization byte counts. The timer includes metadata resolution,
allocation/initialization, rendering and copying; it excludes validation and
deallocation. Counts describe logical pixel work, not cache/DRAM traffic, allocator
overhead or RSS. These single-run host timings are illustrative, not a speed
guarantee or a GPU/guest benchmark. The runner saves all 36 rows and machine/tool
context under `build/rendering-lab/`.

At 64x16, four 1,024-byte leaves require four isolated 4,096-byte allocations:
12,288 bytes of padding. Including the final target, this is 20,480 bytes versus
8,192 packed or 4,096 for direct rows. At 1919x1079 the corresponding total padding
is 11,704 bytes. Page padding is proportionally expensive for small independent
buffers; ancestor depth adds no copies to either flattened path.

## Actual guest boundaries and synchronization

The producer fills four private pages and seals them to the consumer. One
32-byte IPC offer carries their handles. The consumer accepts four immutable
leases, composes 4,096 bytes into one private target, checks every pixel, drops all
readers, then sends an 8-byte completion. Only then does the producer release its
backing. There is no per-row IPC or per-ancestor pixel transfer in this fixture.

The kernel asserts these runtime page counters on two complete sessions:

| Role | Deferred memory calls | Pages allocated | Leaf maps | RO edits | Leaf unmaps |
| --- | ---: | ---: | ---: | ---: | ---: |
| Producer | 12 | 4 | 4 | 4 | 4 |
| Consumer, including separate direct-row trial | 12 | 2 | 6 | 0 | 6 |
| Disposable page-fault probe | 1 | 1 | 1 | 0 | 1 |

The direct-row trial alone allocates/maps/unmaps one page in two deferred calls.
Subtracting it leaves **22 deferred calls and 22 leaf PTE edits** for the flattened
frame's full allocate/seal/accept/release cycle. There are two frame IPC deliveries,
plus two startup deliveries. Address/size queries, task admission, page-table
allocation, CR3 changes, TLB behavior and scheduler cost are outside these counters.
This measures a fresh-buffer cycle; persistent pools could change the tradeoff.

The row probe rejects a checked write past a 128-byte span, then deliberately
writes byte 128 through a native pointer within the same allocated page. That
store succeeds and changes the neighboring sentinel. A separate child stores
beyond its one-page mapping and faults; its supervisor survives. All frames and
heap return to baseline. Existing memory smoke tests additionally exercise stale
handles, budget failures, RO/NX faults, owner/reader death, pinned accepted readers
and cancellation/replacement. The rendering probe does not add a new revocation
or frame cancellation protocol.

## What this leaves open

Flattening the hierarchy removes intermediate pixel copies without requiring
writers to share a writable final buffer. Direct rows remove the remaining
composition copy for this workload, but arbitrary native code can bypass the Rust
wrapper. Exact sub-page authority would need an admitted proof, a suitably enforced
sandbox, or another enforcement mechanism. Mapping a writable page does not grant
hardware-enforced byte ranges within it. See the
[owner question](../../../../OWNER_QUESTIONS.md#writable-row-spans-and-component-admission).

These are offscreen back buffers. Scanout/presentation, tearing, alpha blending,
overlapping windows, scaling, GPU work, partial damage, parallel writers and
deadline recovery are unmeasured. In particular, failing to publish a private
back buffer says nothing about safely aborting a writer already modifying visible
scanout. The next architecture decision needs both an admission/fault model and
representative compositor workloads; the current result does not freeze either.
