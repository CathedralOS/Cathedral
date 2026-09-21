# Inline Integer Field payloads

Status: **tested**. All 483 checked-interpreter positive/control pairs pass,
including 168 new scalar and 315 unchanged object-source pairs. Exact receipt
verification passes all 62 source hashes, generated batches and the pinned runner.
The new
`sequence::integer_payload_at(number, size, field_bits, ordinal)` entry accepts
an inline executor Integer without allocating a temporary namespace object or
byte-storage slot. The existing object-source API delegates its Integer branch
to the same private conversion path. Buffer and String handling is unchanged.

This is a modified composition of rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5),
`src/aml/mod.rs:2612` and its Integer/Field conversion responsibilities, copyright
2018 Isaac Woods, MIT OR Apache-2.0. Existing [inventory](inventory.json),
[notices](../../../../THIRD_PARTY_NOTICES.md) and
[licenses](../../../../licenses/rust-osdev/acpi/) remain authoritative.
`sequence.omg` contains the change; no new nominal value or result model is added.

## Admission and output

Width zero returns `Bounds`; width above 2048 returns `Capacity`. These errors
precede ordinal handling, including a maximum unsigned ordinal. For an admitted
width, ordinal zero produces the single Integer payload and any larger ordinal
returns `End {total:1}`. The number is first normalized to the selected four/eight
byte Integer size, then truncated or zero-extended to the field width by the
existing bit-copy kernel. This follows
[ACPI 6.6 Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules).

Payload length is exactly `ceil(field_bits/8)`, at most 256 bytes. The unused high
bits of the last logical byte and every inactive byte are zero. The result keeps
the canonical `PayloadResult` alternatives and counts. No object identity,
source span, byte-owner identity or synthetic arena entry is required.

This prepares data only. Source evaluation, opcode retirement, repeated writes,
provider operations, synchronization and resource authority remain outside the
helper. ACPI-005 and all whole-source interpreter anchors remain open.

## Evidence boundary

The [separate corpus](../../../../tools/ports/acpi/field-source-inline/README.md)
contains 168 new scalar positive/control pairs plus all 315 unchanged original
object-source pairs. Independent integer arithmetic covers zero/high-bit/MAX
numbers, both Integer sizes, bit/byte/word boundaries through 2048 bits, full
zero tails, nonzero/high-bit/MAX ordinals and width-error precedence. Controls
alter an expected payload byte, sequence count or error variant.

Execution took 644.553 seconds wall time and 1913.560 seconds summed batch time
across three workers and 49 packages. Maximum checked fuel was 316,054.
Every host batch checked dependency and authored bodies before running them.
Receipts bind current source, original fixture bodies, selected entries, generated
package/build text and the pinned runner. The earlier constant proofs and public
Rust observations retain their original source identity; no new constant,
public Rust API, native Omega, firmware or hardware result is implied.
