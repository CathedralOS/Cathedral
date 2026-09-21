# BankField and IndexField namespace installation

Status: **tested in the recorded isolated worktree**. This extends the
normal Field milestone at Git revision `24aed04`; it does not complete ACPI-004
or execute any field access. Dynamic BankValue TermArgs remain ordinary
interpreter implementation work.

## Source and representation

Modified translation from rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5`, copyright 2018 Isaac Woods,
MIT OR Apache-2.0. The retained [notices](../../../../THIRD_PARTY_NOTICES.md) and
[licenses](../../../../licenses/rust-osdev/acpi/) apply. The source maps remain
in the [Field inventory](field-namespace-inventory.json) and
[aggregate AML inventory](inventory.json); general interpreter anchors stay open.

| Pinned responsibility | Canonical implementation |
| --- | --- |
| `src/aml/mod.rs:1025` BankField retirement; `:1422` BankField header; `:1445` IndexField header | `loader.omg`, `field_namespace.omg` |
| `src/aml/mod.rs:1831` named field insertion | Shared parsed descriptors and staged namespace publication in `field_namespace.omg` |
| `src/aml/object.rs:394` FieldUnit and `:402` FieldUnitKind | One `Value::FieldUnit` with `FieldBinding` in `model.omg` |

The existing parser and metadata modules are unchanged from their tested owner
relocation. `FieldBinding` adds explicit Region, Bank and Index alternatives.
Region holds an OperationRegion ID; Bank holds region and selector FieldUnit IDs;
Index holds index and data FieldUnit IDs. BankValue remains in the complete
canonical Declaration. No synthetic region ID, duplicate object arena or
additional field object variant is introduced.

The primary [BankField grammar and rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#bankfield-declare-bank-data-field)
require the region and selector relationships. The
[IndexField rules](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#indexfield-declare-index-data-fields)
require FieldUnit index/data objects. The
[AML encoding](https://uefi.org/specs/ACPI/6.6/20_AML_Specification.html#named-objects-encoding)
defines their headers and shared FieldList. Register access, index alignment,
synchronization and bank selection occur during future access, not installation.

## Admission and atomicity

The loader parses the entire declaration before namespace lookup. BankField
currently accepts the parser's Integer literal encodings, retaining the complete
64-bit metadata value. A dynamic BankValue leaves the existing explicit pending
span and fails UnsupportedSyntax. It is not evaluated, truncated to a selector
width or silently skipped.

Each required name resolves once in declaration scope, using existing ancestor,
absolute and parent-prefix lookup. The primary name is admitted before the
secondary. BankField requires a direct OperationRegion and a direct FieldUnit;
IndexField requires two direct FieldUnits. Aliases preserve canonical object IDs.
Arbitrary reference wrappers and wrong payload types fail UnsupportedSyntax;
missing objects retain the namespace error. This is stricter than the pinned
IndexField warning-only acceptance of an invalid data-register payload.

Existing normal, banked and indexed FieldUnits can be referenced as registers.
The loader retains those object identities without evaluating their dependency
graph. Shared index/data identity is representable. Live graph validation and
access remain open. Externally supplied namespace payloads are inert data.

All required identities are captured before installation. Even if a FieldList
replaces one of its register names, every object in that declaration retains the
same captured IDs. Later name replacement likewise leaves prior FieldUnits
unchanged. The existing pin-compatible name replacement policy remains as
recorded in [the normal Field milestone](field-namespace.PORT.md).

The same candidate Namespace transaction now installs all three families. Any
allocation or binding failure discards the declaration candidate. Any later load
failure restores the initial namespace and method definitions. All metadata,
source spans, access changes and connection descriptions remain canonical.
Existing capacities and term/element budgets are unchanged. No selector write,
region read, mapping, I/O grant or handler invocation occurs.

ObjectType still reports 5. Scalar projection, value admission, named Store and
CopyObject reach UnresolvedRegion for every binding alternative. The detached
[field protocol planner](../field_protocol/PORT.md) remains a separate pure
component; namespace installation does not authorize or execute its actions.

## Verification scope

The extended loader corpus contains 100 behavior/control pairs: the 38 previous
normal-Field scenarios, with the two former unsupported-opcode rows now accurately
named as invalid register-type cases, plus 62 new Bank/Index scenarios. Fixtures
compare all namespace entries, all object slots and links, complete bindings and
metadata, both counts, and every retained method definition.

The six separate boundary pairs cover each binding kind in both Integer widths,
direct and transparent ObjectType queries, scalar/value admission, named Store,
CopyObject source/destination failures and destination-first error precedence.
They compare the full namespace and all 16,384 byte-arena bytes. Their controls
change the final byte so incomplete comparison cannot pass.

All 100 loader pairs pass in 1189.721 seconds, with a maximum 320,898 evaluator
fuel units. All six boundary pairs pass in 204.259 seconds, with a maximum
451,412 fuel units. A separate three-pair check extracts the actual shared
generic bridge comparator and distinguishes Region region identity, Bank
selector identity and Index data identity; it passes in 9.858 seconds, with a
maximum 502,409 fuel units. Every receipt retains unchanged source/tool/binary
hashes and exact fixture/build hashes. Each receipt's strict input/result
verification passes. The earlier broad bridge corpus was not reexecuted.

Commands and exact stage evidence are in the
[loader fixtures](../../../../tools/ports/acpi/aml/field-namespace/README.md) and
[boundary fixtures](../../../../tools/ports/acpi/aml/field-bindings/README.md).
The earlier 38-loader, constant, 20-owner and three-protocol receipts remain
historical evidence at `24aed04`, whose exact committed inputs are preserved.
No new constant-evaluator, native or hardware result is implied.
