# Normal linear Field geometry and scalar chunks

Status: **published and verified at the final repository paths**. All 451
checked positives and 451 changed-body controls passed in 159.93 seconds.
Three constant positives and three rejecting controls passed in 68.484 seconds.
Both stages bind the same 26 current inputs; the retained-record verifier passes.
This bounded slice produces detached access geometry and performs scalar bit
extraction/update. It does not complete `Interpreter::do_field_read/write`.

Modified source derives from rust-osdev/acpi
[`aml/mod.rs` at 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs),
MIT OR Apache-2.0, copyright 2018 Isaac Woods. Upstream notices remain in
Cathedral's `THIRD_PARTY_NOTICES.md` and `licenses/rust-osdev/acpi/`.
`inventory.json` binds both relevant source files and retains whole-method
anchors as **pending**, with explicit partial target mappings.

## Profile and canonical inputs

`geometry::plan(kind, field, region_bytes, size)` consumes existing canonical
`DeclarationKind`, `Field`, `AccessType`, `UpdateRule`, and `IntegerSize`.
Only normal `Field` is admitted. Bank/Index declarations and every Connection
alternative other than `None` fail. The supplied effective access must match its
validated flags; forged kind/update/lock combinations fail. Nonzero attribute,
attribute mode, access length, or `extended=true` fails `UnsupportedMetadata`.
Plain effective AccessAs width changes with zero protocol metadata are compatible;
this is a flags-only boundary, not validation of an entire declaration history.
Field names and source spans do not affect geometry and are not resolved here.

AnyAcc explicitly chooses byte access. Byte/Word/DWord/QWord produce widths
1/2/4/8; BufferAcc is rejected as a non-linear protocol profile. Zero-width fields
are explicitly unsupported. The initialized field may contain 1–2048 bits (at
most 256 logical bytes). An unaligned byte field can require 257 native chunks.
These capacities are Cathedral limits, not ACPI maxima.

[ACPI 6.6 §19.6.48](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#field-declare-field-objects)
specifies native alignment, footprint constraints, update behavior and read
result type. The planner checks the complete aligned footprint against the
supplied region length before constructing a successful plan. Offsets are
relative to the parent region; physical base validity/alignment and held extent
remain later provider obligations. This API receives no region address-space
identity; the caller must separately establish a linear region/provider that
supports the selected native width. Arithmetic compares byte bounds without
multiplying the region length by eight, so `region_bytes=u64::MAX` is admissible.
An overflowing `bit_offset+bit_length` is rejected even when an alternative
byte-address representation could describe that interval: this is the declared
u64 bit-interval profile. No address arithmetic wraps.

## Output and scalar operations

`PlanResult` is either `Ready {plan}` or `Failure {error}`. Failure carries no
partial plan. Ready contains aligned `[start,end)`, width, count, bit interval,
region length, canonical update/access, read shape and `[Chunk;257]`. Every slot
is initialized; slots after `count` are zero. Each live chunk has relative byte
offset, width, logical-field bit position, native bit position, nonzero bit count,
mask and partial-container flag. `preserve_reads` counts partial chunks whose
update rule is Preserve. Integer result shape uses the existing revision width;
larger fields report exactly `ceil(bit_length/8)` Buffer bytes.

`LockRequirement::UnmetGlobalLock` records requested locking. It does not mean
the lock is held, acquired, or available. `NotRequested` likewise grants no
native callback permission. No authority, mutable register view, object token,
region installation or evaluator operation is created.

`chunks::valid` validates ordinary caller-built chunk geometry, including
capacity, native bounds, offset alignment/end, exact mask and partial flag.
`low_mask` yields zero for zero or more than 64 bits, all ones for 64, and
the low-bit mask otherwise. `extract` returns the selected low bits. `merge` applies Preserve, WriteAsOnes
or WriteAsZeros to a supplied logical chunk word. `Previous::{Absent,Value}`
is a semantic optional observation: partial Preserve without a previous value
returns `NeedsPrevious`; a full native write requires no previous value. Supplied
old/native words and new logical values are explicitly normalized to their widths.
These pure computations do not authorize read-modify-write on destructive reads,
W1C bits, FIFO registers, or shared hardware. Such protocols still require an
authored provider under `../ADAPTER.md`.

Bit-mask placement uses explicit wrapping bitvector shifts after geometry bounds
validation; this is not wrapping address arithmetic. Public helpers validate
malformed high-bit/MAX fields through typed numeric locals before comparisons.

## Exact partial source map and differences

| Pinned source | This slice |
| --- | --- |
| `aml/mod.rs:2519 do_field_read` | Normal-region native width/count/alignment, per-chunk bit positions, integer-versus-buffer shape, scalar extraction. |
| `aml/mod.rs:2612 do_field_write` | Normal-region geometry, partial Preserve read requirements and scalar update rules. |
| `aml/object.rs:548 copy_bits` | Scalar chunk extraction/merge only; arbitrary buffer copying and whole-field assembly are not implemented here. |
| `aml/object.rs:588 align_down` | Specialized positive power-of-two native alignment embedded in the planner. |
| `aml/object.rs:441 access_type_bytes`, `459 update_rule` | Existing canonical flags/type decoder is reused, with strict reserved-bit validation. |

The pin allocates Buffer results using the bit length rounded to eight **as a
byte count**, producing eight times the required storage. This profile reports
the correct logical byte count. The pin's normal native callbacks do not check
the declared region length; this planner rejects an out-of-region aligned
footprint before success. The pin may perform a read, or a Preserve read/write,
for an unaligned zero-width field. This profile rejects zero width. Reserved
flags and BufferAcc observations are retained as differences, not compatibility
branches. One retained invalid-update-rule write (`flags=0x61`) catches the
pin's panic before any callback. Requested locking is a TODO in the pin; the detached plan retains it
as an unmet requirement.

Namespace/region lookup, implicit source conversion, complete read assembly,
write sequencing, Bank/Index and connection protocols, lock acquisition,
provider effects and evaluator integration remain outside this bounded slice.
The missing pure assembly/integration work is ordinary future engineering;
this milestone makes no compiler or language blocker claim for it.

## Evidence

Original authored AML defines a synthetic SystemMemory OperationRegion, Field,
and method. The Rust harness calls actual public `Interpreter::new`,
`load_table`, and `evaluate`; callbacks access an initialized 512-byte Vec and
log every `(operation, offset, width, value)`. All other service callbacks trap.
One inert mutex is constructed by interpreter initialization; no lock acquisition
or hardware occurs. No private body mirror or forged ObjectToken is used.

All 362 public observations match independent interval/bit arithmetic, exact
callback order/values and complete final memory images. Classifications are
292 ordinary geometry agreements, 44 Buffer-length corrections, and 26 strict
rejections (10 invalid flags, eight region footprints, two BufferAcc and six
zero-width observations). This is a public host observation stage, separate
from Omega body evaluation.

The generated Omega suite has 451 positive/control pairs. Plan comparisons
inspect every field and all 257 chunk slots, including the zero tail. Controls
mutate an actual expected slot-256 mask (zero tail or final live chunk) or error/word condition. Chunk tests
cover each native width, updates, absent/supplied prior words, upper-bit
normalization, full-width masks, malformed ordinary chunks, and high-bit/MAX
bounds. The maximum checked fuel use was 63,752 per body. Exact counts, hashes and
stage outcomes belong to the retained receipts in `tools/ports/acpi/field-access`; see that directory's README.
