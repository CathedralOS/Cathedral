# Normal Field namespace installation

Status: **tested in the recorded isolated worktree**: 38 checked loader pairs,
one constant-evaluator pair, 20 metadata/consumer compatibility pairs and three
selected upstream protocol owner-migration pairs pass.
This is a bounded ACPI-004 milestone. `loader::load` and
`loader::load_with_definitions` now admit normal AML Field declarations and
install canonical `Value::FieldUnit` objects. IndexField, BankField, executable
CreateField, dynamic declaration operands and general field execution remain
ordinary implementation work. ACPI-004/005/006 stay open.

## Source and ownership

The modified translation derives from rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, copyright 2018 Isaac Woods,
MIT OR Apache-2.0. The retained [notices](../../../../THIRD_PARTY_NOTICES.md) and
[license texts](../../../../licenses/rust-osdev/acpi/) apply.
`src/aml/mod.rs:1412` resolves normal Field regions; `parse_field_list:1831`
constructs and inserts the named FieldUnits. `src/aml/object.rs:394` defines
FieldUnit metadata. The [supplemental inventory](field-namespace-inventory.json)
retains the full pinned file/symbol hashes and leaves the general upstream
interpreter operations pending.

The existing metadata parser has one owner inside `cathedral-acpi-aml`:
`field_model.omg`, `field_flags.omg`, `field_elements.omg` and
`field_declarations.omg`. Its parser/flag bodies move without behavioral changes.
The child `cathedral-acpi-field-syntax` retains ordinary forwarding machines for
its parsing and flag entry points. Consumers now import shared data directly
from `aml::field_model`. No duplicate namespace, object arena, field type or
upward package dependency is introduced.

| Pinned responsibility | Canonical implementation | Scope |
| --- | --- | --- |
| Normal Field opcode loading | `loader.omg`, `field_namespace.omg` | Complete static declaration parsing followed by region lookup and staged namespace installation |
| `parse_field_list` syntax | `field_elements.omg`, `field_flags.omg`, `field_declarations.omg` | Existing bounded metadata behavior, including Access/ExtendedAccess and retained connections |
| `FieldUnit` / normal region binding | `model.omg::Value::FieldUnit`, `field_model.omg` | Stable region object ID plus complete declaration and named-field descriptor |
| Field object type | `object_queries.omg::object_type` | Standard FieldUnit type number 5, no payload evaluation |
| Field evaluation/access | Execution value/target admission | Explicit `UnresolvedRegion`, no handler or successful access stub |

## Admission and transaction

A normal Field declaration is completely parsed before namespace mutation.
The region NameString is resolved once with the existing scope/ancestor rules.
It must identify an allocated direct OperationRegion; aliases already identify
that same canonical object. Missing names fail through the existing namespace
outcomes. Non-region payloads fail `UnsupportedSyntax`; arbitrary reference
wrappers do not silently become region objects.

Every installed object retains the stable region ID, original lexical region
name, declaration scope, initial flags, complete declaration/FieldList spans,
absolute field name, bit offset/length, effective access/update/lock metadata,
connection metadata and named-field source span. A later region-name rebind does
not redirect existing FieldUnits. Connection names retain their declaration
scope; connection Buffer metadata retains either its known literal extent or
its existing explicit deferred encoding. No connection resource is evaluated.

The installer stages a canonical Namespace and publishes it only after every
field allocation and binding succeeds. A later failure elsewhere in the load
also restores the original namespace and method-definition observations through
the existing observed-loader transaction. External records are reset using the
existing load failure rules. No byte arena or live state is modified.

The existing pinned namespace replacement policy is preserved: redeclaration
allocates a fresh object ID, replaces the name binding, and leaves aliases to
older IDs intact. Repeated names within a FieldList likewise allocate successive
IDs. This remains an explicit pin-compatible deviation from the primary
load-time name-collision rule already recorded in [the AML port](PORT.md).

## Limits and authority

Existing bounds remain 1024 initialized source bytes, 16 path segments,
32 namespace entries including root, 64 stable object slots, eight scope
frames and 32 named descriptors per FieldList. The loader's existing
`value_budget` bounds FieldList element turns, independently for each
Field declaration; `term_budget` bounds outer loader turns. Oversized or
exhausted cases fail through `Capacity`, `Depth` or `WorkLimit` and roll back.
IDs are not reclaimed.

Declaration bit extents are inert metadata. They are not admission of a physical
transfer, a complete region-specific connection protocol, or a safe access plan.
The detached [field geometry](../field_access/PORT.md) component still owns
runtime footprint checks. Zero-width and large metadata extents remain
representable, matching the existing syntax parser. A deferred connection
Buffer is retained, not validated as an evaluated Buffer value.

Reads, named writes and CopyObject destinations containing FieldUnit objects
reach the existing explicit unresolved region boundary. No provider, global
lock, physical mapping, I/O capability, firmware invocation or production boot
import is added.

Primary references: [ACPI 6.6 §19.6.48, Field](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#field-declare-field-objects),
[§19.6.100, OperationRegion](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#operationregion-declare-operation-region),
and [ACPI 6.5 Errata A §20.2.5.2 AML encoding](https://uefi.org/specs/ACPI/6.5_A/20_AML_Specification.html#named-objects-encoding).
These govern grammar and inert object relationships, not resource grants.

## Verification

The new original fixtures compare all 32 namespace entries, all path slots,
all 64 object payloads/sibling links, both counts and all 64 method-definition
observations. Behavior controls change an expected final object-slot link.
Cases cover complete metadata, access/connection changes, scopes and aliases,
rebinding, malformed input, exact namespace/object limits, parser fuel, and
whole-load rollback. No firmware dump or external test body is copied.

The loader suite completed in 579.166 seconds with all authored package and
dependency bodies checked. Each scenario and its behavioral control passed;
source and runner hashes remained unchanged. The constant-evaluator pair
separately admitted the positive fixture and rejected its false contract control.
The compatibility suite covers all 18 existing syntax pairs through forwarding
entry points, shared geometry/read/write types, and FieldUnit query/access
boundaries. The selected protocol trio covers BankField read, IndexField write
and IndexField selector overflow after the nominal owner move.

Verification commands and exact stage receipts are owned by
[the fixture directory](../../../../tools/ports/acpi/aml/field-namespace/README.md).
Old field-syntax receipts remain historical records of their original hashes;
relocating the nominal metadata owner does not relabel those receipts as current
execution evidence. No native Omega or hardware execution is claimed.

## Canonical integration

The merge preserves isolated checkpoint `24aed04` and its 38 loader, 20 consumer,
three protocol and one constant pairs as source-bound evidence. The combined
canonical source passed a fresh replay of all 20 unchanged parser/consumer and
FieldUnit boundary pairs; exact receipt verification passed. Named Store routing
was retained while FieldUnit gained the explicit UnresolvedRegion arm. The
[fixture record](../../../../tools/ports/acpi/aml/field-namespace/README.md) lists
the receipt and exact input reconciliation. This does not relabel isolated
loader/constant results as fresh canonical execution.
