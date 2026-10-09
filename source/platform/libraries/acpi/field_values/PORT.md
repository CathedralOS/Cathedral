# Detached normal Field read assembly

Status: **published and tested at the final repository paths**. All 301 checked
positives and 301 changed-body controls passed in 215.759 seconds wall time
(624.925 seconds summed batch time, three independent processes).
Three constant positives and rejecting controls passed in 89.549 seconds.
All 259 public observations reproduce; the verifier passes 32 current input hashes
and all 27 recorded upstream hashes at the exact pin. This adds a
bounded read-data algorithm. It does not complete `Interpreter::do_field_read`
and does not execute a Field against hardware or a namespace.

Modified source derives from rust-osdev/acpi
[`aml/mod.rs:2519` at 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2519),
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Exact notices/licenses remain in
Cathedral's `THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/acpi/`.
`inventory.json` binds two complete pinned source files and retains the 158
whole-method/declaration anchors as pending with explicit partial target maps.

## API and interpretation

`read::assemble(kind, field, region_bytes, size, words:&[u64;257], supplied_count)`
reuses canonical `DeclarationKind`, `Field`, `IntegerSize`, access metadata and
`LockRequirement`. It recomputes `field_access::geometry::plan` internally.
A caller-built Plan is never accepted. The entire native footprint and all
supported-profile checks therefore precede use of supplied words.

A geometry failure is returned first as `ReadError::Geometry {cause}`, retaining
the existing geometry error. Otherwise `supplied_count` must exactly equal the
computed chunk count and must be at most 257; zero, surplus, missing, high-bit
and MAX counts fail `CountMismatch`. Typed u64 locals protect the numeric bounds.
Each supplied word corresponds to that planned chunk's index; unused initialized
word slots are ignored. The words are ordinary numbers and are not evidence that
a device read occurred, that observations were coherent, or that a lock was held.

`ReadResult` defaults to `Failure {Geometry {InvalidFlags}}`, so ordinary
default initialization cannot present a successful Integer. Its semantic alternatives are:

- `Integer {value,size,lock}` for a field no wider than the selected 32/64-bit
  Integer size; higher value bits are zero.
- `Buffer {length,bytes:[u8;256],lock}` otherwise; length is exactly
  `ceil(bit_length/8)`. The final unused bits and every unused byte are zero.
- `Failure {error}` with no partially assembled value.

[ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
and [§19.6.48](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#field-declare-field-objects)
define normalized Field values and the Integer/Buffer choice by field width and
Definition Block revision. The lower field bits become the lower result bits;
bytes use little-endian order. Existing geometry selects AnyAcc byte policy,
1/2/4/8-byte linear accesses and the 1–2048-bit field limit. Its explicit
zero-width, BufferAcc, Bank/Index, connection/protocol metadata and overflowing
bit-interval restrictions apply unchanged. Region length may be u64 MAX because
geometry validates the aligned footprint in byte units.

## Implementation reuse and failure behavior

For each internally produced chunk, existing `chunks::extract` validates its
shape and masks/shifts the supplied numeric word. Native bits outside the
selected width/field are thereby normalized away. Existing
`conversions::integer_to_buffer(EightBytes, ...)` encodes the extracted scalar;
`buffer_fields::copy_bits` places its bits at the planned logical field position
in a private, zero-initialized array. Existing `field_to_integer` forms Integer
results. Buffer results transfer the completed initialized array directly.
No competing canonical Value, IntegerSize, access or buffer-storage model exists.

Any unexpected extraction/copy failure returns `InternalChunk`; partially filled
private bytes never appear in the public failure alternative. All loops are
bounded by computed geometry and fixed array capacities. There are no service
calls, mutable input parameters, allocation, native pointers or device writes.

Both successful alternatives preserve the plan's `UnmetGlobalLock` requirement
when locking was requested. Assembly does not discharge it. `NotRequested`
likewise supplies no provider permission. The adapter still must establish a
supported linear region, actual held extent, native access authorization,
base/alignment validity, observation semantics and any required synchronization;
see `../ADAPTER.md`. Source conversion, Store, object installation, namespace
resolution and evaluator dispatch are outside this slice.

## Exact pinned differences and evidence

The pin allocates a Buffer using the field bit length rounded to eight as a byte
count, yielding eight times the required storage. This module returns the exact
logical byte count. No compatibility branch preserves that allocation bug.
Existing strict geometry checks are also retained: the pin may access beyond a
declared region footprint, accepts some reserved flags/BufferAcc paths, and can
read an unaligned zero-width field. Such inputs are rejected before assembly.

The new public Rust harness executes actual `Interpreter::new`, `load_table`
and `evaluate` against original synthetic AML and initialized Vec memory.
Read callbacks record `(offset,width,value)`; **all writes and other service
callbacks trap**. No private body mirror, forged ObjectToken or hardware fixture
is used. One inert mutex is created by Interpreter initialization; no lock is
acquired. Its 259 observations cover four byte patterns and both revisions:
154 Integer agreements, 92 equal Buffer values with the explicit length
correction, and 13 strict geometry rejections. Independent Python arithmetic
checks exact callback tuples, unchanged complete memory and normalized results.

The Omega suite contains 301 positive/control pairs: 156 Integer cases, 92
Buffer cases, 33 geometry errors, 19 count mismatches, and one explicit default-failure case. It tests all native
widths, unaligned boundaries, full 2048-bit fields, poisoned high native bits and
unused words, locked requirements, both Integer sizes, and geometry-before-count
failure ordering. Buffer comparisons inspect all 256 bytes; controls mutate
byte 255, including zero tails. Integer and failure controls mutate their actual
expected semantic condition. Counts above 257, 2^63 and u64 MAX are explicit.
Maximum checked fuel was 487,490 per body. Exact stage results and current
input hashes are retained in the owned tools.

The source map remains partial: assembly of detached words is complete for this
bounded profile, while region acquisition, native service execution, lock
handling, Bank/Index and evaluator integration are unimplemented here. This is
not a language blocker or a claim that whole Field read execution is complete.
