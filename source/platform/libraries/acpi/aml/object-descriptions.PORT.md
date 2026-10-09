# Direct nonbasic object type descriptions

Status: **tested** at the pure description boundary. All 115 final-path
behavior/control pairs and three constant pairs passed again with explicit
execution-root and generated-build provenance; eleven static pinned labels
verify. Production source is unchanged, and source/tools are frozen.
This is a name-only component of Concatenate, not general formatting or dispatch.

`object_descriptions::describe(store, object)` returns Failure(ConversionFailure)
or String(length, bytes[256]). Default initialization gives Failure(InvalidState).
A staged unsigned object count and ID are checked first: count must be at most
64, and the ID must be below both count and 64. Invalid membership returns
InvalidState before type dispatch. A valid Uninitialized slot has its own label;
an ID outside the logical object table is not an Uninitialized object.

[ACPI 6.6 Table 19.31](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#concatenate-concatenate-data)
and the pinned nested `resolve_as_string` agree on these represented labels:

| Canonical case | Exact logical text |
| --- | --- |
| Uninitialized | `[Uninitialized Object]` |
| Package | `[Package]` |
| BufferField | `[Buffer Field]` |
| Device | `[Device]` |
| Event | `[Event]` |
| Method | `[Control Method]` |
| Mutex | `[Mutex]` |
| OperationRegion | `[Operation Region]` |
| PowerResource | `[Power Resource]` |
| Processor | `[Processor]` |
| ThermalZone | `[Thermal Zone]` |

Lengths exclude a terminator. All 256 output bytes are initialized and every
nonlogical tail byte is zero. Labels are at most 22 bytes and cannot exhaust
this fixed result capacity. Integer, String, Buffer, every Reference kind and
NameReference return UnsupportedValue. The existing canonical model has no
FieldUnit, Debug, RawDataBuffer or NativeMethod case; this adapter invents none.

Only the case of an admitted slot matters. Package member lists, field backing,
bit extents, source spans, Method flags/body, namespace names, links, byte arena
metadata and service parameters are not validated or followed. Even malformed
payload metadata is intentionally descriptive here; a label does not certify
that its object can be read, executed, locked or accessed. There is no source
snapshot, reference resolution, package traversal, Method invocation, mutex/event
operation, region access or target mutation. Shared store access cannot install
a value or allocate an object. This distinction is covered by poisoned-payload
fixtures, including maximum unsigned lengths and IDs.

The licensed upstream component is rust-osdev/acpi
[`src/aml/mod.rs:2176`, pin 257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5](https://github.com/rust-osdev/acpi/blob/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5/src/aml/mod.rs#L2176),
`do_concat`'s nested `resolve_as_string`, copyright 2018 Isaac Woods,
MIT OR Apache-2.0. Exact licenses and upstream notices remain in
licenses/rust-osdev/acpi and THIRD_PARTY_NOTICES.md. The nested formatter and
whole Concatenate operation remain pending: basic value conversion, missing
canonical types, reference/name behavior, operand selection, target stores,
context contribution and retirement are outside this slice. This API makes no
Integer-left/nonbasic-right dispatch policy decision.

Verification uses actual authored Omega bodies and changed expected-body
controls; successful cases compare exact logical lengths and all 256 bytes.
Every label is checked at slots 0, 31 and 63 with ordinary and poisoned metadata.
Basic values, six reference kinds with valid/self/invalid targets, NameReference,
empty/oversized object counts and invalid/MAX IDs have explicit outcomes. Failure
controls demand a different error; successful controls alter expected byte 255.
Representative constants cover a long label, an ignored malformed field
payload and an invalid table count. A static pinned-source label audit is separate
from execution evidence. No public Rust execution or private Rust mirror is claimed.

The complete suite has 115 behavior/control pairs: 66 successful labels,
12 unsupported basic/name values, 18 opaque references, 18 invalid table/ID
combinations and the default failure. Three constant pairs select
Uninitialized at slot 63 with poisoned unrelated state, BufferField at slot 31
with invalid backing metadata, and Device with a MAX object count. The static
label audit records primary table identity, exact upstream pin/hash/lines and
fixture/audit hashes. It does not certify generic Rust Concatenate behavior.

Final receipts bind the exact 14-file source/build dependency closure, authored
tools/cases/label audit, pinned clean Omega revision, fresh compiler binary,
canonical runner source/lock and loaded runner binary for every batch. Each
batch records exactly one checked-package header and one observed 0/1 pair for
every requested behavior. Six constant proofs bind generated source and compiler.
The verifier checks the current static label audit and all 16 original scratch
artifacts retained under `tools/ports/acpi/aml/object-descriptions/history/prepublication`.
The final manifest binds current artifacts and used source hashes. No public Rust,
native, hardware or whole AML interpreter execution is claimed by this slice.

```sh
python3 tools/ports/acpi/aml/object-descriptions/verify_record.py
python3 tools/ports/acpi/aml/object-descriptions/inventory.py --check
```

The tool README lists the complete replay commands. Both do_concat and its
nested resolve_as_string remain pending in the 97-anchor component inventory.

Provenance refinement: the original completed receipts did not record their
dependency root or generated build configuration. Those receipts, tools and
original manifest are preserved unchanged under `history/root-unbound` and
are historical rather than the authoritative final-path receipts. The revised
runtime and constant records include resolved `execution_root`, exact generated
`build_text`, and `build_sha256`. Every batch/proof carries the same root and
build hash; the tools verify the actual generated file before and after execution.
The verifier requires the current repository root and exact reconstructed build
text/hash, so a receipt bound to the scratch dependency tree is rejected.
Production source, the 115 behavior pairs and the three constant cases are unchanged.

The authoritative final replay passed after the provenance refinement. The final
manifest includes the 16 prepublication artifacts and the 31-artifact archive of
the earlier completed release, each preserved against its original manifest.
Earlier root-unbound receipts are not presented as proof of the final dependency
root. No semantic source or fixture change was required by this refinement.
