# VirtIO specification port

**Status: tested** pure protocol/queue behavior; fixed policy declarations
**typechecked** independently. No native layout or device execution claim.

This is a modified translation of `virtio-spec` 0.4.0 at
[`ad565bd701e93fa47bc288d45701e1ae53ac3c12`](https://github.com/rust-osdev/virtio-spec-rs/tree/ad565bd701e93fa47bc288d45701e1ae53ac3c12).
Upstream Rust is reference material, not linked into Cathedral. MIT OR Apache-2.0
applies to the translated material; original texts and notices are retained in
[`licenses/rust-osdev/virtio-spec-rs`](../../../licenses/rust-osdev/virtio-spec-rs)
and [`THIRD_PARTY_NOTICES.md`](../../../THIRD_PARTY_NOTICES.md).
The port changes language representation, replaces pointer projections/allocation
with inert plans, adds validation and tests, and retains unknown numeric values.

The core milestone covers VIRTIO-000 and VIRTIO-001; packed queues are covered in
`packed.PORT.md` for VIRTIO-002. `inventory.json` binds all
16 upstream source files and 629 lexical anchors to their exact hashes. There are
539 translated anchors, seven deliberately omitted presentation/sealing anchors,
and 83 explicitly pending transport anchors for VIRTIO-003. Those are
implementation work, not asserted language blockers. The scanner is not a Rust
semantic parser: `raw-shapes.json` additionally inventories private fields;
`check.py` compares original widths/order and enum carriers/discriminants.

`common.omg` preserves device IDs, every status bit and event discriminant.
`features.omg` retains all common and six device-specific masks, including network
bits 64–70, in a semantic `{low:u64, high:u64}` carrier. Unknown bits survive all
raw operations. A four-word view avoids accidental 64-bit truncation. The pinned
network dependency/recommendation matrix is translated; other device feature
implementations have unconditional predicates in the pin. Negotiation computes
an offered/supported intersection, required-mask inclusion, optional network
requirements, and optional modern VERSION_1 admission. It does not implement a
hardware status handshake or establish driver support for a feature.

Rust bitflag facade operations map to raw exact-width operators and explicit
`features_*` machines. `Features {}` is empty; both halves set to
`0xffffffffffffffff` is all. Identity construction/access retains raw bits.
Generic collection `extend`/`from_iter` maps to repeated union, and iteration to
bounded positions 0–127 with `feature_has`; formatted names and string parsing
are deliberately omitted. Scalar wrappers retain their original raw width and
use its exact bit operators. Trait object and macro mechanics are not exposed.

The six `raw_*` files preserve all 20 device records, including private reserved
fields, and their enum/flag constants. `configuration.omg` preserves every
`pci::CommonCfg` field, wide-address composition and ISR masks;
`notifications.omg` preserves split/packed notification encoding. Generation
checking is a finite predicate over two supplied observations, replacing the
upstream unbounded volatile closure retry. It cannot establish freshness without
an owner supplying correctly ordered device observations and a retry budget.

`split.omg` preserves fixed descriptors, available/used headers, used elements,
raw layout geometry, optional event-word offsets, queue admission and slot
arithmetic. Available storage is `4 + 2*q + event*2` bytes. Used wire data is
`4 + 8*q + event*2`; its upstream allocation pads to four-byte alignment, giving
`4 + 8*q + event*4`. All u16 counts are valid raw geometry inputs, including the
upstream tests' 255/257. Actual split queue admission requires a nonzero power
of two no greater than 32768. Wrapping indices explicitly widen then mask to
16 bits, and event suppression uses the unsigned modular difference rule.
Descriptor codecs explicitly emit/read little-endian bytes. Validation checks
split-only flags, next bounds, indirect negotiation and shape, address-span
arithmetic, and used ID/length limits; it does not prove a whole chain acyclic
or establish that a DMA address is accessible.

Upstream `ring`, `ring_mut`, and pointer projection variants map to checked
`RingView` offset/count/element-width facts. Their mutable spelling confers no
mutation authority here. `new`, `try_new`, and allocator-parameter variants map
to an `AllocationPlan` containing length, alignment and zero-initialization.
A future owner must perform allocation and define failure/lifetime behavior.

## Reviewed differences and specification basis

The numeric protocol basis is the [VirtIO 1.3 Committee Specification Draft 01,
6 October 2023](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.html),
sections 2.7 (split rings), 2.9 (notifications), 4.1.4.3 (common configuration),
and 5.2.4 (block configuration). The pin also contains newer feature values;
the older specification is not used to remove them.

* Pinned `src/blk.rs` declares `Config.num_queues: u8`. Section 5.2.4 uses a
  16-bit count. `raw_blk.Config` deliberately preserves the pinned width and
  its expected geometry. `block_queue_count(low, high)` separately decodes
  the modern 16-bit wire count; the test reads 512, which cannot fit u8.
  This raw record must not be used as an unchecked block configuration overlay.
* Pinned `Avail::from_ptr` subtracts two u16 words without first checking a
  four-byte minimum. `Used::from_ptr` can similarly subtract from a zero-byte
  event-enabled region after its congruence check. The pure region validators
  require the exact requested geometry before exposing offsets; tests reject
  zero/short regions and incorrect alignment. No malformed pointer is created.
* The descriptor's pinned Rust type has natural alignment eight; the split
  descriptor table protocol alignment is sixteen. Both facts are retained
  separately. Packed `vsock::Hdr` is size 44/alignment one in the Rust probe.

## Evidence and limits

Run from the Cathedral root:

```sh
python3 tools/ports/virtio/check.py
python3 tools/ports/virtio/semantic-check.py --omega /path/to/omega
python3 tools/ports/virtio/layout-check.py --omega /path/to/omega
```

`core.vectors.json` contains 343 expected values. The probe compiles the actual
pinned Rust path dependency for `x86_64-unknown-uefi` and reads 255 exported LLVM
constants: every record size/alignment, accessible public-field offsets, and
all 128-bit feature masks. These agree; private-field offsets remain
source-derived expectations. The probe's lockfile fixes transitive dependencies.
Its host executable separately exercises 20 actual upstream available/used
allocations at counts 0, 255, 256, 257 and 65535, checking geometry, zeroed data,
and optional event words. This includes all cases of the pin's two allocation
tests. The upstream feature test's assertions are ported into Omega.

Fresh Omega `eaa7993a23623cd8fabf45350340479c5c9c7879`, binary SHA-256
`2ac9ce5859896c4689ed54ac55f79dd211050a530fe03e3d475cc543b9b523c4`, passes
semantic evaluation of eight behavior groups. Eight independent negative
controls alter expected behavior inside the group bodies; each evaluates to
failure and is rejected by the success contract. This is semantic evaluation,
not a native executable test. All 24 named layout policy declarations also
pass a separate source check. They are requested plans, not selected or
observed native storage layouts. Omega ABI comparison has NOT RUN.

There is one narrow compiler diagnostic: importing `layouts` into the entire
semantic fixture produces three core `PlacedField` read/take/write trait
callable-generic-arity errors. The same policies and the semantic fixture each
check independently. `layout-check.py --combined` reproduces that composition
failure; `layout-import-diagnostic.txt` records it. It does not prevent the pure
port and is not evidence that Layout itself is unavailable.

No live device, volatile access, DMA, barriers, native queue concurrency, or
production integration has been exercised. Before changing the pin, refresh the
full inventory, re-audit private schemas/discriminants and dependency predicates,
regenerate reviewed vectors, rerun Rust observations and all semantic controls,
and review any deliberate difference against the new source and specification.
