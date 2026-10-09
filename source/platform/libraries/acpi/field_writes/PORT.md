# Detached normal Field single-pass write assembly

Historical bulk status: **published and tested at the recorded repository paths**. All 454 checked
positives and 454 changed-body controls passed in 715.718 seconds wall time
(2084.641 seconds summed batch time, three independent processes).
Three constant positives and rejecting controls passed in 108.464 seconds.
All 126 public observations reproduce. The retained verifier binds 31 input hashes,
all 27 recorded upstream hashes, exact upstream HEAD and generated source/build
recipes. This bounded data algorithm does not complete `do_field_write`.

The [single-chunk extension](chunks.PORT.md) shares the extraction/merge kernel
and retains a separate receipt for its own source closure. The numbers above
describe the original bulk implementation, not a rerun of the later extension.
The [single-payload continuation](transfer.PORT.md) now sequences relative native
requests and acknowledgements through the same unchanged merge kernel, with
102 new and eight selected original behavior/control pairs.

Modified source derives from rust-osdev/acpi
[`aml/mod.rs:2612` at 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2612),
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Notices/licenses remain in
`THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/acpi/`. The owned inventory
binds both complete upstream files and keeps all 158 anchors pending, with
explicit partial implementation targets.

## API and payload boundary

`write::assemble(kind, field, region_bytes, size, payload:&[u8;256],
payload_length, previous:&[Previous;257], supplied_count)` reuses canonical
`DeclarationKind`, `Field`, `IntegerSize`, `Previous`, and access metadata.
It recomputes `field_access::geometry::plan` internally; no caller-built Plan is
accepted. The complete native footprint and supported metadata are validated
before payload/count checks. `Geometry {cause}` therefore wins over later errors.

The payload is **already converted to one field-sized logical bit vector**.
Its declared byte length must exactly equal `ceil(field.bit_length/8)` and be at
most 256; otherwise `PayloadLength` is returned. Unused input slots are ignored,
as are high unused bits of the last logical byte. `supplied_count` must then
exactly equal the internally planned count and be at most 257; otherwise
`CountMismatch` is returned. Counts are checked using typed scalar bounds.

Each `Previous` slot corresponds to the same native chunk index. An absent
value is accepted for a full native write and for WriteAsOnes/WriteAsZeros.
A partial Preserve chunk requires `Previous::Value`; otherwise the result is
`Chunk {index,cause:NeedsPrevious}`. Previous values are ordinary initialized
numbers, not proof of a device read, coherence, synchronization, or permission.
Their upper bits are normalized to the chosen native width by canonical merge.

The failure-first semantic `WriteResult` defaults to
`Failure {Geometry {InvalidFlags}}`. Success is
`Writes {count,records:[NativeWrite;257],lock}`, with each live record holding a
relative byte `offset`, native byte `width`, and normalized numeric `value`.
Every unused record is entirely zero. A later error discards all private staged
records; the public Failure alternative cannot expose partial write recipes.
No native write has occurred during assembly.

## Primary semantics and deliberate limits

[ACPI 6.6 §19.6.48](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#field-declare-field-objects)
and [Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
define Field access and conversion rules. Integer sources are zero-extended or
truncated to Field width. A Buffer wider than a FieldUnit is broken into pieces
and repeatedly written, lower pieces first, with the final piece zero-extended.
That conversion and repeated execution are **not implemented by this API**.
Its byte array is a logical bit-vector input, not an AML Buffer source.
In particular a non-byte-aligned field's final storage byte is not permission to
interpret the vector as a wider AML Buffer and claim conversion equivalence.
The pinned `do_field_write` accepts Integer/Buffer objects and executes one pass;
no compatibility branch preserves its missing wider-Buffer repetition behavior.

Geometry retains its documented AnyAcc-as-byte policy, Byte/Word/DWord/QWord
support, positive 1–2048-bit limit, and rejection of BufferAcc, Bank/Index,
Connection/protocol/extended metadata, forged flags, overflowing bit intervals,
and native footprints outside the supplied region length. Region length may be
u64 MAX because fit is checked in byte units without multiplying it by eight.
There is no source conversion, namespace lookup, Store/object installation,
provider call, live memory operation or evaluation dispatch in this slice.

## Implementation and authority

For each internal chunk, `buffer_fields::field_to_integer` extracts the logical
bits using the validated payload length, followed by canonical `chunks::merge`
with the plan's UpdateRule. Extraction chooses the smallest canonical 32/64-bit
integer shape that contains the chunk. `InvalidChunk` is retained if unexpected
internal extraction validation fails. No new nominal access/Integer/Previous
model or duplicate low-level copy/merge implementation is introduced.

All loops are bounded by validated geometry and fixed capacities. Successful
recipes retain `UnmetGlobalLock` when required. Assembly does not acquire a lock
or authorize either a read or a write. `NotRequested` grants no permission.
Adapters must separately establish actual held extent/base/native access rights,
provider semantics and synchronization; see `../ADAPTER.md`. Numeric Preserve
merge alone must not be treated as permission for read-modify-write, destructive
reads, W1C registers, FIFO access, or any other device behavior.

## Evidence

The checked suite has 454 actual body pairs, including geometry, exact payload
and Previous counts, u64 MAX/2^63 inputs, all update modes and native widths,
full/partial/unaligned chunks, 2048-bit fields with 257 native byte writes,
poisoned unused payload bytes/high native bits, both Integer sizes, explicit
lock metadata, default Failure, and early/late absent Previous. Every successful
case compares all three fields of all 257 output records. Controls change the
expected record 256 value, including initialized zero tails, or an actual error
condition. The late error case verifies the public result is only Failure after
earlier private records would have been staged. No final-result-plus-one control
is used.

The public Rust harness uses actual `Interpreter::new`, `load_table` and
`evaluate` on original synthetic AML, with initialized Vec-backed SystemMemory
callbacks. Exact read/write `(offset,width,value)` logs and the entire resulting
512-byte memory are compared to independent interval arithmetic. All other
services trap; one inert mutex is created during Interpreter setup, with no
acquisition. The 90 Integer probes cover both Definition Block revisions; the
36 Buffer probes use exactly field-sized byte-aligned input lengths, preserving
one-pass equivalence. No fake ObjectToken, private-body mirror, firmware sample
or device operation is used. Locked flags in these probes do not prove the pin
acquires a GlobalLock (its implementation has a TODO); Omega retains it unmet.

Three constant pairs cover unaligned Preserve, late MissingPrevious, and MAX
count rejection at maximum geometry. Retained evidence separately identifies
source admission, checked interpretation, and constant proof; no native Omega
execution or whole Field write integration is claimed.

Maximum checked fuel was 788,068 per body. The 454 scenarios comprise 381
successful write recipes, 33 geometry errors, 19 count errors, 18 payload-length
errors, two early/late missing-Previous errors, and one default-failure check.
