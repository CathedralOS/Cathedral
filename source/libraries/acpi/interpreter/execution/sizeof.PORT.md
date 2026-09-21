# Bounded SizeOf execution

Status: **transcribed; checked verification pending**. The 362 behavior/control
pairs are running against the frozen source snapshot. Fixture generation and
76 actual pinned public observations have passed; no passing Omega integration
result is claimed by this draft checkpoint.

This component adds actual AML `SizeOf` (`0x87`) execution through the owned
`prepare_program` / `run_program` pipeline. It reuses the existing ObjectType
metadata operand walk and the canonical `aml::object_queries::size_of` helper.
ObjectType's public descriptor API, lookup behavior and error policy remain
unchanged. The shared descriptor retains its existing `ObjectTypeResult` name.

The primary references are ACPI 6.6
[SizeOf](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#sizeof-get-data-object-size)
and the [AML grammar](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html).
The pinned source is rust-osdev/acpi
[`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`](https://github.com/rust-osdev/acpi/tree/257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5),
`src/aml/mod.rs` SizeOf dispatch and `do_size_of`, copyright 2018 Isaac Woods,
MIT OR Apache-2.0. This adapter is original Cathedral composition. The complete
symbol census in `sizeof-inventory.json` retains pending aggregate dispositions;
licenses remain in `licenses/rust-osdev/acpi` and `THIRD_PARTY_NOTICES.md`.

## Operand and result contract

NameString, Local0–7, Arg0–6 and Debug use the existing metadata path. Named
Methods are inspected without invocation, and FieldUnits are inspected without
region access, including Region, Bank and Index bindings. Metadata lookup stops
at the nearest typeless Scope. Scope, Debug, inline Integer/Uninitialized and
other unsupported payload kinds return `UnsupportedValue` for SizeOf.

Object-backed operands follow existing canonical references under one
64-inspection budget. SizeOf returns String bytes excluding the terminator,
Buffer bytes or Package element count. The canonical helper validates complete
owned/source byte storage or the advertised package sibling chain. It retains
source unit, bounds, ownership, encoding and object-identity validation. No
parallel storage or reference implementation is introduced.

RefOf, DerefOf and Index bytecode expressions remain `UnsupportedOpcode`; their
operands are not evaluated. Existing canonical reference descriptors remain
supported. Literal and unrelated expression operands retain the existing
metadata parser's `UnsupportedOpcode`; bare NullName is `InvalidTarget`.
Truncated operands retain parser failures. Resolution preserves MissingObject,
ReferenceCycle, WorkLimit and Capacity; malformed identities are InvalidState.
Storage failures use the existing generic byte-outcome mapping.

Inspection borrows Frame and ObjectStore read-only. The decoder contributes the
Integer through its existing bounded operand path before advancing the cursor.
Failure publishes no query result or cursor advance. Earlier successful method
effects remain committed. One query uses one existing dispatch turn, with no
allocation, writable target, callback, grant, mapping or synchronization action.

## Evidence boundary

Reproduction and source-bound records are in
[`sizeof-execution`](../../../../../tools/ports/acpi/interpreter/sizeof-execution/README.md).
The authored scenarios retain useful ObjectType coverage against the existing
path and add actual SizeOf methods, malformed storage, references, field kinds,
budgets and direct decoder publication checks. Unchanged ObjectType execution
and exhaustive decoder state fixtures are included as regressions.

The actual pinned public probe traps every device/time/debug callback and
retains known ObjectType typeless-scope and incomplete-operand differences.
The primary specification remains authoritative. These checks establish no
constant-evaluation, native Omega, firmware or hardware execution claim.

Reference-producing bytecode, field execution/providers, other missing opcodes
and complete ACPI-005/006 remain separate work. Future field-read sessions must
continue treating these metadata opcodes without issuing provider requests.
