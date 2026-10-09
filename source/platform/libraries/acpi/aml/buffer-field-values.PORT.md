# Detached BufferField reads

Status: tested at the detached bounded read boundary. Final repository-path
verification passed 281 behavior/control pairs, 3 constant-expression pairs
and 136 actual public Object observations. This read-only adapter does not
modify existing storage, conversion, execution or parser modules.

`buffer_field_values::read(input, length, unit, store, field, size)` accepts an
initialized 1024-byte source snapshot and a direct field object ID. The caller
resolves outer references. The result is `FieldValueRead::Failure(reason)`,
`Integer(number)`, or `Buffer(length, bytes[256])`; the default is
Failure(InvalidState). It is a detached result, not a second AML Value or arena.
No object ID is allocated, no caller state is mutated, and no output is returned
on failure. Successful buffers have zero unused high bits and zero tail bytes.

Admission validates the staged object count and direct ID before reading its
Value. Only BufferField is supported. Backing bytes pass through the existing
`read_bytes`: Named/Local/Arg wrappers are transparent; RefOf/Index/Unresolved
remain opaque and therefore fail as unsupported byte values. Cycles and invalid
IDs keep their existing error outcomes. The backing's complete byte snapshot is
validated before field offset/length admission. Source spans remain immutable;
Owned payloads must name their actual initialized owner slot. Full String ASCII
and NUL validation applies even if the field selects only an earlier prefix.
Unused source metadata is ignored for Owned backing, as in the canonical reader.

The field must be nonempty and wholly inside the logical backing extent. Offset,
length and store counts are staged as unsigned locals before comparisons. Bounds
use `start <= bits` followed by `count <= bits - start`, avoiding overflow. The
backing limit of 256 bytes implies a maximum field width of 2048 bits. Fields at
most the selected integer width (32 or 64 bits) return a normalized Integer;
wider fields return exactly ceil(width / 8) logical bytes. Bit extraction delegates
to the existing `field_to_integer` and `copy_bits` helpers. All byte output starts
zeroed; no initialized but nonlogical Owned tail can leak into the result.

## Primary rules and pinned differences

This is a modified translation of `Object::read_buffer_field` at
[rust-osdev/acpi object.rs:264](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/object.rs),
pin 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5. Copyright 2018 Isaac Woods;
MIT OR Apache-2.0. Exact license texts and translation attribution remain in
Cathedral's licenses/rust-osdev/acpi and THIRD_PARTY_NOTICES.md.

ACPI 6.6 [Table 19.7](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#data-type-conversion-rules)
selects Integer or Buffer by the field's width in bits compared with the
Definition Block integer width. The pin compares bit length with
`integer_size as usize`, whose values are 4 and 8 bytes. Therefore the pin returns
Buffer for widths 5..32 at 32-bit mode and 9..64 at 64-bit mode; this adapter
intentionally returns Integer for those intervals. Public method observations
retain the actual result shapes instead of silently converting them for equality.

[CreateField §19.6.21](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#createfield-create-arbitrary-length-buffer-field)
requires the complete field range to exist and rejects zero width. The adapter
rejects zero/out-of-range stored metadata even though the pinned read can return
Integer(0) for zero width or silently zero-fill beyond the backing extent.
These outcomes are explicit profile corrections. String backing is the internal kernel/Index-compatible
profile inherited from storage and the pin; it does not broaden CreateField's
Buffer-only declaration grammar. Strict ASCII/NUL validation is the canonical
String storage profile, while Rust String permits interior NUL and UTF-8.

The 64-slot/256-byte/1024-source capacities are Cathedral resource limits, not
ACPI maxima. This helper does not access Field Units, operation regions, devices,
mutexes or firmware services. Wider field reads here do not automatically expand
`object_conversions::to_integer`; that separate adapter remains unchanged.

## Evidence

The owned tools live in tools/ports/acpi/aml/buffer-field-values. Tests execute
actual Omega bodies with changed expected results, compare all 256 output bytes,
and include direct/transparent references, Source/Owned backing, dirty tails,
unaligned widths through 2048 bits, malformed metadata and error-order cases.
Three representative constant-expression positive/negative pairs are separate
from checked-interpreter execution. The Rust probe calls the actual public
Object::read_buffer_field; no private-body mirror or ObjectToken is used.
Native ABI, runtime opcode dispatch and hardware behavior are not claimed.

Historical scratch source, tools and receipts remain under the owned tool
directory's history/prepublication path with their original manifest. Final
receipts bind the current 12-file imported source/build closure, compiler and
runner identities, exact generated fixture bodies, and public Rust probe.

Reproduce from the repository root:

```sh
python3 tools/ports/acpi/aml/buffer-field-values/fixtures.py --check
python3 tools/ports/acpi/aml/buffer-field-values/inventory.py --check
python3 tools/ports/acpi/aml/buffer-field-values/reference.py --write
python3 tools/ports/acpi/aml/buffer-field-values/check.py --record tools/ports/acpi/aml/buffer-field-values/verification.json
python3 tools/ports/acpi/aml/buffer-field-values/check_const.py
python3 tools/ports/acpi/aml/buffer-field-values/verify_record.py
```
