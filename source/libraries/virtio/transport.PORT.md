# Transport shapes — VIRTIO-003

This modified translation uses the same pin and retained MIT OR Apache-2.0
notices as `PORT.md`: `src/pci.rs`, `src/mmio.rs`, `src/volatile.rs`, and the
shared configuration/notification definitions. Every source anchor now has a
reviewed translation or deliberate presentation/sealing omission. The full
inventory has 16 files, 620 translated anchors, nine omissions, and no pending
or blocked anchors. Source coverage does not establish an operational driver.

`transport_pci.omg` preserves Cap, Cap64, NotifyCap and CfgCap raw fields, and
CapData facts with an explicit optional-multiplier tag. CapCfgType preserves all
raw u8 values, including unknown values; the actual Rust data-carrying enum is
two bytes and is explicitly not the one-byte wire field. The decoder accepts
unknown types as the pin does. Region admission separately requires a supported
type and BAR 0–5. SharedMemory is 8 and Vendor is 9 in this pin.

The two upstream `read` operations become bounded decoding of a copied 24-byte
prefix plus finite configuration-read plans. A header request reads one dword;
a body plan lists four, five (Notify), or six (SharedMemory) dwords in order.
Only `count` entries are active. The supplied capability offset must be aligned
and inside conventional PCI configuration space; declared length must fit that
space. Captured bytes must cover every consumed field. Larger declared lengths
are accepted without requiring unconsumed extension bytes in the copied prefix.
This adds explicit bounds absent from the live upstream accessor and never
issues configuration cycles. Raw padding, next pointer and unknown types survive.

BAR-relative region and notification plans check requested alignment, width,
length and arithmetic. Notification multiplication widens both operands and
checks the access against the described region. A zero multiplier is valid.
Neither result proves that a BAR exists or that its actual aperture contains
the region; the mapping owner must supply those facts. A raw CfgCap shape does
not by itself authorize PCI configuration writes.

`mmio.omg` has the exact 256-byte register prefix and all 30 macro-defined field
descriptions, including the configuration tail at offset 256. Twenty-nine fixed
register accesses are four bytes even when their logical type is u8, u16 or
bool. The tail is a dynamic region marker, not a fixed register. All five MMIO
wide pairs and all 23 CommonCfg field descriptors/three PCI wide pairs are
retained, with the original read-only/write-only/read-write metadata.
`transport-check.py` audits these macro-generated fields separately from the
lexical inventory so they cannot disappear unnoticed. Copied snapshots permit
inspection without volatile access; modern-device admission checks magic,
version 2 and a nonzero representable ID, retaining unknown nonzero IDs.

`transport_access.omg` replaces volatile pointer wrappers with inert register
and wide-field data, explicit conversion and ordered access plans. Operations
carry kind, offset, physical access width and write value. Wide reads request
first then second words; the supplied words are decoded afterward. Wide writes
encode then request first followed by second. An upstream update is represented
by that read plan, a caller's pure value transformation, and the corresponding
write plan; it has no atomicity claim. Reserved/side-effect behavior must be
reviewed by the actual device owner before it executes any such plan.

The actual endian-num array behavior used by the pin is retained: little-endian
combines the first word as the low half, while big-endian combines it as the
high half. The pinned field names `low`/`high` do not change that BE ordering.
A Rust host witness verifies both directions. Modern VirtIO wide register
access uses the little-endian branch. Overaligned u8/u16 reads return checked
results instead of the pin's narrowing panic; typed ID/status/interrupt carriers
use that raw byte result. Pinned bool conversion is exactly equality with one;
a separate strict bool decoder rejects other values. Widening uses exact u32
casts. No generic callback or volatile authority is embedded in these values.

`transport_seams.omg` names the outstanding owner boundaries through request
data: `DiscoveryRequest`, `MappingRequest`, `DmaRequest`, `InterruptRequest`, and
`QueueActivationRequest`. The corresponding discovery, MMIO mapping/access,
DMA custody, interrupt routing and activation implementations are outside this
pure package. The activation predicate checks only queue geometry. It does not
validate physical ownership, cache coherence, synchronization, payload spans,
feature/status handshakes or completion. An owner must establish those before
any effect, and no request is itself a grant.

The protocol reference for the additional bounds is [VirtIO 1.3 draft section
4.1.4 and 4.2](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.html).
Pinned newer constants remain preserved; the draft does not override the pin.

```sh
python3 tools/ports/virtio/transport-check.py
python3 tools/ports/virtio/transport-semantic-check.py --omega /path/to/omega
python3 tools/ports/virtio/layout-check.py --module transport_layouts --omega /path/to/omega
python3 tools/ports/inventory.py check source/libraries/virtio/inventory.json --checkout reference_code/rust-osdev/virtio-spec-rs --require-transcribed
```

The Rust probe measures 28 actual transport geometry values for
x86_64-unknown-uefi; the expected vectors agree. Five explicit policy declarations
check independently. They remain requested plans, not selected or observed
Omega storage ABI. Fresh Omega eaa7993 passes all seven semantic groups and rejects all seven
independent body-mutating negative controls, each computing failure. The host probe also
checks actual pinned LE/BE word ordering and packed bitfield encodings.

A small compiler inference issue is worked around by explicitly casting the
left-hand field to u64 in the region span bound. On fresh eaa7993, a u64 field
comparison against an inferred `0xffffffffffffffff - other_field` can evaluate
incorrectly; an explicit carrier restores the result. The minimal
`reproduce-unsigned-max.py` compares inferred/explicit maximum forms. This is
not a port blocker. Native Omega execution, device access, DMA, barriers and
production transport integration have NOT RUN.
