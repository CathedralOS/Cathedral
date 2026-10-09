# Packed virtqueues — VIRTIO-002

**Status: tested** pure encodings and queue arithmetic; policy declarations
checked separately, with no observed Omega ABI claim.

Modified translation of the same pinned MIT OR Apache-2.0 source documented in
`PORT.md`, specifically `src/pvirtq.rs` and shared notification/flag facts.
All four packed source records and their fields are retained; generated Rust
bitfield access maps to explicit raw getters, checked constructors and codecs.
Raw constructors retain reserved mode 3. Protocol validation rejects that mode,
nonzero reserved flag bits, out-of-ring offsets and descriptor events without
EVENT_IDX negotiation. Enable/disable modes ignore the offset as specified.

The added algorithms follow [VirtIO 1.3 draft section 2.8](https://docs.oasis-open.org/virtio/virtio/v1.3/virtio-v1.3.html):
packed sizes range from 1 through 32768 and need not be powers of two. Cursor
advance divides by the actual size and toggles wrap parity for every crossing.
The initial wrap value is one. Available flags have AVAIL equal to the current
wrap and USED opposite; used flags have both equal. These are observed-value
predicates, not memory ordering operations.

Event decisions use distance modulo twice the actual queue size. A descriptor
specific event triggers for a matching offset/wrap in the half-open interval
from the old cursor to the new cursor. An advance greater than one queue is
rejected; equal cursors represent zero progress. A two-lap history aliases the
same cursor and cannot be reconstructed from these bits. Callers must bound
progress independently. Enable mode allows even redundant notifications;
disable mode suppresses them. No helper sends an interrupt or notification.

Individual outer descriptor validation preserves opaque buffer IDs, rejects
unknown flags and malformed indirect lengths, and requires negotiated indirect
support. Indirect table elements permit only WRITE. A complete list validator
must additionally enforce writable ordering, chain length, and available-space
constraints. Packed publication requires real ordering/barriers and ownership;
this package provides neither and is not an operational driver.

`packed.vectors.json` has 14 expected geometry values measured from the actual
pinned Rust types for x86_64-unknown-uefi. Native Rust `Desc` is 16 bytes/alignment
eight; the protocol ring region alignment is sixteen. Native Rust
`EventSuppress` is four bytes/alignment two; its protocol region alignment is
four. Explicit byte codecs and separate region constants preserve this
important distinction. The four policies in `packed_layouts.omg` are checked
requested representations, not selected or observed Omega storage layouts.

```sh
python3 tools/ports/virtio/packed-check.py
python3 tools/ports/virtio/packed-semantic-check.py --omega /path/to/omega
python3 tools/ports/virtio/layout-check.py --module packed_layouts --omega /path/to/omega
```

The actual pinned Rust host probe checks offset/wrap, reserved flags, and
notification bitfield encodings. Fresh Omega eaa7993 passes five pure semantic
groups (cursor, flag ownership, event encoding, suppression, descriptors) and
rejects five corresponding body-mutating negative controls, each computing
failure. Fixtures cover non-power-of-two queues, multiple wrap crossings,
maximal offsets/counts, the opposite wrap epoch, empty intervals, invalid modes,
indirect constraints and exact little-endian descriptor/event bytes. No Omega
native executable, physical ABI observation, DMA or device access has run.
