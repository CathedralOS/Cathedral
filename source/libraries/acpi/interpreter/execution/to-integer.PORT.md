# Explicit ToInteger execution

Status: 25 complete Frame/ObjectStore retirement behavior/control pairs passed
in the isolated ToInteger worktree. Actual bytecode verification is pending a
fixture correction: three negative controls attempted the out-of-range literal
`u64::MAX + 1`. ACPI-005 and aggregate upstream anchors remain open.

This modified composition maps `src/aml/mod.rs:2057 do_to_integer` at rust-osdev/acpi
`257aa561aa190f1cfe2de5d1a4f0af9d09ff1db5` to `to_integer_execution.omg`, opcode
`0x99` in `operator_specs.omg`, and retirement dispatch in `retire.omg`. Upstream
copyright 2018 Isaac Woods and MIT OR Apache-2.0 notices remain in
[THIRD_PARTY_NOTICES](../../../../../THIRD_PARTY_NOTICES.md). The
[library charter](../../../CHARTER.md) and
[porting policy](../../../../../wiki/architecture/prior_art_and_hardware_facts.md)
apply. This adds no hardware, region-handler, firmware or boot authority.

The adapter prepares the full conversion through canonical
[`object_conversions::to_integer`](../../aml/object-conversions.PORT.md) before
target publication. Inline Integers normalize to the active 32/64-bit width.
Transparent Named/Local/Arg carriers resolve under the existing 64-inspection
bound; explicit RefOf/Index source identities remain unsupported. Byte backing,
selected-width Buffer prefixes, and all nine conversion errors retain the
canonical helper contract. The existing StrictDecimalHex String profile accepts
complete decimal or 0x/0X digits and rejects empty, whitespace, sign, suffix,
invalid ASCII and overflow. Not every malformed lexical outcome is mandated by
the ACPI specification. Ordinary operand decoding still excludes field reads;
direct helper support for BufferField does not add field bytecode evaluation.

[ACPI 6.6 §19.6.141](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#tointeger-convert-data-to-integer)
defines ToInteger. [§19.3.5.5](https://uefi.org/specs/ACPI/6.6/19_ASL_Reference.html#implicit-result-object-conversion)
excludes explicit conversions from implicit result conversion: the result
directly replaces the destination. The adapter therefore uses the canonical
copy target path. Named Integer/String/Buffer targets become Integer while
retaining object identity and clearing obsolete owned backing. The expression
contributes the converted inline Integer. Local/Arg targets retain canonical
binding and explicit-reference redirection rules. Null discards only the write;
Debug remains UnresolvedService. Regions and invalid destinations fail closed.
The pinned `do_to_integer` calls `do_store`; that behavior is a recorded
difference, not evidence against the primary target rule.

Source conversion and target-admission failures preserve the complete store and
frame. This retirement helper requires a valid engine retirement Frame with room
to contribute to its parent operation; arbitrary forged/full parent operations
are outside that precondition. As in existing retirement helpers, a successful
write is not rolled back if an invalid caller Frame subsequently rejects result
contribution. Earlier AML statements are not rolled back by conversion failure.

The [fixtures](../../../../../tools/ports/acpi/interpreter/to-integer-execution/README.md)
use actual loader/Program/executor bodies and full-state retirement comparators.
Historical public comparison evidence remains in
`tools/ports/acpi/aml/object-conversions/reference-verification.json`: 72 public
Object::to_integer observations and 46 actual Interpreter ToInteger evaluations.
The latter use Null targets. They document width, empty-input, lexical and
overflow differences; they do not attest named target replacement. No new public
Rust run, constant evaluation, native execution or hardware result is claimed.
